//! Dächer: Schichtkörper für das 3D und Bereiche nach lichter Höhe.

use lot_model::{ElementId, Project, Roof};
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

/// Körper je Schicht eines Dachs, von oben nach unten: (Schicht, Material,
/// Körper). Gauben und Dachfenster sind herausgeschnitten.
pub fn roof_solids(project: &Project, id: ElementId, roof: &Roof) -> Vec<(usize, String, Manifold)> {
    let (Some(storey), Some(roof_type)) = (project.storey(roof.storey), project.slab_type(roof.roof_type)) else {
        return Vec::new();
    };
    let base = storey.elevation + roof.eave_height;
    let area = Area::from_ring(&ring(&roof.outer_outline()));
    let cuts = crate::dormers::roof_cuts(project, id);
    let flat = roof.eave_planes().is_empty();
    let mut depth = 0.0;
    roof_type
        .layers
        .iter()
        .enumerate()
        .map(|(k, layer)| {
            let solid = if flat {
                // Flachdach: Schicht direkt hochziehen. Die Differenz zweier Körper mit
                // gemeinsamen Seiten- und Bodenflächen lässt in Manifold Reste stehen.
                let s = crate::extrude(&area, base - depth - layer.thickness, base - depth);
                depth += layer.thickness;
                s
            } else {
                let top = below(roof, base, depth, &area);
                depth += layer.thickness;
                let bottom = below(roof, base, depth, &area);
                top.difference(&bottom)
            };
            let solid = match &cuts {
                Some(c) => solid.difference(c),
                None => solid,
            };
            (k, layer.material.clone(), solid)
        })
        .collect()
}

/// Raum unter der Dachunterseite und unter den Gauben, für Wände, die bis
/// unter das Dach reichen.
pub fn under_roof(project: &Project, id: ElementId, roof: &Roof) -> Option<Manifold> {
    let storey = project.storey(roof.storey)?;
    let area = Area::from_ring(&ring(&roof.outer_outline()));
    let space = below(roof, storey.elevation + roof.eave_height, project.roof_thickness(roof), &area);
    Some(match crate::dormers::under_dormers(project, id) {
        Some(d) => space.union(&d),
        None => space,
    })
}

/// Bereich im Grundriss, in dem die Dachunterseite mindestens `height` über
/// OKFF liegt (für die Anrechnung nach WoFlV § 4 und die Höhenlinien im Plan),
/// einschließlich der Bereiche unter Gauben.
pub fn clear_height_region(project: &Project, id: ElementId, roof: &Roof, height: f64) -> Area {
    roof_clear_region(project, roof, height).union(&crate::dormers::dormer_clear_region(project, id, height))
}

fn roof_clear_region(project: &Project, roof: &Roof, height: f64) -> Area {
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
