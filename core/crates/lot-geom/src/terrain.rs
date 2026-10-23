//! Gelände (1.12) als Körper: das Erdreich unter der Geländeoberfläche, ohne
//! die Baugrube der Geschosse, die unter das Gelände reichen.

use lot_model::{Project, Terrain};
use manifold_rust::linalg::Vec3;
use manifold_rust::manifold::Manifold;
use manifold_rust::types::OpType;

use crate::poly::Area;

/// Material des Erdreichs im Bürostandard.
pub const TERRAIN_MATERIAL: &str = "erdreich";
/// So weit reicht das Erdreich unter den tiefsten Punkt (Gelände oder Geschoss), in m.
const DEPTH: f64 = 1.0;

/// Unterkante des Erdreichs über ±0,00.
pub fn terrain_base(project: &Project, terrain: &Terrain) -> f64 {
    let lowest_storey = project.storeys().first().map_or(0.0, |(_, s)| s.elevation);
    let lowest_point = terrain.z_range().map_or(0.0, |r| r.0);
    lowest_storey.min(lowest_point) - DEPTH
}

/// Erdreich als Körper: je Dreieck ein Prisma unter seiner Ebene, vereinigt,
/// abzüglich der Grundfläche aller Geschosse, die unter das Gelände reichen.
pub fn terrain_solid(project: &Project, terrain: &Terrain) -> Option<Manifold> {
    let triangles = terrain.triangles();
    let (_, top) = terrain.z_range()?;
    let base = terrain_base(project, terrain);
    let prisms: Vec<Manifold> = triangles
        .iter()
        .filter_map(|t| {
            let [a, b, c] = t.map(|i| terrain.points[i]);
            let ring = [[a.x, a.y], [b.x, b.y], [c.x, c.y]];
            let prism = crate::extrude(&Area::from_ring(&ring), base, top + 1.0);
            // Normale der Dreiecksebene, nach oben gerichtet.
            let (u, v) = ([b.x - a.x, b.y - a.y, b.z - a.z], [c.x - a.x, c.y - a.y, c.z - a.z]);
            let mut n = [u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0]];
            if n[2] < 0.0 {
                n = n.map(|x| -x);
            }
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            if len < 1e-12 {
                return None;
            }
            let n = n.map(|x| x / len);
            // Darunter: (p − a) · n ≤ 0  ⇔  p · (−n) ≥ −a · n.
            let offset = -(a.x * n[0] + a.y * n[1] + a.z * n[2]);
            Some(prism.trim_by_plane(Vec3::new(-n[0], -n[1], -n[2]), offset))
        })
        .filter(|m| !m.is_empty())
        .collect();
    if prisms.is_empty() {
        return None;
    }
    let earth = Manifold::batch_boolean(&prisms, OpType::Add);
    // Baugrube: Umriss der Geschosse unter dem Gelände, ohne innere Löcher.
    let pits: Vec<Area> = project
        .storeys()
        .iter()
        .filter(|(_, s)| s.elevation < top)
        .map(|(id, s)| {
            let regions = crate::solid_regions(project, *id, s.elevation + 0.05);
            let all = Area::union_all(regions.iter().map(|r| &r.area));
            let outer: Vec<Area> =
                all.rings().into_iter().filter(|r| crate::signed_area(r) > 0.0).map(|r| Area::from_ring(&r)).collect();
            Area::union_all(outer.iter())
        })
        .collect();
    let pit = Area::union_all(pits.iter());
    if pit.is_empty() {
        return Some(earth);
    }
    Some(earth.difference(&crate::extrude(&pit, base - 1.0, top + 2.0)))
}

#[cfg(test)]
mod tests {
    use lot_model::{Element, ElementId, TerrainPoint};

    use super::*;

    #[test]
    fn earth_lies_under_the_terrain_and_leaves_the_building_out() {
        let mut p = Project::sample();
        let pt = |x, y, z| TerrainPoint { x, y, z };
        let terrain =
            Terrain { points: vec![pt(-5.0, -5.0, 0.5), pt(10.0, -5.0, 0.5), pt(10.0, 5.0, 0.5), pt(-5.0, 5.0, 0.5)] };
        p.elements.insert(ElementId(900), Element::Terrain(terrain.clone()));
        let solid = terrain_solid(&p, &terrain).unwrap();
        let (lo, hi) = crate::to_trimesh(&solid).bounds().unwrap();
        assert!((hi[2] - 0.5).abs() < 1e-9, "{hi:?}");
        // 1 m unter dem Geschoss auf ±0,00.
        assert!((lo[2] + 1.0).abs() < 1e-9, "{lo:?}");
        // Das Geschoss liegt unter dem Gelände; wo die Wand steht, fehlt das Erdreich.
        let full = 15.0 * 10.0 * 1.5;
        assert!(solid.volume() < full - 1.0, "{}", solid.volume());
    }
}
