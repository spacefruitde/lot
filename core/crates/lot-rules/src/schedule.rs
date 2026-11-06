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
    // Kennzeichen aus den Tür- und Fenstertypen des Projekts (ADR 0048).
    let mut chosen: std::collections::BTreeMap<lot_model::ElementId, String> = Default::default();
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
            if let Some(mark) = o.opening_type.and_then(|t| project.opening_type(t)).and_then(|t| t.props.mark.clone())
            {
                chosen.insert(p.id, mark);
            }
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
    // Typen: Öffnungen mit einem Tür- oder Fenstertyp mit Kennzeichen gehören
    // zu diesem; die übrigen nach gleicher Art und Rohbaumaß (bei Fenstern auch
    // Brüstung), nach Größe, mit Kennzeichen, die die Typen frei lassen.
    let size_key = |r: &OpeningRow| {
        let sill = if matches!(r.class, OpeningClass::Window) { mm(r.sill) } else { 0 };
        (r.class.prefix(), r.class, mm(r.width), mm(r.height), sill)
    };
    // Gruppen in der Reihenfolge der Liste: erst die Typen des Projekts nach Kennzeichen, dann nach Größe.
    let mut groups: Vec<(Option<String>, SizeKey)> = rows
        .iter()
        .map(|r| match chosen.get(&r.id) {
            Some(m) => (Some(m.clone()), (r.class.prefix(), r.class, 0, 0, 0)),
            None => (None, size_key(r)),
        })
        .collect();
    groups.sort_by(|a, b| (a.0.is_none(), &a.0, a.1).cmp(&(b.0.is_none(), &b.0, b.1)));
    groups.dedup();
    let taken: Vec<String> = chosen.values().cloned().collect();
    let mut counter = BTreeCounter::default();
    let mut types: Vec<OpeningType> = Vec::new();
    for (typed, k) in groups {
        let mark = match &typed {
            Some(m) => m.clone(),
            None => loop {
                let candidate = format!("{}{}", k.0, counter.next(k.0));
                if !taken.contains(&candidate) {
                    break candidate;
                }
            },
        };
        let members: Vec<usize> = (0..rows.len())
            .filter(|i| match &typed {
                Some(m) => chosen.get(&rows[*i].id) == Some(m),
                None => !chosen.contains_key(&rows[*i].id) && size_key(&rows[*i]) == k,
            })
            .collect();
        let Some(&first) = members.first() else { continue };
        let first = &rows[first];
        let mut t = OpeningType {
            mark: mark.clone(),
            class: first.class,
            width: first.width,
            height: first.height,
            sill: if matches!(first.class, OpeningClass::Window) { first.sill } else { 0.0 },
            count: 0,
            left: 0,
            right: 0,
        };
        for i in members {
            let r = &mut rows[i];
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

/// Art und Rohbaumaß einer Öffnung in mm, bei Fenstern mit Brüstung: (Kürzel, Art, Breite, Höhe, Brüstung).
type SizeKey = (&'static str, OpeningClass, i64, i64, i64);

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

#[cfg(test)]
mod tests {
    use lot_model::{Element, Opening, OpeningKind, OpeningType, Point2, Project, Top, Wall, WallAlignment};

    use super::*;

    #[test]
    fn typed_openings_keep_the_mark_of_their_type() {
        let mut p = Project::template("Test");
        let storey = p.storeys()[0].0;
        let wall_type = p.wall_types().next().unwrap().0;
        let wall = p.next_id();
        p.elements.insert(
            wall,
            Element::Wall(Wall {
                storey,
                wall_type,
                start: Point2::new(0.0, 0.0),
                end: Point2::new(9.0, 0.0),
                alignment: WallAlignment::Center,
                base_offset: 0.0,
                top: Top::Storey { offset: 0.0 },
                sagitta: 0.0,
                phase: Default::default(),
                joins: Default::default(),
                props: Default::default(),
            }),
        );
        let ty = p.next_id();
        p.elements.insert(
            ty,
            Element::OpeningType(OpeningType {
                name: "Fenster groß".into(),
                sill_connection: 0.0,
                width: None,
                height: None,
                props: lot_model::Properties { mark: Some("F1".into()), ..Default::default() },
                locked: Vec::new(),
            }),
        );
        for (offset, typed) in [(1.0, true), (4.0, false), (7.0, false)] {
            let id = p.next_id();
            p.elements.insert(
                id,
                Element::Opening(Opening {
                    host: wall,
                    offset,
                    width: 1.26,
                    height: 1.385,
                    sill_height: 0.9,
                    kind: OpeningKind::Window,
                    phase: Default::default(),
                    opening_type: typed.then_some(ty),
                    recess: None,
                    props: Default::default(),
                    own: Vec::new(),
                }),
            );
        }
        let (rows, types) = opening_schedule(&p);
        assert_eq!(rows.len(), 3);
        let marks: Vec<&str> = rows.iter().map(|r| r.type_mark.as_str()).collect();
        // Der Typ des Projekts behält F1, die übrigen bekommen das nächste freie Kennzeichen.
        assert_eq!(marks.iter().filter(|m| **m == "F1").count(), 1);
        assert_eq!(marks.iter().filter(|m| **m == "F2").count(), 2);
        assert_eq!(types.iter().map(|t| (t.mark.as_str(), t.count)).collect::<Vec<_>>(), vec![("F1", 1), ("F2", 2)]);
    }
}
