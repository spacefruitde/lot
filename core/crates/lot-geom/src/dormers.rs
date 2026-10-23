//! Gauben und Dachfenster (1.05) als Körper, dazu die Ausschnitte im Hauptdach.
//!
//! Alle Maße einer Gaube beziehen sich auf die Dachfläche ihrer Traufe:
//! a entlang der Traufkante, d waagerecht nach innen (siehe [`RoofFace`]).

use lot_model::{Dormer, ElementId, Project, Roof, RoofFace, RoofWindow};
use manifold_rust::linalg::Vec3;
use manifold_rust::manifold::Manifold;
use manifold_rust::types::OpType;

use crate::poly::Area;
use crate::poly::Ring;

/// Reichweite der Hilfskörper nach unten und oben, in Metern.
const FAR: f64 = 200.0;
/// Schichtnummern der Gaube: Dachschichten ab 0, Wandschichten ab 10, Fenster 20 und 21.
pub const DORMER_WALL_LAYER: usize = 10;
pub const DORMER_FRAME_LAYER: usize = 20;
pub const DORMER_GLASS_LAYER: usize = 21;
/// Rahmenbreite von Gauben- und Dachfenstern, in Metern.
const FRAME: f64 = 0.07;

fn area(points: &[lot_model::Point2]) -> Area {
    let ring: Ring = points.iter().map(|p| [p.x, p.y]).collect();
    Area::from_ring(&ring)
}

/// Prisma über dem Rechteck (a, d) der Dachfläche, von ganz unten bis ganz oben.
fn prism(face: &RoofFace, a: (f64, f64), d: (f64, f64)) -> Manifold {
    crate::extrude(&area(&face.rect(a, d)), -FAR, FAR)
}

/// Schneidet an der Ebene z = z0 + tan · d(p) ab und behält, was darunter
/// (`below`) oder darüber liegt.
fn trim(solid: Manifold, face: &RoofFace, z0: f64, tan: f64, below: bool) -> Manifold {
    // z ≤ z0 + tan · ((p − o) · n)  ⇔  tan · (p · n) − z ≥ −z0 + tan · (o · n)
    let (nx, ny) = face.inward;
    let n = Vec3::new(tan * nx, tan * ny, -1.0);
    let len = (n.x * n.x + n.y * n.y + n.z * n.z).sqrt();
    let offset = -z0 + tan * (face.origin.x * nx + face.origin.y * ny);
    let (n, offset) = (Vec3::new(n.x / len, n.y / len, n.z / len), offset / len);
    if below { solid.trim_by_plane(n, offset) } else { solid.trim_by_plane(Vec3::new(-n.x, -n.y, -n.z), -offset) }
}

/// Maße einer Gaube, abgeleitet.
struct Frame {
    face: RoofFace,
    /// Höhe von OKFF des Dachgeschosses über ±0,00.
    floor: f64,
    /// Außenkanten entlang der Traufe.
    span: (f64, f64),
    /// Wo das Gaubendach auf das Hauptdach trifft.
    ridge: f64,
    /// Wanddicke.
    wall: f64,
    /// Dicke des Dachaufbaus.
    roof: f64,
}

fn frame<'a>(project: &'a Project, dormer: &Dormer) -> Option<(Frame, &'a Roof)> {
    let roof = project.roof(dormer.roof)?;
    let face = roof.face(dormer.edge)?;
    let floor = project.storey(roof.storey)?.elevation;
    let wall: f64 = project.wall_type(dormer.wall_type)?.layers.iter().map(|l| l.thickness).sum();
    let ridge = dormer.ridge_distance(roof)?;
    Some((Frame { face, floor, span: dormer.span(), ridge, wall, roof: project.roof_thickness(roof) }, roof))
}

/// Höhe der Ebene des Gaubendachs in Tiefe `depth` unter der Dachhaut, als z0
/// für [`trim`] (Ebene z = z0 + tan · d).
fn dormer_plane(f: &Frame, dormer: &Dormer, depth: f64) -> f64 {
    let (tan, cos) = dormer.pitch();
    f.floor + dormer.eave_height - depth / cos - tan * dormer.setback
}

/// Ebene des Hauptdachs in Tiefe `depth`, als z0 für [`trim`].
fn roof_plane(f: &Frame, roof: &Roof, depth: f64) -> f64 {
    f.floor + roof.eave_height - depth / f.face.cos
}

/// Körper einer Gaube: Dachschichten, Wandschichten, Fenster.
pub fn dormer_solids(project: &Project, dormer: &Dormer) -> Vec<(usize, String, Manifold)> {
    let Some((f, roof)) = frame(project, dormer) else { return Vec::new() };
    let (Some(roof_type), Some(wall_type)) = (project.slab_type(roof.roof_type), project.wall_type(dormer.wall_type))
    else {
        return Vec::new();
    };
    let (tan, _) = dormer.pitch();
    let (a0, a1) = f.span;
    let ov = dormer.overhang;
    let mut out = Vec::new();
    // Dach der Gaube, Schicht für Schicht.
    let mut depth = 0.0;
    for (k, layer) in roof_type.layers.iter().enumerate() {
        let solid = prism(&f.face, (a0 - ov, a1 + ov), (dormer.setback - ov, f.ridge));
        let solid = trim(solid, &f.face, dormer_plane(&f, dormer, depth), tan, true);
        depth += layer.thickness;
        let solid = trim(solid, &f.face, dormer_plane(&f, dormer, depth), tan, false);
        out.push((k, layer.material.clone(), solid));
    }
    // Vorderwand und Wangen: auf dem Hauptdach, bis unter das Gaubendach.
    let window = dormer.window.map(|w| {
        let z = f.floor + w.sill_height;
        let hole = crate::extrude(
            &area(&f.face.rect(
                (dormer.offset - w.width / 2.0, dormer.offset + w.width / 2.0),
                (dormer.setback - 1.0, dormer.setback + f.wall + 1.0),
            )),
            z,
            z + w.height,
        );
        (w, z, hole)
    });
    let mut offset = 0.0;
    for (j, layer) in wall_type.layers.iter().enumerate() {
        let (o0, o1) = (offset, offset + layer.thickness);
        offset = o1;
        let front = prism(&f.face, (a0, a1), (dormer.setback + o0, dormer.setback + o1));
        let left = prism(&f.face, (a0 + o0, a0 + o1), (dormer.setback + f.wall, f.ridge));
        let right = prism(&f.face, (a1 - o1, a1 - o0), (dormer.setback + f.wall, f.ridge));
        let mut walls = Manifold::batch_boolean(&[front, left, right], OpType::Add);
        walls = trim(walls, &f.face, roof_plane(&f, roof, f.roof), f.face.tan, false);
        walls = trim(walls, &f.face, dormer_plane(&f, dormer, f.roof), tan, true);
        if let Some((_, _, hole)) = &window {
            walls = walls.difference(hole);
        }
        out.push((DORMER_WALL_LAYER + j, layer.material.clone(), walls));
    }
    if let Some((w, z, _)) = window {
        // Rahmen und Glas in der Mitte der Vorderwand.
        let mid = dormer.setback + f.wall / 2.0;
        let (b0, b1) = (dormer.offset - w.width / 2.0, dormer.offset + w.width / 2.0);
        let fr = FRAME.min(w.width / 3.0).min(w.height / 3.0);
        let outer = crate::extrude(&area(&f.face.rect((b0, b1), (mid - 0.035, mid + 0.035))), z, z + w.height);
        let inner =
            crate::extrude(&area(&f.face.rect((b0 + fr, b1 - fr), (mid - 0.1, mid + 0.1))), z + fr, z + w.height - fr);
        let glass = crate::extrude(
            &area(&f.face.rect((b0 + fr, b1 - fr), (mid - 0.01, mid + 0.01))),
            z + fr,
            z + w.height - fr,
        );
        out.push((DORMER_FRAME_LAYER, "rahmen".to_string(), outer.difference(&inner)));
        out.push((DORMER_GLASS_LAYER, "glas".to_string(), glass));
    }
    out.into_iter().filter(|(_, _, m)| !m.is_empty()).collect()
}

/// Was aus dem Hauptdach herausgeschnitten wird: das Innere jeder Gaube und
/// die Öffnung jedes Dachfensters, je als Prisma über die ganze Höhe.
pub fn roof_cuts(project: &Project, roof_id: ElementId) -> Option<Manifold> {
    let mut cuts = Vec::new();
    for (_, dormer) in project.dormers_in(roof_id) {
        if let Some((f, _)) = frame(project, dormer) {
            let (a0, a1) = f.span;
            cuts.push(prism(&f.face, (a0 + f.wall, a1 - f.wall), (dormer.setback + f.wall, f.ridge)));
        }
    }
    let roof = project.roof(roof_id)?;
    for (_, w) in project.roof_windows_in(roof_id) {
        if let Some((face, a, d)) = w.extent(roof) {
            cuts.push(prism(&face, a, d));
        }
    }
    (!cuts.is_empty()).then(|| Manifold::batch_boolean(&cuts, OpType::Add))
}

/// Raum unter den Gaubendächern eines Dachs, für Wände bis unter das Dach.
pub fn under_dormers(project: &Project, roof_id: ElementId) -> Option<Manifold> {
    let parts: Vec<Manifold> = project
        .dormers_in(roof_id)
        .filter_map(|(_, dormer)| {
            let (f, _) = frame(project, dormer)?;
            let (a0, a1) = f.span;
            let inside = prism(&f.face, (a0 + f.wall, a1 - f.wall), (dormer.setback + f.wall, f.ridge));
            Some(trim(inside, &f.face, dormer_plane(&f, dormer, f.roof), dormer.pitch().0, true))
        })
        .collect();
    (!parts.is_empty()).then(|| Manifold::batch_boolean(&parts, OpType::Add))
}

/// Bereich im Grundriss unter Gauben, in dem die Unterseite des Gaubendachs
/// mindestens `height` über OKFF liegt.
pub fn dormer_clear_region(project: &Project, roof_id: ElementId, height: f64) -> Area {
    let parts: Vec<Area> = project
        .dormers_in(roof_id)
        .filter_map(|(_, dormer)| {
            let (f, _) = frame(project, dormer)?;
            let (tan, cos) = dormer.pitch();
            let (a0, a1) = f.span;
            let front = dormer.setback + f.wall;
            // Unterseite: eave_height − Dicke / cos + tan · (d − setback) ≥ height.
            let low = dormer.eave_height - f.roof / cos;
            let from = if tan > 1e-9 {
                dormer.setback + (height - low) / tan
            } else if low >= height {
                front
            } else {
                return None;
            };
            let from = from.max(front);
            (from < f.ridge).then(|| area(&f.face.rect((a0 + f.wall, a1 - f.wall), (from, f.ridge))))
        })
        .collect();
    Area::union_all(parts.iter())
}

/// Körper eines Dachfensters: Rahmen (Schicht 0) und Glas (Schicht 1) in der Dachfläche.
pub fn roof_window_solids(project: &Project, window: &RoofWindow) -> Vec<(usize, String, Manifold)> {
    let Some(roof) = project.roof(window.roof) else { return Vec::new() };
    let Some(storey) = project.storey(roof.storey) else { return Vec::new() };
    let Some((face, (a0, a1), (d0, d1))) = window.extent(roof) else { return Vec::new() };
    let z0 = storey.elevation + roof.eave_height;
    // Rahmen 3 cm über bis 10 cm unter der Dachhaut, Glas 1 cm um die Dachhaut.
    let layer = |a: (f64, f64), d: (f64, f64), above: f64, below: f64| {
        let solid = prism(&face, a, d);
        let solid = trim(solid, &face, z0 + above / face.cos, face.tan, true);
        trim(solid, &face, z0 - below / face.cos, face.tan, false)
    };
    let fr = FRAME.min((a1 - a0) / 3.0).min((d1 - d0) / 3.0);
    let frame = layer((a0, a1), (d0, d1), 0.03, 0.10).difference(&prism(&face, (a0 + fr, a1 - fr), (d0 + fr, d1 - fr)));
    let glass = layer((a0 + fr, a1 - fr), (d0 + fr, d1 - fr), 0.01, 0.01);
    vec![(0, "rahmen".to_string(), frame), (1, "glas".to_string(), glass)]
}

#[cfg(test)]
mod tests {
    use lot_model::{DormerWindow, Element, Layer, LayerFunction, Point2, RoofEdge, Storey, WallType};

    use super::*;

    /// Satteldach 45° über 10 × 8 m, Traufen an den Längsseiten, Traufhöhe 1 m über OKFF.
    fn project() -> Project {
        let mut p = Project::new("Gaube");
        p.elements.insert(ElementId(1), Element::Storey(Storey { name: "DG".into(), elevation: 3.0, height: 2.6 }));
        p.elements.insert(
            ElementId(2),
            Element::SlabType(lot_model::SlabType {
                name: "Dach".into(),
                layers: vec![Layer { material: "holz".into(), thickness: 0.2, function: LayerFunction::Structure }],
            }),
        );
        p.elements.insert(
            ElementId(3),
            Element::WallType(WallType {
                name: "Gaubenwand".into(),
                layers: vec![Layer { material: "holz".into(), thickness: 0.15, function: LayerFunction::Structure }],
            }),
        );
        p.elements.insert(
            ElementId(4),
            Element::Roof(Roof {
                storey: ElementId(1),
                roof_type: ElementId(2),
                outline: vec![
                    Point2::new(0.0, 0.0),
                    Point2::new(10.0, 0.0),
                    Point2::new(10.0, 8.0),
                    Point2::new(0.0, 8.0),
                ],
                edges: vec![
                    RoofEdge::Eave { slope: 45.0, overhang: 0.5 },
                    RoofEdge::Verge { overhang: 0.3 },
                    RoofEdge::Eave { slope: 45.0, overhang: 0.5 },
                    RoofEdge::Verge { overhang: 0.3 },
                ],
                eave_height: 1.0,
            }),
        );
        p.elements.insert(
            ElementId(5),
            Element::Dormer(Dormer {
                roof: ElementId(4),
                edge: 0,
                offset: 5.0,
                width: 3.0,
                setback: 0.5,
                eave_height: 2.6,
                slope: 15.0,
                overhang: 0.3,
                wall_type: ElementId(3),
                window: Some(DormerWindow { width: 1.0, height: 0.9, sill_height: 1.3 }),
            }),
        );
        p
    }

    fn dormer(p: &Project) -> &Dormer {
        match p.elements.get(&ElementId(5)) {
            Some(Element::Dormer(d)) => d,
            _ => unreachable!(),
        }
    }

    #[test]
    fn dormer_roof_meets_main_roof() {
        let p = project();
        let d = dormer(&p);
        let roof = p.roof(ElementId(4)).unwrap();
        let ridge = d.ridge_distance(roof).unwrap();
        // 2,6 + tan 15° · (r − 0,5) = 1 + r  ⇒  r = (1,6 − 0,5 · tan 15°) / (1 − tan 15°)
        let t = 15f64.to_radians().tan();
        assert!((ridge - (1.6 - 0.5 * t) / (1.0 - t)).abs() < 1e-9, "{ridge}");
        assert!((d.top_at(ridge) - roof.top_at(Point2::new(5.0, ridge))).abs() < 1e-9);
    }

    #[test]
    fn dormer_has_roof_walls_and_window() {
        let p = project();
        let solids = dormer_solids(&p, dormer(&p));
        let layers: Vec<usize> = solids.iter().map(|s| s.0).collect();
        assert_eq!(layers, [0, DORMER_WALL_LAYER, DORMER_FRAME_LAYER, DORMER_GLASS_LAYER]);
        let walls = crate::to_trimesh(&solids[1].2);
        let (lo, hi) = walls.bounds().unwrap();
        assert!((lo[0] - 3.5).abs() < 1e-6 && (hi[0] - 6.5).abs() < 1e-6, "{lo:?} {hi:?}");
        // Die Vorderwand steht auf der Dachunterseite: 1 + 0,5 − 0,2 · √2 über OKFF.
        assert!((lo[2] - (3.0 + 1.5 - 0.2 * 2f64.sqrt())).abs() < 1e-6, "{lo:?}");
    }

    #[test]
    fn roof_gets_holes_and_dormer_adds_headroom() {
        let p = project();
        let cut = roof_cuts(&p, ElementId(4)).unwrap();
        let (lo, hi) = crate::to_trimesh(&cut).bounds().unwrap();
        assert!((lo[0] - 3.65).abs() < 1e-6 && (hi[0] - 6.35).abs() < 1e-6);
        assert!((lo[1] - 0.65).abs() < 1e-6);
        // Unter der Gaube ist es ab der Vorderwand höher als 2 m.
        let region = dormer_clear_region(&p, ElementId(4), 2.0);
        assert!(region.contains([5.0, 1.0]));
        assert!(!region.contains([2.0, 1.0]));
    }
}
