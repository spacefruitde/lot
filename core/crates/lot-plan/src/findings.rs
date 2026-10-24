//! Hinweise im Modell (PLAN.md, Abschnitt 4): Befunde des Prüfberichts mit
//! Ort erscheinen im Grundriss als farbiger Kreis mit Ausrufezeichen und
//! kurzem Text. Nur auf dem Bildschirm und in Prüfbildern, nicht in den
//! Bauantragsplänen.

use lot_model::ElementId;
use lot_rules::{CheckReport, Status};

use crate::{Anchor, Fill, Line, LineStyle, PaperPoint, Pen, PlanObject, Text, Viewport};

/// Hinweise eines Geschosses: alles, was nicht erfüllt ist, und Hinweise mit Ort.
pub fn draw_findings(objects: &mut Vec<PlanObject>, report: &CheckReport, storey: ElementId, viewport: Viewport) {
    let mut placed: Vec<PaperPoint> = Vec::new();
    for f in report.findings().filter(|f| f.status != Status::Ok && f.storey == Some(storey)) {
        let Some(at) = f.at else { continue };
        let mut c = viewport.to_paper(at.x, at.y);
        // Hinweise am selben Ort untereinander.
        while placed.iter().any(|p| (p.x - c.x).abs() < 4.0 && (p.y - c.y).abs() < 4.0) {
            c.y -= 6.0;
        }
        placed.push(c);
        let color = crate::table::status_color(f.status);
        let r = 2.4;
        let ring: Vec<PaperPoint> = (0..24)
            .map(|k| {
                let t = k as f64 / 24.0 * std::f64::consts::TAU;
                PaperPoint::new(c.x + r * t.cos(), c.y + r * t.sin())
            })
            .collect();
        objects.push(PlanObject::Area(crate::Area { rings: vec![ring.clone()], fill: Fill::Solid(color) }));
        objects.push(PlanObject::Line(Line {
            points: ring,
            closed: true,
            pen: Pen::mm(0.25),
            style: LineStyle::Solid,
            color,
        }));
        objects.push(PlanObject::Text(Text {
            position: PaperPoint::new(c.x, c.y - 1.1),
            text: "!".into(),
            height_mm: 2.4,
            anchor: Anchor::Center,
            rotation: 0.0,
        }));
        let short: String = f.text.chars().take(60).collect();
        let short = if f.text.chars().count() > 60 { format!("{short} …") } else { short };
        objects.push(PlanObject::Text(Text {
            position: PaperPoint::new(c.x + r + 1.0, c.y - 0.8),
            text: short,
            height_mm: 1.8,
            anchor: Anchor::Left,
            rotation: 0.0,
        }));
    }
}
