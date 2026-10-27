//! Temporäre Maße (ADR 0044): Nach der Auswahl eines Bauteils stehen Maße zum
//! nächsten Bezug. Ein Klick auf die Maßzahl, ein neuer Wert, und das Bauteil
//! rückt. Gemessen wird lichte Weite zwischen Wandflächen, wie im Rohbau.

use lot_model::{Element, ElementId, Point2, Project};

use crate::geometry::faces;

/// Ein temporäres Maß von `a` nach `b`. Wird es um 1 m größer, rückt das
/// Bauteil um `grow`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TempDim {
    pub a: Point2,
    pub b: Point2,
    pub value: f64,
    pub grow: (f64, f64),
}

impl TempDim {
    pub fn middle(&self) -> Point2 {
        Point2::new((self.a.x + self.b.x) / 2.0, (self.a.y + self.b.y) / 2.0)
    }
}

/// Größte Entfernung, in der ein Bezug noch gesucht wird, in m.
const REACH: f64 = 30.0;

/// Temporäre Maße für ein gewähltes Bauteil: gerade Wand, Stütze oder Öffnung.
pub fn temp_dims(project: &Project, storey: ElementId, id: ElementId) -> Vec<TempDim> {
    match project.elements.get(&id) {
        Some(Element::Wall(w)) if !w.is_curved() => wall_dims(project, storey, id),
        Some(Element::Column(c)) => column_dims(project, storey, c),
        Some(Element::Opening(o)) => opening_dims(project, id, o),
        _ => Vec::new(),
    }
}

/// Gerade Wand für die Suche nach Bezügen.
struct Straight {
    id: ElementId,
    start: Point2,
    dir: (f64, f64),
    length: f64,
    /// Linke und rechte Fläche als Abstand von der Bezugslinie.
    faces: [f64; 2],
}

/// Gerade Wände des Geschosses.
fn straight_walls(project: &Project, storey: ElementId) -> Vec<Straight> {
    project
        .walls_on(storey)
        .filter(|(_, w)| !w.is_curved())
        .map(|(id, w)| {
            let (l, r) = faces(project, w);
            Straight { id, start: w.start, dir: w.direction(), length: w.length(), faces: [l, r] }
        })
        .collect()
}

/// Lichte Abstände von beiden Flächen einer Wand zur nächsten parallelen Wand.
fn wall_dims(project: &Project, storey: ElementId, id: ElementId) -> Vec<TempDim> {
    let Some(w) = project.wall(id) else { return Vec::new() };
    let (left, right) = faces(project, w);
    let (dx, dy) = w.direction();
    let n = (-dy, dx);
    let mid = w.length() / 2.0;
    let mut out = Vec::new();
    for (face, sign) in [(left, 1.0), (right, -1.0)] {
        let from = w.point_at(mid, face);
        let mut best: Option<f64> = None;
        for Straight { id: other, start, dir, length, faces: [l, r] } in straight_walls(project, storey) {
            if other == id || (dir.0 * dy - dir.1 * dx).abs() > 0.01 {
                continue;
            }
            // Die andere Wand muss gegenüber liegen, nicht daneben.
            let along = (from.x - start.x) * dir.0 + (from.y - start.y) * dir.1;
            if along < 0.0 || along > length {
                continue;
            }
            let normal = (-dir.1, dir.0);
            for t in [l, r] {
                let q = Point2::new(start.x + normal.0 * t, start.y + normal.1 * t);
                let d = ((q.x - from.x) * n.0 + (q.y - from.y) * n.1) * sign;
                if d > 1e-6 && d < REACH && best.is_none_or(|b| d < b) {
                    best = Some(d);
                }
            }
        }
        if let Some(d) = best {
            let to = Point2::new(from.x + n.0 * d * sign, from.y + n.1 * d * sign);
            out.push(TempDim { a: from, b: to, value: d, grow: (-n.0 * sign, -n.1 * sign) });
        }
    }
    out
}

/// Abstände einer Stütze zur nächsten Wandfläche in x und y.
fn column_dims(project: &Project, storey: ElementId, c: &lot_model::Column) -> Vec<TempDim> {
    let (hw, hd) = match c.profile {
        lot_model::ColumnProfile::Rectangle { width, depth } if c.rotation.abs() < 1e-9 => (width / 2.0, depth / 2.0),
        lot_model::ColumnProfile::Circle { diameter } => (diameter / 2.0, diameter / 2.0),
        _ => return Vec::new(),
    };
    let walls = straight_walls(project, storey);
    let mut out = Vec::new();
    for (ray, half) in [((1.0, 0.0), hw), ((-1.0, 0.0), hw), ((0.0, 1.0), hd), ((0.0, -1.0), hd)] {
        let from = Point2::new(c.position.x + ray.0 * half, c.position.y + ray.1 * half);
        let mut best: Option<f64> = None;
        for Straight { start, dir, length, faces: [l, r], .. } in &walls {
            // Nur Wände quer zum Strahl.
            if (dir.0 * ray.1 - dir.1 * ray.0).abs() < 0.99 {
                continue;
            }
            let normal = (-dir.1, dir.0);
            for t in [*l, *r] {
                let q = Point2::new(start.x + normal.0 * t, start.y + normal.1 * t);
                // Schnitt des Strahls mit der Flächenlinie.
                let denom = ray.0 * normal.0 + ray.1 * normal.1;
                let d = ((q.x - from.x) * normal.0 + (q.y - from.y) * normal.1) / denom;
                let hit = Point2::new(from.x + ray.0 * d, from.y + ray.1 * d);
                let along = (hit.x - start.x) * dir.0 + (hit.y - start.y) * dir.1;
                if d > 1e-6 && d < REACH && along >= 0.0 && along <= *length && best.is_none_or(|b| d < b) {
                    best = Some(d);
                }
            }
        }
        if let Some(d) = best {
            let to = Point2::new(from.x + ray.0 * d, from.y + ray.1 * d);
            out.push(TempDim { a: from, b: to, value: d, grow: (-ray.0, -ray.1) });
        }
    }
    out
}

/// Abstände einer Öffnung entlang ihrer Wand: von den Laibungen zur nächsten
/// Wandfläche einer anstoßenden Wand oder zur nächsten Öffnung.
fn opening_dims(project: &Project, id: ElementId, o: &lot_model::Opening) -> Vec<TempDim> {
    let Some(host) = project.wall(o.host) else { return Vec::new() };
    if host.is_curved() {
        return Vec::new();
    }
    let (dx, dy) = host.direction();
    let length = host.length();
    // Hindernisse entlang der Bezugslinie: Flächen anstoßender Wände, Ränder anderer Öffnungen.
    let mut stops: Vec<f64> = Vec::new();
    for Straight { id: other, start, dir, length: olen, faces: [l, r] } in straight_walls(project, host.storey) {
        if other == o.host || (dir.0 * dy - dir.1 * dx).abs() < 0.01 {
            continue;
        }
        let normal = (-dir.1, dir.0);
        for t in [l, r] {
            let q = Point2::new(start.x + normal.0 * t, start.y + normal.1 * t);
            let denom = dx * normal.0 + dy * normal.1;
            let s = ((q.x - host.start.x) * normal.0 + (q.y - host.start.y) * normal.1) / denom;
            let hit = host.point_at(s, 0.0);
            let along = (hit.x - start.x) * dir.0 + (hit.y - start.y) * dir.1;
            // Die Wand muss die Bezugslinie wirklich erreichen (Ecke oder T-Stoß).
            if (-0.6..=length + 0.6).contains(&s) && along >= -0.6 && along <= olen + 0.6 {
                stops.push(s);
            }
        }
    }
    for (other, n) in project.openings_in(o.host) {
        if other != id {
            stops.extend([n.offset, n.offset + n.width]);
        }
    }
    let (start, end) = (o.offset, o.offset + o.width);
    let mut out = Vec::new();
    if let Some(s) = stops.iter().copied().filter(|s| *s < start - 1e-6).max_by(f64::total_cmp) {
        out.push(TempDim { a: host.point_at(s, 0.0), b: host.point_at(start, 0.0), value: start - s, grow: (dx, dy) });
    }
    if let Some(s) = stops.iter().copied().filter(|s| *s > end + 1e-6).min_by(f64::total_cmp) {
        out.push(TempDim { a: host.point_at(end, 0.0), b: host.point_at(s, 0.0), value: s - end, grow: (-dx, -dy) });
    }
    out
}
