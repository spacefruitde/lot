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

/// Grundfläche einer Decke ohne ihre Durchbrüche.
pub fn slab_area(project: &Project, id: ElementId, slab: &Slab) -> Area {
    let ring: Ring = slab.outline.iter().map(|p| [p.x, p.y]).collect();
    let holes: Vec<Area> = project
        .slab_openings_in(id)
        .map(|(_, o)| Area::from_ring(&o.outline.iter().map(|p| [p.x, p.y]).collect::<Ring>()))
        .collect();
    Area::from_ring(&ring).difference(&Area::union_all(holes.iter()))
}

/// Körper je Schicht einer Decke, von oben nach unten: (Schicht, Material, Körper).
pub fn slab_solids(project: &Project, id: ElementId, slab: &Slab) -> Vec<(usize, String, Manifold)> {
    let (Some(storey), Some(slab_type)) = (project.storey(slab.storey), project.slab_type(slab.slab_type)) else {
        return Vec::new();
    };
    let area = slab_area(project, id, slab);
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
