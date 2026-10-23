//! Prüfblatt für Meilenstein M0.
//!
//! Prüft die Planausgabe, bevor es echte Darstellungsregeln gibt: Maßstab,
//! Linienbreiten, Schraffur, Schrift und einen ersten Grundrissschnitt aus
//! dem Modell. Die Darstellung der Wand folgt noch keinen Regeln, die kommen
//! in M1 mit DIN 1356.

use lot_model::Project;

use crate::floor_plan::{FloorPlanView, floor_plan};
use crate::standard::OfficeStandard;

use crate::{
    Anchor, Area, Fill, Gray, PEN_WIDTHS_MM, PaperPoint, Pen, PlanObject, Sheet, Viewport, format_de, rect_points,
};

/// Höhe des Grundrissschnitts über Unterkante Wand, in Metern.
pub const CUT_HEIGHT: f64 = 1.0;

const TITLE: f64 = 3.5;
const LABEL: f64 = 2.5;

pub fn sheet(project: &Project, source: &str) -> Sheet {
    let mut s = Sheet::a4_landscape("Lot M0 Prüfblatt");
    let thin = Pen::mm(0.25);
    let bold = Pen::mm(0.5);

    s.rect(10.0, 10.0, 277.0, 190.0, thin);
    s.text(20.0, 189.0, "Lot M0 Prüfblatt", 5.0, Anchor::Left);
    s.text(
        20.0,
        182.0,
        "Maßstab 1:100. In Originalgröße drucken (100 %), nicht \"An Seite anpassen\".",
        LABEL,
        Anchor::Left,
    );

    // A: Kalibrierrechteck 5,00 x 3,00 m
    let vp = Viewport { scale: 100.0, origin: PaperPoint::new(25.0, 130.0) };
    s.text(25.0, 168.0, "A  Rechteck 5,00 x 3,00 m bei 1:100 = 50 x 30 mm", TITLE, Anchor::Left);
    let a = vp.to_paper(0.0, 0.0);
    let b = vp.to_paper(5.0, 3.0);
    s.rect(a.x, a.y, b.x - a.x, b.y - a.y, bold);
    dimension(&mut s, a.x, b.x, a.y - 8.0, "5,00");
    s.text(b.x + 3.0, (a.y + b.y) / 2.0 - LABEL / 2.0, "3,00", LABEL, Anchor::Left);

    // B: Maßstabsleiste 0 bis 5 m
    s.text(25.0, 108.0, "B  Maßstabsleiste 1:100, ein Feld = 1 m = 10 mm", TITLE, Anchor::Left);
    let bar = Viewport { scale: 100.0, origin: PaperPoint::new(25.0, 96.0) };
    for m in 0..5 {
        let p = bar.to_paper(m as f64, 0.0);
        let q = bar.to_paper(m as f64 + 1.0, 0.0);
        let ring = rect_points(p.x, p.y, q.x - p.x, 2.0);
        if m % 2 == 0 {
            s.objects.push(PlanObject::Area(Area { rings: vec![ring.clone()], fill: Fill::Solid(Gray::BLACK) }));
        }
        s.polygon(ring, Pen::mm(0.18));
    }
    for m in 0..=5 {
        let p = bar.to_paper(m as f64, 0.0);
        s.text(p.x, p.y - 5.0, format!("{m}"), LABEL, Anchor::Center);
    }
    s.text(bar.to_paper(5.0, 0.0).x + 4.0, 95.5, "m", LABEL, Anchor::Left);

    // D: Grundrissschnitt der Wand aus dem Modell
    s.text(
        25.0,
        72.0,
        format!("D  Grundriss aus dem Modell, Schnitt bei +{} m, 1:100", format_de(CUT_HEIGHT, 2)),
        TITLE,
        Anchor::Left,
    );
    let plan = Viewport { scale: 100.0, origin: PaperPoint::new(25.0, 48.0) };
    if let Some((storey, _)) = project.storeys().first() {
        let view = FloorPlanView { storey: *storey, cut_height: CUT_HEIGHT, scale: 100.0 };
        s.objects.extend(floor_plan(project, view, OfficeStandard::default_standard(), plan));
    }
    if let Some((min, max)) = lot_geom::plan_extent(project) {
        let a = plan.to_paper(min[0], min[1]);
        let b = plan.to_paper(max[0], max[1]);
        dimension(&mut s, a.x, b.x, a.y - 8.0, &format_de(max[0] - min[0], 2));
    }
    s.text(
        25.0,
        28.0,
        "Darstellung nach Bürostandard 1:100 (Vorschlag M1), Fenster noch ohne Rahmen.",
        LABEL,
        Anchor::Left,
    );

    // C: Stiftreihe nach DIN EN ISO 128-20
    s.text(160.0, 168.0, "C  Linienbreiten in mm", TITLE, Anchor::Left);
    for (i, width) in PEN_WIDTHS_MM.iter().enumerate() {
        let y = 160.0 - i as f64 * 7.0;
        s.line(vec![PaperPoint::new(160.0, y), PaperPoint::new(220.0, y)], Pen::mm(*width));
        s.text(226.0, y - LABEL / 2.0, format_de(*width, 2), LABEL, Anchor::Left);
    }

    // E: Schraffur und Füllung
    s.text(160.0, 100.0, "E  Schraffur 45°, Abstand 2 mm, Linie 0,18 / Fläche grau", TITLE, Anchor::Left);
    let hatch = rect_points(160.0, 72.0, 30.0, 20.0);
    s.objects.push(PlanObject::Area(Area {
        rings: vec![hatch.clone()],
        fill: Fill::Hatch { angle_deg: 45.0, spacing_mm: 2.0, pen: Pen::mm(0.18) },
    }));
    s.polygon(hatch, Pen::mm(0.35));
    let gray = rect_points(200.0, 72.0, 30.0, 20.0);
    s.objects.push(PlanObject::Area(Area { rings: vec![gray.clone()], fill: Fill::Solid(Gray(0.8)) }));
    s.polygon(gray, Pen::mm(0.35));

    // F: Schrift
    s.text(160.0, 60.0, "F  Schrift, Versalhöhe in mm", TITLE, Anchor::Left);
    s.text(160.0, 51.0, "2,5  Wohnen 24,31 m² ÄÖÜ äöü ß", 2.5, Anchor::Left);
    s.text(160.0, 44.0, "3,5  Wohnen 24,31 m² ÄÖÜ äöü ß", 3.5, Anchor::Left);
    s.text(160.0, 35.0, "5,0  Wohnen 24,31 m²", 5.0, Anchor::Left);

    s.text(282.0, 13.0, format!("Erzeugt von lot-cli {} aus {source}", env!("CARGO_PKG_VERSION")), 2.0, Anchor::Right);
    s
}

/// Einfache Maßlinie mit Schrägstrichen an den Enden und Maßzahl darüber.
/// Die Form der Bemaßung ist ein Platzhalter, die Regeln folgen in M2.
fn dimension(s: &mut Sheet, x0: f64, x1: f64, y: f64, label: &str) {
    let pen = Pen::mm(0.18);
    s.line(vec![PaperPoint::new(x0 - 2.0, y), PaperPoint::new(x1 + 2.0, y)], pen);
    for x in [x0, x1] {
        s.line(vec![PaperPoint::new(x, y - 2.0), PaperPoint::new(x, y + 2.0)], pen);
        s.line(vec![PaperPoint::new(x - 1.0, y - 1.0), PaperPoint::new(x + 1.0, y + 1.0)], Pen::mm(0.35));
    }
    s.text((x0 + x1) / 2.0, y + 1.2, label, LABEL, Anchor::Center);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calibration_rectangle_is_50_by_30_mm() {
        let s = sheet(&Project::sample(), "test");
        let rect = s
            .objects
            .iter()
            .find_map(|o| match o {
                PlanObject::Line(l) if l.closed && l.pen.width_mm == 0.5 => Some(l.points.clone()),
                _ => None,
            })
            .unwrap();
        assert_eq!(rect[1].x - rect[0].x, 50.0);
        assert_eq!(rect[2].y - rect[1].y, 30.0);
    }

    #[test]
    fn wall_cut_shows_window_gap() {
        // Schnitt bei 1,00 m geht durch das Fenster (Brüstung 0,90 m): zwei Wandstücke.
        let regions = lot_geom::layer_regions(&Project::sample(), lot_model::ElementId(1), CUT_HEIGHT);
        assert_eq!(regions[0].area.rings().len(), 2);
    }
}
