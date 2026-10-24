//! Aufzüge im Grundriss. Vorschlag nach DIN 1356-1:1995-02: Schacht mit
//! Diagonalkreuz, darin der Fahrkorb. Für den Bauantrag steht die nutzbare
//! Grundfläche des Fahrkorbs dabei (MBauVorlV § 8 Abs. 2 Nr. 2 e).

use lot_model::{ElementId, Project};

use crate::standard::ScaleStyle;
use crate::{Anchor, Color, Line, LineStyle, PaperPoint, Pen, PlanObject, Text, Viewport, format_de};

/// Alle Aufzüge mit Haltestelle in diesem Geschoss.
pub fn draw_elevators(
    objects: &mut Vec<PlanObject>,
    project: &Project,
    storey: ElementId,
    style: &ScaleStyle,
    viewport: Viewport,
) {
    let elevators: Vec<&lot_model::Elevator> = project
        .elements
        .values()
        .filter_map(|e| match e {
            lot_model::Element::Elevator(x) if x.stops_at(storey) => Some(x),
            _ => None,
        })
        .collect();
    if elevators.is_empty() {
        return;
    }
    let finder = lot_geom::spaces::SpaceFinder::new(project, storey);
    let line = |points: Vec<PaperPoint>, width: f64, closed: bool| {
        PlanObject::Line(Line { points, closed, pen: Pen::mm(width), style: LineStyle::Solid, color: Color::BLACK })
    };
    for e in elevators {
        let cabin: Vec<PaperPoint> = e.cabin().iter().map(|p| viewport.to_paper(p.x, p.y)).collect();
        objects.push(line(cabin.clone(), style.pens.view, true));
        // Kreuz über den Schacht, oder über den Fahrkorb, wenn kein Schacht ihn umschließt.
        let (lo, hi) = finder
            .area_at([e.position.x, e.position.y])
            .and_then(|a| a.bounds())
            .map(|(lo, hi)| (viewport.to_paper(lo[0], lo[1]), viewport.to_paper(hi[0], hi[1])))
            .unwrap_or_else(|| {
                let xs = cabin.iter().map(|p| p.x);
                let ys = cabin.iter().map(|p| p.y);
                (
                    PaperPoint::new(xs.clone().fold(f64::INFINITY, f64::min), ys.clone().fold(f64::INFINITY, f64::min)),
                    PaperPoint::new(xs.fold(f64::NEG_INFINITY, f64::max), ys.fold(f64::NEG_INFINITY, f64::max)),
                )
            });
        objects.push(line(vec![lo, hi], style.pens.hatch, false));
        objects.push(line(vec![PaperPoint::new(lo.x, hi.y), PaperPoint::new(hi.x, lo.y)], style.pens.hatch, false));
        // Tür an der Vorderkante des Fahrkorbs, breit gezeichnet.
        let (_, r) = e.frame();
        let c = e.door_center();
        let half = e.door_width / 2.0;
        let a = viewport.to_paper(c.x + r[0] * half, c.y + r[1] * half);
        let b = viewport.to_paper(c.x - r[0] * half, c.y - r[1] * half);
        objects.push(line(vec![a, b], style.pens.cut, false));
        // Beschriftung im Fahrkorb.
        let center = viewport.to_paper(e.position.x, e.position.y);
        let lines = [
            ("Aufzug".to_string(), style.space_name),
            (format!("{} x {}", format_de(e.cabin_width, 2), format_de(e.cabin_depth, 2)), style.space_text),
            (format!("{} m²", format_de(e.cabin_area(), 2)), style.space_text),
        ];
        let total: f64 = lines.iter().map(|(_, h)| h * 1.6).sum::<f64>() - lines[2].1 * 0.6;
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

/// Ob an diesem Punkt ein Aufzug beschriftet wird; dort entfällt der Raumstempel.
pub fn labels_space(project: &Project, storey: ElementId, p: lot_model::Point2) -> bool {
    project.elements.values().any(|e| match e {
        lot_model::Element::Elevator(x) if x.stops_at(storey) => {
            let (d, r) = x.frame();
            let (dx, dy) = (p.x - x.position.x, p.y - x.position.y);
            (dx * d[0] + dy * d[1]).abs() <= x.cabin_depth / 2.0 + 0.3
                && (dx * r[0] + dy * r[1]).abs() <= x.cabin_width / 2.0 + 0.3
        }
        _ => false,
    })
}
