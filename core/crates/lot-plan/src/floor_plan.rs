//! Grundriss aus dem Modell (3.01) mit Darstellung je Maßstab (3.03) nach
//! DIN 1356-1:1995-02 (3.04).
//!
//! Was gezeichnet wird, in Zeichenreihenfolge:
//!
//! 1. Sichtbare Kanten unterhalb der Schnittebene (Brüstungen, niedrige Wände)
//! 2. Muster der geschnittenen Schichten, je nach Detailgrad
//! 3. Grenzen zwischen Schichten aus verschiedenem Material
//! 4. Umriss aller geschnittenen Bauteile (Schnittkante)
//! 5. Achsen mit Achskreisen
//!
//! Gleiches Material benachbarter Schichten und Wände wird zu einer Fläche
//! vereinigt, damit an Anschlüssen keine Fugen erscheinen.

use std::collections::BTreeMap;

use lot_geom::{Area, LayerRegion};
use lot_model::{ElementId, LayerFunction, Project};

use crate::clip::clip_polyline;
use crate::standard::{CutPattern, Detail, OfficeStandard, ScaleStyle};
use crate::{Anchor, Area as PlanArea, Fill, Gray, Line, LineStyle, PaperPoint, Pen, PlanObject, Text, Viewport};

/// Ein Grundriss: Geschoss, Schnitthöhe über OKFF und Maßstab.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloorPlanView {
    pub storey: ElementId,
    /// Schnitthöhe über OKFF, in Metern.
    pub cut_height: f64,
    /// Maßstabszahl, 100 für 1:100.
    pub scale: f64,
}

/// Übliche Schnitthöhe im Grundriss, in Metern über OKFF. Vorschlag, Ben prüft.
pub const DEFAULT_CUT_HEIGHT: f64 = 1.0;

/// Zeichnet den Grundriss. `viewport` bildet Modellmeter auf Blattmillimeter ab.
pub fn floor_plan(
    project: &Project,
    view: FloorPlanView,
    standard: &OfficeStandard,
    viewport: Viewport,
) -> Vec<PlanObject> {
    let Some(storey) = project.storey(view.storey) else { return Vec::new() };
    let style = standard.style_for(view.scale);
    let z_cut = storey.elevation + view.cut_height;
    let cut = lot_geom::layer_regions(project, view.storey, z_cut);
    let cut_all = Area::union_all(cut.iter().map(|r| &r.area));
    let mut objects = Vec::new();

    // 1. Sichtbares unterhalb der Schnittebene.
    let below: Vec<Area> = lot_geom::heights(project, view.storey)
        .windows(2)
        .filter(|w| w[0] < z_cut && w[1] - w[0] > 1e-6)
        .map(|w| {
            let z = (w[0] + w[1].min(z_cut)) / 2.0;
            Area::union_all(lot_geom::layer_regions(project, view.storey, z).iter().map(|r| &r.area))
        })
        .collect();
    let below = Area::union_all(below.iter()).difference(&cut_all);
    for ring in below.rings() {
        objects.push(closed(paper(&ring, viewport), style.pens.view));
    }

    // 2. bis 4. Geschnittene Bauteile.
    match style.detail {
        Detail::Coarse => {
            let rings: Vec<Vec<PaperPoint>> = cut_all.rings().iter().map(|r| paper(r, viewport)).collect();
            objects.push(PlanObject::Area(PlanArea { rings, fill: Fill::Solid(Gray::BLACK) }));
        }
        Detail::Medium | Detail::Fine => {
            draw_patterns(&mut objects, &cut, standard, style, viewport);
            for area in material_areas(&cut).values() {
                for ring in area.rings() {
                    objects.push(closed(paper(&ring, viewport), style.pens.layer));
                }
            }
        }
    }
    for ring in cut_all.rings() {
        objects.push(closed(paper(&ring, viewport), style.pens.cut));
    }

    // 5. Achsen.
    for (_, grid) in project.grid_lines() {
        draw_grid_line(&mut objects, grid, style, viewport);
    }
    objects
}

/// Muster der geschnittenen Schichten.
fn draw_patterns(
    objects: &mut Vec<PlanObject>,
    regions: &[LayerRegion],
    standard: &OfficeStandard,
    style: &ScaleStyle,
    viewport: Viewport,
) {
    let shown = |r: &&LayerRegion| match style.detail {
        Detail::Fine => true,
        // Bei mittlerem Detail nur Tragschichten und Stützen.
        Detail::Medium => matches!(r.function, Some(LayerFunction::Structure) | None),
        Detail::Coarse => false,
    };
    let hatch = Pen::mm(style.pens.hatch);
    // Linienmuster über die vereinigte Fläche je Material, damit sie an Ecken durchlaufen.
    let mut by_material: BTreeMap<&str, Vec<&LayerRegion>> = BTreeMap::new();
    for r in regions.iter().filter(shown) {
        by_material.entry(r.material.as_str()).or_default().push(r);
    }
    for (key, parts) in by_material {
        let material = standard.material(key);
        let area = Area::union_all(parts.iter().map(|r| &r.area));
        let rings: Vec<Vec<PaperPoint>> = area.rings().iter().map(|r| paper(r, viewport)).collect();
        match material.cut {
            CutPattern::Empty => {}
            CutPattern::Solid { gray } => {
                objects.push(PlanObject::Area(PlanArea { rings, fill: Fill::Solid(Gray(gray)) }));
            }
            CutPattern::Lines { angle, spacing } => {
                objects.push(PlanObject::Area(PlanArea {
                    rings,
                    fill: Fill::Hatch { angle_deg: angle, spacing_mm: spacing, pen: hatch },
                }));
            }
            CutPattern::Cross { angle, spacing } => {
                for a in [angle, angle + 90.0] {
                    objects.push(PlanObject::Area(PlanArea {
                        rings: rings.clone(),
                        fill: Fill::Hatch { angle_deg: a, spacing_mm: spacing, pen: hatch },
                    }));
                }
            }
            CutPattern::Insulation { soft } => {
                for r in parts {
                    insulation(objects, r, soft, hatch, viewport);
                }
            }
        }
    }
}

/// Dämmstoffmuster entlang der Schicht: Schlangenlinie (weich) oder Zickzack (hart).
/// Die Welle beginnt am Wandanfang, damit sie über die ganze Wand durchläuft.
fn insulation(objects: &mut Vec<PlanObject>, region: &LayerRegion, soft: bool, pen: Pen, viewport: Viewport) {
    let Some(band) = region.band else { return };
    let thickness = band.left - band.right;
    if thickness <= 0.0 {
        return;
    }
    let rings: Vec<Vec<PaperPoint>> = region.area.rings().iter().map(|r| paper(r, viewport)).collect();
    let Some((lo, hi)) = region.area.bounds() else { return };
    // Längsbereich der Fläche auf der Wandrichtung.
    let corners = [[lo[0], lo[1]], [hi[0], lo[1]], [hi[0], hi[1]], [lo[0], hi[1]]];
    let along =
        corners.map(|c| (c[0] - band.origin[0]) * band.direction[0] + (c[1] - band.origin[1]) * band.direction[1]);
    let (s0, s1) = along.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), s| (a.min(*s), b.max(*s)));
    let mid = (band.left + band.right) / 2.0;
    let amplitude = thickness / 2.0 * 0.9;
    let period = thickness;
    let normal = [-band.direction[1], band.direction[0]];
    let point = |s: f64, t: f64| {
        let x = band.origin[0] + band.direction[0] * s + normal[0] * t;
        let y = band.origin[1] + band.direction[1] * s + normal[1] * t;
        viewport.to_paper(x, y)
    };
    let start = (s0 / period).floor() * period;
    let mut points = Vec::new();
    if soft {
        let steps_per_period = 24;
        let n = ((s1 - start) / period * steps_per_period as f64).ceil() as usize + 1;
        for i in 0..=n {
            let s = start + i as f64 * period / steps_per_period as f64;
            let phase = (s / period) * std::f64::consts::TAU;
            points.push(point(s, mid + amplitude * phase.sin()));
        }
    } else {
        let n = ((s1 - start) / (period / 2.0)).ceil() as usize + 1;
        for i in 0..=n {
            let s = start + i as f64 * period / 2.0;
            let side = if i % 2 == 0 { 1.0 } else { -1.0 };
            points.push(point(s, mid + side * amplitude));
        }
    }
    for [a, b] in clip_polyline(&points, &rings) {
        objects.push(PlanObject::Line(Line {
            points: vec![a, b],
            closed: false,
            pen,
            style: LineStyle::Solid,
            color: Gray::BLACK,
        }));
    }
}

/// Vereinigte Fläche je Material über alle geschnittenen Schichten.
fn material_areas(regions: &[LayerRegion]) -> BTreeMap<&str, Area> {
    let mut by_material: BTreeMap<&str, Vec<&Area>> = BTreeMap::new();
    for r in regions {
        by_material.entry(r.material.as_str()).or_default().push(&r.area);
    }
    by_material.into_iter().map(|(k, v)| (k, Area::union_all(v))).collect()
}

/// Achse als Strichpunktlinie mit Achskreis und Beschriftung am Anfang.
fn draw_grid_line(objects: &mut Vec<PlanObject>, grid: &lot_model::GridLine, style: &ScaleStyle, viewport: Viewport) {
    let a = viewport.to_paper(grid.start.x, grid.start.y);
    let b = viewport.to_paper(grid.end.x, grid.end.y);
    let len = (b.x - a.x).hypot(b.y - a.y);
    if len < 1e-6 {
        return;
    }
    let d = ((b.x - a.x) / len, (b.y - a.y) / len);
    let r = style.grid_bubble / 2.0;
    // Strichpunktlinie nach DIN EN ISO 128-20: langer Strich, Lücke, Punkt, Lücke.
    objects.push(PlanObject::Line(Line {
        points: vec![a, b],
        closed: false,
        pen: Pen::mm(style.pens.grid),
        style: LineStyle::Dashed(vec![12.0, 1.5, 0.5, 1.5]),
        color: Gray::BLACK,
    }));
    let center = PaperPoint::new(a.x - d.0 * r, a.y - d.1 * r);
    let circle: Vec<PaperPoint> = (0..48)
        .map(|i| {
            let t = i as f64 / 48.0 * std::f64::consts::TAU;
            PaperPoint::new(center.x + r * t.cos(), center.y + r * t.sin())
        })
        .collect();
    objects.push(closed(circle, style.pens.annotation));
    let height = (style.grid_bubble * 0.45).min(5.0);
    objects.push(PlanObject::Text(Text {
        position: PaperPoint::new(center.x, center.y - height / 2.0),
        text: grid.label.clone(),
        height_mm: height,
        anchor: Anchor::Center,
    }));
}

fn paper(ring: &[[f64; 2]], viewport: Viewport) -> Vec<PaperPoint> {
    ring.iter().map(|p| viewport.to_paper(p[0], p[1])).collect()
}

fn closed(points: Vec<PaperPoint>, width: f64) -> PlanObject {
    PlanObject::Line(Line { points, closed: true, pen: Pen::mm(width), style: LineStyle::Solid, color: Gray::BLACK })
}
