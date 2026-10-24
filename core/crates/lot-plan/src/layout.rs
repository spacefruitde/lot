//! Planblätter mit Plankopf (3.07). Vorschlag nach DIN EN ISO 5457 und
//! DIN EN ISO 7200: Blatt quer, Heftrand links 20 mm, sonst 10 mm, Schriftfeld
//! unten rechts. Jede Ansicht steht mit ihrer Mitte an der angegebenen Stelle,
//! darunter Titel und Maßstab.

use lot_geom::ModelGeometry;
use lot_model::{Element, PlanSheet, Project, ViewContent};

use crate::floor_plan::{DEFAULT_CUT_HEIGHT, FloorPlanView, floor_plan};
use crate::standard::OfficeStandard;
use crate::{Anchor, PaperPoint, Pen, PlanObject, Sheet, Viewport, bounds, rect_points};

/// Blattgrößen quer in mm nach DIN EN ISO 216.
pub fn format_size(format: &str) -> (f64, f64) {
    match format {
        "A4" => (297.0, 210.0),
        "A3" => (420.0, 297.0),
        "A2" => (594.0, 420.0),
        "A1" => (841.0, 594.0),
        _ => (1189.0, 841.0),
    }
}

/// Heftrand links und Rand sonst, in mm.
const BINDING: f64 = 20.0;
const MARGIN: f64 = 10.0;
/// Schriftfeld: Breite und Zeilenhöhe in mm.
const BLOCK_WIDTH: f64 = 180.0;
const ROW: f64 = 13.0;

/// Eine Ansicht, an ihren Platz auf dem Blatt geschoben.
struct Drawing {
    objects: Vec<PlanObject>,
    title: String,
    scale: f64,
    center: PaperPoint,
    /// Umriss der Zeichnung ohne Titel.
    lo: PaperPoint,
    hi: PaperPoint,
}

/// Abstand von der Zeichnung bis zur Titelzeile und Höhe des Titelbereichs, in mm.
const TITLE_GAP: f64 = 8.0;
const TITLE_HEIGHT: f64 = 9.0;

impl Drawing {
    /// Umriss einschließlich Titel und Maßstab.
    fn extent(&self) -> (PaperPoint, PaperPoint) {
        let half = 2.6 * self.title.chars().count() as f64 / 2.0;
        let lo = PaperPoint::new(self.lo.x.min(self.center.x - half), self.lo.y - TITLE_GAP - TITLE_HEIGHT);
        let hi = PaperPoint::new(self.hi.x.max(self.center.x + half), self.hi.y);
        (lo, hi)
    }
}

fn drawings(project: &Project, geometry: &ModelGeometry, sheet: &PlanSheet, standard: &OfficeStandard) -> Vec<Drawing> {
    let mut out = Vec::new();
    for placed in &sheet.views {
        let viewport = Viewport { scale: placed.scale, origin: PaperPoint::new(0.0, 0.0) };
        let (mut objects, title) = match placed.content {
            ViewContent::FloorPlan { storey } => {
                let view = FloorPlanView { storey, cut_height: DEFAULT_CUT_HEIGHT, scale: placed.scale };
                let name = project.storey(storey).map_or("?".to_string(), |s| s.name.clone());
                (floor_plan(project, view, standard, viewport), format!("Grundriss {name}"))
            }
            ViewContent::Section { view } => match project.elements.get(&view) {
                Some(Element::SectionView(v)) => {
                    (crate::section::section(project, geometry, v, standard, viewport), v.name.clone())
                }
                _ => (Vec::new(), "Ansicht fehlt".to_string()),
            },
            ViewContent::Presentation { storey } => {
                let name = project.storey(storey).map_or("?".to_string(), |s| s.name.clone());
                (
                    crate::presentation::presentation_plan(project, storey, placed.scale, standard, viewport),
                    format!("Grundriss {name}"),
                )
            }
            ViewContent::SitePlan => {
                (crate::site_plan::site_plan(project, geometry, standard, viewport), "Lageplan".to_string())
            }
        };
        let Some((lo, hi)) = bounds(&objects) else { continue };
        let (dx, dy) = (placed.center.x - (lo.x + hi.x) / 2.0, placed.center.y - (lo.y + hi.y) / 2.0);
        for o in &mut objects {
            o.translate(dx, dy);
        }
        out.push(Drawing {
            objects,
            title,
            scale: placed.scale,
            center: PaperPoint::new(placed.center.x, placed.center.y),
            lo: PaperPoint::new(lo.x + dx, lo.y + dy),
            hi: PaperPoint::new(hi.x + dx, hi.y + dy),
        });
    }
    out
}

/// Zeichnet ein Planblatt.
pub fn plan_sheet(project: &Project, geometry: &ModelGeometry, sheet: &PlanSheet, standard: &OfficeStandard) -> Sheet {
    let (width, height) = format_size(&sheet.format);
    let mut out = Sheet {
        title: format!("{}, Plan {} {}", project.name, sheet.number, sheet.title),
        width_mm: width,
        height_mm: height,
        objects: Vec::new(),
    };
    out.polygon(rect_points(BINDING, MARGIN, width - BINDING - MARGIN, height - 2.0 * MARGIN), Pen::mm(0.5));
    for d in drawings(project, geometry, sheet, standard) {
        out.objects.extend(d.objects);
        // Titel unter der Ansicht, unterstrichen, mit Maßstab.
        let (x, y) = (d.center.x, d.lo.y - TITLE_GAP);
        out.text(x, y, d.title.clone(), 5.0, Anchor::Center);
        let half = 2.6 * d.title.chars().count() as f64 / 2.0;
        out.line(vec![PaperPoint::new(x - half, y - 1.5), PaperPoint::new(x + half, y - 1.5)], Pen::mm(0.35));
        out.text(x, y - 6.0, format!("M 1:{}", d.scale), 2.5, Anchor::Center);
    }
    if let Some(id) = project.plan_sheets().into_iter().find(|(_, s)| *s == sheet).map(|(id, _)| id) {
        let style = standard.style_for(100.0);
        let annotations = lot_model::AnnotationView::Sheet { sheet: id };
        crate::annotations::draw_annotations(&mut out.objects, project, annotations, standard, style, |p| {
            PaperPoint::new(p.x, p.y)
        });
    }
    title_block(&mut out, project, sheet, standard);
    out
}

/// Was am Blatt nicht stimmt: Ansichten über den Rand, auf dem Schriftfeld
/// oder übereinander. Leer, wenn alles passt.
pub fn layout_problems(
    project: &Project,
    geometry: &ModelGeometry,
    sheet: &PlanSheet,
    standard: &OfficeStandard,
) -> Vec<String> {
    let (width, height) = format_size(&sheet.format);
    let frame = (PaperPoint::new(BINDING, MARGIN), PaperPoint::new(width - MARGIN, height - MARGIN));
    let block = (
        PaperPoint::new(width - MARGIN - BLOCK_WIDTH, MARGIN),
        PaperPoint::new(width - MARGIN, MARGIN + block_rows(standard) * ROW),
    );
    let overlap = |a: (PaperPoint, PaperPoint), b: (PaperPoint, PaperPoint)| {
        a.0.x < b.1.x && b.0.x < a.1.x && a.0.y < b.1.y && b.0.y < a.1.y
    };
    let items = drawings(project, geometry, sheet, standard);
    let mut problems = Vec::new();
    for (i, d) in items.iter().enumerate() {
        let e = d.extent();
        if e.0.x < frame.0.x || e.0.y < frame.0.y || e.1.x > frame.1.x || e.1.y > frame.1.y {
            problems.push(format!("Plan {}: {} ragt über den Rand", sheet.number, d.title));
        }
        if overlap(e, block) {
            problems.push(format!("Plan {}: {} liegt auf dem Schriftfeld", sheet.number, d.title));
        }
        for other in &items[i + 1..] {
            if overlap(e, other.extent()) {
                problems.push(format!("Plan {}: {} und {} überdecken sich", sheet.number, d.title, other.title));
            }
        }
    }
    problems
}

/// Zeilen des Schriftfelds: vier, mit Unterschriften fünf.
fn block_rows(standard: &OfficeStandard) -> f64 {
    if standard.sheets.signatures { 5.0 } else { 4.0 }
}

/// Schriftfeld unten rechts (Vorschlag): Bauvorhaben; Bauherr, Bauort mit
/// Grundstück, Entwurfsverfasser; Planinhalt; Maßstab, Format, Datum, Index,
/// Plannummer; im Bürostandard wählbar Unterschriften von Bauherr und
/// Entwurfsverfasser (Bauvorlagen).
fn title_block(out: &mut Sheet, project: &Project, sheet: &PlanSheet, standard: &OfficeStandard) {
    let info = project.info();
    let w = out.width_mm;
    let (x0, y0) = (w - MARGIN - BLOCK_WIDTH, MARGIN);
    let x1 = w - MARGIN;
    let rows = block_rows(standard);
    out.polygon(rect_points(x0, y0, BLOCK_WIDTH, rows * ROW), Pen::mm(0.5));
    for r in 1..rows as usize {
        let y = y0 + r as f64 * ROW;
        out.line(vec![PaperPoint::new(x0, y), PaperPoint::new(x1, y)], Pen::mm(0.25));
    }
    let scales: Vec<f64> = sheet.views.iter().map(|v| v.scale).collect();
    let scale = match scales.first() {
        Some(s) if scales.iter().all(|x| x == s) => format!("1:{s}"),
        Some(_) => "wie angegeben".to_string(),
        None => String::new(),
    };
    let office = &standard.office;
    let architect = if office.name.is_empty() {
        info.architect.clone()
    } else {
        [office.name.as_str(), office.address.as_str(), office.contact.as_str()]
            .iter()
            .filter(|t| !t.is_empty())
            .copied()
            .collect::<Vec<_>>()
            .join("\n")
    };
    let site = if info.parcel.is_empty() { info.site.clone() } else { format!("{}\n{}", info.site, info.parcel) };
    let title = if info.description.is_empty() { project.name.clone() } else { info.description.clone() };
    // Zeilen von oben nach unten, je Zeile Felder: (Breite, Bezeichnung, Inhalt, Schrifthöhe).
    let mut rows_content: Vec<Vec<(f64, &str, String, f64)>> = vec![
        vec![(BLOCK_WIDTH, "Bauvorhaben", title, 3.5)],
        vec![
            (60.0, "Bauherr", info.client.clone(), 2.5),
            (60.0, "Bauort, Grundstück", site, 2.5),
            (60.0, "Entwurfsverfasser", architect, 2.5),
        ],
        vec![(BLOCK_WIDTH, "Planinhalt", sheet.title.clone(), 3.5)],
        vec![
            (35.0, "Maßstab", scale, 2.5),
            (20.0, "Format", sheet.format.clone(), 2.5),
            (40.0, "Datum", sheet.date.clone(), 2.5),
            (20.0, "Index", sheet.index.clone(), 2.5),
            (65.0, "Plan-Nr.", sheet.number.clone(), 5.0),
        ],
    ];
    if standard.sheets.signatures {
        rows_content.push(vec![
            (90.0, "Bauherr: Datum, Unterschrift", String::new(), 2.5),
            (90.0, "Entwurfsverfasser: Datum, Unterschrift", String::new(), 2.5),
        ]);
    }
    for (r, cells) in rows_content.iter().enumerate() {
        let top = y0 + (rows - r as f64) * ROW;
        let mut x = x0;
        for (i, (cw, label, value, size)) in cells.iter().enumerate() {
            if i > 0 {
                out.line(vec![PaperPoint::new(x, top), PaperPoint::new(x, top - ROW)], Pen::mm(0.25));
            }
            out.text(x + 1.5, top - 3.0, *label, 1.8, Anchor::Left);
            // Mehrere Zeilen kleiner und von unten nach oben.
            let lines: Vec<&str> = value.lines().collect();
            if lines.len() > 1 {
                for (i, line) in lines.iter().rev().enumerate() {
                    out.text(x + 1.5, top - ROW + 1.8 + i as f64 * 2.6, *line, 1.9, Anchor::Left);
                }
            } else {
                out.text(x + 1.5, top - ROW + 2.5, value.clone(), *size, Anchor::Left);
            }
            x += cw;
        }
    }
}

/// Alle Planblätter des Projekts, nach Plannummer.
pub fn plan_sheets(project: &Project, geometry: &ModelGeometry, standard: &OfficeStandard) -> Vec<Sheet> {
    project.plan_sheets().into_iter().map(|(_, s)| plan_sheet(project, geometry, s, standard)).collect()
}

#[cfg(test)]
mod tests {
    use lot_model::{PlacedView, Point2};

    use super::*;

    fn sheet(project: &Project, centers: &[(f64, f64)]) -> PlanSheet {
        let storey = project.storeys()[0].0;
        PlanSheet {
            number: "001".into(),
            title: "Test".into(),
            format: "A3".into(),
            date: String::new(),
            index: String::new(),
            views: centers
                .iter()
                .map(|&(x, y)| PlacedView {
                    content: ViewContent::FloorPlan { storey },
                    scale: 100.0,
                    center: Point2::new(x, y),
                })
                .collect(),
        }
    }

    #[test]
    fn layout_problems_name_edge_title_block_and_overlap() {
        let p = Project::sample();
        let g = lot_geom::build(&p).unwrap();
        let standard = OfficeStandard::default_standard();
        assert!(layout_problems(&p, &g, &sheet(&p, &[(150.0, 200.0)]), standard).is_empty());
        let problems = layout_problems(&p, &g, &sheet(&p, &[(25.0, 200.0)]), standard);
        assert!(problems[0].contains("Rand"), "{problems:?}");
        let problems = layout_problems(&p, &g, &sheet(&p, &[(330.0, 40.0)]), standard);
        assert!(problems.iter().any(|m| m.contains("Schriftfeld")), "{problems:?}");
        let problems = layout_problems(&p, &g, &sheet(&p, &[(150.0, 200.0), (160.0, 200.0)]), standard);
        assert!(problems.iter().any(|m| m.contains("überdecken")), "{problems:?}");
    }

    #[test]
    fn title_block_shows_number_and_common_scale() {
        let p = Project::sample();
        let g = lot_geom::build(&p).unwrap();
        let out = plan_sheet(&p, &g, &sheet(&p, &[(150.0, 200.0)]), OfficeStandard::default_standard());
        let texts: Vec<&str> = out
            .objects
            .iter()
            .filter_map(|o| match o {
                PlanObject::Text(t) => Some(t.text.as_str()),
                _ => None,
            })
            .collect();
        for expected in ["001", "1:100", "A3", "Plan-Nr.", "M 1:100"] {
            assert!(texts.contains(&expected), "{expected} fehlt in {texts:?}");
        }
        assert_eq!((out.width_mm, out.height_mm), (420.0, 297.0));
    }
}
