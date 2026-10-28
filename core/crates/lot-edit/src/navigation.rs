//! Navigation (ADR 0049): was die App zeigen kann, die Projekt-Seitenleiste
//! mit Suche, Geschosse nach oben und unten, Heimat einer Ansicht auf den
//! Planblättern und der Verlauf für Zurück und Vorwärts.

use lot_model::{Element, ElementId, Project, SectionKind, ViewContent};
use serde::{Deserialize, Serialize};

/// Eine Ansicht der App. Die Kennung ist ihre RON-Schreibweise, etwa `FloorPlan(3)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ViewRef {
    /// Grundriss eines Geschosses.
    FloorPlan(u64),
    /// 3D-Modell.
    Model,
    /// Schnitt oder Ansicht (`SectionView`).
    Section(u64),
    SitePlan,
    /// Planblatt (`PlanSheet`).
    Sheet(u64),
    Schedule(Schedule),
}

/// Listen in der Seitenleiste.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Schedule {
    /// Flächenaufstellung nach DIN 277 und WoFlV.
    Areas,
    /// Tür- und Fensterliste.
    Openings,
    /// Prüfbericht.
    Checks,
}

impl ViewRef {
    pub fn id(&self) -> String {
        ron::to_string(self).unwrap_or_default()
    }

    pub fn parse(id: &str) -> Option<ViewRef> {
        ron::from_str(id).ok()
    }

    /// Ob es die Ansicht im Projekt (noch) gibt.
    pub fn exists(&self, project: &Project) -> bool {
        match *self {
            ViewRef::FloorPlan(id) => project.storey(ElementId(id)).is_some(),
            ViewRef::Section(id) => section(project, id).is_some(),
            ViewRef::Sheet(id) => matches!(project.elements.get(&ElementId(id)), Some(Element::PlanSheet(_))),
            ViewRef::Model | ViewRef::SitePlan | ViewRef::Schedule(_) => true,
        }
    }

    /// Titel, etwa „Grundriss EG“, „Schnitt A-A“ oder „Plan 201 Schnitte“.
    pub fn title(&self, project: &Project) -> String {
        match *self {
            ViewRef::FloorPlan(id) => {
                format!("Grundriss {}", project.storey(ElementId(id)).map_or("?", |s| s.name.as_str()))
            }
            ViewRef::Model => "3D-Modell".into(),
            ViewRef::Section(id) => section(project, id).map_or("Ansicht fehlt".into(), |v| v.name.clone()),
            ViewRef::SitePlan => "Lageplan".into(),
            ViewRef::Sheet(id) => match project.elements.get(&ElementId(id)) {
                Some(Element::PlanSheet(s)) => format!("Plan {} {}", s.number, s.title),
                _ => "Blatt fehlt".into(),
            },
            ViewRef::Schedule(Schedule::Areas) => "Flächen".into(),
            ViewRef::Schedule(Schedule::Openings) => "Türen und Fenster".into(),
            ViewRef::Schedule(Schedule::Checks) => "Prüfbericht".into(),
        }
    }

    /// Was auf einem Planblatt für diese Ansicht steht; Listen und 3D lassen sich nicht platzieren.
    pub fn content(&self) -> Option<ViewContent> {
        match *self {
            ViewRef::FloorPlan(id) => Some(ViewContent::FloorPlan { storey: ElementId(id) }),
            ViewRef::Section(id) => Some(ViewContent::Section { view: ElementId(id) }),
            ViewRef::SitePlan => Some(ViewContent::SitePlan),
            _ => None,
        }
    }

    /// Die Ansicht, die ein Inhalt eines Planblatts zeigt.
    pub fn of_content(content: ViewContent) -> ViewRef {
        match content {
            ViewContent::FloorPlan { storey } | ViewContent::Presentation { storey } => ViewRef::FloorPlan(storey.0),
            ViewContent::Section { view } => ViewRef::Section(view.0),
            ViewContent::SitePlan => ViewRef::SitePlan,
        }
    }
}

fn section(project: &Project, id: u64) -> Option<&lot_model::SectionView> {
    match project.elements.get(&ElementId(id)) {
        Some(Element::SectionView(v)) => Some(v),
        _ => None,
    }
}

/// Wohin ein Sprung führt: eine Ansicht oder ein Element im Grundriss seines Geschosses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Target {
    View(ViewRef),
    /// Bisher Räume: Grundriss ihres Geschosses, mittig auf dem Raumstempel.
    Element(u64),
}

impl Target {
    pub fn id(&self) -> String {
        ron::to_string(self).unwrap_or_default()
    }

    pub fn parse(id: &str) -> Option<Target> {
        ron::from_str(id).ok()
    }
}

/// Eine Zeile der Seitenleiste.
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub view: ViewRef,
    pub title: String,
    /// Zweite Zeile, etwa „OKFF +2,75 · Höhe 2,75 m“ oder „auf Plan 201, 301“.
    pub detail: String,
    /// Bei Ansichten auf einem Planblatt: Blatt und Stelle in seiner Liste.
    pub placement: Option<(ElementId, usize)>,
    /// Diese Platzierung ist die Heimat der Ansicht (Schnittmarken verweisen hierher).
    pub home: bool,
    /// Ansichten auf einem Planblatt.
    pub children: Vec<Entry>,
}

impl Entry {
    fn new(view: ViewRef, title: String, detail: String) -> Self {
        Self { view, title, detail, placement: None, home: false, children: Vec::new() }
    }
}

/// Eine Gruppe der Seitenleiste, etwa „Schnitte und Ansichten“.
#[derive(Debug, Clone, PartialEq)]
pub struct Group {
    pub title: String,
    pub entries: Vec<Entry>,
}

/// Höhe im Plan, „±0,00“, „+2,75“, „-2,50“.
pub fn format_level(z: f64) -> String {
    let v = (z * 100.0).round() / 100.0;
    let text = format!("{:.2}", v.abs()).replace('.', ",");
    if v == 0.0 {
        format!("±{text}")
    } else if v > 0.0 {
        format!("+{text}")
    } else {
        format!("-{text}")
    }
}

/// Die Seitenleiste „Projekt“: Geschosse von oben nach unten, Grundrisse,
/// Schnitte und Ansichten, Lageplan, Planblätter mit ihren Ansichten, Listen.
pub fn tree(project: &Project) -> Vec<Group> {
    let storeys: Vec<_> = project.storeys().into_iter().rev().collect();
    let storey_entries = storeys
        .iter()
        .map(|(id, s)| {
            let detail = format!("OKFF {} · Höhe {}", format_level(s.elevation), crate::format_m(s.height));
            Entry::new(ViewRef::FloorPlan(id.0), s.name.clone(), detail)
        })
        .collect();
    let mut plans: Vec<Entry> = storeys
        .iter()
        .map(|(id, _)| {
            let view = ViewRef::FloorPlan(id.0);
            Entry::new(view, view.title(project), sheets_text(project, view))
        })
        .collect();
    plans.push(Entry::new(ViewRef::Model, "3D-Modell".into(), String::new()));
    let mut sections: Vec<_> = project.section_views().collect();
    sections.sort_by(|a, b| {
        (a.1.kind == SectionKind::Elevation, &a.1.name).cmp(&(b.1.kind == SectionKind::Elevation, &b.1.name))
    });
    let sections = sections
        .into_iter()
        .map(|(id, v)| {
            let view = ViewRef::Section(id.0);
            Entry::new(view, v.name.clone(), sheets_text(project, view))
        })
        .collect();
    let site = vec![Entry::new(ViewRef::SitePlan, "Lageplan".into(), sheets_text(project, ViewRef::SitePlan))];
    let sheets = project
        .plan_sheets()
        .into_iter()
        .map(|(id, s)| {
            let children = s
                .views
                .iter()
                .enumerate()
                .map(|(i, placed)| {
                    let view = ViewRef::of_content(placed.content);
                    let mut title = view.title(project);
                    if let ViewContent::Presentation { .. } = placed.content {
                        title = format!("{title} (Präsentation)");
                    }
                    let is_home = home(project, view) == Some((id, i));
                    let detail = if is_home && placements(project, view).len() > 1 {
                        format!("1:{} · Heimat", placed.scale)
                    } else {
                        format!("1:{}", placed.scale)
                    };
                    Entry { placement: Some((id, i)), home: is_home, ..Entry::new(view, title, detail) }
                })
                .collect();
            let count = s.views.len();
            let detail = match count {
                1 => format!("{}, 1 Ansicht", s.format),
                n => format!("{}, {n} Ansichten", s.format),
            };
            Entry { children, ..Entry::new(ViewRef::Sheet(id.0), format!("{} {}", s.number, s.title), detail) }
        })
        .collect();
    let lists = [Schedule::Areas, Schedule::Openings, Schedule::Checks]
        .into_iter()
        .map(|s| {
            let view = ViewRef::Schedule(s);
            Entry::new(view, view.title(project), String::new())
        })
        .collect();
    [
        ("Geschosse", storey_entries),
        ("Grundrisse", plans),
        ("Schnitte und Ansichten", sections),
        ("Lageplan", site),
        ("Planblätter", sheets),
        ("Listen", lists),
    ]
    .into_iter()
    .map(|(title, entries)| Group { title: title.into(), entries })
    .collect()
}

/// „auf Plan 201, 301“ oder leer, wenn die Ansicht auf keinem Blatt liegt.
fn sheets_text(project: &Project, view: ViewRef) -> String {
    let mut numbers: Vec<String> = Vec::new();
    for (sheet, _) in placements(project, view) {
        if let Some(Element::PlanSheet(s)) = project.elements.get(&sheet)
            && !numbers.contains(&s.number)
        {
            numbers.push(s.number.clone());
        }
    }
    if numbers.is_empty() { String::new() } else { format!("auf Plan {}", numbers.join(", ")) }
}

/// Wo die Ansicht auf Planblättern liegt: Blatt und Stelle, nach Plannummer.
pub fn placements(project: &Project, view: ViewRef) -> Vec<(ElementId, usize)> {
    project
        .plan_sheets()
        .into_iter()
        .flat_map(|(id, s)| {
            s.views
                .iter()
                .enumerate()
                .filter(move |(_, p)| ViewRef::of_content(p.content) == view)
                .map(move |(i, _)| (id, i))
        })
        .collect()
}

/// Heimat einer Ansicht (ADR 0049): bei Schnitten das gewählte Blatt, sonst
/// und ohne Wahl die erste Platzierung nach Plannummer.
pub fn home(project: &Project, view: ViewRef) -> Option<(ElementId, usize)> {
    let all = placements(project, view);
    let chosen = match view {
        ViewRef::Section(id) => section(project, id).and_then(|v| v.home),
        _ => None,
    };
    chosen.and_then(|sheet| all.iter().copied().find(|(s, _)| *s == sheet)).or_else(|| all.first().copied())
}

/// Das Geschoss über (`up`) oder unter dem gegebenen.
pub fn storey_step(project: &Project, storey: ElementId, up: bool) -> Option<ElementId> {
    let storeys = project.storeys();
    let i = storeys.iter().position(|(id, _)| *id == storey)?;
    let j = if up { i.checked_add(1)? } else { i.checked_sub(1)? };
    storeys.get(j).map(|(id, _)| *id)
}

/// Ein Treffer der Suche oder der Befehlspalette.
#[derive(Debug, Clone, PartialEq)]
pub struct Hit {
    pub target: Target,
    pub title: String,
    pub detail: String,
}

/// Ansichten und Räume, deren Titel oder Zusatz alle Wörter der Suche enthält,
/// etwa „bad 1.og“. Ansichten zuerst, dann Räume; leere Suche findet alle Ansichten.
pub fn search(project: &Project, query: &str) -> Vec<Hit> {
    let words: Vec<String> = query.split_whitespace().map(str::to_lowercase).collect();
    let matches = |title: &str, detail: &str| {
        let text = format!("{title} {detail}").to_lowercase();
        words.iter().all(|w| text.contains(w.as_str()))
    };
    let mut out = Vec::new();
    for group in tree(project) {
        if group.title == "Geschosse" {
            continue;
        }
        for e in group.entries {
            let title = match e.view {
                ViewRef::Sheet(_) => format!("Plan {}", e.title),
                _ => e.title.clone(),
            };
            if matches(&title, &e.detail) {
                out.push(Hit { target: Target::View(e.view), title, detail: e.detail });
            }
        }
    }
    if words.is_empty() {
        return out;
    }
    for (storey_id, storey) in project.storeys() {
        let mut spaces: Vec<_> = project.spaces_on(storey_id).collect();
        spaces.sort_by(|a, b| a.1.number.cmp(&b.1.number).then(a.1.name.cmp(&b.1.name)));
        for (id, space) in spaces {
            let title =
                if space.number.is_empty() { space.name.clone() } else { format!("{} {}", space.number, space.name) };
            let detail = format!("Raum, {}", storey.name);
            if matches(&title, &detail) {
                out.push(Hit { target: Target::Element(id.0), title, detail });
            }
        }
    }
    out
}

/// Ein Ort in der App: Ansicht und Kamera (Mitte in Blattmillimetern und
/// Zoom in Pixeln je Millimeter), damit Zurück an dieselbe Stelle führt.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Place {
    pub view: ViewRef,
    pub camera: Option<[f64; 3]>,
}

/// Verlauf für Zurück (⌘[) und Vorwärts (⌘]) wie in Safari und Xcode.
#[derive(Debug, Clone, Default)]
pub struct History {
    back: Vec<Place>,
    forward: Vec<Place>,
}

/// So viele Orte merkt sich der Verlauf.
const HISTORY_LIMIT: usize = 100;

impl History {
    /// Vor einem Wechsel in eine andere Ansicht: der bisherige Ort kommt auf den
    /// Zurück-Stapel, Vorwärts verfällt.
    pub fn leave(&mut self, from: Place, to: ViewRef) {
        if from.view == to {
            return;
        }
        self.back.push(from);
        if self.back.len() > HISTORY_LIMIT {
            self.back.remove(0);
        }
        self.forward.clear();
    }

    /// Ein Schritt zurück; `current` kommt auf den Vorwärts-Stapel.
    pub fn back(&mut self, current: Place) -> Option<Place> {
        let place = self.back.pop()?;
        self.forward.push(current);
        Some(place)
    }

    pub fn forward(&mut self, current: Place) -> Option<Place> {
        let place = self.forward.pop()?;
        self.back.push(current);
        Some(place)
    }

    pub fn can_go_back(&self) -> bool {
        !self.back.is_empty()
    }

    pub fn can_go_forward(&self) -> bool {
        !self.forward.is_empty()
    }

    /// Vergisst Orte, deren Ansicht es nicht mehr gibt, etwa nach Löschen oder Widerrufen.
    pub fn retain(&mut self, project: &Project) {
        self.back.retain(|p| p.view.exists(project));
        self.forward.retain(|p| p.view.exists(project));
        self.back.dedup_by(|a, b| a.view == b.view);
        self.forward.dedup_by(|a, b| a.view == b.view);
    }
}

/// Ein neuer Schnitt oder eine neue Ansicht von `start` nach `end`, mit Blick
/// nach `look`. Schnitte bekommen den nächsten freien Kennbuchstaben, Ansichten
/// den Namen nach der Himmelsrichtung der Fassade, die sie zeigen. Die
/// Blicktiefe reicht bis hinter die letzte Wand.
pub fn new_section(
    project: &Project,
    start: lot_model::Point2,
    end: lot_model::Point2,
    look: lot_model::WallSide,
    kind: SectionKind,
) -> lot_model::SectionView {
    let mut view = lot_model::SectionView {
        name: String::new(),
        mark: String::new(),
        start,
        end,
        look,
        depth: 10.0,
        kind,
        home: None,
    };
    let (n, _) = view.frame();
    let reach = project
        .elements
        .values()
        .filter_map(|e| match e {
            Element::Wall(w) => Some([w.start, w.end]),
            _ => None,
        })
        .flatten()
        .map(|p| (p.x - start.x) * n.0 + (p.y - start.y) * n.1)
        .fold(f64::NEG_INFINITY, f64::max);
    if reach.is_finite() {
        view.depth = ((reach + 1.0).max(3.0) * 10.0).ceil() / 10.0;
    }
    let names: Vec<String> = project.section_views().map(|(_, v)| v.name.clone()).collect();
    let unique = |base: String| {
        (1..)
            .map(|i| if i == 1 { base.clone() } else { format!("{base} {i}") })
            .find(|n| !names.contains(n))
            .unwrap_or(base.clone())
    };
    match kind {
        SectionKind::Section => {
            let marks: Vec<String> = project.section_views().map(|(_, v)| v.mark.clone()).collect();
            view.mark = ('A'..='Z').map(String::from).find(|m| !marks.contains(m)).unwrap_or_else(|| "?".into());
            view.name = unique(format!("Schnitt {0}-{0}", view.mark));
        }
        SectionKind::Elevation => view.name = unique(format!("Ansicht {}", facade_direction(project, n))),
    }
    view
}

/// Himmelsrichtung der Fassade, auf die man mit Blick `look` schaut: Blick
/// nach Norden zeigt die Südfassade. Nord nach `ProjectInfo::north`.
pub fn facade_direction(project: &Project, look: (f64, f64)) -> &'static str {
    let a = project.info().north.to_radians();
    let north = (-a.sin(), a.cos());
    let east = (north.1, -north.0);
    let facing = (-look.0, -look.1);
    let bearing = (facing.0 * east.0 + facing.1 * east.1).atan2(facing.0 * north.0 + facing.1 * north.1).to_degrees();
    let names = ["Nord", "Nordost", "Ost", "Südost", "Süd", "Südwest", "West", "Nordwest"];
    names[((bearing.rem_euclid(360.0) + 22.5) / 45.0) as usize % 8]
}

/// Geschosshöhe ändern: Die Geschosse darüber rücken um die Differenz mit,
/// damit OKFF bis OKFF darüber stimmt.
pub fn storey_height_ops(project: &Project, storey: ElementId, height: f64) -> Vec<lot_model::Operation> {
    let Some(s) = project.storey(storey) else { return Vec::new() };
    let delta = height - s.height;
    if delta.abs() < 1e-9 {
        return Vec::new();
    }
    let mut ops = vec![lot_model::Operation::SetParam {
        id: storey,
        param: lot_model::Param::StoreyHeight,
        old: s.height,
        new: height,
    }];
    ops.extend(shift_above(project, storey, delta));
    ops
}

/// OKFF eines Geschosses ändern; die Geschosse darüber rücken mit.
pub fn storey_elevation_ops(project: &Project, storey: ElementId, elevation: f64) -> Vec<lot_model::Operation> {
    let Some(s) = project.storey(storey) else { return Vec::new() };
    let delta = elevation - s.elevation;
    if delta.abs() < 1e-9 {
        return Vec::new();
    }
    let mut ops = vec![lot_model::Operation::SetParam {
        id: storey,
        param: lot_model::Param::StoreyElevation,
        old: s.elevation,
        new: elevation,
    }];
    ops.extend(shift_above(project, storey, delta));
    ops
}

fn shift_above(project: &Project, storey: ElementId, delta: f64) -> Vec<lot_model::Operation> {
    let storeys = project.storeys();
    let Some(i) = storeys.iter().position(|(id, _)| *id == storey) else { return Vec::new() };
    storeys[i + 1..]
        .iter()
        .map(|(id, s)| lot_model::Operation::SetParam {
            id: *id,
            param: lot_model::Param::StoreyElevation,
            old: s.elevation,
            new: s.elevation + delta,
        })
        .collect()
}
