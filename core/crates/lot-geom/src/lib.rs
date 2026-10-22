//! Geometrie: aus Parametern des Modells werden Schichtflächen, Volumen und Netze.
//!
//! Geometrie ist immer abgeleitet und wird nie gespeichert (CLAUDE.md, Regel 6).
//!
//! - [`joins`]: Wandanschlüsse im Grundriss, je Schicht eine Fläche
//! - [`build`]: 3D-Körper aus den Schichtflächen, Höhenabschnitt für
//!   Höhenabschnitt hochgezogen und mit manifold-rust vereinigt (ADR 0004)

pub mod joins;
mod mesh;
pub mod poly;

use std::collections::BTreeMap;

pub use joins::{Band, LayerRegion, column_area, layer_regions};
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
/// Abschnitt hochgezogen und je Schicht vereinigt.
pub fn build(project: &Project) -> Result<ModelGeometry, GeomError> {
    let mut solids: BTreeMap<(ElementId, usize), (String, Vec<Manifold>)> = BTreeMap::new();
    for (storey, _) in project.storeys() {
        let heights = heights(project, storey);
        for pair in heights.windows(2) {
            let (z0, z1) = (pair[0], pair[1]);
            if z1 - z0 < 1e-6 {
                continue;
            }
            for region in layer_regions(project, storey, (z0 + z1) / 2.0) {
                let polygons: Vec<Vec<Vec2>> = region
                    .area
                    .rings()
                    .into_iter()
                    .map(|r| r.into_iter().map(|p| Vec2::new(p[0], p[1])).collect())
                    .collect();
                let solid = Manifold::extrude(&polygons, z1 - z0, 0, 0.0, Vec2::new(1.0, 1.0))
                    .translate(Vec3::new(0.0, 0.0, z0));
                if !solid.is_empty() {
                    solids
                        .entry((region.element, region.layer))
                        .or_insert_with(|| (region.material, Vec::new()))
                        .1
                        .push(solid);
                }
            }
        }
    }
    let elements = solids
        .into_iter()
        .map(|((id, layer), (material, parts))| {
            let solid = Manifold::batch_boolean(&parts, OpType::Add);
            ElementGeometry { id, layer, material, mesh: to_trimesh(&solid) }
        })
        .filter(|e| !e.mesh.is_empty())
        .collect();
    Ok(ModelGeometry { elements })
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
                let (dx, dy) = w.direction();
                for off in [offsets[0].0, offsets[offsets.len() - 1].1] {
                    points.push([w.start.x - dy * off, w.start.y + dx * off]);
                    points.push([w.end.x - dy * off, w.end.y + dx * off]);
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
