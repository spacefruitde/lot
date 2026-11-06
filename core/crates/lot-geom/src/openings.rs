//! Fenster und Türen im 3D: Rahmen und Glas, Türblatt. Im Grundriss zeichnet
//! sie `lot-plan`; die Lage des Fensterrahmens gilt für beide.

use lot_model::{LayerFunction, Opening, OpeningKind, Project, WallSide, WallType};
use manifold_rust::manifold::Manifold;

use crate::joins::wall_section;

/// Bautiefe des Fensterrahmens, in Metern. Vorschlag, Ben prüft.
pub const WINDOW_FRAME: f64 = 0.07;
/// Ansichtsbreite des Rahmens, in Metern.
const FRAME_WIDTH: f64 = 0.07;
/// Dicke von Glas und Türblatt, in Metern.
const PANE: f64 = 0.02;
const DOOR_LEAF: f64 = 0.04;
const DOOR_INSET: f64 = 0.01;

/// Lage des Fensterrahmens quer zur Wand: (außen, innen) als Abstand von der
/// Bezugslinie. Außen gedämmt: innen an der Außenkante der Tragschicht; sonst
/// in der Mitte der Tragschicht.
pub fn window_frame(wall_type: &WallType, offsets: &[(f64, f64)]) -> (f64, f64) {
    let structure = wall_type.layers.iter().position(|l| l.function == LayerFunction::Structure);
    match structure {
        Some(k) if wall_type.layers[..k].iter().any(|l| l.function == LayerFunction::Insulation) => {
            (offsets[k].0, offsets[k].0 - WINDOW_FRAME)
        }
        Some(k) => {
            let mid = (offsets[k].0 + offsets[k].1) / 2.0;
            (mid + WINDOW_FRAME / 2.0, mid - WINDOW_FRAME / 2.0)
        }
        None => {
            let mid = (offsets[0].0 + offsets[offsets.len() - 1].1) / 2.0;
            (mid + WINDOW_FRAME / 2.0, mid - WINDOW_FRAME / 2.0)
        }
    }
}

/// Körper einer Tür oder eines Fensters: (Teil, Material, Körper). Fenster
/// haben Rahmen und Glas, Türen ein geschlossenes Blatt an der Seite, zu der
/// sie aufschlagen.
pub fn opening_solids(project: &Project, opening: &Opening) -> Vec<(usize, String, Manifold)> {
    let Some(wall) = project.wall(opening.host) else { return Vec::new() };
    let (Some(wall_type), Some((z0, z1))) = (project.wall_type(wall.wall_type), project.opening_span(opening)) else {
        return Vec::new();
    };
    let offsets = wall.layer_offsets(wall_type);
    let (a, b) = (opening.offset, opening.offset + opening.width);
    match opening.kind {
        OpeningKind::Plain => Vec::new(),
        OpeningKind::Window => {
            let (out, inn) = window_frame(wall_type, &offsets);
            let outer = crate::extrude(&wall_section(wall, (a, b), (out, inn)), z0, z1);
            let f = FRAME_WIDTH.min(opening.width / 3.0).min(opening.height / 3.0);
            let hole = crate::extrude(&wall_section(wall, (a + f, b - f), (out + 0.01, inn - 0.01)), z0 + f, z1 - f);
            let mid = (out + inn) / 2.0;
            let glass = crate::extrude(
                &wall_section(wall, (a + f, b - f), (mid + PANE / 2.0, mid - PANE / 2.0)),
                z0 + f,
                z1 - f,
            );
            vec![(0, "rahmen".into(), outer.difference(&hole)), (1, "glas".into(), glass)]
        }
        OpeningKind::Door(door) => {
            let face = match door.opens_to {
                WallSide::Left => offsets[0].0,
                WallSide::Right => offsets[offsets.len() - 1].1,
            };
            // 1 cm hinter der Wandfläche, damit das Blatt in der Ansicht einen Umriss hat.
            let (t0, t1) = match door.opens_to {
                WallSide::Left => (face - DOOR_INSET, face - DOOR_INSET - DOOR_LEAF),
                WallSide::Right => (face + DOOR_INSET + DOOR_LEAF, face + DOOR_INSET),
            };
            vec![(0, "tuerblatt".into(), crate::extrude(&wall_section(wall, (a, b), (t0, t1)), z0, z1))]
        }
    }
}
