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
//!    Diagonalkreuz, in der Decke darüber gestrichelt. Treppen und Geländer
//!    (siehe [`crate::stairs`])
//! 6. Dach: Umriss mit Überstand gestrichelt, Linien gleicher lichter Höhe
//!    (1,00 m und 2,00 m) in den Räumen darunter
//! 7. Achsen mit Achskreisen
//! 8. Maßketten
//! 9. Raumstempel: Nummer, Name, Netto-Raumfläche
//! 10. 2D-Elemente dieses Grundrisses (siehe [`crate::annotations`])
//!
//! Gleiches Material benachbarter Schichten und Wände wird zu einer Fläche
//! vereinigt, damit an Anschlüssen keine Fugen erscheinen.

use std::collections::BTreeMap;

use lot_geom::{Area, LayerRegion};
use lot_model::{ElementId, LayerFunction, OpeningKind, Project, WallSide};

use crate::clip::clip_polyline;
use crate::standard::{CutPattern, Detail, OfficeStandard, ScaleStyle};
use crate::views::Paint;
use crate::{Anchor, Area as PlanArea, Color, Fill, Line, LineStyle, PaperPoint, Pen, PlanObject, Text, Viewport};

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
    floor_plan_layers(project, view, standard, viewport).into_iter().flat_map(|(_, objects)| objects).collect()
}

/// Grundriss mit den Farben und dem Detailgrad einer Ansicht (ADR 0050).
pub fn floor_plan_with(
    project: &Project,
    view: FloorPlanView,
    standard: &OfficeStandard,
    viewport: Viewport,
    paint: &Paint,
) -> Vec<PlanObject> {
    floor_plan_layers_with(project, view, standard, viewport, paint).into_iter().flat_map(|(_, o)| o).collect()
}

/// Wie [`floor_plan`], nach Layern getrennt, für den Austausch über DXF und DWG
/// (6.04). Die Namen sind Vorschläge ohne Umlaute; leere Layer fehlen.
pub fn floor_plan_layers(
    project: &Project,
    view: FloorPlanView,
    standard: &OfficeStandard,
    viewport: Viewport,
) -> Vec<(&'static str, Vec<PlanObject>)> {
    floor_plan_layers_with(project, view, standard, viewport, &Paint::default())
}

/// Wie [`floor_plan_layers`], mit den Farben und dem Detailgrad einer Ansicht.
pub fn floor_plan_layers_with(
    project: &Project,
    view: FloorPlanView,
    standard: &OfficeStandard,
    viewport: Viewport,
    paint: &Paint,
) -> Vec<(&'static str, Vec<PlanObject>)> {
    let Some(storey) = project.storey(view.storey) else { return Vec::new() };
    let mut owned = standard.style_for(view.scale).clone();
    if let Some(d) = paint.detail {
        owned.detail = d;
    }
    let style = &owned;
    let z_cut = storey.elevation + view.cut_height;
    // Im Umbauplan durchbrechen zugemauerte und neue Öffnungen in Bestand die Wand nicht.
    let phased = project.has_phases().then(|| project.without_phase_openings());
    let geo = phased.as_ref().unwrap_or(project);
    let cut = lot_geom::layer_regions(geo, view.storey, z_cut);
    let cut_all = Area::union_all(cut.iter().map(|r| &r.area));
    let mut layers: Vec<(&'static str, Vec<PlanObject>)> = Vec::new();
    let mut layer = |name: &'static str, draw: &mut dyn FnMut(&mut Vec<PlanObject>)| {
        let mut objects = Vec::new();
        draw(&mut objects);
        if !objects.is_empty() {
            layers.push((name, objects));
        }
    };

    // 0. Räume farbig nach den Regeln der Ansicht (ADR 0050), ganz unten.
    layer("A_Raumfarben", &mut |objects| draw_space_colors(objects, project, view.storey, paint, viewport));

    // 1. Sichtbares unterhalb der Schnittebene.
    layer("A_Sichtkanten", &mut |objects| {
        let below: Vec<Area> = lot_geom::heights(geo, view.storey)
            .windows(2)
            .filter(|w| w[0] < z_cut && w[1] - w[0] > 1e-6)
            .map(|w| {
                let z = (w[0] + w[1].min(z_cut)) / 2.0;
                Area::union_all(lot_geom::layer_regions(geo, view.storey, z).iter().map(|r| &r.area))
            })
            .collect();
        let below = Area::union_all(below.iter()).difference(&cut_all);
        for ring in below.rings() {
            objects.push(closed(paper(&ring, viewport), style.pens.view));
        }
    });

    // 2. bis 4. Geschnittene Bauteile, im Umbauplan farbig.
    layer("A_Bauteile_Schnitt", &mut |objects| {
        draw_cut_for(objects, project, &cut, Some((view.storey, z_cut)), standard, style, viewport, paint)
    });

    // 5. Türen und Fenster, Deckendurchbrüche.
    layer("A_Tueren_Fenster", &mut |objects| draw_openings(objects, project, view.storey, z_cut, style, viewport));
    layer("A_Deckendurchbrueche", &mut |objects| draw_slab_openings(objects, project, view.storey, style, viewport));
    // 5c. Gefälle (ADR 0051): Pfeil in Fließrichtung mit Prozent, Abläufe, Notüberläufe.
    layer("A_Gefaelle", &mut |objects| draw_falls(objects, project, view.storey, style, viewport));

    // 5a. Treppen und Geländer.
    layer("A_Treppen", &mut |objects| {
        crate::stairs::draw_stairs(objects, project, view.storey, view.cut_height, style, viewport)
    });

    // 5b. Aufzüge.
    layer("A_Aufzuege", &mut |objects| {
        crate::elevators::draw_elevators(objects, project, view.storey, style, viewport)
    });

    // 6. Dach.
    layer("A_Dach", &mut |objects| draw_roof_lines(objects, project, view.storey, style, viewport));

    // 6a. Schnittlinien.
    layer("A_Schnittlinien", &mut |objects| draw_section_marks(objects, project, style, viewport));

    // 7. Achsen.
    layer("A_Achsen", &mut |objects| {
        for (_, grid) in project.grid_lines() {
            draw_grid_line(objects, grid, style, viewport);
        }
    });

    // 8. Maßketten, Rohbaumaße der Türen ohne Maßkette.
    layer("A_Bemassung", &mut |objects| {
        crate::dimensions::draw_dimensions(objects, project, view.storey, style, viewport);
        crate::dimensions::draw_door_sizes(objects, project, view.storey, z_cut, style, viewport);
    });

    // 9. Raumstempel.
    layer("A_Raumstempel", &mut |objects| draw_space_tags(objects, project, view.storey, style, viewport));

    // 10. 2D-Elemente.
    layer("A_2D", &mut |objects| {
        let annotations = lot_model::AnnotationView::FloorPlan { storey: view.storey };
        crate::annotations::draw_annotations(
            objects,
            project,
            annotations,
            standard,
            style,
            crate::annotations::model(viewport),
        )
    });
    layers
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
    // Im Umbauplan stehen die Räume nach dem Umbau, mit ihren Flächen ohne die
    // abgebrochenen Wände.
    let planned = project.has_phases().then(|| project.phase_view(lot_model::PhaseFilter::Planned));
    let spaces: Vec<_> = project
        .spaces_on(storey)
        .filter(|(_, r)| r.phase != lot_model::Phase::Demolished || planned.is_none())
        .collect();
    if spaces.is_empty() {
        return;
    }
    let finder = lot_geom::spaces::SpaceFinder::new(planned.as_ref().unwrap_or(project), storey);
    for (_, space) in spaces {
        // Im Aufzugsschacht steht die Beschriftung des Aufzugs.
        if crate::elevators::labels_space(project, storey, space.position) {
            continue;
        }
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
pub(crate) fn draw_openings(
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
            color: Color::BLACK,
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
            // Zugemauerte Öffnungen zeigt der Umbauplan als rote Fläche ohne Symbol,
            // Öffnungen in abgebrochenen Wänden ohne Symbol.
            if project.has_phases()
                && (o.phase == lot_model::Phase::Demolished || wall.phase == lot_model::Phase::Demolished)
            {
                continue;
            }
            let Some((o0, o1)) = project.opening_span(o) else { continue };
            let (a, b) = (o.offset, o.offset + o.width);
            if o0 > z_cut {
                // Über der Schnittebene: Umriss gestrichelt, bei Nischen nur so tief wie die Nische.
                let (left, right) = match o.recess {
                    Some(lot_model::Recess { side: WallSide::Left, depth }) => (left, left - depth),
                    Some(lot_model::Recess { side: WallSide::Right, depth }) => (right + depth, right),
                    None => (left, right),
                };
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
            color: Color::BLACK,
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
                // Diagonalkreuz über die Ecken des umschließenden Rechtecks der
                // ersten vier Punkte; nicht, wo eine Treppe von unten heraufkommt.
                if ring.len() >= 4 && !stair_from_below(project, storey, &ring) {
                    objects.push(line(vec![ring[0], ring[2]], style.pens.hatch, false, false));
                    objects.push(line(vec![ring[1], ring[3]], style.pens.hatch, false, false));
                }
            } else {
                objects.push(line(ring, style.pens.hidden, true, true));
            }
        }
    }
}

/// Ob eine Treppe aus dem Geschoss darunter im Durchbruch `ring` ankommt.
fn stair_from_below(project: &Project, storey: ElementId, ring: &[[f64; 2]]) -> bool {
    let storeys = project.storeys();
    let Some(below) = storeys.iter().position(|(id, _)| *id == storey).and_then(|i| i.checked_sub(1)) else {
        return false;
    };
    let hole = Area::from_ring(ring);
    project.stairs_on(storeys[below].0).filter_map(|(_, s)| project.stair_layout(s)).any(|layout| {
        layout.flights.iter().flat_map(|f| layout.treads(f)).any(|t| {
            let c = t.outline.iter().fold([0.0, 0.0], |acc, p| [acc[0] + p.x / 4.0, acc[1] + p.y / 4.0]);
            hole.contains(c)
        })
    })
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
    for (roof_id, roof) in roofs {
        let outer: Vec<[f64; 2]> = roof.outer_outline().iter().map(|p| [p.x, p.y]).collect();
        // Gauben und Dachfenster gestrichelt wie der Dachumriss.
        let mut outlines: Vec<Vec<lot_model::Point2>> = Vec::new();
        for (_, d) in project.dormers_in(roof_id) {
            if let (Some(face), Some(ridge)) = (roof.face(d.edge), d.ridge_distance(roof)) {
                outlines.push(face.rect(d.span(), (d.setback, ridge)));
            }
        }
        for (_, w) in project.roof_windows_in(roof_id) {
            if let Some((face, a, d)) = w.extent(roof) {
                outlines.push(face.rect(a, d));
            }
        }
        for o in outlines {
            let ring: Vec<[f64; 2]> = o.iter().map(|p| [p.x, p.y]).collect();
            objects.push(PlanObject::Line(Line {
                points: paper_ring(&ring),
                closed: true,
                pen: Pen::mm(style.pens.hidden),
                style: LineStyle::Dashed(HIDDEN_DASH.to_vec()),
                color: Color::BLACK,
            }));
        }
        objects.push(PlanObject::Line(Line {
            points: paper_ring(&outer),
            closed: true,
            pen: Pen::mm(style.pens.hidden),
            style: LineStyle::Dashed(HIDDEN_DASH.to_vec()),
            color: Color::BLACK,
        }));
        // Flachdächer haben überall dieselbe lichte Höhe; Linien braucht es nur unter Schrägen.
        let heights: &[f64] = if roof.eave_planes().is_empty() { &[] } else { &[1.0, 2.0] };
        for &height in heights {
            let region = lot_geom::roofs::clear_height_region(project, roof_id, roof, height);
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
                            color: Color::BLACK,
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
            PlanObject::Line(Line { points, closed: false, pen: Pen::mm(width), style, color: Color::BLACK })
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
            objects
                .push(PlanObject::Area(PlanArea { rings: vec![vec![tip, s0, s1]], fill: Fill::Solid(Color::BLACK) }));
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

/// Gefälle der Decken eines Geschosses (ADR 0051): ein Pfeil in Fließrichtung
/// mit dem Gefälle in Prozent, Abläufe als Kreis mit Kreuz, Notüberläufe als
/// Rechteck mit „NÜ“. Vorschlag, Ben prüft.
fn draw_falls(
    objects: &mut Vec<PlanObject>,
    project: &Project,
    storey: ElementId,
    style: &ScaleStyle,
    viewport: Viewport,
) {
    let pen = Pen::mm(style.pens.annotation);
    let line = |points: Vec<PaperPoint>, closed: bool| {
        PlanObject::Line(Line { points, closed, pen, style: LineStyle::Solid, color: Color::BLACK })
    };
    for (_, slab) in project.slabs_on(storey) {
        let Some(fall) = &slab.fall else { continue };
        let n = slab.outline.len().max(1) as f64;
        let c = viewport.to_paper(
            slab.outline.iter().map(|p| p.x).sum::<f64>() / n,
            slab.outline.iter().map(|p| p.y).sum::<f64>() / n,
        );
        let (dx, dy) = fall.flow();
        let (len, head) = (12.0, 2.5);
        let tip = PaperPoint::new(c.x + dx * len / 2.0, c.y + dy * len / 2.0);
        let tail = PaperPoint::new(c.x - dx * len / 2.0, c.y - dy * len / 2.0);
        objects.push(line(vec![tail, tip], false));
        let back = PaperPoint::new(tip.x - dx * head, tip.y - dy * head);
        objects.push(PlanObject::Area(PlanArea {
            rings: vec![vec![
                tip,
                PaperPoint::new(back.x - dy * head * 0.4, back.y + dx * head * 0.4),
                PaperPoint::new(back.x + dy * head * 0.4, back.y - dx * head * 0.4),
            ]],
            fill: Fill::Solid(Color::BLACK),
        }));
        // Prozent neben dem Pfeil, lesbar zwischen -90° und 90° (DIN 1356-1).
        let mut angle = dy.atan2(dx).to_degrees();
        if angle > 90.0 {
            angle -= 180.0;
        } else if angle < -90.0 {
            angle += 180.0;
        }
        let off = style.text * 0.6;
        objects.push(PlanObject::Text(Text {
            position: PaperPoint::new(c.x - dy * off, c.y + dx * off),
            text: format!("{} %", crate::format_de(fall.percent, 1)),
            height_mm: style.text,
            anchor: Anchor::Center,
            rotation: angle,
        }));
        for d in &fall.drains {
            let p = viewport.to_paper(d.x, d.y);
            let r = 2.0;
            let ring: Vec<PaperPoint> = (0..24)
                .map(|i| {
                    let a = i as f64 / 24.0 * std::f64::consts::TAU;
                    PaperPoint::new(p.x + r * a.cos(), p.y + r * a.sin())
                })
                .collect();
            objects.push(line(ring, true));
            let k = r * std::f64::consts::FRAC_1_SQRT_2;
            objects.push(line(vec![PaperPoint::new(p.x - k, p.y - k), PaperPoint::new(p.x + k, p.y + k)], false));
            objects.push(line(vec![PaperPoint::new(p.x - k, p.y + k), PaperPoint::new(p.x + k, p.y - k)], false));
        }
        for o in &fall.overflows {
            let p = viewport.to_paper(o.x, o.y);
            objects.push(line(crate::rect_points(p.x - 1.5, p.y - 0.75, 3.0, 1.5), true));
            objects.push(PlanObject::Text(Text {
                position: PaperPoint::new(p.x, p.y + 1.5),
                text: "NÜ".into(),
                height_mm: style.text * 0.8,
                anchor: Anchor::Center,
                rotation: 0.0,
            }));
        }
    }
}

/// Grundriss eines Geschosses blass, als Hilfe unter dem aktuellen
/// (ADR 0049): nur geschnittene Bauteile, Türen, Fenster, Treppen und Aufzüge,
/// in Hellgrau und dünn, ohne Texte und Schraffuren. Nur am Bildschirm.
pub fn faded_storey(
    project: &Project,
    storey: ElementId,
    standard: &OfficeStandard,
    viewport: Viewport,
) -> Vec<PlanObject> {
    let fade = |c: Color| Color { r: c.r + (1.0 - c.r) * 0.7, g: c.g + (1.0 - c.g) * 0.7, b: c.b + (1.0 - c.b) * 0.7 };
    let view = FloorPlanView { storey, cut_height: DEFAULT_CUT_HEIGHT, scale: viewport.scale };
    floor_plan_layers(project, view, standard, viewport)
        .into_iter()
        .filter(|(name, _)| ["A_Bauteile_Schnitt", "A_Tueren_Fenster", "A_Treppen", "A_Aufzuege"].contains(name))
        .flat_map(|(_, objects)| objects)
        .filter_map(|o| match o {
            PlanObject::Line(mut l) => {
                l.color = fade(l.color);
                l.pen = Pen::mm(l.pen.width_mm.min(0.18));
                Some(PlanObject::Line(l))
            }
            PlanObject::Area(PlanArea { rings, fill: Fill::Solid(c) }) => {
                Some(PlanObject::Area(PlanArea { rings, fill: Fill::Solid(fade(c)) }))
            }
            PlanObject::Area(_) | PlanObject::Text(_) => None,
        })
        .collect()
}

/// Fläche im deutschen Format, „24,35 m²“.
pub fn format_area(m2: f64) -> String {
    format!("{} m²", crate::format_de(m2, 2))
}

/// Geschnittene Bauteile, im Grundriss wie im Schnitt: je nach Detailgrad
/// schwarz gefüllt (grob) oder mit Mustern und Materialgrenzen, darüber die
/// Schnittkante um alles Geschnittene.
/// Geschnittene Bauteile, im Umbauplan (es gibt Bestand oder Abbruch) farbig
/// je Bauzustand statt mit Materialschraffur. `opening_cut` (Geschoss und Höhe)
/// legt im Grundriss Öffnungen farbig an, die die Wand nicht durchbrechen:
/// neue Öffnungen in Bestand gelb (Ausbruch), zugemauerte rot.
/// Räume, denen eine Regel eine Farbe gibt, als Fläche.
fn draw_space_colors(
    objects: &mut Vec<PlanObject>,
    project: &Project,
    storey: ElementId,
    paint: &Paint,
    viewport: Viewport,
) {
    if paint.colors.is_empty() {
        return;
    }
    let finder = lot_geom::spaces::SpaceFinder::new(project, storey);
    for (id, space) in project.spaces_on(storey) {
        let (Some(color), Some(area)) = (paint.colors.get(&id), finder.area_at([space.position.x, space.position.y]))
        else {
            continue;
        };
        let rings: Vec<Vec<PaperPoint>> = area.rings().iter().map(|r| paper(r, viewport)).collect();
        objects.push(PlanObject::Area(PlanArea { rings, fill: Fill::Solid(*color) }));
    }
}

/// Geschnittene Bauteile; Bauteile mit einer Farbe aus den Regeln der Ansicht
/// (ADR 0050) voll in dieser Farbe, die übrigen wie gewohnt, im Umbauplan nach Bauzustand.
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_cut_for(
    objects: &mut Vec<PlanObject>,
    project: &Project,
    cut: &[LayerRegion],
    opening_cut: Option<(ElementId, f64)>,
    standard: &OfficeStandard,
    style: &ScaleStyle,
    viewport: Viewport,
    paint: &Paint,
) {
    let (painted, rest): (Vec<LayerRegion>, Vec<LayerRegion>) =
        cut.iter().cloned().partition(|r| paint.colors.contains_key(&r.element));
    if !rest.is_empty() {
        draw_cut_phased(objects, project, &rest, opening_cut, standard, style, viewport);
    }
    if painted.is_empty() {
        return;
    }
    let mut colors: Vec<Color> = Vec::new();
    for r in &painted {
        let c = paint.colors[&r.element];
        if !colors.contains(&c) {
            colors.push(c);
        }
    }
    for c in colors {
        let area = Area::union_all(painted.iter().filter(|r| paint.colors[&r.element] == c).map(|r| &r.area));
        let rings: Vec<Vec<PaperPoint>> = area.rings().iter().map(|r| paper(r, viewport)).collect();
        objects.push(PlanObject::Area(PlanArea { rings, fill: Fill::Solid(c) }));
    }
    for area in material_areas(&painted).values() {
        for ring in area.rings() {
            objects.push(closed(paper(&ring, viewport), style.pens.layer));
        }
    }
    for ring in Area::union_all(painted.iter().map(|r| &r.area)).rings() {
        objects.push(closed(paper(&ring, viewport), style.pens.cut));
    }
}

fn draw_cut_phased(
    objects: &mut Vec<PlanObject>,
    project: &Project,
    cut: &[LayerRegion],
    opening_cut: Option<(ElementId, f64)>,
    standard: &OfficeStandard,
    style: &ScaleStyle,
    viewport: Viewport,
) {
    if !project.has_phases() {
        draw_cut(objects, cut, standard, style, viewport);
        return;
    }
    // Was keinen Bauzustand hat (Erdreich, Referenzmodelle), bleibt wie gewohnt.
    let (cut, plain): (Vec<LayerRegion>, Vec<LayerRegion>) = cut
        .iter()
        .cloned()
        .partition(|r| project.elements.get(&r.element).and_then(lot_model::Element::phase).is_some());
    if !plain.is_empty() {
        draw_cut(objects, &plain, standard, style, viewport);
    }
    let cut = cut.as_slice();
    let colors = &standard.phases;
    let cut_all = Area::union_all(cut.iter().map(|r| &r.area));
    for phase in [lot_model::Phase::Existing, lot_model::Phase::Demolished, lot_model::Phase::New] {
        let area = Area::union_all(cut.iter().filter(|r| project.phase_of(r.element) == phase).map(|r| &r.area));
        let rings: Vec<Vec<PaperPoint>> = area.rings().iter().map(|r| paper(r, viewport)).collect();
        if !rings.is_empty() {
            objects.push(PlanObject::Area(PlanArea { rings, fill: Fill::Solid(colors.of(phase)) }));
        }
    }
    if let Some((storey, z)) = opening_cut {
        for (wall_id, wall) in project.walls_on(storey) {
            let Some(wall_type) = project.wall_type(wall.wall_type) else { continue };
            let offsets = wall.layer_offsets(wall_type);
            let (left, right) = (offsets[0].0, offsets[offsets.len() - 1].1);
            for (id, o) in project.openings_in(wall_id) {
                let spans = project.opening_span(o).is_some_and(|(o0, o1)| o0 <= z && z < o1);
                if project.opening_cuts_in_plan(id) || !spans {
                    continue;
                }
                let phase =
                    if o.phase == lot_model::Phase::New { lot_model::Phase::Demolished } else { lot_model::Phase::New };
                let area =
                    lot_geom::wall_section(wall, (o.offset, o.offset + o.width), (left, right)).intersection(&cut_all);
                let rings: Vec<Vec<PaperPoint>> = area.rings().iter().map(|r| paper(r, viewport)).collect();
                if !rings.is_empty() {
                    objects.push(PlanObject::Area(PlanArea { rings, fill: Fill::Solid(colors.of(phase)) }));
                    for ring in area.rings() {
                        objects.push(closed(paper(&ring, viewport), style.pens.layer));
                    }
                }
            }
        }
    }
    for area in material_areas(cut).values() {
        for ring in area.rings() {
            objects.push(closed(paper(&ring, viewport), style.pens.layer));
        }
    }
    for ring in cut_all.rings() {
        objects.push(closed(paper(&ring, viewport), style.pens.cut));
    }
}

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
            objects.push(PlanObject::Area(PlanArea { rings, fill: Fill::Solid(Color::BLACK) }));
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
                objects.push(PlanObject::Area(PlanArea { rings, fill: Fill::Solid(Color::gray(gray)) }));
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
            color: Color::BLACK,
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
        color: Color::BLACK,
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
    PlanObject::Line(Line { points, closed: true, pen: Pen::mm(width), style: LineStyle::Solid, color: Color::BLACK })
}

#[cfg(test)]
mod tests {
    use lot_model::{Door, Element, ElementId, Handing, OpeningKind, Project, WallSide};

    use super::*;

    #[test]
    fn faded_storey_is_light_thin_and_without_text() {
        let p = Project::sample();
        let viewport = Viewport { scale: 100.0, origin: PaperPoint::new(0.0, 0.0) };
        let objects = faded_storey(&p, ElementId(1), OfficeStandard::default_standard(), viewport);
        assert!(!objects.is_empty());
        for o in &objects {
            match o {
                PlanObject::Line(l) => {
                    assert!(l.color.r >= 0.69 && l.pen.width_mm <= 0.18, "{l:?}");
                }
                PlanObject::Area(a) => assert!(matches!(a.fill, Fill::Solid(c) if c.r >= 0.69)),
                PlanObject::Text(t) => panic!("Text im blassen Geschoss: {}", t.text),
            }
        }
    }

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

    fn fills(objects: &[PlanObject]) -> Vec<Color> {
        objects
            .iter()
            .filter_map(|o| match o {
                PlanObject::Area(PlanArea { fill: Fill::Solid(c), .. }) => Some(*c),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn existing_wall_with_new_opening_is_grey_with_yellow_breakthrough() {
        let mut p = Project::sample();
        let wall = p.walls().next().map(|(id, _)| id).unwrap();
        if let Some(Element::Wall(w)) = p.elements.get_mut(&wall) {
            w.phase = lot_model::Phase::Existing;
        }
        // Die Öffnung der Beispielwand reicht über die Schnittebene und ist neu.
        let view = FloorPlanView { storey: ElementId(1), cut_height: DEFAULT_CUT_HEIGHT, scale: 100.0 };
        let viewport = Viewport { scale: 100.0, origin: PaperPoint::new(0.0, 0.0) };
        let standard = OfficeStandard::default_standard();
        let colors = fills(&floor_plan(&p, view, standard, viewport));
        assert!(colors.contains(&standard.phases.of(lot_model::Phase::Existing)), "{colors:?}");
        assert!(colors.contains(&standard.phases.of(lot_model::Phase::Demolished)), "{colors:?}");
        // Im Bestandsplan ist alles wie gewohnt schwarz-weiß, die neue Öffnung fehlt.
        let existing = p.phase_view(lot_model::PhaseFilter::Existing);
        assert!(!existing.has_phases());
        assert_eq!(existing.elements.values().filter(|e| matches!(e, Element::Opening(_))).count(), 0);
        let colors = fills(&floor_plan(&existing, view, standard, viewport));
        assert!(!colors.contains(&standard.phases.of(lot_model::Phase::Existing)));
    }

    #[test]
    fn demolished_wall_is_yellow_and_gone_after_the_conversion() {
        let mut p = Project::sample();
        let wall = p.walls().next().map(|(id, _)| id).unwrap();
        if let Some(Element::Wall(w)) = p.elements.get_mut(&wall) {
            w.phase = lot_model::Phase::Demolished;
        }
        let view = FloorPlanView { storey: ElementId(1), cut_height: DEFAULT_CUT_HEIGHT, scale: 100.0 };
        let viewport = Viewport { scale: 100.0, origin: PaperPoint::new(0.0, 0.0) };
        let standard = OfficeStandard::default_standard();
        assert!(
            fills(&floor_plan(&p, view, standard, viewport))
                .contains(&standard.phases.of(lot_model::Phase::Demolished))
        );
        let planned = p.phase_view(lot_model::PhaseFilter::Planned);
        assert!(planned.walls().next().is_none());
        assert!(planned.elements.values().all(|e| !matches!(e, Element::Opening(_))));
    }
}
