//! Grundriss aus dem Modell (3.01) mit Darstellung je Maßstab (3.03) nach
//! DIN 1356-1:1995-02 (3.04).
//!
//! Was gezeichnet wird, in Zeichenreihenfolge:
//!
//! 1. Sichtbare Kanten unterhalb der Schnittebene (Brüstungen, niedrige Wände)
//! 2. Muster der geschnittenen Schichten, je nach Detailgrad
//! 3. Grenzen zwischen Schichten aus verschiedenem Material
//! 4. Umriss aller geschnittenen Bauteile (Schnittkante)
//! 5. Türen und Fenster: Türblatt und Aufschlag, Fensterrahmen; Öffnungen
//!    über der Schnittebene gestrichelt. Deckendurchbrüche im Boden mit
//!    Diagonalkreuz, in der Decke darüber gestrichelt
//! 6. Dach: Umriss mit Überstand gestrichelt, Linien gleicher lichter Höhe
//!    (1,00 m und 2,00 m) in den Räumen darunter
//! 7. Achsen mit Achskreisen
//! 8. Maßketten
//! 9. Raumstempel: Nummer, Name, Netto-Raumfläche
//!
//! Gleiches Material benachbarter Schichten und Wände wird zu einer Fläche
//! vereinigt, damit an Anschlüssen keine Fugen erscheinen.

use std::collections::BTreeMap;

use lot_geom::{Area, LayerRegion};
use lot_model::{ElementId, LayerFunction, OpeningKind, Project, WallSide};

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
    draw_cut(&mut objects, &cut, standard, style, viewport);

    // 5. Türen und Fenster, Deckendurchbrüche.
    draw_openings(&mut objects, project, view.storey, z_cut, style, viewport);
    draw_slab_openings(&mut objects, project, view.storey, style, viewport);

    // 6. Dach.
    draw_roof_lines(&mut objects, project, view.storey, style, viewport);

    // 6a. Schnittlinien.
    draw_section_marks(&mut objects, project, style, viewport);

    // 7. Achsen.
    for (_, grid) in project.grid_lines() {
        draw_grid_line(&mut objects, grid, style, viewport);
    }

    // 8. Maßketten.
    crate::dimensions::draw_dimensions(&mut objects, project, view.storey, style, viewport);

    // 9. Raumstempel.
    draw_space_tags(&mut objects, project, view.storey, style, viewport);
    objects
}

/// Raumstempel (3.06), Vorschlag: Nummer klein über dem Namen, darunter die
/// Netto-Raumfläche nach DIN 277:2021-08. Ein Raum ohne geschlossene Wände
/// zeigt „Fläche offen“.
fn draw_space_tags(
    objects: &mut Vec<PlanObject>,
    project: &Project,
    storey: ElementId,
    style: &ScaleStyle,
    viewport: Viewport,
) {
    let spaces: Vec<_> = project.spaces_on(storey).collect();
    if spaces.is_empty() {
        return;
    }
    let finder = lot_geom::spaces::SpaceFinder::new(project, storey);
    for (_, space) in spaces {
        let area = finder.area_at([space.position.x, space.position.y]).map(|a| a.area());
        let mut lines: Vec<(String, f64)> = Vec::new();
        if !space.number.is_empty() {
            lines.push((space.number.clone(), style.space_text));
        }
        lines.push((space.name.clone(), style.space_name));
        lines.push((area.map_or_else(|| "Fläche offen".to_string(), format_area), style.space_text));
        // Zeilenabstand das 1,6-fache der Schrifthöhe, der Block steht mittig auf dem Raumpunkt.
        let total: f64 = lines.iter().map(|(_, h)| h * 1.6).sum::<f64>() - lines.last().map_or(0.0, |(_, h)| h * 0.6);
        let center = viewport.to_paper(space.position.x, space.position.y);
        let mut top = center.y + total / 2.0;
        for (text, height) in lines {
            top -= height;
            objects.push(PlanObject::Text(Text {
                position: PaperPoint::new(center.x, top),
                text,
                height_mm: height,
                anchor: Anchor::Center,
                rotation: 0.0,
            }));
            top -= height * 0.6;
        }
    }
}

/// Dicke des Türblatts in 1:50, in Metern.
const DOOR_LEAF: f64 = 0.04;
/// Strichlänge und Lücke verdeckter Kanten, in mm auf dem Blatt.
const HIDDEN_DASH: [f64; 2] = [2.0, 1.0];

/// Türen und Fenster (1.06), Vorschlag nach DIN 1356-1:1995-02:
///
/// - Tür: Türblatt rechtwinklig zur Wand an den Bändern, Aufschlag als
///   Viertelkreis; in 1:50 das Türblatt mit Dicke.
/// - Fenster: in 1:200 eine Linie, in 1:100 der Rahmen, in 1:50 Rahmen und
///   Glaslinie. Der Rahmen sitzt an der Außenkante der Tragschicht, wenn außen
///   gedämmt ist, sonst in der Mitte der Tragschicht.
/// - Öffnungen ganz über der Schnittebene (Oberlichter): Umriss gestrichelt.
fn draw_openings(
    objects: &mut Vec<PlanObject>,
    project: &Project,
    storey: ElementId,
    z_cut: f64,
    style: &ScaleStyle,
    viewport: Viewport,
) {
    let line = |points: Vec<[f64; 2]>, width: f64, dashed: bool| {
        PlanObject::Line(Line {
            points: points.iter().map(|p| viewport.to_paper(p[0], p[1])).collect(),
            closed: false,
            pen: Pen::mm(width),
            style: if dashed { LineStyle::Dashed(HIDDEN_DASH.to_vec()) } else { LineStyle::Solid },
            color: Gray::BLACK,
        })
    };
    for (wall_id, wall) in project.walls_on(storey) {
        let (Some(wall_type), Some((bottom, top))) = (project.wall_type(wall.wall_type), project.wall_extent(wall))
        else {
            continue;
        };
        if !(bottom <= z_cut && z_cut < top) {
            continue;
        }
        let offsets = wall.layer_offsets(wall_type);
        let (left, right) = (offsets[0].0, offsets[offsets.len() - 1].1);
        let at = |s: f64, t: f64| {
            let p = wall.point_at(s, t);
            [p.x, p.y]
        };
        for (_, o) in project.openings_in(wall_id) {
            let (o0, o1) = (bottom + o.sill_height, bottom + o.sill_height + o.height);
            let (a, b) = (o.offset, o.offset + o.width);
            if o0 > z_cut {
                // Über der Schnittebene: Umriss gestrichelt.
                let ring = vec![at(a, left), at(b, left), at(b, right), at(a, right), at(a, left)];
                objects.push(line(ring, style.pens.hidden, true));
                continue;
            }
            if o1 <= z_cut {
                continue;
            }
            match o.kind {
                OpeningKind::Plain => {}
                OpeningKind::Door(door) => {
                    let (face, sign) = match door.opens_to {
                        WallSide::Left => (left, 1.0),
                        WallSide::Right => (right, -1.0),
                    };
                    let (hinge_s, free_s) = if door.hinge_at_end() { (b, a) } else { (a, b) };
                    let hinge = at(hinge_s, face);
                    let free = at(free_s, face);
                    let (tx, ty) = wall.tangent_at(hinge_s);
                    let normal = [-ty * sign, tx * sign];
                    let radius = o.width;
                    let tip = [hinge[0] + normal[0] * radius, hinge[1] + normal[1] * radius];
                    if style.detail == Detail::Fine {
                        // Türblatt mit Dicke, zur freien Seite hin.
                        let toward = [(free[0] - hinge[0]) / radius, (free[1] - hinge[1]) / radius];
                        let d = [toward[0] * DOOR_LEAF, toward[1] * DOOR_LEAF];
                        let leaf =
                            vec![hinge, tip, [tip[0] + d[0], tip[1] + d[1]], [hinge[0] + d[0], hinge[1] + d[1]], hinge];
                        objects.push(line(leaf, style.pens.view, false));
                    } else {
                        objects.push(line(vec![hinge, tip], style.pens.view, false));
                    }
                    // Aufschlag: Viertelkreis von der Spitze des Türblatts zur Gegenseite.
                    let start = (tip[1] - hinge[1]).atan2(tip[0] - hinge[0]);
                    let end = (free[1] - hinge[1]).atan2(free[0] - hinge[0]);
                    let mut sweep = end - start;
                    while sweep > std::f64::consts::PI {
                        sweep -= std::f64::consts::TAU;
                    }
                    while sweep < -std::f64::consts::PI {
                        sweep += std::f64::consts::TAU;
                    }
                    let arc: Vec<[f64; 2]> = (0..=24)
                        .map(|i| {
                            let angle = start + sweep * i as f64 / 24.0;
                            [hinge[0] + radius * angle.cos(), hinge[1] + radius * angle.sin()]
                        })
                        .collect();
                    objects.push(line(arc, style.pens.hatch, false));
                }
                OpeningKind::Window => {
                    let (f_out, f_in) = lot_geom::openings::window_frame(wall_type, &offsets);
                    let mid = (f_out + f_in) / 2.0;
                    match style.detail {
                        Detail::Coarse => objects.push(line(vec![at(a, mid), at(b, mid)], style.pens.view, false)),
                        Detail::Medium | Detail::Fine => {
                            let frame = vec![at(a, f_out), at(b, f_out), at(b, f_in), at(a, f_in), at(a, f_out)];
                            objects.push(line(frame, style.pens.view, false));
                            if style.detail == Detail::Fine {
                                objects.push(line(vec![at(a, mid), at(b, mid)], style.pens.hatch, false));
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Deckendurchbrüche (1.11), Vorschlag nach DIN 1356-1: in der Decke unter den
/// Füßen (Oberkante auf OKFF dieses Geschosses) Umriss mit Diagonalkreuz, in
/// der Decke darüber (OKFF des nächsten Geschosses) Umriss gestrichelt.
fn draw_slab_openings(
    objects: &mut Vec<PlanObject>,
    project: &Project,
    storey: ElementId,
    style: &ScaleStyle,
    viewport: Viewport,
) {
    let storeys = project.storeys();
    let above = storeys.iter().position(|(id, _)| *id == storey).and_then(|i| storeys.get(i + 1)).map(|(id, _)| *id);
    let line = |points: Vec<[f64; 2]>, width: f64, dashed: bool, closed: bool| {
        PlanObject::Line(Line {
            points: points.iter().map(|p| viewport.to_paper(p[0], p[1])).collect(),
            closed,
            pen: Pen::mm(width),
            style: if dashed { LineStyle::Dashed(HIDDEN_DASH.to_vec()) } else { LineStyle::Solid },
            color: Gray::BLACK,
        })
    };
    for (slab_id, slab) in project.elements.iter().filter_map(|(id, e)| match e {
        lot_model::Element::Slab(s) => Some((*id, s)),
        _ => None,
    }) {
        let own = slab.storey == storey;
        if !own && Some(slab.storey) != above {
            continue;
        }
        for (_, o) in project.slab_openings_in(slab_id) {
            let ring: Vec<[f64; 2]> = o.outline.iter().map(|p| [p.x, p.y]).collect();
            if own {
                objects.push(line(ring.clone(), style.pens.view, false, true));
                // Diagonalkreuz über die Ecken des umschließenden Rechtecks der ersten vier Punkte.
                if ring.len() >= 4 {
                    objects.push(line(vec![ring[0], ring[2]], style.pens.hatch, false, false));
                    objects.push(line(vec![ring[1], ring[3]], style.pens.hatch, false, false));
                }
            } else {
                objects.push(line(ring, style.pens.hidden, true, true));
            }
        }
    }
}

/// Strichlänge und Lücke der Linien gleicher lichter Höhe, in mm auf dem Blatt.
const HEIGHT_LINE_DASH: [f64; 4] = [6.0, 1.0, 1.0, 1.0];

/// Dach im Grundriss seines Geschosses (Vorschlag): Umriss mit Überstand
/// gestrichelt wie verdeckte Kanten über der Schnittebene; in den Räumen die
/// Linien, an denen die lichte Höhe 1,00 m und 2,00 m beträgt (WoFlV § 4),
/// beschriftet mit der Höhe.
fn draw_roof_lines(
    objects: &mut Vec<PlanObject>,
    project: &Project,
    storey: ElementId,
    style: &ScaleStyle,
    viewport: Viewport,
) {
    let roofs: Vec<_> = project.roofs_on(storey).collect();
    if roofs.is_empty() {
        return;
    }
    let paper_ring = |ring: &[[f64; 2]]| ring.iter().map(|p| viewport.to_paper(p[0], p[1])).collect::<Vec<_>>();
    let finder = lot_geom::spaces::SpaceFinder::new(project, storey);
    let rooms: Vec<Vec<PaperPoint>> = finder.enclosed.iter().flat_map(|a| a.rings()).map(|r| paper_ring(&r)).collect();
    for (_, roof) in roofs {
        let outer: Vec<[f64; 2]> = roof.outer_outline().iter().map(|p| [p.x, p.y]).collect();
        objects.push(PlanObject::Line(Line {
            points: paper_ring(&outer),
            closed: true,
            pen: Pen::mm(style.pens.hidden),
            style: LineStyle::Dashed(HIDDEN_DASH.to_vec()),
            color: Gray::BLACK,
        }));
        for height in [1.0, 2.0] {
            let region = lot_geom::roofs::clear_height_region(project, roof, height);
            let mut labelled = Vec::new();
            for ring in region.rings() {
                let ring = paper_ring(&ring);
                let n = ring.len();
                for i in 0..n {
                    for [a, b] in crate::clip::clip_segment(ring[i], ring[(i + 1) % n], &rooms) {
                        // Jede Linie ab 20 mm Länge bekommt ihre Höhe.
                        if (b.x - a.x).hypot(b.y - a.y) >= 20.0 {
                            labelled.push([a, b]);
                        }
                        objects.push(PlanObject::Line(Line {
                            points: vec![a, b],
                            closed: false,
                            pen: Pen::mm(style.pens.hatch),
                            style: LineStyle::Dashed(HEIGHT_LINE_DASH.to_vec()),
                            color: Gray::BLACK,
                        }));
                    }
                }
            }
            for [a, b] in labelled {
                let angle = (b.y - a.y).atan2(b.x - a.x).to_degrees();
                let angle = if angle > 90.0 {
                    angle - 180.0
                } else if angle <= -90.0 {
                    angle + 180.0
                } else {
                    angle
                };
                let (s, c) = angle.to_radians().sin_cos();
                let mid = PaperPoint::new((a.x + b.x) / 2.0 - s * 0.8, (a.y + b.y) / 2.0 + c * 0.8);
                objects.push(PlanObject::Text(Text {
                    position: mid,
                    text: format!("{} m", crate::format_de(height, 2)),
                    height_mm: style.space_text,
                    anchor: Anchor::Center,
                    rotation: angle,
                }));
            }
        }
    }
}

/// Schnittlinien (Vorschlag nach DIN 1356-1): nur die breiten Enden der
/// Strichpunktlinie, Pfeile in Blickrichtung und der Kennbuchstabe an beiden
/// Enden; die dünne Linie durch das Gebäude entfällt, damit sie keine
/// Raumstempel kreuzt.
fn draw_section_marks(objects: &mut Vec<PlanObject>, project: &Project, style: &ScaleStyle, viewport: Viewport) {
    for (_, view) in project.section_views().filter(|(_, v)| v.kind == lot_model::SectionKind::Section) {
        let a = viewport.to_paper(view.start.x, view.start.y);
        let b = viewport.to_paper(view.end.x, view.end.y);
        let len = (b.x - a.x).hypot(b.y - a.y);
        if len < 1e-6 {
            continue;
        }
        let d = ((b.x - a.x) / len, (b.y - a.y) / len);
        let (look, _) = view.frame();
        let line = |points: Vec<PaperPoint>, width: f64, style: LineStyle| {
            PlanObject::Line(Line { points, closed: false, pen: Pen::mm(width), style, color: Gray::BLACK })
        };
        let end = 8.0f64.min(len / 3.0);
        for (p, dir) in [(a, d), (b, (-d.0, -d.1))] {
            let q = PaperPoint::new(p.x + dir.0 * end, p.y + dir.1 * end);
            objects.push(line(vec![p, q], style.pens.cut, LineStyle::Solid));
            // Pfeil in Blickrichtung, gefüllt.
            let tip = PaperPoint::new(p.x + look.0 * 5.0, p.y + look.1 * 5.0);
            let (s0, s1) = (
                PaperPoint::new(p.x + look.0 * 2.5 + dir.0 * 1.2, p.y + look.1 * 2.5 + dir.1 * 1.2),
                PaperPoint::new(p.x + look.0 * 2.5 - dir.0 * 1.2, p.y + look.1 * 2.5 - dir.1 * 1.2),
            );
            objects.push(line(
                vec![p, PaperPoint::new(p.x + look.0 * 2.5, p.y + look.1 * 2.5)],
                style.pens.annotation,
                LineStyle::Solid,
            ));
            objects.push(PlanObject::Area(PlanArea { rings: vec![vec![tip, s0, s1]], fill: Fill::Solid(Gray::BLACK) }));
            if !view.mark.is_empty() {
                let at = PaperPoint::new(p.x + look.0 * 8.0, p.y + look.1 * 8.0 - style.text / 2.0);
                objects.push(PlanObject::Text(Text {
                    position: at,
                    text: view.mark.clone(),
                    height_mm: style.text,
                    anchor: Anchor::Center,
                    rotation: 0.0,
                }));
            }
        }
    }
}

/// Fläche im deutschen Format, „24,35 m²“.
pub fn format_area(m2: f64) -> String {
    format!("{} m²", crate::format_de(m2, 2))
}

/// Geschnittene Bauteile, im Grundriss wie im Schnitt: je nach Detailgrad
/// schwarz gefüllt (grob) oder mit Mustern und Materialgrenzen, darüber die
/// Schnittkante um alles Geschnittene.
pub(crate) fn draw_cut(
    objects: &mut Vec<PlanObject>,
    cut: &[LayerRegion],
    standard: &OfficeStandard,
    style: &ScaleStyle,
    viewport: Viewport,
) {
    let cut_all = Area::union_all(cut.iter().map(|r| &r.area));
    match style.detail {
        Detail::Coarse => {
            let rings: Vec<Vec<PaperPoint>> = cut_all.rings().iter().map(|r| paper(r, viewport)).collect();
            objects.push(PlanObject::Area(PlanArea { rings, fill: Fill::Solid(Gray::BLACK) }));
        }
        Detail::Medium | Detail::Fine => {
            draw_patterns(objects, cut, standard, style, viewport);
            for area in material_areas(cut).values() {
                for ring in area.rings() {
                    objects.push(closed(paper(&ring, viewport), style.pens.layer));
                }
            }
        }
    }
    for ring in cut_all.rings() {
        objects.push(closed(paper(&ring, viewport), style.pens.cut));
    }
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
    let model_rings = region.area.rings();
    let rings: Vec<Vec<PaperPoint>> = model_rings.iter().map(|r| paper(r, viewport)).collect();
    // Längsbereich der Fläche entlang der Bezugslinie (bei Bögen entlang des Bogens).
    let (s0, s1) = model_rings
        .iter()
        .flatten()
        .map(|p| band.locate(*p).0)
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), s| (a.min(s), b.max(s)));
    if !s0.is_finite() {
        return;
    }
    let mid = (band.left + band.right) / 2.0;
    let amplitude = thickness / 2.0 * 0.9;
    // Auf Bögen gilt die Periode in Schichtmitte; gemessen wird auf der Bezugslinie.
    let period = thickness * band.arc.map_or(1.0, |a| a.radius / a.radius_at(mid).max(1e-9));
    let point = |s: f64, t: f64| {
        let [x, y] = band.point(s, t);
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
        rotation: 0.0,
    }));
}

fn paper(ring: &[[f64; 2]], viewport: Viewport) -> Vec<PaperPoint> {
    ring.iter().map(|p| viewport.to_paper(p[0], p[1])).collect()
}

fn closed(points: Vec<PaperPoint>, width: f64) -> PlanObject {
    PlanObject::Line(Line { points, closed: true, pen: Pen::mm(width), style: LineStyle::Solid, color: Gray::BLACK })
}

#[cfg(test)]
mod tests {
    use lot_model::{Door, Element, ElementId, Handing, OpeningKind, Project, WallSide};

    use super::*;

    /// Beispielwand (5 m nach Osten, 30 cm) mit einer Tür statt des Fensters.
    fn door_project(door: Door) -> Project {
        let mut p = Project::sample();
        if let Some(Element::Opening(o)) = p.elements.get_mut(&ElementId(4)) {
            o.sill_height = 0.0;
            o.height = 2.0;
            o.width = 1.0;
            o.offset = 2.0;
            o.kind = OpeningKind::Door(door);
        }
        p
    }

    fn lines(p: &Project) -> Vec<Vec<PaperPoint>> {
        let view = FloorPlanView { storey: ElementId(1), cut_height: DEFAULT_CUT_HEIGHT, scale: 100.0 };
        let viewport = Viewport { scale: 100.0, origin: PaperPoint::new(0.0, 0.0) };
        floor_plan(p, view, OfficeStandard::default_standard(), viewport)
            .into_iter()
            .filter_map(|o| match o {
                PlanObject::Line(l) => Some(l.points),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn door_swings_to_its_side_from_its_hinge() {
        // Aufschlag nach Norden (links), DIN links: Bänder am östlichen Ende bei x = 3 m.
        let p = door_project(Door { opens_to: WallSide::Left, hinge: Handing::DinLeft });
        let all = lines(&p);
        // Türblatt: von (3,0; 0,15) nach Norden, 1 m lang (10 mm auf dem Blatt).
        let leaf = all.iter().find(|l| l.len() == 2 && (l[0].x - 30.0).abs() < 1e-6 && (l[0].y - 1.5).abs() < 1e-6);
        let leaf = leaf.expect("Türblatt fehlt");
        assert!((leaf[1].x - 30.0).abs() < 1e-6 && (leaf[1].y - 11.5).abs() < 1e-6, "{leaf:?}");
        // Aufschlag: Viertelkreis um die Bänder mit Radius 10 mm, endet an der Gegenseite bei x = 2 m.
        let arc = all.iter().find(|l| l.len() == 25).expect("Aufschlag fehlt");
        assert!(arc.iter().all(|q| ((q.x - 30.0).hypot(q.y - 1.5) - 10.0).abs() < 1e-6));
        assert!((arc[24].x - 20.0).abs() < 1e-6 && (arc[24].y - 1.5).abs() < 1e-6);
    }

    #[test]
    fn window_above_cut_is_dashed() {
        let mut p = Project::sample();
        if let Some(Element::Opening(o)) = p.elements.get_mut(&ElementId(4)) {
            o.sill_height = 1.6;
            o.height = 0.6;
            o.kind = OpeningKind::Window;
        }
        let view = FloorPlanView { storey: ElementId(1), cut_height: DEFAULT_CUT_HEIGHT, scale: 100.0 };
        let viewport = Viewport { scale: 100.0, origin: PaperPoint::new(0.0, 0.0) };
        let dashed = floor_plan(&p, view, OfficeStandard::default_standard(), viewport)
            .into_iter()
            .filter(|o| matches!(o, PlanObject::Line(Line { style: LineStyle::Dashed(_), .. })))
            .count();
        assert_eq!(dashed, 1);
    }
}
