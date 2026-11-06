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

/// Höhen einer Decke über ±0,00: Oberkante, Ober- und Unterkante der
/// Tragschicht (Rohdecke), Unterkante. Ohne Tragschicht gilt die ganze Decke.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloorLevels {
    pub top: f64,
    pub structure_top: f64,
    pub structure_bottom: f64,
    pub bottom: f64,
}

pub fn floor_levels(project: &Project, slab: &Slab) -> Option<FloorLevels> {
    let storey = project.storey(slab.storey)?;
    let slab_type = project.slab_type(slab.slab_type)?;
    let top = storey.elevation + slab.top_offset;
    let bottom = top - slab_type.thickness();
    let mut depth = 0.0;
    let mut structure: Option<(f64, f64)> = None;
    for l in &slab_type.layers {
        if l.function == LayerFunction::Structure {
            structure = Some((structure.map_or(top - depth, |s| s.0), top - depth - l.thickness));
        }
        depth += l.thickness;
    }
    let (structure_top, structure_bottom) = structure.unwrap_or((top, bottom));
    Some(FloorLevels { top, structure_top, structure_bottom, bottom })
}

/// Körper je Schicht einer Decke, von oben nach unten: (Schicht, Material, Körper).
///
/// Schichten nach Bauablauf (ADR 0045): Die Rohdecke liegt auf den Wänden
/// auf. Estrich und Trittschalldämmung darüber stoßen an den Innenputz der
/// Wände, die auf der Decke stehen; der Deckenputz darunter endet am
/// Innenputz der Wände, die bis unter die Decke reichen.
pub fn slab_solids(project: &Project, id: ElementId, slab: &Slab) -> Vec<(usize, String, Manifold)> {
    let (Some(storey), Some(slab_type)) = (project.storey(slab.storey), project.slab_type(slab.slab_type)) else {
        return Vec::new();
    };
    let area = slab_area(project, id, slab);
    let structure: Vec<usize> = slab_type
        .layers
        .iter()
        .enumerate()
        .filter(|(_, l)| l.function == LayerFunction::Structure)
        .map(|(k, _)| k)
        .collect();
    let (first, last) = (structure.first().copied(), structure.last().copied());
    let union = |regions: Vec<crate::LayerRegion>| Area::union_all(regions.iter().map(|r| &r.area));
    let mut top = storey.elevation + slab.top_offset;
    // Wände, die auf der Decke stehen; in Türöffnungen läuft der Estrich durch.
    let standing = first.filter(|f| *f > 0).map(|_| union(crate::layer_regions(project, slab.storey, top + FLOOR_CUT)));
    // Wände des Geschosses darunter, die bis unter die Decke reichen.
    let below = project
        .storeys()
        .into_iter()
        .filter(|(_, s)| s.elevation < top - 1e-6)
        .max_by(|a, b| a.1.elevation.total_cmp(&b.1.elevation))
        .map(|(id, _)| id);
    slab_type
        .layers
        .iter()
        .enumerate()
        .map(|(k, layer)| {
            let bottom = top - layer.thickness;
            let layer_area = match (first, last) {
                (Some(f), _) if k < f => standing.as_ref().map_or_else(|| area.clone(), |w| area.difference(w)),
                (_, Some(l)) if k > l => match below {
                    Some(b) => area.difference(&union(solid_regions(project, b, (top + bottom) / 2.0))),
                    None => area.clone(),
                },
                _ => area.clone(),
            };
            let solid = crate::extrude(&layer_area, bottom, top);
            top = bottom;
            (k, layer.material.clone(), solid)
        })
        .collect()
}
