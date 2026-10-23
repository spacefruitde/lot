//! Fangen: Der Zeiger rastet an Wandenden, Wänden, Achsen und Winkeln ein.

use lot_model::{ElementId, Point2, Project};

use crate::geometry::{intersect, local};

/// Woran der Zeiger eingerastet ist.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapKind {
    /// Ende einer Wand (Bezugslinie)
    Endpoint,
    /// Schnittpunkt zweier Achsen
    GridIntersection,
    /// Punkt auf der Bezugslinie einer Wand
    OnWall,
    /// Punkt auf einer Achse
    Grid,
    /// Richtung in 15°-Schritten vom Anfangspunkt
    Angle,
    /// Frei, auf 1 cm gerundet
    Free,
    /// Endpunkt einer Linie in einer Unterlage aus DXF oder DWG
    Underlay,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Snap {
    pub point: Point2,
    pub kind: SnapKind,
}

/// Raster für freie Punkte und Längen, in Metern.
pub const ROUND: f64 = 0.01;
/// Winkelschritt beim Zeichnen, in Grad.
const ANGLE_STEP: f64 = 15.0;
/// Bis zu dieser Abweichung rastet der Winkel ein, in Grad.
const ANGLE_CATCH: f64 = 3.0;

/// Fangpunkt zum Zeiger `p`. `from` ist der Anfangspunkt beim Zeichnen;
/// mit `orthogonal` rastet die Richtung nur in 90°-Schritten ein.
/// `exclude` nimmt Wände aus, etwa die gerade verschobenen.
pub fn snap(
    project: &Project,
    storey: ElementId,
    p: Point2,
    tolerance: f64,
    from: Option<Point2>,
    orthogonal: bool,
    exclude: &[ElementId],
) -> Snap {
    let s = find(project, storey, p, tolerance, from, orthogonal, exclude);
    // Rechenrauschen aus Winkelfunktionen entfernen (4,0 statt 4,000000000000001).
    let clean = |v: f64| (v * 1e9).round() / 1e9;
    Snap { point: Point2::new(clean(s.point.x), clean(s.point.y)), kind: s.kind }
}

fn find(
    project: &Project,
    storey: ElementId,
    p: Point2,
    tolerance: f64,
    from: Option<Point2>,
    orthogonal: bool,
    exclude: &[ElementId],
) -> Snap {
    let walls: Vec<_> = project.walls_on(storey).filter(|(id, _)| !exclude.contains(id)).collect();

    // 1. Wandenden
    let nearest_end = walls
        .iter()
        .flat_map(|(_, w)| [w.start, w.end])
        .map(|q| (q, q.distance(p)))
        .filter(|(_, d)| *d <= tolerance)
        .min_by(|a, b| a.1.total_cmp(&b.1));
    if let Some((q, _)) = nearest_end {
        return Snap { point: q, kind: SnapKind::Endpoint };
    }

    // 2. Achsschnittpunkte
    let grids: Vec<_> = project.grid_lines().map(|(_, g)| g).collect();
    let mut best: Option<(Point2, f64)> = None;
    for (i, a) in grids.iter().enumerate() {
        for b in &grids[i + 1..] {
            let da = unit(a.start, a.end);
            let db = unit(b.start, b.end);
            if let Some(q) = intersect(a.start, da, b.start, db) {
                let d = q.distance(p);
                if d <= tolerance && best.is_none_or(|(_, bd)| d < bd) {
                    best = Some((q, d));
                }
            }
        }
    }
    if let Some((q, _)) = best {
        return Snap { point: q, kind: SnapKind::GridIntersection };
    }

    // 3. Mit Anfangspunkt: Winkel in Schritten, Länge auf 1 cm.
    let angled = from.and_then(|start| {
        let (dx, dy) = (p.x - start.x, p.y - start.y);
        let len = dx.hypot(dy);
        if len < 1e-6 {
            return None;
        }
        let angle = dy.atan2(dx).to_degrees();
        let step = if orthogonal { 90.0 } else { ANGLE_STEP };
        let snapped = (angle / step).round() * step;
        if !orthogonal && (angle - snapped).abs() > ANGLE_CATCH {
            return None;
        }
        let len = (len / ROUND).round() * ROUND;
        let (s, c) = snapped.to_radians().sin_cos();
        Some(Point2::new(start.x + c * len, start.y + s * len))
    });

    // 4. Auf einer Wand oder Achse; mit Winkel: Schnitt der Richtung mit der Wand.
    let mut on_line: Option<(Point2, f64, SnapKind)> = None;
    for (_, w) in walls.iter().filter(|(_, w)| w.is_curved()) {
        let ray = from.zip(angled).map(|(start, q)| (start, unit(start, q)));
        if let Some(q) = on_arc(w, p, ray) {
            let dist = q.distance(p);
            if dist <= tolerance && on_line.is_none_or(|(_, bd, _)| dist < bd) {
                on_line = Some((q, dist, SnapKind::OnWall));
            }
        }
    }
    let lines = walls
        .iter()
        .filter(|(_, w)| !w.is_curved())
        .map(|(_, w)| (w.start, w.end, SnapKind::OnWall))
        .chain(grids.iter().map(|g| (g.start, g.end, SnapKind::Grid)));
    for (a, b, kind) in lines {
        let d = unit(a, b);
        let len = a.distance(b);
        let candidate = match (from, angled) {
            (Some(start), Some(q)) => intersect(start, unit(start, q), a, d),
            _ => {
                let s = ((p.x - a.x) * d.0 + (p.y - a.y) * d.1).clamp(0.0, len);
                Some(Point2::new(a.x + d.0 * s, a.y + d.1 * s))
            }
        };
        let Some(q) = candidate else { continue };
        let s = (q.x - a.x) * d.0 + (q.y - a.y) * d.1;
        if s < -1e-9 || s > len + 1e-9 {
            continue;
        }
        let dist = q.distance(p);
        if dist <= tolerance && on_line.is_none_or(|(_, bd, _)| dist < bd) {
            on_line = Some((q, dist, kind));
        }
    }
    if let Some((q, _, kind)) = on_line {
        return Snap { point: q, kind };
    }
    if let Some(q) = angled {
        return Snap { point: q, kind: SnapKind::Angle };
    }
    Snap { point: Point2::new((p.x / ROUND).round() * ROUND, (p.y / ROUND).round() * ROUND), kind: SnapKind::Free }
}

/// Punkt auf dem Bogen einer gebogenen Wand: der nächste zu `p`, oder mit
/// `ray` (Anfang, Richtung) der Schnitt dieses Strahls mit dem Bogen.
fn on_arc(w: &lot_model::Wall, p: Point2, ray: Option<(Point2, (f64, f64))>) -> Option<Point2> {
    let arc = w.arc()?;
    let on = |q: Point2| {
        let (s, _) = w.locate(q);
        (-1e-9..=w.length() + 1e-9).contains(&s)
    };
    match ray {
        None => {
            let (s, _) = w.locate(p);
            Some(w.point_at(s.clamp(0.0, w.length()), 0.0))
        }
        Some((start, d)) => {
            // |start + u d - c|² = r²
            let (ox, oy) = (start.x - arc.center.x, start.y - arc.center.y);
            let b = d.0 * ox + d.1 * oy;
            let disc = b * b - (ox * ox + oy * oy - arc.radius * arc.radius);
            if disc < 0.0 {
                return None;
            }
            [-b - disc.sqrt(), -b + disc.sqrt()]
                .into_iter()
                .map(|u| Point2::new(start.x + d.0 * u, start.y + d.1 * u))
                .filter(|q| on(*q))
                .min_by(|a, b| a.distance(p).total_cmp(&b.distance(p)))
        }
    }
}

fn unit(a: Point2, b: Point2) -> (f64, f64) {
    let len = a.distance(b).max(1e-12);
    ((b.x - a.x) / len, (b.y - a.y) / len)
}

/// Ob der Punkt auf der Bezugslinie der Wand liegt (für Tests).
pub fn on_reference(project: &Project, wall: ElementId, p: Point2) -> bool {
    project.wall(wall).is_some_and(|w| {
        let (s, t) = local(w, p);
        t.abs() < 1e-6 && s >= -1e-6 && s <= w.length() + 1e-6
    })
}
