//! Lageplan mit Abstandsflächen (5.01, 5.02). Vorschlag nach MBauVorlV § 7:
//! Grundstück und Nachbarn mit Bezeichnung, öffentliche Flächen, das Gebäude
//! mit Außenmaßen, Geschosszahl und Dachform, Höhenlage über NHN, Abstände zu
//! den Grenzen, Geländehöhen, Nordpfeil und die Abstandsflächen mit Tiefe.
//! Abstandsflächen außerhalb der zulässigen Fläche sind rot (PLAN.md,
//! Abschnitt 4: „rot markierte Abstandsfläche“).

use lot_geom::ModelGeometry;
use lot_model::{Point2, Project};
use lot_rules::setbacks::{SetbackArea, SetbackReport, building_plot};

use crate::dimensions::format_dimension;
use crate::section::format_height;
use crate::standard::OfficeStandard;
use crate::{Anchor, Area, Color, Fill, Line, LineStyle, PaperPoint, Pen, PlanObject, Text, Viewport, format_de};

/// Strich, Lücke für Abstandsflächen und Straßenmitte, in mm.
const DASH: [f64; 2] = [3.0, 1.5];
const DASH_DOT: [f64; 4] = [6.0, 1.0, 0.5, 1.0];

/// Rechnet Gebäudeklasse und Abstandsflächen für den Lageplan.
pub fn setback_report(project: &Project, geometry: &ModelGeometry) -> SetbackReport {
    let (rules, _) = lot_rules::StateRules::for_state(&project.info().state);
    let class = lot_rules::building_class(project, rules);
    lot_rules::setbacks(project, geometry, &class)
}

/// Zeichnet den Lageplan.
pub fn site_plan(
    project: &Project,
    geometry: &ModelGeometry,
    standard: &OfficeStandard,
    viewport: Viewport,
) -> Vec<PlanObject> {
    let report = setback_report(project, geometry);
    site_plan_with(project, &report, standard, viewport)
}

/// Zeichnet den Lageplan aus schon gerechneten Abstandsflächen.
pub fn site_plan_with(
    project: &Project,
    report: &SetbackReport,
    standard: &OfficeStandard,
    viewport: Viewport,
) -> Vec<PlanObject> {
    let style = standard.style_for(viewport.scale);
    let thin = Pen::mm(style.pens.annotation);
    let text = style.text;
    let to_paper = |p: [f64; 2]| viewport.to_paper(p[0], p[1]);
    let mut out = Vec::new();
    let info = project.info();
    let red = standard.phases.of(lot_model::Phase::New);

    // Abstandsflächen unten: leicht schraffiert, gestrichelt umrandet.
    for a in &report.areas {
        let rings: Vec<Vec<PaperPoint>> =
            a.area.rings().iter().map(|r| r.iter().map(|p| to_paper(*p)).collect()).collect();
        out.push(PlanObject::Area(Area {
            rings: rings.clone(),
            fill: Fill::Hatch { angle_deg: 45.0, spacing_mm: 2.0, pen: Pen::mm(0.13) },
        }));
    }
    let violations: Vec<Vec<PaperPoint>> =
        report.violations.rings().iter().map(|r| r.iter().map(|p| to_paper(*p)).collect()).collect();
    if !violations.is_empty() {
        out.push(PlanObject::Area(Area { rings: violations.clone(), fill: Fill::Solid(red) }));
    }
    for a in &report.areas {
        for ring in a.area.rings() {
            out.push(line(ring.iter().map(|p| to_paper(*p)).collect(), true, thin, LineStyle::Dashed(DASH.to_vec())));
        }
    }

    // Nachbargrundstücke und Baugrundstück.
    let plots: Vec<&lot_model::Plot> = project
        .elements
        .values()
        .filter_map(|e| match e {
            lot_model::Element::Plot(p) => Some(p),
            _ => None,
        })
        .collect();
    let main = building_plot(project);
    for p in &plots {
        let is_main = main.is_some_and(|m| std::ptr::eq(m, *p));
        let ring: Vec<PaperPoint> = p.outline.iter().map(|q| to_paper([q.x, q.y])).collect();
        let pen = if is_main { Pen::mm(style.pens.cut) } else { thin };
        out.push(line(ring, true, pen, LineStyle::Solid));
        let c = centroid(&p.outline);
        if is_main {
            continue;
        }
        out.push(label(to_paper([c.x, c.y]), p.name.clone(), text, Anchor::Center, 0.0));
    }
    if let Some(plot) = main {
        // Bezeichnung und Fläche in der Ecke mit dem meisten Platz: oben links innen.
        let (lo, hi) = bbox(&plot.outline);
        let at = to_paper([lo[0], hi[1]]);
        out.push(label(PaperPoint::new(at.x + 2.0, at.y - 2.0 - text), plot.name.clone(), text, Anchor::Left, 0.0));
        out.push(label(
            PaperPoint::new(at.x + 2.0, at.y - 3.0 - 2.0 * text),
            format!("{} m²", format_de(plot.area(), 0)),
            text * 0.8,
            Anchor::Left,
            0.0,
        ));
        // Öffentliche Flächen: Gegenseite dünn, Mitte strichpunktiert, Name.
        let ccw = signed(&plot.outline) > 0.0;
        for e in &plot.public_edges {
            let Some((a, b)) = plot.edge(e.edge) else { continue };
            let len = a.distance(b);
            if len < 1e-9 {
                continue;
            }
            let d = [(b.x - a.x) / len, (b.y - a.y) / len];
            let n = if ccw { [d[1], -d[0]] } else { [-d[1], d[0]] };
            let at = |s: f64, t: f64| to_paper([a.x + d[0] * s + n[0] * t, a.y + d[1] * s + n[1] * t]);
            out.push(line(vec![at(0.0, e.width), at(len, e.width)], false, thin, LineStyle::Solid));
            out.push(line(
                vec![at(0.0, e.width / 2.0), at(len, e.width / 2.0)],
                false,
                thin,
                LineStyle::Dashed(DASH_DOT.to_vec()),
            ));
            let angle = readable(d[1].atan2(d[0]).to_degrees());
            let name = if e.name.is_empty() { "öffentliche Fläche".to_string() } else { e.name.clone() };
            let mid = at(len / 2.0, e.width * 0.75);
            out.push(label(mid, format!("{name}, {} m breit", format_de(e.width, 2)), text, Anchor::Center, angle));
            let mid = at(len * 0.85, e.width / 2.0);
            out.push(label(PaperPoint::new(mid.x, mid.y + 0.8), "Mitte".into(), text * 0.7, Anchor::Center, angle));
        }
    }

    // Gebäude: Umriss dick, rot schraffiert als Neubau, mit Geschosszahl, Dachform, Höhen.
    let outline = lot_rules::building::building_outline(project);
    for ring in outline.rings() {
        let paper: Vec<PaperPoint> = ring.iter().map(|p| to_paper(*p)).collect();
        out.push(PlanObject::Area(Area { rings: vec![paper.clone()], fill: Fill::Solid(Color::rgb8(255, 255, 255)) }));
        for seg in crate::hatch_segments(std::slice::from_ref(&paper), 45.0, 1.5) {
            out.push(PlanObject::Line(Line {
                points: seg.to_vec(),
                closed: false,
                pen: Pen::mm(0.18),
                style: LineStyle::Solid,
                color: red,
            }));
        }
        out.push(line(paper, true, Pen::mm(style.pens.cut + 0.2), LineStyle::Solid));
    }
    if let Some((lo, hi)) = outline.bounds() {
        let c = to_paper([(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0]);
        let (rules, _) = lot_rules::StateRules::for_state(&info.state);
        let class = lot_rules::building_class(project, rules);
        let storeys = roman(class.above_ground.len());
        let roof = roof_form(project);
        out.push(label(PaperPoint::new(c.x, c.y + 0.5), format!("{storeys} {roof}"), text * 1.4, Anchor::Center, 0.0));
        let top = project_top(report);
        let mut lines = vec![match info.datum {
            Some(d) => format!("±0,00 = {} m ü. NHN", format_de(d, 2)),
            None => "±0,00 = Höhe über NHN fehlt".to_string(),
        }];
        if let Some(top) = top {
            lines.push(match info.datum {
                Some(d) => format!("OK {} = {} m ü. NHN", format_height(top), format_de(d + top, 2)),
                None => format!("OK {}", format_height(top)),
            });
        }
        lines.push(format!("Gebäudeklasse {}", class.roman()));
        for (i, l) in lines.into_iter().enumerate() {
            out.push(label(
                PaperPoint::new(c.x, c.y - 1.5 - (i as f64 + 1.0) * text * 1.3),
                l,
                text * 0.8,
                Anchor::Center,
                0.0,
            ));
        }
        // Außenmaße innen an der unteren und linken Kante.
        let (a, b) = (to_paper([lo[0], lo[1]]), to_paper([hi[0], hi[1]]));
        let size = text * 0.8;
        out.push(label(
            PaperPoint::new((a.x + b.x) / 2.0, a.y + 1.5),
            format_dimension(hi[0] - lo[0]),
            size,
            Anchor::Center,
            0.0,
        ));
        out.push(label(
            PaperPoint::new(a.x + 1.5 + size, (a.y + b.y) / 2.0),
            format_dimension(hi[1] - lo[1]),
            size,
            Anchor::Center,
            90.0,
        ));
    }

    // Tiefe jeder Abstandsfläche und Abstand bis zur Grenze, in der Mitte der Wand.
    for a in &report.areas {
        draw_setback_labels(&mut out, a, &to_paper, thin, text, red);
    }

    // Geländehöhen an den Höhenpunkten im Grundstück.
    if let (Some(t), Some(plot)) = (project.terrain(), main) {
        let ring: Vec<[f64; 2]> = plot.outline.iter().map(|p| [p.x, p.y]).collect();
        let inside = lot_geom::Area::from_ring(&ring).offset(0.01);
        for p in t.points.iter().filter(|p| inside.contains([p.x, p.y])) {
            let at = to_paper([p.x, p.y]);
            let s = 0.8;
            out.push(line(
                vec![PaperPoint::new(at.x - s, at.y - s), PaperPoint::new(at.x + s, at.y + s)],
                false,
                thin,
                LineStyle::Solid,
            ));
            out.push(line(
                vec![PaperPoint::new(at.x - s, at.y + s), PaperPoint::new(at.x + s, at.y - s)],
                false,
                thin,
                LineStyle::Solid,
            ));
            let value = match info.datum {
                Some(d) => format_de(d + p.z, 2),
                None => format_height(p.z),
            };
            out.push(label(PaperPoint::new(at.x + 1.2, at.y + 0.6), value, text * 0.7, Anchor::Left, 0.0));
        }
    }

    // Nordpfeil rechts oben neben allen Grundstücken, Legende darunter.
    let all: Vec<Point2> = plots.iter().flat_map(|p| p.outline.iter().copied()).collect();
    let (all_lo, all_hi) = if all.is_empty() {
        let (lo, hi) = outline.bounds().unwrap_or(([0.0, 0.0], [0.0, 0.0]));
        (lo, hi)
    } else {
        bbox(&all)
    };
    let corner = to_paper([all_hi[0], all_hi[1]]);
    out.extend(north_arrow(PaperPoint::new(corner.x + 12.0, corner.y - 10.0), info.north, thin));

    if main.is_some() {
        let at = to_paper([all_lo[0], all_lo[1]]);
        let s = &report.rules.setbacks;
        let mut legend = vec![format!(
            "Abstandsflächen nach {} ({}): T = {} H, mindestens {} m",
            s.source,
            report.rules.name,
            format_de(s.factor, 1),
            format_de(s.minimum, 2)
        )];
        if report.small_building
            && let Some(sb) = s.small_buildings
        {
            legend[0] =
                format!("Abstandsflächen nach {}: kleines Wohngebäude, T = {} m", s.source, format_de(sb.depth, 2));
        }
        legend.push(if report.ok() {
            "Alle Abstandsflächen liegen auf dem Grundstück oder bis zur Mitte öffentlicher Flächen".to_string()
        } else {
            format!("Rot: Abstandsfläche außerhalb, {} m²", format_de(report.violations.area(), 2))
        });
        for (i, l) in legend.into_iter().enumerate() {
            out.push(label(
                PaperPoint::new(at.x, at.y - 6.0 - i as f64 * text * 1.4),
                l,
                text * 0.8,
                Anchor::Left,
                0.0,
            ));
        }
    }
    out
}

/// Tiefe T und Abstand zur Grenze einer Abstandsfläche.
fn draw_setback_labels(
    out: &mut Vec<PlanObject>,
    a: &SetbackArea,
    to_paper: &impl Fn([f64; 2]) -> PaperPoint,
    pen: Pen,
    text: f64,
    red: Color,
) {
    let len = a.length();
    if len < 1.0 {
        return;
    }
    let d = [(a.end.x - a.start.x) / len, (a.end.y - a.start.y) / len];
    let n = a.outward;
    let (t_lo, t_hi) = a.depth();
    let (h_lo, h_hi) = a.height();
    let span = |lo: f64, hi: f64| {
        if hi - lo < 0.005 { format_de(lo, 2) } else { format!("{} bis {}", format_de(lo, 2), format_de(hi, 2)) }
    };
    let angle = readable(d[1].atan2(d[0]).to_degrees());
    // Text knapp innerhalb der Abstandsfläche, nahe der Wand, in Wandrichtung um ein Viertel versetzt.
    let s = len * 0.25;
    let base = [a.start.x + d[0] * s, a.start.y + d[1] * s];
    let t_min = t_lo.min(t_hi);
    let p1 = to_paper([base[0] + n[0] * t_min * 0.55, base[1] + n[1] * t_min * 0.55]);
    let p2 = to_paper([base[0] + n[0] * t_min * 0.35, base[1] + n[1] * t_min * 0.35]);
    out.push(label(p1, format!("{} T = {} m", a.facing, span(t_lo, t_hi)), text * 0.8, Anchor::Center, angle));
    out.push(label(p2, format!("H = {} m", span(h_lo, h_hi)), text * 0.7, Anchor::Center, angle));
    // Vorhandener Abstand bis zur Grenze: Maßlinie bei drei Vierteln der Wand.
    if let Some(avail) = a.available {
        let at = [a.start.x + d[0] * len * 0.75, a.start.y + d[1] * len * 0.75];
        let p = to_paper(at);
        let q = to_paper([at[0] + n[0] * avail, at[1] + n[1] * avail]);
        out.push(line(vec![p, q], false, pen, LineStyle::Solid));
        for end in [p, q] {
            out.push(tick(end, [n[0], n[1]], pen));
        }
        let at = PaperPoint::new((p.x + q.x) / 2.0, (p.y + q.y) / 2.0);
        let across = readable(n[1].atan2(n[0]).to_degrees());
        let color = if a.ok() { Color::BLACK } else { red };
        out.push(PlanObject::Text(Text {
            position: PaperPoint::new(at.x - n[1] * 1.2, at.y + n[0] * 1.2),
            text: format_dimension(avail),
            height_mm: text * 0.8,
            anchor: Anchor::Center,
            rotation: across,
        }));
        if !a.ok() {
            out.push(line(vec![p, q], false, Pen::mm(0.35), LineStyle::Solid));
            if let Some(PlanObject::Line(l)) = out.last_mut() {
                l.color = color;
            }
        }
    }
}

/// Maßbegrenzung als Schrägstrich unter 45°.
fn tick(at: PaperPoint, d: [f64; 2], pen: Pen) -> PlanObject {
    let (c, s) = (std::f64::consts::FRAC_1_SQRT_2, std::f64::consts::FRAC_1_SQRT_2);
    let r = [d[0] * c - d[1] * s, d[0] * s + d[1] * c];
    line(
        vec![PaperPoint::new(at.x - r[0], at.y - r[1]), PaperPoint::new(at.x + r[0], at.y + r[1])],
        false,
        Pen::mm(pen.width_mm * 2.0),
        LineStyle::Solid,
    )
}

/// Nordpfeil: Kreis mit Pfeil nach Norden und „N“.
fn north_arrow(at: PaperPoint, north: f64, pen: Pen) -> Vec<PlanObject> {
    let r = 6.0;
    let (sin, cos) = north.to_radians().sin_cos();
    let rot = |x: f64, y: f64| PaperPoint::new(at.x + x * cos - y * sin, at.y + x * sin + y * cos);
    let circle: Vec<PaperPoint> = (0..48)
        .map(|i| {
            let t = i as f64 / 48.0 * std::f64::consts::TAU;
            PaperPoint::new(at.x + r * t.cos(), at.y + r * t.sin())
        })
        .collect();
    let arrow = vec![rot(0.0, r + 2.5), rot(2.2, -r * 0.6), rot(0.0, -r * 0.2), rot(-2.2, -r * 0.6)];
    vec![
        line(circle, true, pen, LineStyle::Solid),
        PlanObject::Area(Area { rings: vec![arrow.clone()], fill: Fill::Solid(Color::BLACK) }),
        line(arrow, true, pen, LineStyle::Solid),
        PlanObject::Text(Text {
            position: rot(0.0, r + 4.0),
            text: "N".into(),
            height_mm: 3.5,
            anchor: Anchor::Center,
            rotation: 0.0,
        }),
    ]
}

/// Höchste Oberkante aller Abstandsflächen über ±0,00.
fn project_top(report: &SetbackReport) -> Option<f64> {
    report.areas.iter().flat_map(|a| a.samples.iter().map(|s| s.top)).reduce(f64::max)
}

/// Dachform als Kürzel: FD Flachdach, SD Satteldach, PD Pultdach, WD Walmdach.
fn roof_form(project: &Project) -> &'static str {
    let roofs: Vec<&lot_model::Roof> = project
        .elements
        .values()
        .filter_map(|e| match e {
            lot_model::Element::Roof(r) => Some(r),
            _ => None,
        })
        .collect();
    let Some(main) = roofs.iter().max_by(|a, b| poly_area(&a.outline).total_cmp(&poly_area(&b.outline))) else {
        return "";
    };
    let eaves =
        main.edges.iter().filter(|e| matches!(e, lot_model::RoofEdge::Eave { slope, .. } if *slope > 5.0)).count();
    match eaves {
        0 => "FD",
        1 => "PD",
        2 => "SD",
        _ => "WD",
    }
}

fn poly_area(points: &[Point2]) -> f64 {
    signed(points).abs()
}

fn signed(points: &[Point2]) -> f64 {
    let n = points.len();
    (0..n).map(|i| points[i].x * points[(i + 1) % n].y - points[(i + 1) % n].x * points[i].y).sum::<f64>() / 2.0
}

fn centroid(points: &[Point2]) -> Point2 {
    let n = points.len().max(1) as f64;
    Point2::new(points.iter().map(|p| p.x).sum::<f64>() / n, points.iter().map(|p| p.y).sum::<f64>() / n)
}

fn bbox(points: &[Point2]) -> ([f64; 2], [f64; 2]) {
    points.iter().fold(([f64::INFINITY; 2], [f64::NEG_INFINITY; 2]), |(lo, hi), p| {
        ([lo[0].min(p.x), lo[1].min(p.y)], [hi[0].max(p.x), hi[1].max(p.y)])
    })
}

/// Römische Zahl für die Geschosszahl im Lageplan.
fn roman(n: usize) -> String {
    const R: [&str; 10] = ["", "I", "II", "III", "IV", "V", "VI", "VII", "VIII", "IX"];
    R.get(n).map_or_else(|| n.to_string(), |s| s.to_string())
}

/// Winkel so, dass Text von unten oder rechts lesbar bleibt (DIN 1356-1).
fn readable(angle: f64) -> f64 {
    let mut a = angle.rem_euclid(360.0);
    if a > 90.0 + 1e-9 && a <= 270.0 + 1e-9 {
        a -= 180.0;
    }
    if a > 180.0 {
        a -= 360.0;
    }
    a
}

fn line(points: Vec<PaperPoint>, closed: bool, pen: Pen, style: LineStyle) -> PlanObject {
    PlanObject::Line(Line { points, closed, pen, style, color: Color::BLACK })
}

fn label(position: PaperPoint, text: String, height: f64, anchor: Anchor, rotation: f64) -> PlanObject {
    PlanObject::Text(Text { position, text, height_mm: height, anchor, rotation })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn readable_angles_stay_between_minus_and_plus_ninety() {
        assert_eq!(readable(0.0), 0.0);
        assert_eq!(readable(180.0), 0.0);
        assert_eq!(readable(-90.0), 90.0);
        assert_eq!(readable(270.0), 90.0);
        assert!((readable(135.0) + 45.0).abs() < 1e-9);
    }

    #[test]
    fn roman_numerals() {
        assert_eq!(roman(4), "IV");
        assert_eq!(roman(12), "12");
    }
}
