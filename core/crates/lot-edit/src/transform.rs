//! Verschieben, Drehen und Spiegeln von Bauteilen, auch als Kopie (ADR 0044).
//!
//! Gespeichert werden weiter nur Parameter (CLAUDE.md, Regel 6): Eine
//! Abbildung ändert Punkte, Richtungen und Drehwinkel der Elemente. Beim
//! Spiegeln laufen Wände rückwärts, damit innen und außen auf ihrer
//! gespiegelten Seite bleiben.

use std::collections::HashMap;

use lot_model::{DimRef, Element, ElementId, Operation, Point2, Project, WallEnd};

use crate::geometry::JOIN_TOLERANCE;

/// Ebene Abbildung p ↦ M·p + t: Verschiebung, Drehung, Spiegelung.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    m: [[f64; 2]; 2],
    t: (f64, f64),
}

impl Transform {
    pub fn translation(dx: f64, dy: f64) -> Self {
        Self { m: [[1.0, 0.0], [0.0, 1.0]], t: (dx, dy) }
    }

    /// Drehung um `center`, gegen den Uhrzeigersinn in Grad.
    pub fn rotation(center: Point2, degrees: f64) -> Self {
        let (sin, cos) = degrees.to_radians().sin_cos();
        let m = [[cos, -sin], [sin, cos]];
        let t = (center.x - (cos * center.x - sin * center.y), center.y - (sin * center.x + cos * center.y));
        Self { m, t }
    }

    /// Spiegelung an der Geraden durch `a` und `b`.
    pub fn mirror(a: Point2, b: Point2) -> Option<Self> {
        let len = a.distance(b);
        if len < 1e-9 {
            return None;
        }
        let (ux, uy) = ((b.x - a.x) / len, (b.y - a.y) / len);
        let m = [[2.0 * ux * ux - 1.0, 2.0 * ux * uy], [2.0 * ux * uy, 2.0 * uy * uy - 1.0]];
        let t = (a.x - (m[0][0] * a.x + m[0][1] * a.y), a.y - (m[1][0] * a.x + m[1][1] * a.y));
        Some(Self { m, t })
    }

    pub fn point(&self, p: Point2) -> Point2 {
        Point2::new(
            self.m[0][0] * p.x + self.m[0][1] * p.y + self.t.0,
            self.m[1][0] * p.x + self.m[1][1] * p.y + self.t.1,
        )
    }

    pub fn vector(&self, v: Point2) -> Point2 {
        Point2::new(self.m[0][0] * v.x + self.m[0][1] * v.y, self.m[1][0] * v.x + self.m[1][1] * v.y)
    }

    /// Ob die Abbildung spiegelt (links und rechts tauschen).
    pub fn mirrors(&self) -> bool {
        self.m[0][0] * self.m[1][1] - self.m[0][1] * self.m[1][0] < 0.0
    }

    /// Neuer Drehwinkel in Grad für ein Bauteil mit Winkel `degrees`.
    pub fn angle(&self, degrees: f64) -> f64 {
        let (sin, cos) = degrees.to_radians().sin_cos();
        let v = self.vector(Point2::new(cos, sin));
        let a = v.y.atan2(v.x).to_degrees();
        // Glatte Winkel bleiben glatt.
        let r = (a * 1e6).round() / 1e6;
        if r <= -180.0 { r + 360.0 } else { r }
    }

    /// Ob die Abbildung nur verschiebt.
    pub fn is_translation(&self) -> bool {
        self.m == [[1.0, 0.0], [0.0, 1.0]]
    }
}

/// Das Element nach der Abbildung, ohne Bezüge auf andere Elemente zu ändern.
/// Öffnungen bleiben an ihrer Stelle in der Wand; wird die Wand gespiegelt,
/// rechnet [`transformed_opening`] sie um. `None` für Elemente, die sich so
/// noch nicht abbilden lassen (Treppen beim Spiegeln, Unterlagen).
pub fn transformed(e: &Element, t: &Transform) -> Option<Element> {
    let p = |q: Point2| t.point(q);
    let ring = |r: &[Point2]| -> Vec<Point2> {
        let mut v: Vec<Point2> = r.iter().map(|q| t.point(*q)).collect();
        // Die Umlaufrichtung bleibt dieselbe.
        if t.mirrors() {
            v.reverse();
        }
        v
    };
    Some(match e {
        Element::Wall(w) => {
            let mut n = w.clone();
            n.start = p(w.start);
            n.end = p(w.end);
            if t.mirrors() {
                // Rückwärts, damit die Schichten auf ihrer gespiegelten Seite bleiben;
                // damit bleibt auch der Bogen auf seiner Seite.
                std::mem::swap(&mut n.start, &mut n.end);
            }
            Element::Wall(n)
        }
        Element::Column(c) => {
            let mut n = c.clone();
            n.position = p(c.position);
            n.rotation = t.angle(c.rotation);
            Element::Column(n)
        }
        Element::Space(r) => {
            let mut n = r.clone();
            n.position = p(r.position);
            Element::Space(n)
        }
        Element::Slab(s) => {
            let mut n = s.clone();
            n.outline = ring(&s.outline);
            Element::Slab(n)
        }
        Element::SlabOpening(o) => {
            let mut n = o.clone();
            n.outline = ring(&o.outline);
            Element::SlabOpening(n)
        }
        Element::Roof(r) => {
            // Kante i läuft von Punkt i zu i + 1; beim Spiegeln bleiben die Punkte in
            // ihrer Reihenfolge, das Dach bestimmt seine Umlaufrichtung selbst.
            let mut n = r.clone();
            n.outline = r.outline.iter().map(|q| p(*q)).collect();
            Element::Roof(n)
        }
        Element::DimensionChain(d) => {
            let mut n = d.clone();
            n.position = p(d.position);
            n.direction = t.vector(d.direction);
            n.refs = d
                .refs
                .iter()
                .map(|r| match *r {
                    DimRef::Point(q) => DimRef::Point(p(q)),
                    DimRef::WallEnd { wall, end, side } if t.mirrors() => {
                        DimRef::WallEnd { wall, end: other_end(end), side }
                    }
                    DimRef::Jamb { opening, end, side } if t.mirrors() => {
                        DimRef::Jamb { opening, end: other_end(end), side }
                    }
                    r => r,
                })
                .collect();
            Element::DimensionChain(n)
        }
        Element::GridLine(g) => {
            let mut n = g.clone();
            n.start = p(g.start);
            n.end = p(g.end);
            Element::GridLine(n)
        }
        Element::SectionView(v) => {
            // Gespiegelt bleibt der Blick auf derselben Seite des Gebäudes.
            let mut n = v.clone();
            n.start = p(v.start);
            n.end = p(v.end);
            if t.mirrors() {
                n.look = match v.look {
                    lot_model::WallSide::Left => lot_model::WallSide::Right,
                    lot_model::WallSide::Right => lot_model::WallSide::Left,
                };
            }
            Element::SectionView(n)
        }
        Element::Annotation(a) => {
            use lot_model::AnnotationShape as S;
            let mut n = a.clone();
            n.shape = match &a.shape {
                S::Polyline(pts) => S::Polyline(pts.iter().map(|q| p(*q)).collect()),
                S::Polygon { points, hatch } => S::Polygon { points: ring(points), hatch: hatch.clone() },
                S::Circle { center, radius } => S::Circle { center: p(*center), radius: *radius },
                // Texte bleiben lesbar: nur Lage und Drehung, nie Spiegelschrift.
                S::Text { position, text, height, rotation } => S::Text {
                    position: p(*position),
                    text: text.clone(),
                    height: *height,
                    rotation: if t.mirrors() { *rotation } else { t.angle(*rotation) },
                },
            };
            Element::Annotation(n)
        }
        Element::Stair(s) if !t.mirrors() => {
            let mut n = s.clone();
            n.start = p(s.start);
            n.direction = t.vector(s.direction);
            Element::Stair(n)
        }
        Element::Railing(r) => {
            let mut n = r.clone();
            if let lot_model::RailingPath::Points(pts) = &r.path {
                n.path = lot_model::RailingPath::Points(pts.iter().map(|q| p(*q)).collect());
            }
            Element::Railing(n)
        }
        Element::Elevator(e) => {
            let mut n = e.clone();
            n.position = p(e.position);
            n.door_direction = t.vector(e.door_direction);
            Element::Elevator(n)
        }
        _ => return None,
    })
}

fn other_end(end: WallEnd) -> WallEnd {
    match end {
        WallEnd::Start => WallEnd::End,
        WallEnd::End => WallEnd::Start,
    }
}

/// Öffnung in einer Wand, die mit abgebildet wurde: Beim Spiegeln läuft die
/// Wand rückwärts, der Abstand zählt dann vom anderen Ende, und Türen
/// wechseln DIN links und rechts; die Aufschlagseite bleibt.
pub fn transformed_opening(o: &lot_model::Opening, wall_length: f64, t: &Transform) -> lot_model::Opening {
    let mut n = o.clone();
    if t.mirrors() {
        n.offset = wall_length - o.offset - o.width;
        if let lot_model::OpeningKind::Door(d) = &mut n.kind {
            d.hinge = d.hinge.other();
        }
    }
    n
}

/// Operationen, die die Elemente `ids` abgebildet als neue Elemente einfügen
/// (Kopieren, Reihe, Einfügen). Öffnungen kommen mit ihrer Wand, Durchbrüche
/// mit ihrer Decke; eine Öffnung ohne ihre Wand wird in derselben Wand
/// kopiert, um den Anteil der Verschiebung entlang der Wand. Maßketten nur,
/// wenn alle ihre Bezüge mitkommen. Räume und Achsen bekommen neue Nummern.
pub fn copy_ops(project: &Project, ids: &[ElementId], t: &Transform, storey: Option<ElementId>) -> Vec<Operation> {
    copy_ops_into(project, project, ids, t, storey)
}

/// Wie [`copy_ops`], aber die Elemente kommen aus `project` (etwa der
/// Zwischenablage) und landen in `target`: Kennungen, Raumnummern und
/// Achsnamen richten sich nach `target`.
pub fn copy_ops_into(
    project: &Project,
    target: &Project,
    ids: &[ElementId],
    t: &Transform,
    storey: Option<ElementId>,
) -> Vec<Operation> {
    let mut cp = Copier { next: target.next_id().0, map: HashMap::new(), ops: Vec::new(), scratch: target.clone() };
    // Erst Wände und Decken, dann was in ihnen sitzt, dann der Rest.
    let rank = |id: &ElementId| match project.elements.get(id) {
        Some(Element::Wall(_) | Element::Slab(_)) => 0,
        Some(Element::Opening(_) | Element::SlabOpening(_)) => 1,
        Some(Element::DimensionChain(_)) => 3,
        _ => 2,
    };
    let mut order: Vec<ElementId> = ids.to_vec();
    order.sort_by_key(rank);
    let mut with_host: Vec<ElementId> = Vec::new();
    for id in &order {
        let Some(e) = project.elements.get(id) else { continue };
        let on = |s: ElementId| storey.unwrap_or(s);
        match e {
            Element::Wall(w) => {
                let Some(Element::Wall(mut n)) = transformed(e, t) else { continue };
                n.storey = on(w.storey);
                let new = cp.insert(Some(*id), Element::Wall(n));
                for (oid, o) in project.openings_in(*id) {
                    let mut c = transformed_opening(o, w.length(), t);
                    c.host = new;
                    cp.insert(Some(oid), Element::Opening(c));
                    with_host.push(oid);
                }
            }
            Element::Slab(s) => {
                let Some(Element::Slab(mut n)) = transformed(e, t) else { continue };
                n.storey = on(s.storey);
                let new = cp.insert(Some(*id), Element::Slab(n));
                for (oid, o) in project.slab_openings_in(*id) {
                    let Some(Element::SlabOpening(mut c)) = transformed(&Element::SlabOpening(o.clone()), t) else {
                        continue;
                    };
                    c.host = new;
                    cp.insert(Some(oid), Element::SlabOpening(c));
                    with_host.push(oid);
                }
            }
            Element::Opening(o) if !with_host.contains(id) => {
                // Allein kopiert: in derselben Wand, um den Anteil entlang der Wand.
                let Some(wall) = project.wall(o.host).filter(|_| target.wall(o.host).is_some()) else { continue };
                let from = wall.point_at(o.offset + o.width / 2.0, 0.0);
                let to = t.point(from);
                let (tx, ty) = wall.tangent_at(o.offset + o.width / 2.0);
                let along = (to.x - from.x) * tx + (to.y - from.y) * ty;
                let mut c = o.clone();
                c.offset = ((o.offset + along) / 0.01).round() * 0.01;
                cp.insert(Some(*id), Element::Opening(c));
            }
            Element::SlabOpening(o) if !with_host.contains(id) => {
                if target.elements.contains_key(&o.host)
                    && let Some(Element::SlabOpening(c)) = transformed(e, t)
                {
                    cp.insert(Some(*id), Element::SlabOpening(lot_model::SlabOpening { host: o.host, ..c }));
                }
            }
            Element::Opening(_) | Element::SlabOpening(_) => {}
            Element::Space(r) => {
                let Some(Element::Space(mut n)) = transformed(e, t) else { continue };
                n.storey = on(r.storey);
                n.number = crate::next_space_number(&cp.scratch, n.storey, 0);
                cp.insert(Some(*id), Element::Space(n));
            }
            Element::GridLine(_) => {
                let Some(Element::GridLine(mut n)) = transformed(e, t) else { continue };
                n.label = crate::next_grid_label(&cp.scratch, n.start, n.end);
                cp.insert(Some(*id), Element::GridLine(n));
            }
            Element::DimensionChain(d) => {
                let refs_copied = d.refs.iter().all(|r| match r {
                    DimRef::WallEnd { wall, .. } | DimRef::WallFace { wall, .. } => cp.map.contains_key(wall),
                    DimRef::Jamb { opening, .. } => cp.map.contains_key(opening),
                    DimRef::Grid(g) => cp.map.contains_key(g),
                    DimRef::Point(_) => true,
                });
                if !refs_copied {
                    continue;
                }
                let Some(Element::DimensionChain(mut n)) = transformed(e, t) else { continue };
                n.storey = on(d.storey);
                for r in &mut n.refs {
                    match r {
                        DimRef::WallEnd { wall, .. } | DimRef::WallFace { wall, .. } => *wall = cp.map[wall],
                        DimRef::Jamb { opening, .. } => *opening = cp.map[opening],
                        DimRef::Grid(g) => *g = cp.map[g],
                        DimRef::Point(_) => {}
                    }
                }
                cp.insert(Some(*id), Element::DimensionChain(n));
            }
            _ => {
                let Some(mut n) = transformed(e, t) else { continue };
                if let Some(s) = storey {
                    set_storey(&mut n, s);
                }
                cp.insert(Some(*id), n);
            }
        }
    }
    cp.ops
}

/// Neue Kennungen und Operationen beim Kopieren; `scratch` kennt die Kopien
/// schon, damit Nummern für Räume und Achsen nicht doppelt vergeben werden.
struct Copier {
    next: u64,
    map: HashMap<ElementId, ElementId>,
    ops: Vec<Operation>,
    scratch: Project,
}

impl Copier {
    fn insert(&mut self, old: Option<ElementId>, e: Element) -> ElementId {
        let id = ElementId(self.next);
        self.next += 1;
        if let Some(old) = old {
            self.map.insert(old, id);
        }
        self.scratch.elements.insert(id, e.clone());
        self.ops.push(Operation::Insert { id, element: e });
        id
    }
}

fn set_storey(e: &mut Element, storey: ElementId) {
    match e {
        Element::Column(c) => c.storey = storey,
        Element::Roof(r) => r.storey = storey,
        Element::Stair(s) => s.storey = storey,
        Element::Railing(r) => r.storey = storey,
        Element::Annotation(a) => {
            if let lot_model::AnnotationView::FloorPlan { storey: s } = &mut a.view {
                *s = storey;
            }
        }
        _ => {}
    }
}

/// Operationen, die die Elemente `ids` abbilden (Drehen, Spiegeln). Wände
/// außerhalb der Auswahl, die Ende an Ende anschließen, ziehen mit.
pub fn transform_ops(project: &Project, ids: &[ElementId], t: &Transform) -> Vec<Operation> {
    let mut ops = Vec::new();
    let mut moved_ends: Vec<(Point2, Point2)> = Vec::new();
    for id in ids {
        let Some(e) = project.elements.get(id) else { continue };
        // Öffnungen gehen mit ihrer Wand; allein lassen sie sich nicht drehen oder spiegeln.
        if let Element::Opening(_) = e {
            continue;
        }
        if let Element::Wall(w) = e {
            moved_ends.extend([(w.start, t.point(w.start)), (w.end, t.point(w.end))]);
            for (oid, o) in project.openings_in(*id) {
                let n = transformed_opening(o, w.length(), t);
                if n != *o {
                    ops.push(Operation::Replace {
                        id: oid,
                        old: Element::Opening(o.clone()),
                        new: Element::Opening(n),
                    });
                }
            }
        }
        if let Some(n) = transformed(e, t) {
            ops.push(Operation::Replace { id: *id, old: e.clone(), new: n });
        }
    }
    for (id, w) in project.walls() {
        if ids.contains(&id) {
            continue;
        }
        let follow = |p: Point2| moved_ends.iter().find(|(a, _)| a.distance(p) <= JOIN_TOLERANCE).map(|(_, b)| *b);
        let (s, e) = (follow(w.start), follow(w.end));
        if s.is_some() || e.is_some() {
            let mut n = w.clone();
            n.start = s.unwrap_or(w.start);
            n.end = e.unwrap_or(w.end);
            ops.push(Operation::Replace { id, old: Element::Wall(w.clone()), new: Element::Wall(n) });
        }
    }
    ops
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: Point2, b: Point2) -> bool {
        a.distance(b) < 1e-9
    }

    #[test]
    fn rotation_and_mirror_map_points() {
        let r = Transform::rotation(Point2::new(1.0, 1.0), 90.0);
        assert!(close(r.point(Point2::new(2.0, 1.0)), Point2::new(1.0, 2.0)));
        assert!(!r.mirrors());
        assert!((r.angle(0.0) - 90.0).abs() < 1e-9);
        let m = Transform::mirror(Point2::new(0.0, 0.0), Point2::new(0.0, 1.0)).unwrap();
        assert!(close(m.point(Point2::new(2.0, 3.0)), Point2::new(-2.0, 3.0)));
        assert!(m.mirrors());
    }
}
