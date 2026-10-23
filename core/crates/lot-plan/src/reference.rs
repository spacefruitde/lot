//! Referenzmodelle (6.02) im Grundriss: Bauteile eines Fachplanermodells,
//! geschnitten in der Schnittebene, hellgrau mit dünnem Rand.

use lot_geom::{Area, ElementGeometry};

use crate::{Color, Fill, Line, LineStyle, PaperPoint, Pen, PlanObject, Viewport};

/// Zeichnet, was von den Referenzbauteilen in der Höhe `z` (über ±0,00) geschnitten wird.
pub fn draw_reference_cut(objects: &mut Vec<PlanObject>, parts: &[ElementGeometry], z: f64, viewport: Viewport) {
    let areas: Vec<Area> = parts
        .iter()
        .map(|e| {
            let uvw: Vec<[f64; 3]> = e.mesh.positions.iter().map(|p| [p[0], p[1], p[2] - z]).collect();
            crate::section::slice(&e.mesh, &uvw)
        })
        .filter(|a| !a.is_empty())
        .collect();
    let all = Area::union_all(areas.iter());
    let rings: Vec<Vec<PaperPoint>> =
        all.rings().iter().map(|r| r.iter().map(|p| viewport.to_paper(p[0], p[1])).collect()).collect();
    if rings.is_empty() {
        return;
    }
    objects.push(PlanObject::Area(crate::Area { rings: rings.clone(), fill: Fill::Solid(Color::gray(0.82)) }));
    for ring in rings {
        objects.push(PlanObject::Line(Line {
            points: ring,
            closed: true,
            pen: Pen::mm(0.18),
            style: LineStyle::Solid,
            color: Color::gray(0.35),
        }));
    }
}
