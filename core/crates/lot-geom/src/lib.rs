//! Geometrie: aus Parametern des Modells werden Schichtflächen, Volumen und Netze.
//!
//! Geometrie ist immer abgeleitet und wird nie gespeichert (CLAUDE.md, Regel 6).
//!
//! - [`joins`]: Wandanschlüsse im Grundriss, je Schicht eine Fläche
//! - [`spaces`]: Räume als von Wänden umschlossene Flächen
//! - [`slabs`]: Decken und Bodenplatten
//! - [`roofs`]: Dächer und lichte Höhen darunter
//! - [`stairs`]: Treppen und Geländer
//! - [`dormers`]: Gauben und Dachfenster
//! - [`build`]: 3D-Körper aus den Schichtflächen, Höhenabschnitt für
//!   Höhenabschnitt hochgezogen und mit manifold-rust vereinigt (ADR 0004)

pub mod dimensions;
pub mod dormers;
pub mod joins;
mod mesh;
pub mod openings;
pub mod poly;
pub mod roofs;
pub mod slabs;
pub mod spaces;
pub mod stairs;

use std::collections::BTreeMap;

pub use joins::{Band, LayerRegion, column_area, layer_regions, solid_regions, wall_section};
use lot_model::{Element, ElementId, Project};
use manifold_rust::linalg::{Vec2, Vec3};
use manifold_rust::manifold::Manifold;
use manifold_rust::types::OpType;
pub use mesh::{FlatVertex, TriMesh, signed_area};
pub use poly::{Area, Ring};

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum GeomError {
    #[error("Die Geometrie von Element {0:?} ist leer")]
    Empty(ElementId),
}

/// Geometrie einer Schicht eines Bauteils (oder eines ganzen Bauteils ohne Schichten).
#[derive(Debug, Clone, PartialEq)]
pub struct ElementGeometry {
    pub id: ElementId,
    pub layer: usize,
    /// Schlüssel des Materials im Bürostandard.
    pub material: String,
    pub mesh: TriMesh,
}

/// Geometrie aller sichtbaren Elemente eines Projekts.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ModelGeometry {
    pub elements: Vec<ElementGeometry>,
}

impl ModelGeometry {
    pub fn bounds(&self) -> Option<([f64; 3], [f64; 3])> {
        self.elements.iter().filter_map(|e| e.mesh.bounds()).reduce(|(a_lo, a_hi), (b_lo, b_hi)| {
            (
                [a_lo[0].min(b_lo[0]), a_lo[1].min(b_lo[1]), a_lo[2].min(b_lo[2])],
                [a_hi[0].max(b_hi[0]), a_hi[1].max(b_hi[1]), a_hi[2].max(b_hi[2])],
            )
        })
    }

    pub fn triangle_count(&self) -> usize {
        self.elements.iter().map(|e| e.mesh.triangles.len()).sum()
    }
}

/// Rechnet die 3D-Geometrie aller Wände und Stützen.
///
/// Jedes Geschoss wird an allen Höhen geteilt, an denen sich etwas ändert
/// (Unter- und Oberkanten, Brüstungen, Stürze). In jedem Abschnitt gelten
/// dieselben Anschlüsse wie im Grundriss; die Schichtflächen werden über den
/// Abschnitt hochgezogen und je Schicht vereinigt. Übereinanderliegende
/// Abschnitte mit gleicher Fläche werden vorher zu einem zusammengefasst,
/// so bleibt die teure Vereinigung den Wänden mit Öffnungen vorbehalten.
pub fn build(project: &Project) -> Result<ModelGeometry, GeomError> {
    let mut bands = Vec::new();
    for (storey, _) in project.storeys() {
        let heights = heights(project, storey);
        bands.extend(heights.windows(2).filter(|p| p[1] - p[0] >= 1e-6).map(|p| (storey, p[0], p[1])));
    }
    // Die Abschnitte sind unabhängig voneinander und werden parallel gerechnet.
    let regions = parallel_map(&bands, |&(storey, z0, z1)| layer_regions(project, storey, (z0 + z1) / 2.0));
    let mut stacks: BTreeMap<(ElementId, usize), (String, Vec<Extrusion>)> = BTreeMap::new();
    for (&(_, z0, z1), band) in bands.iter().zip(regions) {
        for region in band {
            let parts =
                &mut stacks.entry((region.element, region.layer)).or_insert_with(|| (region.material, Vec::new())).1;
            match parts.last_mut() {
                Some(last) if (last.1 - z0).abs() < 1e-9 && last.2.same_shape(&region.area) => last.1 = z1,
                _ => parts.push((z0, z1, region.area)),
            }
        }
    }
    let stacks: Vec<_> = stacks.into_iter().collect();
    let slabs: Vec<_> = project
        .elements
        .iter()
        .filter_map(|(id, e)| match e {
            lot_model::Element::Slab(s) => Some((*id, s)),
            _ => None,
        })
        .collect();
    let slab_solids: Vec<(ElementId, usize, String, Manifold)> = parallel_map(&slabs, |(id, slab)| {
        slabs::slab_solids(project, *id, slab)
            .into_iter()
            .map(|(layer, material, solid)| (*id, layer, material, solid))
            .collect::<Vec<_>>()
    })
    .into_iter()
    .flatten()
    .collect();
    // Wände und Stützen enden unter den Decken, die auf ihnen liegen.
    let all_slabs: Vec<Manifold> = slab_solids.iter().map(|s| s.3.clone()).collect();
    let cut = if all_slabs.is_empty() { None } else { Some(Manifold::batch_boolean(&all_slabs, OpType::Add)) };
    // Dächer, und der Raum unter jedem Dach für Wände, die bis unter das Dach reichen.
    let roofs: Vec<_> = project
        .elements
        .iter()
        .filter_map(|(id, e)| match e {
            lot_model::Element::Roof(r) => Some((*id, r)),
            _ => None,
        })
        .collect();
    let roof_elements: Vec<ElementGeometry> = parallel_map(&roofs, |(id, roof)| {
        roofs::roof_solids(project, *id, roof)
            .into_iter()
            .map(|(layer, material, solid)| ElementGeometry { id: *id, layer, material, mesh: to_trimesh(&solid) })
            .collect::<Vec<_>>()
    })
    .into_iter()
    .flatten()
    .collect();
    let openings: Vec<_> = project
        .elements
        .iter()
        .filter_map(|(id, e)| match e {
            lot_model::Element::Opening(o) => Some((*id, o)),
            _ => None,
        })
        .collect();
    let opening_elements: Vec<ElementGeometry> = parallel_map(&openings, |(id, o)| {
        openings::opening_solids(project, o)
            .into_iter()
            .map(|(layer, material, solid)| ElementGeometry { id: *id, layer, material, mesh: to_trimesh(&solid) })
            .collect::<Vec<_>>()
    })
    .into_iter()
    .flatten()
    .collect();
    // Treppen, Geländer, Gauben und Dachfenster.
    let stairs: Vec<_> = project
        .elements
        .iter()
        .filter(|(_, e)| {
            matches!(
                e,
                lot_model::Element::Stair(_)
                    | lot_model::Element::Railing(_)
                    | lot_model::Element::Dormer(_)
                    | lot_model::Element::RoofWindow(_)
            )
        })
        .collect();
    let stair_elements: Vec<ElementGeometry> = parallel_map(&stairs, |(id, e)| {
        let solids = match e {
            lot_model::Element::Stair(s) => stairs::stair_solids(project, s),
            lot_model::Element::Railing(r) => stairs::railing_solids(project, r),
            lot_model::Element::Dormer(d) => dormers::dormer_solids(project, d),
            lot_model::Element::RoofWindow(w) => dormers::roof_window_solids(project, w),
            _ => Vec::new(),
        };
        solids
            .into_iter()
            .map(|(layer, material, solid)| ElementGeometry { id: **id, layer, material, mesh: to_trimesh(&solid) })
            .collect::<Vec<_>>()
    })
    .into_iter()
    .flatten()
    .collect();
    let under: BTreeMap<ElementId, Manifold> =
        roofs.iter().filter_map(|(id, r)| Some((r.storey, roofs::under_roof(project, *id, r)?))).collect();
    let slab_elements: Vec<ElementGeometry> = slab_solids
        .into_iter()
        .map(|(id, layer, material, solid)| ElementGeometry { id, layer, material, mesh: to_trimesh(&solid) })
        .collect();
    let elements = parallel_map(&stacks, |((id, layer), (material, parts))| {
        let mut solids: Vec<Manifold> =
            parts.iter().map(|(z0, z1, area)| extrude(area, *z0, *z1)).filter(|s| !s.is_empty()).collect();
        let solid = if solids.len() == 1 {
            solids.pop().unwrap_or_default()
        } else {
            Manifold::batch_boolean(&solids, OpType::Add)
        };
        let solid = match &cut {
            Some(cut) => solid.difference(cut),
            None => solid,
        };
        // Bis unter das Dach: oberhalb der Dachunterseite abschneiden.
        let to_roof = match project.elements.get(id) {
            Some(lot_model::Element::Wall(w)) => matches!(w.top, lot_model::Top::Roof { .. }).then_some(w.storey),
            Some(lot_model::Element::Column(c)) => matches!(c.top, lot_model::Top::Roof { .. }).then_some(c.storey),
            _ => None,
        };
        let solid = match to_roof.and_then(|s| under.get(&s)) {
            Some(space) => solid.intersection(space),
            None => solid,
        };
        ElementGeometry { id: *id, layer: *layer, material: material.clone(), mesh: to_trimesh(&solid) }
    });
    Ok(ModelGeometry {
        elements: elements
            .into_iter()
            .chain(slab_elements)
            .chain(roof_elements)
            .chain(opening_elements)
            .chain(stair_elements)
            .filter(|e| !e.mesh.is_empty())
            .collect(),
    })
}

/// `f` auf alle Einträge angewandt, verteilt auf die Prozessorkerne. Die
/// Reihenfolge des Ergebnisses entspricht der Eingabe.
fn parallel_map<T: Sync, R: Send>(items: &[T], f: impl Fn(&T) -> R + Sync) -> Vec<R> {
    let threads = std::thread::available_parallelism().map_or(1, |n| n.get()).min(8);
    if threads <= 1 || items.len() < 2 {
        return items.iter().map(&f).collect();
    }
    let chunk = items.len().div_ceil(threads);
    std::thread::scope(|scope| {
        let handles: Vec<_> =
            items.chunks(chunk).map(|part| scope.spawn(|| part.iter().map(&f).collect::<Vec<R>>())).collect();
        handles.into_iter().flat_map(|h| h.join().unwrap_or_else(|e| std::panic::resume_unwind(e))).collect()
    })
}

/// Fläche einer Schicht, hochgezogen von unten bis oben.
type Extrusion = (f64, f64, Area);

/// Fläche von `z0` bis `z1` hochgezogen.
pub(crate) fn extrude(area: &Area, z0: f64, z1: f64) -> Manifold {
    let polygons: Vec<Vec<Vec2>> =
        area.rings().into_iter().map(|r| r.into_iter().map(|p| Vec2::new(p[0], p[1])).collect()).collect();
    Manifold::extrude(&polygons, z1 - z0, 0, 0.0, Vec2::new(1.0, 1.0)).translate(Vec3::new(0.0, 0.0, z0))
}

/// Alle Höhen eines Geschosses, an denen sich Bauteile ändern, aufsteigend.
pub fn heights(project: &Project, storey: ElementId) -> Vec<f64> {
    let mut z = Vec::new();
    for (id, w) in project.walls_on(storey) {
        if let Some((bottom, top)) = project.wall_extent(w) {
            z.extend([bottom, top]);
            for (_, o) in project.openings_in(id) {
                z.extend([bottom + o.sill_height, bottom + o.sill_height + o.height]);
            }
        }
    }
    for (_, c) in project.columns_on(storey) {
        if let Some((bottom, top)) = project.vertical_extent(c.storey, c.base_offset, c.top) {
            z.extend([bottom, top]);
        }
    }
    z.sort_by(f64::total_cmp);
    z.dedup_by(|a, b| (*a - *b).abs() < 1e-6);
    z
}

/// Ausdehnung aller Wände und Stützen im Grundriss (min, max), ohne Anschlüsse zu rechnen.
pub fn plan_extent(project: &Project) -> Option<([f64; 2], [f64; 2])> {
    let mut points = Vec::new();
    for e in project.elements.values() {
        match e {
            Element::Wall(w) => {
                let Some(t) = project.wall_type(w.wall_type) else { continue };
                let offsets = w.layer_offsets(t);
                let faces = (offsets[0].0, offsets[offsets.len() - 1].1);
                if let Some((lo, hi)) = joins::wall_section(w, (0.0, w.length()), faces).bounds() {
                    points.extend([lo, hi]);
                }
            }
            Element::Column(c) => {
                if let Some((lo, hi)) = column_area(c).bounds() {
                    points.extend([lo, hi]);
                }
            }
            _ => {}
        }
    }
    let first = *points.first()?;
    Some(
        points.iter().fold((first, first), |(lo, hi), p| {
            ([lo[0].min(p[0]), lo[1].min(p[1])], [hi[0].max(p[0]), hi[1].max(p[1])])
        }),
    )
}

fn to_trimesh(solid: &Manifold) -> TriMesh {
    let mesh = solid.get_mesh_gl64(0);
    TriMesh {
        positions: mesh.vert_properties.chunks_exact(mesh.num_prop as usize).map(|v| [v[0], v[1], v[2]]).collect(),
        triangles: mesh.tri_verts.as_chunks::<3>().0.iter().map(|t| t.map(|i| i as u32)).collect(),
    }
}

#[cfg(test)]
mod tests;
