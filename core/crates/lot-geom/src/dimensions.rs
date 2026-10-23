//! Maßketten: Bezüge auf Wände, Öffnungen und Achsen werden zu Punkten auf der Maßlinie.

use lot_model::{DimRef, DimensionChain, Element, ElementId, Point2, Project, Wall, WallEnd, WallSide};

/// Toleranz, mit der Wandenden als verbunden gelten, in Metern (wie bei den Anschlüssen).
const JOIN_TOLERANCE: f64 = 0.001;

/// Ein aufgelöster Bezugspunkt.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DimPoint {
    /// Lage entlang der Maßlinie, ab `DimensionChain::position`, in Metern.
    pub along: f64,
    /// Der Bezugspunkt im Grundriss.
    pub point: [f64; 2],
    /// Öffnung, zu deren Laibung der Punkt gehört.
    pub opening: Option<ElementId>,
}

/// Einheitsvektor der Maßlinie.
pub fn direction(d: &DimensionChain) -> [f64; 2] {
    let len = d.direction.x.hypot(d.direction.y).max(1e-12);
    [d.direction.x / len, d.direction.y / len]
}

/// Alle Bezugspunkte der Kette, entlang der Maßlinie sortiert. Fehlende Bezüge
/// (gelöschte Elemente) fallen weg, zusammenfallende Punkte zählen einmal.
pub fn resolve(project: &Project, d: &DimensionChain) -> Vec<DimPoint> {
    let dir = direction(d);
    let mut points: Vec<DimPoint> = d
        .refs
        .iter()
        .filter_map(|r| {
            let (p, opening) = match *r {
                DimRef::WallEnd { wall, end, side } => (wall_corner(project, wall, end, side)?, None),
                DimRef::Jamb { opening, end, side } => (jamb(project, opening, end, side)?, Some(opening)),
                DimRef::WallFace { wall, side } => {
                    let w = project.wall(wall)?;
                    (on_face(project, w, w.length() / 2.0, side)?, None)
                }
                DimRef::Grid(id) => match project.elements.get(&id)? {
                    Element::GridLine(g) => (grid_point(g.start, g.end, d.position, dir)?, None),
                    _ => return None,
                },
                DimRef::Point(p) => ([p.x, p.y], None),
            };
            let along = (p[0] - d.position.x) * dir[0] + (p[1] - d.position.y) * dir[1];
            Some(DimPoint { along, point: p, opening })
        })
        .collect();
    points.sort_by(|a, b| a.along.total_cmp(&b.along));
    points.dedup_by(|a, b| (a.along - b.along).abs() < 1e-6);
    points
}

/// Punkt auf einer Wandfläche im Abstand `along` vom Wandanfang.
fn on_face(project: &Project, wall: &Wall, along: f64, side: WallSide) -> Option<[f64; 2]> {
    let t = face(project, wall, side)?;
    let p = wall.point_at(along, t);
    Some([p.x, p.y])
}

/// Abstand einer Wandfläche von der Bezugslinie (links positiv).
fn face(project: &Project, wall: &Wall, side: WallSide) -> Option<f64> {
    let offsets = wall.layer_offsets(project.wall_type(wall.wall_type)?);
    Some(match side {
        WallSide::Left => offsets[0].0,
        WallSide::Right => offsets[offsets.len() - 1].1,
    })
}

fn jamb(project: &Project, opening: ElementId, end: WallEnd, side: WallSide) -> Option<[f64; 2]> {
    let Element::Opening(o) = project.elements.get(&opening)? else { return None };
    let wall = project.wall(o.host)?;
    let along = match end {
        WallEnd::Start => o.offset,
        WallEnd::End => o.offset + o.width,
    };
    on_face(project, wall, along, side)
}

/// Ecke am Wandende auf der Seite `side`. Trifft dort genau eine andere Wand
/// als L-Ecke an, gilt die Ecke beider Wände: auf der Außenseite der Ecke der
/// Schnitt mit der äußeren Fläche der anderen Wand, auf der Innenseite mit der inneren.
fn wall_corner(project: &Project, id: ElementId, end: WallEnd, side: WallSide) -> Option<[f64; 2]> {
    let wall = project.wall(id)?;
    let (at, along) = match end {
        WallEnd::Start => (wall.start, 0.0),
        WallEnd::End => (wall.end, wall.length()),
    };
    let base = on_face(project, wall, along, side)?;
    let (tx, ty) = wall.tangent_at(along);
    // Richtung aus der Wand hinaus über das Ende.
    let out = if end == WallEnd::Start { [-tx, -ty] } else { [tx, ty] };
    let others: Vec<&Wall> = project
        .walls_on(wall.storey)
        .filter(|(other, w)| {
            *other != id && (w.start.distance(at) < JOIN_TOLERANCE || w.end.distance(at) < JOIN_TOLERANCE)
        })
        .map(|(_, w)| w)
        .collect();
    let [other] = others[..] else { return Some(base) };
    if other.is_curved() {
        return Some(base);
    }
    let (ox, oy) = other.direction();
    // Läuft die andere Wand zur Seite `side` weg, liegt diese Seite innen in der Ecke.
    let away = if other.start.distance(at) < JOIN_TOLERANCE { [ox, oy] } else { [-ox, -oy] };
    let normal = match side {
        WallSide::Left => [-ty, tx],
        WallSide::Right => [ty, -tx],
    };
    let inner = away[0] * normal[0] + away[1] * normal[1] > 0.0;
    let candidates = [WallSide::Left, WallSide::Right].into_iter().filter_map(|s| {
        let q = on_face(project, other, 0.0, s)?;
        let p = intersect(base, out, q, [ox, oy])?;
        Some((p, (p[0] - base[0]) * out[0] + (p[1] - base[1]) * out[1]))
    });
    let best = if inner {
        candidates.min_by(|a, b| a.1.total_cmp(&b.1))
    } else {
        candidates.max_by(|a, b| a.1.total_cmp(&b.1))
    };
    Some(best.map_or(base, |(p, _)| p))
}

/// Schnitt einer Achse mit der Maßlinie.
fn grid_point(a: Point2, b: Point2, origin: Point2, dir: [f64; 2]) -> Option<[f64; 2]> {
    let len = a.distance(b).max(1e-12);
    intersect([origin.x, origin.y], dir, [a.x, a.y], [(b.x - a.x) / len, (b.y - a.y) / len])
}

fn intersect(p: [f64; 2], d: [f64; 2], q: [f64; 2], e: [f64; 2]) -> Option<[f64; 2]> {
    let denom = d[0] * e[1] - d[1] * e[0];
    if denom.abs() < 1e-9 {
        return None;
    }
    let t = ((q[0] - p[0]) * e[1] - (q[1] - p[1]) * e[0]) / denom;
    Some([p[0] + d[0] * t, p[1] + d[1] * t])
}
