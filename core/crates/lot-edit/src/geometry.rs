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
/// Bei gebogenen Wänden entlang des Bogens.
pub fn local(wall: &Wall, p: Point2) -> (f64, f64) {
    wall.locate(p)
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

/// Umriss einer Wand ohne Anschlüsse in Metern: ein Rechteck, bei
/// gebogenen Wänden ein Ringstück.
pub fn footprint(project: &Project, wall: &Wall) -> Vec<[f64; 2]> {
    let (left, right) = faces(project, wall);
    let n = if wall.is_curved() { 32 } else { 1 };
    let at = |i: usize, t: f64| {
        let p = wall.point_at(wall.length() * i as f64 / n as f64, t);
        [p.x, p.y]
    };
    let mut ring: Vec<[f64; 2]> = (0..=n).map(|i| at(i, right)).collect();
    ring.extend((0..=n).rev().map(|i| at(i, left)));
    ring
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

/// Halbe Breite und Höhe des Bereichs um einen Raumpunkt, in dem ein Klick den
/// Raum wählt, in Metern (etwa die Größe des Raumstempels in 1:100).
const ROOM_PICK: (f64, f64) = (0.9, 0.5);

/// Das Element unter dem Punkt, siehe [`elements_at`].
pub fn element_at(project: &Project, storey: ElementId, p: Point2, tolerance: f64) -> Option<ElementId> {
    elements_at(project, storey, p, tolerance).into_iter().next()
}

/// Alle wählbaren Elemente unter dem Punkt, das Wichtigste zuerst: Öffnungen,
/// 2D-Elemente, Maßketten, Stützen, Wände, Achsen, Deckendurchbrüche, Dächer,
/// Deckenränder, Raumstempel; innerhalb einer Art das nächste zuerst.
/// Unterlagen fehlen: Sie sind nie wählbar, aber fangbar (ADR 0043).
pub fn elements_at(project: &Project, storey: ElementId, p: Point2, tolerance: f64) -> Vec<ElementId> {
    let mut out = openings_at(project, storey, p, tolerance);
    out.extend(annotations_at(project, storey, p, tolerance));
    out.extend(nearest(
        project.dimensions_on(storey).filter_map(|(id, d)| {
            let (a, b) = dimension_extent(project, d)?;
            Some((id, segment_distance(a, b, p)))
        }),
        tolerance,
    ));
    out.extend(nearest(project.columns_on(storey).map(|(id, c)| (id, column_distance(c, p))), tolerance));
    out.extend(walls_at(project, storey, p, tolerance));
    out.extend(nearest(project.grid_lines().map(|(id, g)| (id, segment_distance(g.start, g.end, p))), tolerance));
    out.extend(nearest(
        project
            .slabs_on(storey)
            .flat_map(|(slab, _)| project.slab_openings_in(slab))
            .map(|(id, o)| (id, ring_distance(&o.outline, p))),
        tolerance,
    ));
    out.extend(nearest(project.roofs_on(storey).map(|(id, r)| (id, ring_distance(&r.outer_outline(), p))), tolerance));
    out.extend(nearest(project.slabs_on(storey).map(|(id, s)| (id, ring_distance(&s.outline, p))), tolerance));
    out.extend(nearest(
        project
            .spaces_on(storey)
            .filter(|(_, r)| {
                (p.x - r.position.x).abs() <= ROOM_PICK.0 + tolerance
                    && (p.y - r.position.y).abs() <= ROOM_PICK.1 + tolerance
            })
            .map(|(id, r)| (id, r.position.distance(p))),
        f64::INFINITY,
    ));
    out
}

/// Bereich um einen Raumpunkt, in dem ein Klick den Raum wählt.
pub fn space_stamp(r: &lot_model::Space) -> [Point2; 4] {
    let (x, y, w, h) = (r.position.x, r.position.y, ROOM_PICK.0, ROOM_PICK.1);
    [Point2::new(x - w, y - h), Point2::new(x + w, y - h), Point2::new(x + w, y + h), Point2::new(x - w, y + h)]
}

/// Kennungen mit Abstand bis `tolerance`, das nächste zuerst.
fn nearest(items: impl Iterator<Item = (ElementId, f64)>, tolerance: f64) -> Vec<ElementId> {
    let mut v: Vec<(ElementId, f64)> = items.filter(|(_, d)| *d <= tolerance).collect();
    v.sort_by(|a, b| a.1.total_cmp(&b.1));
    v.into_iter().map(|(id, _)| id).collect()
}

/// Abstand vom Rand eines geschlossenen Umrisses.
fn ring_distance(ring: &[Point2], p: Point2) -> f64 {
    let n = ring.len();
    (0..n).map(|i| segment_distance(ring[i], ring[(i + 1) % n], p)).fold(f64::INFINITY, f64::min)
}

/// Umriss einer Öffnung in ihrer Wand, über die ganze Wanddicke.
pub fn opening_outline(project: &Project, o: &lot_model::Opening) -> Option<Vec<[f64; 2]>> {
    let wall = project.wall(o.host)?;
    let (left, right) = faces(project, wall);
    lot_geom::wall_section(wall, (o.offset, o.offset + o.width), (left, right)).rings().into_iter().next()
}

/// Die Öffnungen, in denen der Punkt liegt.
fn openings_at(project: &Project, storey: ElementId, p: Point2, tolerance: f64) -> Vec<ElementId> {
    project
        .walls_on(storey)
        .flat_map(|(wall_id, wall)| {
            let (s, t) = wall.locate(p);
            let (left, right) = faces(project, wall);
            let inside = t <= left + tolerance && t >= right - tolerance;
            project
                .openings_in(wall_id)
                .filter(move |(_, o)| {
                    inside && s >= o.offset - tolerance / 2.0 && s <= o.offset + o.width + tolerance / 2.0
                })
                .map(|(id, _)| id)
        })
        .collect()
}

/// Anfang und Ende der Maßlinie.
fn dimension_extent(project: &Project, d: &lot_model::DimensionChain) -> Option<(Point2, Point2)> {
    let points = lot_geom::dimensions::resolve(project, d);
    let (first, last) = (points.first()?, points.last()?);
    let dir = lot_geom::dimensions::direction(d);
    let at = |s: f64| Point2::new(d.position.x + dir[0] * s, d.position.y + dir[1] * s);
    Some((at(first.along), at(last.along)))
}

/// Ungefähre Ausdehnung eines Texts in m, für 1:100 (Schrifthöhe in mm auf dem Papier).
fn text_box(position: Point2, text: &str, height: f64) -> [[f64; 2]; 4] {
    let (w, h) = (text.chars().count() as f64 * height * 0.06, height * 0.1);
    let (x, y) = (position.x, position.y);
    [[x, y], [x + w, y], [x + w, y + h], [x, y + h]]
}

/// Kanten eines 2D-Elements im Grundriss; Texte als Rechteck.
fn annotation_edges(a: &lot_model::Annotation) -> Vec<(Point2, Point2)> {
    let pairs = |pts: &[Point2], closed: bool| -> Vec<(Point2, Point2)> {
        let n = pts.len();
        let count = if closed { n } else { n.saturating_sub(1) };
        (0..count).map(|i| (pts[i], pts[(i + 1) % n])).collect()
    };
    match &a.shape {
        lot_model::AnnotationShape::Polyline(p) => pairs(p, false),
        lot_model::AnnotationShape::Polygon { points, .. } => pairs(points, true),
        lot_model::AnnotationShape::Circle { center, radius } => {
            let ring: Vec<Point2> = (0..32)
                .map(|i| {
                    let t = i as f64 / 32.0 * std::f64::consts::TAU;
                    Point2::new(center.x + radius * t.cos(), center.y + radius * t.sin())
                })
                .collect();
            pairs(&ring, true)
        }
        lot_model::AnnotationShape::Text { position, text, height, .. } => {
            let b = text_box(*position, text, *height).map(|q| Point2::new(q[0], q[1]));
            pairs(&b, true)
        }
    }
}

/// Die 2D-Elemente des Grundrisses unter dem Punkt; bei Texten zählt die ganze Fläche.
fn annotations_at(project: &Project, storey: ElementId, p: Point2, tolerance: f64) -> Vec<ElementId> {
    nearest(
        project.annotations_in(lot_model::AnnotationView::FloorPlan { storey }).map(|(id, a)| {
            if let lot_model::AnnotationShape::Text { position, text, height, .. } = &a.shape {
                let b = text_box(*position, text, *height);
                if p.x >= b[0][0] && p.x <= b[1][0] && p.y >= b[0][1] && p.y <= b[2][1] {
                    return (id, 0.0);
                }
            }
            (id, annotation_edges(a).iter().map(|(a, b)| segment_distance(*a, *b, p)).fold(f64::INFINITY, f64::min))
        }),
        tolerance,
    )
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
        Element::Opening(o) => opening_outline(project, o),
        Element::SlabOpening(o) => Some(o.outline.iter().map(|p| [p.x, p.y]).collect()),
        Element::Slab(s) => Some(s.outline.iter().map(|p| [p.x, p.y]).collect()),
        Element::Roof(r) => Some(r.outer_outline().iter().map(|p| [p.x, p.y]).collect()),
        Element::Underlay(u) => Some(u.corners().iter().map(|p| [p.x, p.y]).collect()),
        Element::Annotation(a) => match &a.shape {
            lot_model::AnnotationShape::Polyline(points) if points.len() == 2 => {
                let (s, e) = (points[0], points[1]);
                let len = s.distance(e).max(1e-12);
                let (nx, ny) = (-(e.y - s.y) / len * 0.03, (e.x - s.x) / len * 0.03);
                Some(vec![[s.x - nx, s.y - ny], [e.x - nx, e.y - ny], [e.x + nx, e.y + ny], [s.x + nx, s.y + ny]])
            }
            lot_model::AnnotationShape::Text { position, text, height, .. } => {
                Some(text_box(*position, text, *height).to_vec())
            }
            _ => {
                let edges = annotation_edges(a);
                Some(edges.iter().map(|(p, _)| [p.x, p.y]).chain(edges.last().map(|(_, q)| [q.x, q.y])).collect())
            }
        },
        Element::DimensionChain(d) => {
            let (a, b) = dimension_extent(project, d)?;
            let len = a.distance(b).max(1e-9);
            let (nx, ny) = (-(b.y - a.y) / len * 0.05, (b.x - a.x) / len * 0.05);
            Some(vec![[a.x - nx, a.y - ny], [b.x - nx, b.y - ny], [b.x + nx, b.y + ny], [a.x + nx, a.y + ny]])
        }
        Element::Space(r) => {
            // Die Fläche des Raums, bei offenen Räumen der Bereich des Stempels.
            let finder = lot_geom::spaces::SpaceFinder::new(project, r.storey);
            let ring = finder.area_at([r.position.x, r.position.y]).and_then(|a| {
                a.rings().into_iter().max_by(|x, y| lot_geom::signed_area(x).total_cmp(&lot_geom::signed_area(y)))
            });
            Some(ring.unwrap_or_else(|| {
                let (x, y, w, h) = (r.position.x, r.position.y, ROOM_PICK.0, ROOM_PICK.1);
                vec![[x - w, y - h], [x + w, y - h], [x + w, y + h], [x - w, y + h]]
            }))
        }
        Element::SectionView(v) => {
            let len = v.start.distance(v.end).max(1e-12);
            let (dx, dy) = ((v.end.x - v.start.x) / len, (v.end.y - v.start.y) / len);
            let at = |p: Point2, t: f64| [p.x - dy * t, p.y + dx * t];
            Some(vec![at(v.start, -0.05), at(v.end, -0.05), at(v.end, 0.05), at(v.start, 0.05)])
        }
        Element::GridLine(g) => {
            let len = g.start.distance(g.end).max(1e-12);
            let (dx, dy) = ((g.end.x - g.start.x) / len, (g.end.y - g.start.y) / len);
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
            Some(Element::Space(r)) => {
                let mut n = r.clone();
                n.position = shift(r.position);
                changes.push((*id, Element::Space(n)));
            }
            Some(Element::SlabOpening(o)) => {
                let mut n = o.clone();
                n.outline = o.outline.iter().map(|p| shift(*p)).collect();
                changes.push((*id, Element::SlabOpening(n)));
            }
            Some(Element::Roof(r)) => {
                let mut n = r.clone();
                n.outline = r.outline.iter().map(|p| shift(*p)).collect();
                changes.push((*id, Element::Roof(n)));
            }
            Some(Element::Slab(s)) => {
                let mut n = s.clone();
                n.outline = s.outline.iter().map(|p| shift(*p)).collect();
                changes.push((*id, Element::Slab(n)));
            }
            Some(Element::DimensionChain(d)) => {
                let mut n = d.clone();
                n.position = shift(d.position);
                changes.push((*id, Element::DimensionChain(n)));
            }
            // Öffnungen gleiten entlang ihrer Wand; mit der Wand bewegen sie sich ohnehin.
            Some(Element::Opening(o)) if !walls.contains(&o.host) => {
                let Some(wall) = project.wall(o.host) else { continue };
                let (tx, ty) = wall.tangent_at(o.offset + o.width / 2.0);
                let along = delta.0 * tx + delta.1 * ty;
                let mut n = o.clone();
                n.offset = ((o.offset + along) / 0.01).round() * 0.01;
                n.offset = n.offset.clamp(0.0, (wall.length() - o.width).max(0.0));
                changes.push((*id, Element::Opening(n)));
            }
            Some(Element::Underlay(u)) => {
                let mut n = u.clone();
                n.origin = shift(u.origin);
                changes.push((*id, Element::Underlay(n)));
            }
            Some(Element::Annotation(a)) => {
                let mut n = a.clone();
                n.shape = a.shape.translated(delta.0, delta.1);
                changes.push((*id, Element::Annotation(n)));
            }
            Some(Element::GridLine(g)) => {
                let mut n = g.clone();
                n.start = shift(g.start);
                n.end = shift(g.end);
                changes.push((*id, Element::GridLine(n)));
            }
            Some(Element::SectionView(v)) => {
                let mut n = v.clone();
                n.start = shift(v.start);
                n.end = shift(v.end);
                changes.push((*id, Element::SectionView(n)));
            }
            _ => {}
        }
    }
    changes
}

/// Die Wand unter dem Punkt (innerhalb `tolerance`), die nächste zuerst.
pub fn wall_at(project: &Project, storey: ElementId, p: Point2, tolerance: f64) -> Option<ElementId> {
    walls_at(project, storey, p, tolerance).into_iter().next()
}

/// Alle Wände unter dem Punkt, die nächste zuerst.
pub fn walls_at(project: &Project, storey: ElementId, p: Point2, tolerance: f64) -> Vec<ElementId> {
    nearest(project.walls_on(storey).map(|(id, w)| (id, distance_to_wall(project, w, p))), tolerance)
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
    if m.is_curved() {
        // An einer gebogenen Wand wandert das Ende einfach mit.
        return Some(Point2::new(e.x + delta.0, e.y + delta.1));
    }
    let (mx, my) = m.direction();
    let line_point = Point2::new(m.start.x + delta.0 - my * t, m.start.y + delta.1 + mx * t);
    intersect(e, w.direction(), line_point, (mx, my))
}

/// Ersetzungen als Elemente.
pub fn as_elements(changes: Vec<(ElementId, Wall)>) -> Vec<(ElementId, Element)> {
    changes.into_iter().map(|(id, w)| (id, Element::Wall(w))).collect()
}

/// Schnitte und Ansichten unter dem Zeiger (ADR 0049), der nächste zuerst.
/// Schnitte trifft man an ihren Marken: den breiten Enden der Linie, dem
/// Pfeil und dem Buchstaben, je `mark` lang; die Linie durchs Gebäude bleibt
/// frei für die Bauteile. Ansichten liegen vor der Fassade und sind auf der
/// ganzen Linie zu treffen.
pub fn views_at(project: &Project, p: Point2, tolerance: f64, mark: f64) -> Vec<ElementId> {
    nearest(
        project.section_views().map(|(id, v)| {
            let d = match v.kind {
                lot_model::SectionKind::Elevation => segment_distance(v.start, v.end, p),
                lot_model::SectionKind::Section => {
                    let len = v.start.distance(v.end).max(1e-12);
                    let dir = ((v.end.x - v.start.x) / len, (v.end.y - v.start.y) / len);
                    let (look, _) = v.frame();
                    let k = mark.min(len / 3.0);
                    let along = |q: Point2, d: (f64, f64), t: f64| Point2::new(q.x + d.0 * t, q.y + d.1 * t);
                    [
                        segment_distance(v.start, along(v.start, dir, k), p),
                        segment_distance(v.end, along(v.end, dir, -k), p),
                        segment_distance(v.start, along(v.start, look, mark * 1.2), p),
                        segment_distance(v.end, along(v.end, look, mark * 1.2), p),
                    ]
                    .into_iter()
                    .fold(f64::INFINITY, f64::min)
                }
            };
            (id, d)
        }),
        tolerance,
    )
}

/// Die Schnittlinie mit einem Ende an `to`; `None`, wenn nichts zu ändern ist
/// oder die Linie zu kurz würde.
pub fn moved_section_end(project: &Project, id: ElementId, end: End, to: Point2) -> Option<(ElementId, Element)> {
    let Some(Element::SectionView(v)) = project.elements.get(&id) else { return None };
    let mut n = v.clone();
    match end {
        End::Start => n.start = to,
        End::End => n.end = to,
    }
    (n != *v && n.start.distance(n.end) >= 0.1).then_some((id, Element::SectionView(n)))
}
