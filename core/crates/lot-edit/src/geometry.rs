//! Geometrische Hilfen für das Bearbeiten: Lage von Punkten zu Wänden,
//! Verschieben mit angeschlossenen Wänden.

use lot_model::{Element, ElementId, Point2, Project, Wall};

/// Toleranz, mit der Wandenden als verbunden gelten, in Metern (wie in lot-geom).
pub const JOIN_TOLERANCE: f64 = 0.001;

/// Ende einer Wand.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum End {
    Start,
    End,
}

pub fn end_point(wall: &Wall, end: End) -> Point2 {
    match end {
        End::Start => wall.start,
        End::End => wall.end,
    }
}

fn set_end(wall: &mut Wall, end: End, p: Point2) {
    match end {
        End::Start => wall.start = p,
        End::End => wall.end = p,
    }
}

/// Lage eines Punkts im System der Wand: (längs ab Anfang, quer, links positiv).
pub fn local(wall: &Wall, p: Point2) -> (f64, f64) {
    let (dx, dy) = wall.direction();
    let (px, py) = (p.x - wall.start.x, p.y - wall.start.y);
    (px * dx + py * dy, -px * dy + py * dx)
}

/// Linke und rechte Wandfläche als Abstand von der Bezugslinie.
pub fn faces(project: &Project, wall: &Wall) -> (f64, f64) {
    match project.wall_type(wall.wall_type) {
        Some(t) => {
            let o = wall.layer_offsets(t);
            (o[0].0, o[o.len() - 1].1)
        }
        None => (0.0, 0.0),
    }
}

/// Umriss einer Wand ohne Anschlüsse, als Rechteck in Metern.
pub fn footprint(project: &Project, wall: &Wall) -> Vec<[f64; 2]> {
    let (left, right) = faces(project, wall);
    let (dx, dy) = wall.direction();
    let at = |p: Point2, t: f64| [p.x - dy * t, p.y + dx * t];
    vec![at(wall.start, right), at(wall.end, right), at(wall.end, left), at(wall.start, left)]
}

/// Abstand des Punkts von der Fläche der Wand (0 innerhalb), in Metern.
pub fn distance_to_wall(project: &Project, wall: &Wall, p: Point2) -> f64 {
    let (s, t) = local(wall, p);
    let (left, right) = faces(project, wall);
    let ds = if s < 0.0 {
        -s
    } else if s > wall.length() {
        s - wall.length()
    } else {
        0.0
    };
    let dt = if t > left {
        t - left
    } else if t < right {
        right - t
    } else {
        0.0
    };
    ds.hypot(dt)
}

/// Das Element unter dem Punkt: Stützen vor Wänden vor Achsen, jeweils das nächste.
pub fn element_at(project: &Project, storey: ElementId, p: Point2, tolerance: f64) -> Option<ElementId> {
    let column = project
        .columns_on(storey)
        .map(|(id, c)| (id, column_distance(c, p)))
        .filter(|(_, d)| *d <= tolerance)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(id, _)| id);
    column.or_else(|| wall_at(project, storey, p, tolerance)).or_else(|| {
        project
            .grid_lines()
            .map(|(id, g)| (id, segment_distance(g.start, g.end, p)))
            .filter(|(_, d)| *d <= tolerance)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(id, _)| id)
    })
}

/// Abstand vom Querschnitt einer Stütze (0 innerhalb).
fn column_distance(c: &lot_model::Column, p: Point2) -> f64 {
    let (sin, cos) = (-c.rotation.to_radians()).sin_cos();
    let (dx, dy) = (p.x - c.position.x, p.y - c.position.y);
    let (x, y) = (dx * cos - dy * sin, dx * sin + dy * cos);
    match c.profile {
        lot_model::ColumnProfile::Rectangle { width, depth } => {
            ((x.abs() - width / 2.0).max(0.0)).hypot((y.abs() - depth / 2.0).max(0.0))
        }
        lot_model::ColumnProfile::Circle { diameter } => (x.hypot(y) - diameter / 2.0).max(0.0),
    }
}

/// Umriss eines Elements für die Auswahl, in Metern.
pub fn outline(project: &Project, id: ElementId) -> Option<Vec<[f64; 2]>> {
    match project.elements.get(&id)? {
        Element::Wall(w) => Some(footprint(project, w)),
        Element::Column(c) => Some(column_outline(c)),
        Element::GridLine(g) => {
            let w = Wall {
                storey: id,
                wall_type: id,
                start: g.start,
                end: g.end,
                alignment: lot_model::WallAlignment::Center,
                base_offset: 0.0,
                top: lot_model::Top::Height(1.0),
            };
            let (dx, dy) = w.direction();
            let at = |p: Point2, t: f64| [p.x - dy * t, p.y + dx * t];
            Some(vec![at(g.start, -0.05), at(g.end, -0.05), at(g.end, 0.05), at(g.start, 0.05)])
        }
        _ => None,
    }
}

/// Umriss einer Stütze.
pub fn column_outline(c: &lot_model::Column) -> Vec<[f64; 2]> {
    let (sin, cos) = c.rotation.to_radians().sin_cos();
    match c.profile {
        lot_model::ColumnProfile::Rectangle { width, depth } => [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)]
            .iter()
            .map(|(sx, sy)| {
                let (x, y) = (sx * width / 2.0, sy * depth / 2.0);
                [c.position.x + x * cos - y * sin, c.position.y + x * sin + y * cos]
            })
            .collect(),
        lot_model::ColumnProfile::Circle { diameter } => (0..32)
            .map(|i| {
                let a = i as f64 / 32.0 * std::f64::consts::TAU;
                [c.position.x + diameter / 2.0 * a.cos(), c.position.y + diameter / 2.0 * a.sin()]
            })
            .collect(),
    }
}

fn segment_distance(a: Point2, b: Point2, p: Point2) -> f64 {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let len2 = dx * dx + dy * dy;
    let t = if len2 > 0.0 { (((p.x - a.x) * dx + (p.y - a.y) * dy) / len2).clamp(0.0, 1.0) } else { 0.0 };
    p.distance(Point2::new(a.x + dx * t, a.y + dy * t))
}

/// Neue Stände aller Elemente, wenn die Auswahl um `delta` verschoben wird:
/// Wände mit ihren Anschlüssen, Stützen und Achsen.
pub fn moved_elements(project: &Project, ids: &[ElementId], delta: (f64, f64)) -> Vec<(ElementId, Element)> {
    let walls: Vec<ElementId> = ids.iter().copied().filter(|id| project.wall(*id).is_some()).collect();
    let mut changes = as_elements(moved_walls(project, &walls, delta));
    let shift = |p: Point2| Point2::new(p.x + delta.0, p.y + delta.1);
    for id in ids {
        match project.elements.get(id) {
            Some(Element::Column(c)) => {
                let mut n = c.clone();
                n.position = shift(c.position);
                changes.push((*id, Element::Column(n)));
            }
            Some(Element::GridLine(g)) => {
                let mut n = g.clone();
                n.start = shift(g.start);
                n.end = shift(g.end);
                changes.push((*id, Element::GridLine(n)));
            }
            _ => {}
        }
    }
    changes
}

/// Die Wand unter dem Punkt (innerhalb `tolerance`), die nächste zuerst.
pub fn wall_at(project: &Project, storey: ElementId, p: Point2, tolerance: f64) -> Option<ElementId> {
    project
        .walls_on(storey)
        .map(|(id, w)| (id, distance_to_wall(project, w, p)))
        .filter(|(_, d)| *d <= tolerance)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(id, _)| id)
}

/// Schnittpunkt der Geraden durch `p` mit Richtung `d` und der Geraden durch `q` mit Richtung `e`.
pub fn intersect(p: Point2, d: (f64, f64), q: Point2, e: (f64, f64)) -> Option<Point2> {
    let denom = d.0 * e.1 - d.1 * e.0;
    if denom.abs() < 1e-9 {
        return None;
    }
    let t = ((q.x - p.x) * e.1 - (q.y - p.y) * e.0) / denom;
    Some(Point2::new(p.x + d.0 * t, p.y + d.1 * t))
}

/// Neue Stände aller Wände, wenn die Wände `ids` um `delta` verschoben werden.
/// Wände, deren Ende an einer verschobenen Wand hängt, werden mitgezogen:
/// an Ecken wandert das Ende mit, an T-Stößen bleibt die anstoßende Wand in
/// ihrer Richtung und wird verlängert oder gekürzt.
pub fn moved_walls(project: &Project, ids: &[ElementId], delta: (f64, f64)) -> Vec<(ElementId, Wall)> {
    let mut changes: Vec<(ElementId, Wall)> = Vec::new();
    let moved: Vec<(ElementId, &Wall)> = ids.iter().filter_map(|id| Some((*id, project.wall(*id)?))).collect();
    for (id, w) in &moved {
        let mut n = (*w).clone();
        n.start = Point2::new(w.start.x + delta.0, w.start.y + delta.1);
        n.end = Point2::new(w.end.x + delta.0, w.end.y + delta.1);
        changes.push((*id, n));
    }
    let Some(storey) = moved.first().map(|(_, w)| w.storey) else { return changes };
    for (id, w) in project.walls_on(storey) {
        if ids.contains(&id) {
            continue;
        }
        let mut n = w.clone();
        let mut changed = false;
        for end in [End::Start, End::End] {
            let e = end_point(w, end);
            for (_, m) in &moved {
                if e.distance(m.start) <= JOIN_TOLERANCE || e.distance(m.end) <= JOIN_TOLERANCE {
                    set_end(&mut n, end, Point2::new(e.x + delta.0, e.y + delta.1));
                    changed = true;
                    break;
                }
                if let Some(p) = follow_tee(project, w, e, m, delta) {
                    set_end(&mut n, end, p);
                    changed = true;
                    break;
                }
            }
        }
        if changed {
            changes.push((id, n));
        }
    }
    changes
}

/// Neue Stände, wenn das Ende `end` der Wand `id` nach `to` gezogen wird.
/// Wände mit demselben Ende ziehen mit.
pub fn moved_endpoint(project: &Project, id: ElementId, end: End, to: Point2) -> Vec<(ElementId, Wall)> {
    let Some(w) = project.wall(id) else { return Vec::new() };
    let from = end_point(w, end);
    let mut changes = Vec::new();
    let mut n = w.clone();
    set_end(&mut n, end, to);
    changes.push((id, n));
    for (other, ow) in project.walls_on(w.storey) {
        if other == id {
            continue;
        }
        let mut n = ow.clone();
        let mut changed = false;
        for e in [End::Start, End::End] {
            if end_point(ow, e).distance(from) <= JOIN_TOLERANCE {
                set_end(&mut n, e, to);
                changed = true;
            }
        }
        if changed {
            changes.push((other, n));
        }
    }
    changes
}

/// Ende `e` der Wand `w` sitzt an der verschobenen Wand `m` (T-Stoß): neuer
/// Punkt auf der verschobenen Wand, in Richtung von `w`.
fn follow_tee(project: &Project, w: &Wall, e: Point2, m: &Wall, delta: (f64, f64)) -> Option<Point2> {
    let (s, t) = local(m, e);
    let (left, right) = faces(project, m);
    let inside = s > JOIN_TOLERANCE
        && s < m.length() - JOIN_TOLERANCE
        && t <= left + JOIN_TOLERANCE
        && t >= right - JOIN_TOLERANCE;
    if !inside {
        return None;
    }
    let (mx, my) = m.direction();
    let line_point = Point2::new(m.start.x + delta.0 - my * t, m.start.y + delta.1 + mx * t);
    intersect(e, w.direction(), line_point, (mx, my))
}

/// Ersetzungen als Elemente.
pub fn as_elements(changes: Vec<(ElementId, Wall)>) -> Vec<(ElementId, Element)> {
    changes.into_iter().map(|(id, w)| (id, Element::Wall(w))).collect()
}
