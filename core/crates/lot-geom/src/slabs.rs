//! Decken und Bodenplatten: Umriss aus den Wänden und Körper für das 3D.

use lot_model::{ElementId, LayerFunction, Project, Slab};
use manifold_rust::manifold::Manifold;

use crate::joins::solid_regions;
use crate::poly::{Area, Ring};

/// Höhe über OKFF, in der die Wände für den Umriss geschnitten werden, in Metern.
const FLOOR_CUT: f64 = 0.05;

/// Umrisse für Decken eines Geschosses: die Außenkanten der Tragschichten aller
/// Wände, je zusammenhängendem Gebäudeteil ein Ring. Schichten außerhalb der
/// Tragschicht (Dämmung, Außenputz, Vormauerschale) liegen außerhalb, so wie
/// eine Decke auf dem Mauerwerk aufliegt und die Dämmung vor der Stirn
/// durchläuft. Wände ohne Tragschicht zählen ganz.
pub fn slab_outlines(project: &Project, storey: ElementId) -> Vec<Ring> {
    let z = project.storey(storey).map_or(0.0, |s| s.elevation) + FLOOR_CUT;
    let regions = solid_regions(project, storey, z);
    let inner: Vec<&Area> = regions
        .iter()
        .filter(|r| {
            let Some(wall) = project.wall(r.element) else { return true };
            let Some(t) = project.wall_type(wall.wall_type) else { return true };
            let first = t.layers.iter().position(|l| l.function == LayerFunction::Structure).unwrap_or(0);
            r.layer >= first
        })
        .map(|r| &r.area)
        .collect();
    Area::union_all(inner).rings().into_iter().filter(|r| crate::signed_area(r) > 0.0).collect()
}

/// Körper je Schicht einer Decke, von oben nach unten: (Schicht, Material, Körper).
pub fn slab_solids(project: &Project, slab: &Slab) -> Vec<(usize, String, Manifold)> {
    let (Some(storey), Some(slab_type)) = (project.storey(slab.storey), project.slab_type(slab.slab_type)) else {
        return Vec::new();
    };
    let ring: Ring = slab.outline.iter().map(|p| [p.x, p.y]).collect();
    let area = Area::from_ring(&ring);
    let mut top = storey.elevation + slab.top_offset;
    slab_type
        .layers
        .iter()
        .enumerate()
        .map(|(k, layer)| {
            let bottom = top - layer.thickness;
            let solid = crate::extrude(&area, bottom, top);
            top = bottom;
            (k, layer.material.clone(), solid)
        })
        .collect()
}
