//! Bearbeiten-Befehle für Wände (ADR 0044): Teilen, Stutzen und Dehnen zur
//! Ecke, Ausrichten an einer Linie.

use lot_model::{DimRef, Element, ElementId, Operation, Point2, Project, WallEnd};

use crate::geometry::{self, intersect};
use crate::snap::ROUND;

/// Teilt eine gerade Wand am Punkt `p` (auf 1 cm gerundet). Der vordere Teil
/// behält die Kennung, der hintere wird neu; Öffnungen bleiben auf ihrer
/// Seite, Maßketten am Wandende gehen mit dem hinteren Teil.
pub fn split_wall_ops(project: &Project, id: ElementId, p: Point2) -> Result<Vec<Operation>, String> {
    let wall = project.wall(id).ok_or("Hier ist keine Wand")?;
    if wall.is_curved() {
        return Err("Bogenwände lassen sich noch nicht teilen".into());
    }
    let length = wall.length();
    let s = ((wall.locate(p).0 / ROUND).round() * ROUND).clamp(0.0, length);
    if s < 0.05 || s > length - 0.05 {
        return Err("Zum Teilen nicht so nah am Wandende klicken".into());
    }
    let openings: Vec<_> = project.openings_in(id).map(|(oid, o)| (oid, o.clone())).collect();
    if openings.iter().any(|(_, o)| o.offset < s && o.offset + o.width > s) {
        return Err("Teilen geht nicht durch eine Öffnung".into());
    }
    let cut = wall.point_at(s, 0.0);
    let new = project.next_id();
    let mut front = wall.clone();
    front.end = cut;
    let mut back = wall.clone();
    back.start = cut;
    let mut ops = vec![
        Operation::Replace { id, old: Element::Wall(wall.clone()), new: Element::Wall(front) },
        Operation::Insert { id: new, element: Element::Wall(back) },
    ];
    for (oid, o) in openings.into_iter().filter(|(_, o)| o.offset >= s) {
        let mut n = o.clone();
        n.host = new;
        n.offset = o.offset - s;
        ops.push(Operation::Replace { id: oid, old: Element::Opening(o), new: Element::Opening(n) });
    }
    for (did, e) in &project.elements {
        let Element::DimensionChain(d) = e else { continue };
        let did = *did;
        let mut n = d.clone();
        for r in &mut n.refs {
            if let DimRef::WallEnd { wall: w, end: WallEnd::End, .. } = r
                && *w == id
            {
                *w = new;
            }
        }
        if n != *d {
            ops.push(Operation::Replace {
                id: did,
                old: Element::DimensionChain(d.clone()),
                new: Element::DimensionChain(n),
            });
        }
    }
    Ok(ops)
}

/// Stutzen oder Dehnen zweier gerader Wände zu einer Ecke (TR wie Revit):
/// Beide enden im Schnittpunkt ihrer Bezugslinien; von jeder Wand bleibt der
/// Teil, auf den geklickt wurde.
pub fn trim_ops(project: &Project, a: (ElementId, Point2), b: (ElementId, Point2)) -> Result<Vec<Operation>, String> {
    if a.0 == b.0 {
        return Err("Zwei verschiedene Wände anklicken".into());
    }
    let (wa, wb) = (project.wall(a.0).ok_or("Hier ist keine Wand")?, project.wall(b.0).ok_or("Hier ist keine Wand")?);
    if wa.is_curved() || wb.is_curved() {
        return Err("Bogenwände lassen sich noch nicht stutzen".into());
    }
    let (da, db) = (wa.direction(), wb.direction());
    let corner = intersect(wa.start, da, wb.start, db).ok_or("Die Wände sind parallel")?;
    let mut ops = Vec::new();
    for ((id, clicked), w) in [(a, wa), (b, wb)] {
        let at = w.locate(corner).0;
        let keep = w.locate(clicked).0;
        let mut n = w.clone();
        if keep < at {
            n.end = corner;
        } else {
            n.start = corner;
        }
        if n.start.distance(n.end) < 0.01 {
            return Err("Danach bliebe von einer Wand nichts übrig".into());
        }
        ops.push(Operation::Replace { id, old: Element::Wall(w.clone()), new: Element::Wall(n) });
    }
    Ok(ops)
}

/// Eine Linie zum Ausrichten: ein Punkt darauf und die Richtung (Einheitsvektor).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Line {
    pub point: Point2,
    pub dir: (f64, f64),
}

impl Line {
    fn distance(&self, p: Point2) -> f64 {
        ((p.x - self.point.x) * self.dir.1 - (p.y - self.point.y) * self.dir.0).abs()
    }
}

/// Die Linie unter dem Zeiger, an der sich etwas ausrichten lässt: Flächen
/// und Bezugslinien gerader Wände, Achsen, 2D-Linien; mit dem Element, zu dem sie gehört.
pub fn line_at(project: &Project, storey: ElementId, p: Point2, tolerance: f64) -> Option<(ElementId, Line)> {
    let mut best: Option<(f64, ElementId, Line)> = None;
    let mut consider = |id: ElementId, line: Line, along_ok: bool| {
        let d = line.distance(p);
        if along_ok && d <= tolerance && best.as_ref().is_none_or(|b| d < b.0) {
            best = Some((d, id, line));
        }
    };
    for (id, w) in project.walls_on(storey).filter(|(_, w)| !w.is_curved()) {
        let (s, _) = w.locate(p);
        let along = s >= -tolerance && s <= w.length() + tolerance;
        let (left, right) = geometry::faces(project, w);
        let (dx, dy) = w.direction();
        for t in [left, right, 0.0] {
            consider(id, Line { point: w.point_at(0.0, t), dir: (dx, dy) }, along);
        }
    }
    for (id, g) in project.grid_lines() {
        let len = g.start.distance(g.end).max(1e-12);
        let dir = ((g.end.x - g.start.x) / len, (g.end.y - g.start.y) / len);
        consider(id, Line { point: g.start, dir }, true);
    }
    for (id, a) in project.annotations_in(lot_model::AnnotationView::FloorPlan { storey }) {
        if let lot_model::AnnotationShape::Polyline(pts) = &a.shape {
            for seg in pts.windows(2) {
                let len = seg[0].distance(seg[1]);
                if len < 1e-9 {
                    continue;
                }
                let dir = ((seg[1].x - seg[0].x) / len, (seg[1].y - seg[0].y) / len);
                let s = (p.x - seg[0].x) * dir.0 + (p.y - seg[0].y) * dir.1;
                consider(id, Line { point: seg[0], dir }, s >= -tolerance && s <= len + tolerance);
            }
        }
    }
    best.map(|(_, id, l)| (id, l))
}

/// Verschiebung, die `target` auf die Linie `reference` legt; beide müssen parallel sein.
pub fn align_delta(reference: Line, target: Line) -> Result<(f64, f64), String> {
    let cross = reference.dir.0 * target.dir.1 - reference.dir.1 * target.dir.0;
    if cross.abs() > 0.5f64.to_radians().sin() {
        return Err("Ausrichten geht nur an einer parallelen Linie".into());
    }
    // Lot vom Zielpunkt auf die Bezugslinie.
    let (nx, ny) = (-reference.dir.1, reference.dir.0);
    let d = (reference.point.x - target.point.x) * nx + (reference.point.y - target.point.y) * ny;
    Ok((nx * d, ny * d))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn align_moves_perpendicular_only() {
        let reference = Line { point: Point2::new(0.0, 2.0), dir: (1.0, 0.0) };
        let target = Line { point: Point2::new(5.0, 2.5), dir: (-1.0, 0.0) };
        let (dx, dy) = align_delta(reference, target).unwrap();
        assert!(dx.abs() < 1e-12 && (dy + 0.5).abs() < 1e-12);
        let skew = Line { point: Point2::new(0.0, 0.0), dir: (0.0, 1.0) };
        assert!(align_delta(reference, skew).is_err());
    }
}
