//! Maßketten im Grundriss (3.05). Vorschlag nach DIN 1356-1:1995-02 und
//! DIN 406-11:1992-12:
//!
//! - Maßlinie und Maßhilfslinien dünn, Maßhilfslinien kurz beidseits der Maßlinie
//! - Maßbegrenzung als Schrägstrich unter 45°
//! - Maßzahl über der Maßlinie, lesbar von unten oder von rechts
//! - unter 1 m in Zentimetern, sonst in Metern, Millimeter hochgestellt (36⁵, 1,13⁵)
//! - bei Öffnungen die Höhe unter der Maßlinie

use lot_model::{Element, ElementId, Project};

use crate::standard::ScaleStyle;
use crate::{Anchor, Color, Line, LineStyle, Pen, PlanObject, Text, Viewport};

/// Länge der Maßhilfslinie über die Maßlinie hinaus und zum Bauteil hin, in mm auf dem Blatt.
const EXTENSION: (f64, f64) = (1.5, 3.0);
/// Überstand der Maßlinie über die äußeren Maßhilfslinien, in mm auf dem Blatt.
const OVERSHOOT: f64 = 1.5;
/// Halbe Länge des Schrägstrichs, in mm auf dem Blatt.
const TICK: f64 = 1.0;
/// Abstand der Maßzahl von der Maßlinie, in mm auf dem Blatt.
const TEXT_GAP: f64 = 0.8;

const SUPERSCRIPT: [char; 10] = ['⁰', '¹', '²', '³', '⁴', '⁵', '⁶', '⁷', '⁸', '⁹'];

/// Maßzahl nach deutscher Planungspraxis: 0,365 m wird „36⁵“, 1,135 m wird
/// „1,13⁵“, 11 m wird „11,00“. Gerundet auf Millimeter.
pub fn format_dimension(metres: f64) -> String {
    let mm = (metres.abs() * 1000.0 * (1.0 + 1e-12)).round() as i64;
    let tail = |mm: i64| if mm % 10 == 0 { String::new() } else { SUPERSCRIPT[(mm % 10) as usize].to_string() };
    if mm < 1000 {
        format!("{}{}", mm / 10, tail(mm))
    } else {
        format!("{},{:02}{}", mm / 1000, (mm % 1000) / 10, tail(mm))
    }
}

/// Alle Maßketten eines Geschosses.
pub fn draw_dimensions(
    objects: &mut Vec<PlanObject>,
    project: &Project,
    storey: ElementId,
    style: &ScaleStyle,
    viewport: Viewport,
) {
    let mm = viewport.scale / 1000.0;
    for (_, dim) in project.dimensions_on(storey) {
        let points = lot_geom::dimensions::resolve(project, dim);
        if points.len() < 2 {
            continue;
        }
        let d = lot_geom::dimensions::direction(dim);
        let n = [-d[1], d[0]];
        let foot = |along: f64| [dim.position.x + d[0] * along, dim.position.y + d[1] * along];
        let paper = |p: [f64; 2]| viewport.to_paper(p[0], p[1]);
        let line = |a: [f64; 2], b: [f64; 2], width: f64| {
            PlanObject::Line(Line {
                points: vec![paper(a), paper(b)],
                closed: false,
                pen: Pen::mm(width),
                style: LineStyle::Solid,
                color: Color::BLACK,
            })
        };
        let (first, last) = (points[0].along, points[points.len() - 1].along);
        objects.push(line(foot(first - OVERSHOOT * mm), foot(last + OVERSHOOT * mm), style.pens.annotation));

        // Lesbar von unten oder von rechts: Leserichtung zwischen −90° und 90°.
        let mut read = d;
        let mut angle = d[1].atan2(d[0]).to_degrees();
        if angle > 90.0 + 1e-9 || angle <= -90.0 + 1e-9 {
            read = [-d[0], -d[1]];
            angle = read[1].atan2(read[0]).to_degrees();
        }
        let up = [-read[1], read[0]];
        let tick = [(read[0] + up[0]) * TICK * mm / 2f64.sqrt(), (read[1] + up[1]) * TICK * mm / 2f64.sqrt()];

        for p in &points {
            let f = foot(p.along);
            // Maßhilfslinie: zum Bauteil hin und ein Stück darüber hinaus.
            let side = if (p.point[0] - f[0]) * n[0] + (p.point[1] - f[1]) * n[1] >= 0.0 { 1.0 } else { -1.0 };
            let toward = [n[0] * side, n[1] * side];
            let a = [f[0] - toward[0] * EXTENSION.0 * mm, f[1] - toward[1] * EXTENSION.0 * mm];
            let b = [f[0] + toward[0] * EXTENSION.1 * mm, f[1] + toward[1] * EXTENSION.1 * mm];
            objects.push(line(a, b, style.pens.annotation));
            objects.push(line([f[0] - tick[0], f[1] - tick[1]], [f[0] + tick[0], f[1] + tick[1]], style.pens.view));
        }

        let height = style.dimension_text;
        for pair in points.windows(2) {
            let mid = foot((pair[0].along + pair[1].along) / 2.0);
            let above = [mid[0] + up[0] * TEXT_GAP * mm, mid[1] + up[1] * TEXT_GAP * mm];
            objects.push(PlanObject::Text(Text {
                position: paper(above),
                text: format_dimension(pair[1].along - pair[0].along),
                height_mm: height,
                anchor: Anchor::Center,
                rotation: angle,
            }));
            // Öffnung: Höhe unter der Maßlinie.
            if let (Some(a), Some(b)) = (pair[0].opening, pair[1].opening)
                && a == b
                && let Some(Element::Opening(o)) = project.elements.get(&a)
            {
                let below = [mid[0] - up[0] * (TEXT_GAP + height) * mm, mid[1] - up[1] * (TEXT_GAP + height) * mm];
                objects.push(PlanObject::Text(Text {
                    position: paper(below),
                    text: format_dimension(o.height),
                    height_mm: height,
                    anchor: Anchor::Center,
                    rotation: angle,
                }));
            }
        }
    }
}

/// Rohbaumaße von Türen und Durchgängen, die keine Maßkette erfasst, als
/// „Breite/Höhe“ neben der Öffnung auf der Seite ohne Aufschlag. Der Bauantrag
/// verlangt die Öffnungsmaße der Türen (MBauVorlV § 8 Abs. 2 Nr. 2 b).
pub fn draw_door_sizes(
    objects: &mut Vec<PlanObject>,
    project: &Project,
    storey: ElementId,
    z_cut: f64,
    style: &ScaleStyle,
    viewport: Viewport,
) {
    let mm = viewport.scale / 1000.0;
    let dimensioned: std::collections::BTreeSet<ElementId> = project
        .dimensions_on(storey)
        .flat_map(|(_, d)| lot_geom::dimensions::resolve(project, d))
        .filter_map(|p| p.opening)
        .collect();
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
        for (id, o) in project.openings_in(wall_id) {
            if dimensioned.contains(&id) || matches!(o.kind, lot_model::OpeningKind::Window) {
                continue;
            }
            let Some((o0, o1)) = project.opening_span(o) else { continue };
            if o0 > z_cut || o1 <= z_cut {
                continue;
            }
            // Auf der Seite ohne Aufschlag; bei Durchgängen links der Wand.
            let side = match o.kind {
                lot_model::OpeningKind::Door(d) => d.opens_to.other(),
                _ => lot_model::WallSide::Left,
            };
            let height = style.dimension_text;
            let (face, out) = match side {
                lot_model::WallSide::Left => (left, 1.0),
                lot_model::WallSide::Right => (right, -1.0),
            };
            let s = o.offset + o.width / 2.0;
            let gap = (TEXT_GAP + 0.5 * height) * mm;
            let p = wall.point_at(s, face + out * gap);
            let (tx, ty) = wall.tangent_at(s);
            let mut angle = ty.atan2(tx).to_degrees();
            if angle > 90.0 + 1e-9 || angle <= -90.0 + 1e-9 {
                angle = if angle > 0.0 { angle - 180.0 } else { angle + 180.0 };
            }
            objects.push(PlanObject::Text(Text {
                position: viewport.to_paper(p.x, p.y),
                text: format!("{}/{}", format_dimension(o.width), format_dimension(o.height)),
                height_mm: height,
                anchor: Anchor::Center,
                rotation: angle,
            }));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_follow_german_practice() {
        assert_eq!(format_dimension(0.365), "36⁵");
        assert_eq!(format_dimension(0.24), "24");
        assert_eq!(format_dimension(1.135), "1,13⁵");
        assert_eq!(format_dimension(2.01), "2,01");
        assert_eq!(format_dimension(11.0), "11,00");
        assert_eq!(format_dimension(0.885), "88⁵");
        assert_eq!(format_dimension(0.06), "6");
    }
}
