//! Auswählen (ADR 0043): Kandidaten unter dem Zeiger zum Durchblättern mit
//! Tab, Wandketten, Rahmen und Lasso, gleiche Bauteile und Bezeichnungen.

use std::collections::BTreeMap;

use lot_model::{AnnotationShape, AnnotationView, Element, ElementId, OpeningKind, Point2, Project};

use crate::format_m;
use crate::geometry::{self, JOIN_TOLERANCE};

/// Was ein Klick an dieser Stelle wählen würde: ein Bauteil oder eine Wandkette.
#[derive(Debug, Clone, PartialEq)]
pub struct Candidate {
    pub ids: Vec<ElementId>,
    pub label: String,
}

/// Alle Kandidaten unter dem Zeiger, in der Reihenfolge von
/// [`geometry::elements_at`]. Nach der nächsten Wand folgt ihre Wandkette,
/// wenn sie mit anderen Wänden Ende an Ende verbunden ist (wie Tab in Revit).
pub fn candidates_at(project: &Project, storey: ElementId, p: Point2, tolerance: f64) -> Vec<Candidate> {
    let mut out = Vec::new();
    let mut chained = false;
    for id in geometry::elements_at(project, storey, p, tolerance) {
        out.push(Candidate { ids: vec![id], label: label(project, id) });
        if !chained && project.wall(id).is_some() {
            chained = true;
            let chain = wall_chain(project, id);
            if chain.len() > 1 {
                out.push(Candidate { label: format!("Wandkette ({} Wände)", chain.len()), ids: chain });
            }
        }
    }
    out
}

/// Die Wände, die mit `start` über Ecken und Fortsetzungen Ende an Ende
/// verbunden sind, im selben Geschoss und Bauzustand (ADR 0045: verschiedene
/// Bauzustände verbinden sich nicht). `start` steht vorne.
pub fn wall_chain(project: &Project, start: ElementId) -> Vec<ElementId> {
    let Some(first) = project.wall(start) else { return Vec::new() };
    let walls: Vec<(ElementId, [Point2; 2])> = project
        .walls_on(first.storey)
        .filter(|(_, w)| w.phase == first.phase)
        .map(|(id, w)| (id, [w.start, w.end]))
        .collect();
    let mut chain = vec![start];
    let mut i = 0;
    while i < chain.len() {
        let ends = walls.iter().find(|(id, _)| *id == chain[i]).map(|(_, e)| *e).unwrap_or([first.start, first.end]);
        for (id, other) in &walls {
            if !chain.contains(id) && other.iter().any(|q| ends.iter().any(|e| q.distance(*e) <= JOIN_TOLERANCE)) {
                chain.push(*id);
            }
        }
        i += 1;
    }
    chain
}

/// Bereich eines Rahmens oder Lassos in Modellkoordinaten.
#[derive(Debug, Clone, PartialEq)]
pub enum Region {
    Rect(Point2, Point2),
    Lasso(Vec<Point2>),
}

impl Region {
    pub fn ring(&self) -> Vec<Point2> {
        match self {
            Region::Rect(a, b) => vec![*a, Point2::new(b.x, a.y), *b, Point2::new(a.x, b.y)],
            Region::Lasso(points) => points.clone(),
        }
    }

    fn contains(&self, p: Point2) -> bool {
        match self {
            Region::Rect(a, b) => {
                p.x >= a.x.min(b.x) && p.x <= a.x.max(b.x) && p.y >= a.y.min(b.y) && p.y <= a.y.max(b.y)
            }
            Region::Lasso(ring) => point_in_ring(ring, p),
        }
    }
}

/// Die Elemente des Geschosses, die ein Rahmen oder Lasso wählt: ganz innen
/// liegende, mit `crossing` auch angeschnittene. Flächen wie Decken, Dächer
/// und Räume zählen nur über ihren Rand, damit ein Rahmen in einem Raum nicht
/// die Decke mitnimmt; Räume über ihren Stempel.
pub fn in_region(project: &Project, storey: ElementId, region: &Region, crossing: bool) -> Vec<ElementId> {
    let frame = region.ring();
    pickable(project, storey)
        .into_iter()
        .filter(|id| {
            let ring: Vec<Point2> = match project.elements.get(id) {
                Some(Element::Space(r)) => geometry::space_stamp(r).to_vec(),
                _ => match geometry::outline(project, *id) {
                    Some(o) => o.iter().map(|p| Point2::new(p[0], p[1])).collect(),
                    None => return false,
                },
            };
            if ring.is_empty() {
                return false;
            }
            if !crossing {
                return ring.iter().all(|p| region.contains(*p));
            }
            ring.iter().any(|p| region.contains(*p)) || rings_cross(&ring, &frame)
        })
        .collect()
}

/// Alles, was im Grundriss des Geschosses wählbar ist. Unterlagen fehlen (ADR 0043).
pub fn pickable(project: &Project, storey: ElementId) -> Vec<ElementId> {
    let mut out: Vec<ElementId> = Vec::new();
    for (id, _) in project.walls_on(storey) {
        out.push(id);
        out.extend(project.openings_in(id).map(|(o, _)| o));
    }
    out.extend(project.columns_on(storey).map(|(id, _)| id));
    out.extend(project.spaces_on(storey).map(|(id, _)| id));
    for (id, _) in project.slabs_on(storey) {
        out.push(id);
        out.extend(project.slab_openings_in(id).map(|(o, _)| o));
    }
    out.extend(project.roofs_on(storey).map(|(id, _)| id));
    out.extend(project.dimensions_on(storey).map(|(id, _)| id));
    out.extend(project.annotations_in(AnnotationView::FloorPlan { storey }).map(|(id, _)| id));
    out.extend(project.grid_lines().map(|(id, _)| id));
    out
}

/// Gleiche Bauteile wie die Auswahl (SA wie in Revit): gleiche Art und
/// gleicher Typ, erst im Geschoss; ist das schon die Auswahl, im ganzen Projekt.
pub fn similar(project: &Project, storey: ElementId, selection: &[ElementId]) -> Vec<ElementId> {
    let keys: Vec<String> = selection.iter().filter_map(|id| similarity_key(project, *id)).collect();
    let matching = |ids: Vec<ElementId>| -> Vec<ElementId> {
        ids.into_iter().filter(|id| similarity_key(project, *id).is_some_and(|k| keys.contains(&k))).collect()
    };
    let on_storey = matching(pickable(project, storey));
    let mut sorted = selection.to_vec();
    sorted.sort();
    let mut found = on_storey.clone();
    found.sort();
    if found != sorted {
        return on_storey;
    }
    let mut all = Vec::new();
    for (id, _) in project.storeys() {
        for e in matching(pickable(project, id)) {
            if !all.contains(&e) {
                all.push(e);
            }
        }
    }
    all
}

/// Art und Typ eines Bauteils, für „gleiche auswählen“.
fn similarity_key(project: &Project, id: ElementId) -> Option<String> {
    Some(match project.elements.get(&id)? {
        Element::Wall(w) => format!("Wand {}", w.wall_type.0),
        Element::Opening(o) => format!("{} {:.3} {:.3}", opening_kind(&o.kind), o.width, o.height),
        Element::Column(c) => format!("Stütze {:?}", c.profile),
        Element::Space(_) => "Raum".into(),
        Element::Slab(s) => format!("Decke {}", s.slab_type.0),
        Element::SlabOpening(_) => "Deckendurchbruch".into(),
        Element::Roof(r) => format!("Dach {}", r.roof_type.0),
        Element::DimensionChain(_) => "Maßkette".into(),
        Element::Annotation(a) => annotation_kind(&a.shape).into(),
        Element::GridLine(_) => "Achse".into(),
        _ => return None,
    })
}

fn opening_kind(kind: &OpeningKind) -> &'static str {
    match kind {
        OpeningKind::Plain => "Öffnung",
        OpeningKind::Door(_) => "Tür",
        OpeningKind::Window => "Fenster",
    }
}

fn annotation_kind(shape: &AnnotationShape) -> &'static str {
    match shape {
        AnnotationShape::Text { .. } => "Text",
        AnnotationShape::Polyline(_) => "Linie",
        AnnotationShape::Polygon { .. } => "Fläche",
        AnnotationShape::Circle { .. } => "Kreis",
    }
}

/// Art eines Elements für Gruppen im Inspektor, etwa „Wand · KS 17,5“ oder „Tür“.
pub fn kind_label(project: &Project, id: ElementId) -> String {
    let p = project;
    match p.elements.get(&id) {
        Some(Element::Wall(w)) => format!("Wand · {}", p.wall_type(w.wall_type).map_or("?", |t| t.name.as_str())),
        Some(Element::Opening(o)) => opening_kind(&o.kind).into(),
        Some(Element::Storey(s)) => format!("Geschoss {}", s.name),
        Some(Element::Column(_)) => "Stütze".into(),
        Some(Element::DimensionChain(_)) => "Maßkette".into(),
        Some(Element::Slab(s)) => match s.kind {
            lot_model::SlabKind::Floor => "Decke".into(),
            lot_model::SlabKind::Base => "Bodenplatte".into(),
        },
        Some(Element::SlabType(t)) => format!("Deckenaufbau {}", t.name),
        Some(Element::OpeningType(t)) => format!("Tür- oder Fenstertyp {}", t.name),
        Some(Element::SlabOpening(_)) => "Deckendurchbruch".into(),
        Some(Element::Roof(_)) => "Dach".into(),
        Some(Element::RoofWindow(_)) => "Dachfenster".into(),
        Some(Element::Dormer(_)) => "Gaube".into(),
        Some(Element::Stair(_)) => "Treppe".into(),
        Some(Element::Railing(_)) => "Geländer".into(),
        Some(Element::SectionView(v)) => v.name.clone(),
        Some(Element::Underlay(u)) => format!("Unterlage · {}", u.source),
        Some(Element::Terrain(_)) => "Gelände".into(),
        Some(Element::ReferenceModel(r)) => format!("Referenzmodell · {}", r.source),
        Some(Element::VariantSet(v)) => format!("Varianten {}", v.name),
        Some(Element::Annotation(a)) => annotation_kind(&a.shape).into(),
        Some(Element::ProjectInfo(_)) => "Projektangaben".into(),
        Some(Element::Plot(p)) => {
            if p.neighbour {
                "Nachbargrundstück".into()
            } else {
                "Grundstück".into()
            }
        }
        Some(Element::Elevator(_)) => "Aufzug".into(),
        Some(Element::PlanSheet(_)) => "Planblatt".into(),
        Some(Element::ViewSettings(_)) => "Ansichtseinstellungen".into(),
        Some(Element::Space(_)) => "Raum".into(),
        Some(Element::GridLine(_)) => "Achse".into(),
        Some(Element::WallType(t)) => format!("Wandaufbau {}", t.name),
        None => String::new(),
    }
}

/// Bezeichnung eines Elements für Zeiger und Inspektor, etwa
/// „Wand · KS 17,5 · 4,20 m“, „Tür · 1,01 × 2,135“ oder „Raum 0.01 Wohnen“.
pub fn label(project: &Project, id: ElementId) -> String {
    let kind = kind_label(project, id);
    match project.elements.get(&id) {
        Some(Element::Wall(w)) => format!("{kind} · {}", format_m(w.length())),
        Some(Element::Opening(o)) => format!("{kind} · {} × {}", meters(o.width), meters(o.height)),
        Some(Element::Column(c)) => match c.profile {
            lot_model::ColumnProfile::Rectangle { width, depth } => {
                format!("{kind} {}/{}", (width * 100.0).round(), (depth * 100.0).round())
            }
            lot_model::ColumnProfile::Circle { diameter } => format!("{kind} Ø {}", (diameter * 100.0).round()),
        },
        Some(Element::Stair(s)) => format!("{kind} · {} Steigungen", s.risers()),
        Some(Element::Terrain(t)) => format!("{kind} · {} Höhenpunkte", t.points.len()),
        Some(Element::VariantSet(v)) => format!("{kind} · {}", v.active_name()),
        Some(Element::Annotation(a)) => match &a.shape {
            AnnotationShape::Text { text, .. } => format!("{kind} · {text}"),
            _ => kind,
        },
        Some(Element::Plot(p)) => format!("{kind} {}", p.name),
        Some(Element::Elevator(e)) => {
            format!("{kind} · Fahrkorb {:.2} x {:.2} m", e.cabin_width, e.cabin_depth).replace('.', ",")
        }
        Some(Element::PlanSheet(s)) => format!("{kind} {} {}", s.number, s.title),
        Some(Element::Space(r)) => format!("{kind} {} {}", r.number, r.name).trim_end().to_string(),
        Some(Element::GridLine(g)) => format!("{kind} {}", g.label),
        _ => kind,
    }
}

/// Die Auswahl nach Art und Typ gruppiert, für den Inspektor: je Gruppe die
/// Bezeichnung und die Kennungen, in der Reihenfolge des ersten Auftretens.
pub fn groups(project: &Project, ids: &[ElementId]) -> Vec<(String, Vec<ElementId>)> {
    let mut order: Vec<String> = Vec::new();
    let mut by: BTreeMap<String, Vec<ElementId>> = BTreeMap::new();
    for id in ids {
        let k = kind_label(project, *id);
        if !by.contains_key(&k) {
            order.push(k.clone());
        }
        by.entry(k).or_default().push(*id);
    }
    order.into_iter().map(|k| (k.clone(), by.remove(&k).unwrap_or_default())).collect()
}

/// Maß in m mit zwei Stellen, mit drei, wenn die dritte zählt (1,01 und 2,135).
pub fn meters(v: f64) -> String {
    let s = if (v * 1000.0).round() as i64 % 10 != 0 { format!("{v:.3}") } else { format!("{v:.2}") };
    s.replace('.', ",")
}

fn point_in_ring(ring: &[Point2], p: Point2) -> bool {
    let n = ring.len();
    let mut inside = false;
    for i in 0..n {
        let (a, b) = (ring[i], ring[(i + n - 1) % n]);
        if (a.y > p.y) != (b.y > p.y) && p.x < (b.x - a.x) * (p.y - a.y) / (b.y - a.y) + a.x {
            inside = !inside;
        }
    }
    inside
}

/// Ob sich zwei geschlossene Umrisse an ihren Kanten schneiden.
fn rings_cross(a: &[Point2], b: &[Point2]) -> bool {
    let edges =
        |r: &[Point2]| -> Vec<(Point2, Point2)> { (0..r.len()).map(|i| (r[i], r[(i + 1) % r.len()])).collect() };
    let eb = edges(b);
    edges(a).iter().any(|(p, q)| eb.iter().any(|(r, s)| segments_cross(*p, *q, *r, *s)))
}

fn segments_cross(p: Point2, q: Point2, r: Point2, s: Point2) -> bool {
    let cross = |o: Point2, a: Point2, b: Point2| (a.x - o.x) * (b.y - o.y) - (a.y - o.y) * (b.x - o.x);
    let (d1, d2, d3, d4) = (cross(r, s, p), cross(r, s, q), cross(p, q, r), cross(p, q, s));
    (d1 > 0.0) != (d2 > 0.0) && (d3 > 0.0) != (d4 > 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lasso_contains_points_inside_only() {
        let ring = vec![Point2::new(0.0, 0.0), Point2::new(4.0, 0.0), Point2::new(4.0, 4.0), Point2::new(0.0, 4.0)];
        assert!(point_in_ring(&ring, Point2::new(1.0, 1.0)));
        assert!(!point_in_ring(&ring, Point2::new(5.0, 1.0)));
    }

    #[test]
    fn crossing_edges_are_found() {
        let a = vec![Point2::new(0.0, 0.0), Point2::new(2.0, 0.0), Point2::new(2.0, 2.0), Point2::new(0.0, 2.0)];
        let b = vec![Point2::new(1.0, 1.0), Point2::new(3.0, 1.0), Point2::new(3.0, 3.0), Point2::new(1.0, 3.0)];
        let c = vec![Point2::new(5.0, 5.0), Point2::new(6.0, 5.0), Point2::new(6.0, 6.0)];
        assert!(rings_cross(&a, &b));
        assert!(!rings_cross(&a, &c));
    }
}
