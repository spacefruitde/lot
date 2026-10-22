//! Geometrie: aus Parametern des Modells werden Volumen, Netze und Schnitte.
//!
//! Geometrie ist immer abgeleitet und wird nie gespeichert (CLAUDE.md, Regel 6).
//! Booleans laufen über manifold-rust, eine Rust-Portierung von Manifold (ADR 0004): Eine Wand ist ein Quader, von
//! dem jede Öffnung als etwas tieferer Quader abgezogen wird.

mod mesh;

use lot_model::{Element, ElementId, Opening, Project, Wall};
use manifold_rust::linalg::Vec3;
use manifold_rust::manifold::Manifold;
use manifold_rust::types::OpType;
pub use mesh::{FlatVertex, TriMesh, signed_area};

/// Überstand des Öffnungskörpers vor beiden Wandseiten, in Metern. Verhindert
/// deckungsgleiche Flächen beim Abziehen.
const OPENING_OVERSHOOT: f64 = 0.01;

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum GeomError {
    #[error("Die Geometrie von Element {0:?} ist leer")]
    Empty(ElementId),
}

/// Geometrie eines Elements.
#[derive(Debug, Clone, PartialEq)]
pub struct ElementGeometry {
    pub id: ElementId,
    pub mesh: TriMesh,
}

/// Geometrie aller sichtbaren Elemente eines Projekts.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ModelGeometry {
    pub elements: Vec<ElementGeometry>,
}

impl ModelGeometry {
    pub fn bounds(&self) -> Option<([f64; 3], [f64; 3])> {
        self.elements
            .iter()
            .filter_map(|e| e.mesh.bounds())
            .reduce(|(a_lo, a_hi), (b_lo, b_hi)| {
                (
                    [a_lo[0].min(b_lo[0]), a_lo[1].min(b_lo[1]), a_lo[2].min(b_lo[2])],
                    [a_hi[0].max(b_hi[0]), a_hi[1].max(b_hi[1]), a_hi[2].max(b_hi[2])],
                )
            })
    }
}

/// Rechnet die Geometrie aller Wände mit ihren Öffnungen.
pub fn build(project: &Project) -> Result<ModelGeometry, GeomError> {
    let mut elements = Vec::new();
    for (id, element) in &project.elements {
        if let Element::Wall(wall) = element {
            let solid = wall_solid(project, *id, wall);
            if solid.is_empty() {
                return Err(GeomError::Empty(*id));
            }
            elements.push(ElementGeometry { id: *id, mesh: to_trimesh(&solid) });
        }
    }
    Ok(ModelGeometry { elements })
}

/// Waagerechter Schnitt durch alle Wände in Höhe `z` (Meter über Unterkante).
/// Ergibt geschlossene Ringe im Grundriss, in fester Reihenfolge.
pub fn plan_cut(project: &Project, z: f64) -> Vec<Vec<[f64; 2]>> {
    let mut rings: Vec<Vec<[f64; 2]>> = project
        .elements
        .iter()
        .filter_map(|(id, e)| match e {
            Element::Wall(w) => Some(wall_solid(project, *id, w).slice(z).to_polygons()),
            _ => None,
        })
        .flatten()
        .map(|ring| ring.into_iter().map(|p| [p.x, p.y]).collect())
        .map(normalize_ring)
        .collect();
    rings.sort_by(|a, b| a[0][0].total_cmp(&b[0][0]).then(a[0][1].total_cmp(&b[0][1])));
    rings
}

/// Ausdehnung aller Wände im Grundriss (min, max), ohne Geometrie zu rechnen.
pub fn plan_extent(project: &Project) -> Option<([f64; 2], [f64; 2])> {
    project
        .elements
        .values()
        .filter_map(|e| match e {
            Element::Wall(w) => Some(wall_footprint(w)),
            _ => None,
        })
        .flatten()
        .fold(None, |acc: Option<([f64; 2], [f64; 2])>, p| {
            Some(match acc {
                None => (p, p),
                Some((lo, hi)) => ([lo[0].min(p[0]), lo[1].min(p[1])], [hi[0].max(p[0]), hi[1].max(p[1])]),
            })
        })
}

/// Die vier Ecken der Wand im Grundriss.
fn wall_footprint(w: &Wall) -> [[f64; 2]; 4] {
    let (dx, dy) = (w.end.x - w.start.x, w.end.y - w.start.y);
    let len = dx.hypot(dy).max(f64::MIN_POSITIVE);
    let (nx, ny) = (-dy / len * w.thickness / 2.0, dx / len * w.thickness / 2.0);
    [
        [w.start.x + nx, w.start.y + ny],
        [w.end.x + nx, w.end.y + ny],
        [w.end.x - nx, w.end.y - ny],
        [w.start.x - nx, w.start.y - ny],
    ]
}

/// Volumen einer Wand mit allen ihren Öffnungen.
fn wall_solid(project: &Project, id: ElementId, wall: &Wall) -> Manifold {
    let length = wall.length();
    let body = Manifold::cube(Vec3::new(length, wall.thickness, wall.height), false)
        .translate(Vec3::new(0.0, -wall.thickness / 2.0, 0.0));
    let openings: Vec<Manifold> = project.openings_in(id).map(|(_, o)| opening_solid(wall, o)).collect();
    let local = if openings.is_empty() {
        body
    } else {
        body.difference(&Manifold::batch_boolean(&openings, OpType::Add))
    };
    place(local, wall)
}

/// Öffnungskörper im lokalen System der Wand (x entlang der Achse).
fn opening_solid(wall: &Wall, o: &Opening) -> Manifold {
    let depth = wall.thickness + 2.0 * OPENING_OVERSHOOT;
    Manifold::cube(Vec3::new(o.width, depth, o.height), false).translate(Vec3::new(o.offset, -depth / 2.0, o.sill_height))
}

/// Vom lokalen System der Wand in das Projektsystem.
fn place(solid: Manifold, wall: &Wall) -> Manifold {
    let angle = (wall.end.y - wall.start.y).atan2(wall.end.x - wall.start.x).to_degrees();
    solid.rotate(0.0, 0.0, angle).translate(Vec3::new(wall.start.x, wall.start.y, 0.0))
}

fn to_trimesh(solid: &Manifold) -> TriMesh {
    let mesh = solid.get_mesh_gl64(0);
    TriMesh {
        positions: mesh.vert_properties.chunks_exact(mesh.num_prop as usize).map(|v| [v[0], v[1], v[2]]).collect(),
        triangles: mesh.tri_verts.as_chunks::<3>().0.iter().map(|t| t.map(|i| i as u32)).collect(),
    }
}

/// Raster für Schnittpunkte im Grundriss, in Metern (1 µm).
const PLAN_GRID: f64 = 1e-6;

/// Ring gegen den Uhrzeigersinn, beginnend beim Punkt mit kleinstem x, dann y,
/// Punkte auf 1 µm gerundet. So bleibt die Ausgabe zwischen Läufen gleich
/// (wichtig für Golden-Tests). Manifold rastet Schnitte intern auf ein
/// Clipper2-Raster von etwa 10 nm ein; das Runden macht daraus wieder 1,75
/// statt 1,7499999.
fn normalize_ring(ring: Vec<[f64; 2]>) -> Vec<[f64; 2]> {
    let mut ring: Vec<[f64; 2]> = ring
        .into_iter()
        .map(|p| p.map(|v| (v / PLAN_GRID).round() * PLAN_GRID))
        .collect();
    ring.dedup();
    if signed_area(&ring) < 0.0 {
        ring.reverse();
    }
    let start = ring
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| a[0].total_cmp(&b[0]).then(a[1].total_cmp(&b[1])))
        .map_or(0, |(i, _)| i);
    ring.rotate_left(start);
    ring
}

#[cfg(test)]
mod tests {
    use super::*;
    use lot_model::Point2;

    fn volume(mesh: &TriMesh) -> f64 {
        mesh.triangles
            .iter()
            .map(|t| {
                let [a, b, c] = t.map(|i| mesh.positions[i as usize]);
                mesh::dot(a, mesh::cross(b, c)) / 6.0
            })
            .sum()
    }

    #[test]
    fn wall_volume_minus_window() {
        let g = build(&Project::sample()).unwrap();
        assert_eq!(g.elements.len(), 1);
        // 5,00 x 0,30 x 2,60 m minus Fenster 1,50 x 0,30 x 1,30 m
        let expected = 5.0 * 0.3 * 2.6 - 1.5 * 0.3 * 1.3;
        assert!((volume(&g.elements[0].mesh) - expected).abs() < 1e-9);
    }

    #[test]
    fn mesh_is_closed() {
        let g = build(&Project::sample()).unwrap();
        let mesh = &g.elements[0].mesh;
        // In einem geschlossenen Netz gehört jede Kante zu genau zwei Dreiecken.
        let mut edges = std::collections::HashMap::new();
        for t in &mesh.triangles {
            for i in 0..3 {
                let (a, b) = (t[i], t[(i + 1) % 3]);
                *edges.entry((a.min(b), a.max(b))).or_insert(0) += 1;
            }
        }
        assert!(edges.values().all(|n| *n == 2));
    }

    #[test]
    fn rotated_wall_keeps_volume() {
        let mut p = Project::sample();
        if let Some(Element::Wall(w)) = p.elements.get_mut(&ElementId(1)) {
            w.start = Point2::new(2.0, 1.0);
            w.end = Point2::new(2.0 + 3.0, 1.0 + 4.0);
        }
        let g = build(&p).unwrap();
        let expected = 5.0 * 0.3 * 2.6 - 1.5 * 0.3 * 1.3;
        assert!((volume(&g.elements[0].mesh) - expected).abs() < 1e-9);
    }

    #[test]
    fn cut_through_window_gives_two_pieces() {
        let rings = plan_cut(&Project::sample(), 1.0);
        assert_eq!(rings.len(), 2);
        let areas: Vec<f64> = rings.iter().map(|r| signed_area(r)).collect();
        // Links 1,75 m, rechts 5,00 - 1,75 - 1,50 = 1,75 m, jeweils 0,30 m dick.
        for a in areas {
            assert!((a - 1.75 * 0.3).abs() < 1e-9, "Fläche {a}");
        }
    }

    #[test]
    fn cut_below_window_is_one_piece() {
        let rings = plan_cut(&Project::sample(), 0.5);
        assert_eq!(rings.len(), 1);
        assert!((signed_area(&rings[0]) - 5.0 * 0.3).abs() < 1e-9);
    }

    #[test]
    fn feature_edges_of_wall_with_window() {
        let g = build(&Project::sample()).unwrap();
        // Quader: 12 Kanten, Fensterloch: 4 Laibungskanten + 2 x 4 Kanten in den Wandflächen.
        assert_eq!(g.elements[0].mesh.feature_edges(20.0).len(), 12 + 4 + 8);
    }

    #[test]
    fn extent_covers_thickness() {
        let (lo, hi) = plan_extent(&Project::sample()).unwrap();
        assert_eq!(lo, [0.0, -0.15]);
        assert_eq!(hi, [5.0, 0.15]);
    }
}
