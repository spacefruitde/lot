//! Dächer: Schichtkörper für das 3D und Bereiche nach lichter Höhe.

use lot_model::{Project, Roof};
use manifold_rust::linalg::Vec3;
use manifold_rust::manifold::Manifold;

use crate::poly::{Area, Ring, strip};

/// Reichweite der Hilfskörper nach unten und oben, in Metern.
const FAR: f64 = 200.0;

fn ring(points: &[lot_model::Point2]) -> Ring {
    points.iter().map(|p| [p.x, p.y]).collect()
}

/// Alles unterhalb der Ebenen, die um `depth` (senkrecht zur Dachfläche) unter
/// der Dachhaut liegen, innerhalb des Dachumrisses mit Überstand. `base` ist
/// die Höhe der Traufe über ±0,00.
fn below(roof: &Roof, base: f64, depth: f64, area: &Area) -> Manifold {
    let mut solid = crate::extrude(area, base - FAR, base + FAR);
    let planes = roof.eave_planes();
    if planes.is_empty() {
        return solid.trim_by_plane(Vec3::new(0.0, 0.0, -1.0), -(base - depth));
    }
    for e in planes {
        // z ≤ base − depth / cos + tan · ((p − o) · n)  ⇔  tan · (p · n) − z ≥ depth / cos − base + tan · (o · n)
        let n = Vec3::new(e.tan * e.inward.0, e.tan * e.inward.1, -1.0);
        let len = (n.x * n.x + n.y * n.y + n.z * n.z).sqrt();
        let offset = depth / e.cos - base + e.tan * (e.origin.x * e.inward.0 + e.origin.y * e.inward.1);
        solid = solid.trim_by_plane(Vec3::new(n.x / len, n.y / len, n.z / len), offset / len);
    }
    solid
}

/// Körper je Schicht eines Dachs, von oben nach unten: (Schicht, Material, Körper).
pub fn roof_solids(project: &Project, roof: &Roof) -> Vec<(usize, String, Manifold)> {
    let (Some(storey), Some(roof_type)) = (project.storey(roof.storey), project.slab_type(roof.roof_type)) else {
        return Vec::new();
    };
    let base = storey.elevation + roof.eave_height;
    let area = Area::from_ring(&ring(&roof.outer_outline()));
    let mut depth = 0.0;
    roof_type
        .layers
        .iter()
        .enumerate()
        .map(|(k, layer)| {
            let top = below(roof, base, depth, &area);
            depth += layer.thickness;
            let bottom = below(roof, base, depth, &area);
            (k, layer.material.clone(), top.difference(&bottom))
        })
        .collect()
}

/// Raum unter der Dachunterseite, für Wände, die bis unter das Dach reichen.
pub fn under_roof(project: &Project, roof: &Roof) -> Option<Manifold> {
    let storey = project.storey(roof.storey)?;
    let area = Area::from_ring(&ring(&roof.outer_outline()));
    Some(below(roof, storey.elevation + roof.eave_height, project.roof_thickness(roof), &area))
}

/// Bereich im Grundriss, in dem die Dachunterseite mindestens `height` über
/// OKFF liegt (für die Anrechnung nach WoFlV § 4 und die Höhenlinien im Plan).
pub fn clear_height_region(project: &Project, roof: &Roof, height: f64) -> Area {
    let thickness = project.roof_thickness(roof);
    let mut area = Area::from_ring(&ring(&roof.outer_outline()));
    for e in roof.eave_planes() {
        // underside ≥ height  ⇔  d ≥ (height − eave_height + thickness / cos) / tan
        if e.tan.abs() < 1e-12 {
            if roof.eave_height - thickness < height {
                return Area::empty();
            }
            continue;
        }
        let d = (height - roof.eave_height + thickness / e.cos) / e.tan;
        let along = [-e.inward.1, e.inward.0];
        let origin = [e.origin.x + e.inward.0 * d, e.origin.y + e.inward.1 * d];
        let half = strip(origin, along, (-500.0, 500.0), (0.0, -500.0));
        area = area.intersection(&half);
    }
    if roof.eave_planes().is_empty() && roof.eave_height - thickness < height {
        return Area::empty();
    }
    area
}
