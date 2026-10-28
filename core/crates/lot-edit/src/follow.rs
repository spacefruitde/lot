//! Nachführen (ADR 0051): Decken, die den Wänden folgen, bekommen nach jeder
//! Änderung den Umriss an der Außenkante der Tragschicht der Wände darunter.
//! Gelingt das nicht (Wände nicht geschlossen), bleibt der letzte Umriss, und
//! die Decke zeigt es an, statt zu verschwinden.

use lot_geom::Area;
use lot_model::{Element, ElementId, Operation, Point2, Project, Slab, SlabKind};

/// Geschoss, dessen Wände die Decke tragen: bei der Bodenplatte das eigene, sonst das darunter.
pub fn support_storey(project: &Project, slab: &Slab) -> Option<ElementId> {
    if slab.kind == SlabKind::Base {
        return Some(slab.storey);
    }
    let storeys = project.storeys();
    let i = storeys.iter().position(|(id, _)| *id == slab.storey)?;
    i.checked_sub(1).map(|j| storeys[j].0)
}

fn ring_area(points: &[Point2]) -> Area {
    Area::from_ring(&points.iter().map(|p| [p.x, p.y]).collect::<Vec<_>>())
}

/// Umriss aus den Wänden: der Ring, der den bisherigen Umriss am meisten überdeckt.
pub fn derived_outline(project: &Project, slab: &Slab) -> Option<Vec<Point2>> {
    let support = support_storey(project, slab)?;
    let current = ring_area(&slab.outline);
    lot_geom::slabs::slab_outlines(project, support)
        .into_iter()
        .map(|ring| {
            let points: Vec<Point2> = ring.iter().map(|p| Point2::new(p[0], p[1])).collect();
            let overlap = ring_area(&points).intersection(&current).area();
            (overlap, points)
        })
        .filter(|(overlap, _)| *overlap > 1e-6)
        .max_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, points)| points)
}

/// Ob zwei Umrisse gleich sind (gleiche Punkte in gleicher Folge, auf 1 mm).
fn same(a: &[Point2], b: &[Point2]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(p, q)| p.distance(*q) < 1e-3)
}

/// Neuer Umriss einer folgenden Decke, wenn er aus denselben Wänden entsteht,
/// denen sie folgt; sonst `None` (die Wände sind offen, eine fehlt oder eine kam dazu).
pub fn followed_outline(project: &Project, slab: &Slab) -> Option<Vec<Point2>> {
    let outline = derived_outline(project, slab)?;
    if slab.boundary_walls.is_empty() {
        return Some(outline);
    }
    let probe = Slab { outline: outline.clone(), ..slab.clone() };
    let mut walls: Vec<ElementId> = edge_walls(project, &probe).into_iter().flatten().collect();
    walls.sort();
    walls.dedup();
    let mut expected = slab.boundary_walls.clone();
    expected.sort();
    expected.dedup();
    (walls == expected).then_some(outline)
}

/// Alles, was nach einer Änderung nachgeführt wird: Decken, die den Wänden
/// folgen, Treppenlöcher und ihre Umwehrung. Für [`lot_model::Document::set_follow`].
pub fn follow_ops(project: &Project) -> Vec<Operation> {
    let mut ops = slab_ops(project);
    ops.extend(crate::stairs::follow_ops(project));
    ops
}

/// Operationen, die alle folgenden Decken auf den Umriss ihrer Wände bringen.
fn slab_ops(project: &Project) -> Vec<Operation> {
    project
        .elements
        .iter()
        .filter_map(|(id, e)| match e {
            Element::Slab(s) if s.follows_walls => {
                let outline = followed_outline(project, s)?;
                if same(&outline, &s.outline) {
                    return None;
                }
                let mut new = s.clone();
                new.outline = outline;
                Some(Operation::Replace { id: *id, old: e.clone(), new: Element::Slab(new) })
            }
            _ => None,
        })
        .collect()
}

/// Die Decke soll den Wänden folgen, kann es aber nicht.
pub fn broken(project: &Project, slab: &Slab) -> bool {
    slab.follows_walls && followed_outline(project, slab).is_none()
}

/// Übernimmt den Umriss neu aus den Wänden, die ihn jetzt umschließen, und
/// folgt danach diesen Wänden; `None`, wenn keine Wände die Decke umschließen.
pub fn refollow(project: &Project, slab: &Slab) -> Option<Slab> {
    let outline = derived_outline(project, slab)?;
    let mut new = Slab { outline, follows_walls: true, ..slab.clone() };
    let mut walls: Vec<ElementId> = edge_walls(project, &new).into_iter().flatten().collect();
    walls.sort();
    walls.dedup();
    new.boundary_walls = walls;
    Some(new)
}

/// Je Kante des Umrisses die Wand, deren Tragschicht dort endet; `None` bei freien Kanten.
pub fn edge_walls(project: &Project, slab: &Slab) -> Vec<Option<ElementId>> {
    let n = slab.outline.len();
    let Some(support) = support_storey(project, slab) else { return vec![None; n] };
    let z = project.storey(support).map_or(0.0, |s| s.elevation) + 0.05;
    let regions = lot_geom::solid_regions(project, support, z);
    let area = ring_area(&slab.outline);
    (0..n)
        .map(|i| {
            let (a, b) = (slab.outline[i], slab.outline[(i + 1) % n]);
            let len = a.distance(b).max(1e-9);
            let m = Point2::new((a.x + b.x) / 2.0, (a.y + b.y) / 2.0);
            let normal = (-(b.y - a.y) / len, (b.x - a.x) / len);
            // Ein Zentimeter in die Decke hinein liegt die Tragschicht der Wand.
            let inside = [1.0, -1.0]
                .into_iter()
                .map(|s| [m.x + normal.0 * 0.01 * s, m.y + normal.1 * 0.01 * s])
                .find(|p| area.contains(*p))?;
            regions.iter().find(|r| project.wall(r.element).is_some() && r.area.contains(inside)).map(|r| r.element)
        })
        .collect()
}

/// Die Decke mit den Wänden ihres Umrisses als die, denen sie folgt.
pub fn with_boundary(project: &Project, slab: Slab) -> Slab {
    let mut walls: Vec<ElementId> = edge_walls(project, &slab).into_iter().flatten().collect();
    walls.sort();
    walls.dedup();
    Slab { boundary_walls: walls, ..slab }
}
