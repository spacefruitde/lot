//! Tür- und Fensterliste (4.05): jede Öffnung mit Position, Lage zwischen den
//! Räumen, Rohbaumaßen, Brüstung, lichter Breite und Anschlag, dazu die
//! Zusammenstellung gleicher Typen mit Stückzahl.

use lot_model::{Handing, OpeningKind, Project};

use crate::accessible::AccessibleNorm;
use crate::rooms::StoreyRooms;

/// Art einer Öffnung in der Liste.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum OpeningClass {
    Door,
    Passage,
    FrenchWindow,
    Window,
}

impl OpeningClass {
    pub fn title(self) -> &'static str {
        match self {
            OpeningClass::Door => "Tür",
            OpeningClass::Passage => "Durchgang",
            OpeningClass::FrenchWindow => "Fenstertür",
            OpeningClass::Window => "Fenster",
        }
    }

    fn prefix(self) -> &'static str {
        match self {
            OpeningClass::Door | OpeningClass::Passage => "T",
            OpeningClass::FrenchWindow | OpeningClass::Window => "F",
        }
    }
}

/// Eine Zeile der Liste.
#[derive(Debug, Clone, PartialEq)]
pub struct OpeningRow {
    pub id: lot_model::ElementId,
    /// Position, etwa „T 0.03“.
    pub mark: String,
    /// Typ, etwa „T2“.
    pub type_mark: String,
    pub storey: String,
    pub class: OpeningClass,
    /// Raum, in dem die Öffnung gezählt wird (bei Türen der Raum, in den sie aufschlägt).
    pub room: String,
    /// Raum auf der anderen Seite oder „außen“.
    pub other: String,
    pub unit: String,
    pub width: f64,
    pub height: f64,
    pub sill: f64,
    /// Lichte Durchgangsbreite bei Türen und Durchgängen.
    pub clear_width: Option<f64>,
    pub handing: Option<Handing>,
    pub wall: String,
}

/// Gleiche Öffnungen als Typ.
#[derive(Debug, Clone, PartialEq)]
pub struct OpeningType {
    pub mark: String,
    pub class: OpeningClass,
    pub width: f64,
    pub height: f64,
    pub sill: f64,
    pub count: usize,
    pub left: usize,
    pub right: usize,
}

fn mm(v: f64) -> i64 {
    (v * 1000.0).round() as i64
}

/// Tür- und Fensterliste: alle Öffnungen nach Geschoss und Position, dazu die Typen.
pub fn opening_schedule(project: &Project) -> (Vec<OpeningRow>, Vec<OpeningType>) {
    let norm = AccessibleNorm::get();
    let mut rows = Vec::new();
    for (index, (storey, s)) in project.storeys().into_iter().enumerate() {
        let rooms = StoreyRooms::new(project, storey);
        let prefix = storey_prefix(&s.name, index, project);
        let mut items: Vec<OpeningRow> = Vec::new();
        for p in &rooms.passages {
            let o = p.opening;
            let class = match o.kind {
                OpeningKind::Door(_) => OpeningClass::Door,
                OpeningKind::Plain => OpeningClass::Passage,
                OpeningKind::Window if p.sill < 0.05 => OpeningClass::FrenchWindow,
                OpeningKind::Window => OpeningClass::Window,
            };
            let label = |side: Option<usize>, outside: bool| match side {
                Some(i) => rooms.rooms[i].label(),
                None if outside => "außen".to_string(),
                None => String::new(),
            };
            let (room, other) = match p.swing_room() {
                Some(i) if Some(i) == p.left => (label(p.left, p.left_outside), label(p.right, p.right_outside)),
                Some(_) => (label(p.right, p.right_outside), label(p.left, p.left_outside)),
                None if p.left.is_none() => (label(p.right, p.right_outside), label(p.left, p.left_outside)),
                None => (label(p.left, p.left_outside), label(p.right, p.right_outside)),
            };
            let unit = [p.swing_room(), p.left, p.right]
                .into_iter()
                .flatten()
                .map(|i| rooms.rooms[i].space.unit.clone())
                .find(|u| !u.is_empty())
                .unwrap_or_default();
            items.push(OpeningRow {
                id: p.id,
                mark: String::new(),
                type_mark: String::new(),
                storey: s.name.clone(),
                class,
                room,
                other,
                unit,
                width: o.width,
                height: o.height,
                sill: p.sill,
                clear_width: matches!(class, OpeningClass::Door | OpeningClass::Passage)
                    .then(|| norm.clear_door(o.width)),
                handing: match o.kind {
                    OpeningKind::Door(d) => Some(d.hinge),
                    _ => None,
                },
                wall: project
                    .wall(p.wall)
                    .and_then(|w| project.wall_type(w.wall_type))
                    .map_or(String::new(), |t| t.name.clone()),
            });
        }
        // Positionen je Geschoss: Türen, dann Fenster, nach Raumnummer.
        items.sort_by(|a, b| a.class.prefix().cmp(b.class.prefix()).then(a.room.cmp(&b.room)).then(a.id.cmp(&b.id)));
        let mut counter: BTreeCounter = BTreeCounter::default();
        for item in &mut items {
            let n = counter.next(item.class.prefix());
            item.mark = format!("{} {prefix}.{n:02}", item.class.prefix());
        }
        rows.extend(items);
    }
    // Typen: gleiche Art und Rohbaumaße (bei Fenstern auch Brüstung), nach Größe.
    let mut types: Vec<OpeningType> = Vec::new();
    let key = |r: &OpeningRow| {
        let sill = if matches!(r.class, OpeningClass::Window) { mm(r.sill) } else { 0 };
        (r.class.prefix(), r.class, mm(r.width), mm(r.height), sill)
    };
    let mut keys: Vec<_> = rows.iter().map(key).collect();
    keys.sort();
    keys.dedup();
    let mut counter = BTreeCounter::default();
    for k in keys {
        let n = counter.next(k.0);
        let mark = format!("{}{n}", k.0);
        let mut t = OpeningType {
            mark: mark.clone(),
            class: k.1,
            width: k.2 as f64 / 1000.0,
            height: k.3 as f64 / 1000.0,
            sill: k.4 as f64 / 1000.0,
            count: 0,
            left: 0,
            right: 0,
        };
        for r in rows.iter_mut().filter(|r| key(r) == k) {
            r.type_mark = mark.clone();
            t.count += 1;
            match r.handing {
                Some(Handing::DinLeft) => t.left += 1,
                Some(Handing::DinRight) => t.right += 1,
                None => {}
            }
        }
        types.push(t);
    }
    (rows, types)
}

/// Kennziffer des Geschosses in der Position: wie die Raumnummern („0“, „1“, „-1“).
fn storey_prefix(name: &str, index: usize, project: &Project) -> String {
    // Aus der ersten Raumnummer des Geschosses, sonst die Reihenfolge ab 0.
    let storeys = project.storeys();
    let id = storeys[index].0;
    project
        .spaces_on(id)
        .filter_map(|(_, s)| s.number.split_once('.').map(|(p, _)| p.to_string()))
        .min()
        .unwrap_or_else(|| {
            let _ = name;
            index.to_string()
        })
}

#[derive(Default)]
struct BTreeCounter(std::collections::BTreeMap<&'static str, usize>);

impl BTreeCounter {
    fn next(&mut self, key: &'static str) -> usize {
        let n = self.0.entry(key).or_insert(0);
        *n += 1;
        *n
    }
}
