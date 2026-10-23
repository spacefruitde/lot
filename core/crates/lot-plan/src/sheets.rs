//! Blätter für Testprojekte und Prüfpakete: Grundrisse und der Wandknoten-Katalog.
//!
//! Planköpfe und frei gestaltete Planblätter folgen in M3 (3.07). Bis dahin
//! trägt jedes Blatt eine schlichte Zeile mit Projekt, Inhalt und Maßstab.

use lot_model::{Element, ElementId, Project};

use crate::floor_plan::{DEFAULT_CUT_HEIGHT, FloorPlanView, floor_plan};
use crate::standard::OfficeStandard;
use crate::{Anchor, PaperPoint, Pen, Sheet, Viewport, format_de, rect_points};

/// Blattformate nach DIN EN ISO 216, quer: (Name, Breite, Höhe) in mm.
const FORMATS: [(&str, f64, f64); 4] =
    [("A4", 297.0, 210.0), ("A3", 420.0, 297.0), ("A2", 594.0, 420.0), ("A1", 841.0, 594.0)];
const MARGIN: f64 = 10.0;
/// Höhe der Beschriftungszeile unten.
const FOOTER: f64 = 12.0;

/// Grundriss eines Geschosses auf dem kleinsten Blatt, auf das er passt.
pub fn floor_plan_sheet(project: &Project, storey: ElementId, scale: f64, standard: &OfficeStandard) -> Sheet {
    let extent = lot_geom::plan_extent(project).unwrap_or(([0.0, 0.0], [10.0, 10.0]));
    // Platz für Achskreise rundherum.
    let pad = 2.0 * standard.style_for(scale).grid_bubble * scale / 1000.0 + 0.5;
    let model_w = extent.1[0] - extent.0[0] + 2.0 * pad;
    let model_h = extent.1[1] - extent.0[1] + 2.0 * pad;
    let paper_w = model_w * 1000.0 / scale;
    let paper_h = model_h * 1000.0 / scale;
    let (format, width, height) = FORMATS
        .iter()
        .copied()
        .find(|(_, w, h)| paper_w + 2.0 * MARGIN <= *w && paper_h + 2.0 * MARGIN + FOOTER <= *h)
        .unwrap_or(FORMATS[FORMATS.len() - 1]);
    let storey_name = project.storey(storey).map_or("?".to_string(), |s| s.name.clone());
    let mut sheet = Sheet {
        title: format!("{}, Grundriss {storey_name}, 1:{}", project.name, scale),
        width_mm: width,
        height_mm: height,
        objects: Vec::new(),
    };
    frame(&mut sheet);

    let center_model = [(extent.0[0] + extent.1[0]) / 2.0, (extent.0[1] + extent.1[1]) / 2.0];
    let center_paper = PaperPoint::new(width / 2.0, (height + FOOTER) / 2.0);
    let f = 1000.0 / scale;
    let viewport = Viewport {
        scale,
        origin: PaperPoint::new(center_paper.x - center_model[0] * f, center_paper.y - center_model[1] * f),
    };
    let view = FloorPlanView { storey, cut_height: DEFAULT_CUT_HEIGHT, scale };
    sheet.objects.extend(floor_plan(project, view, standard, viewport));

    footer(
        &mut sheet,
        &format!("{}  ·  Grundriss {storey_name}  ·  M 1:{}  ·  {format}", project.name, scale),
        &format!(
            "Schnitthöhe +{} m über OKFF. Darstellung nach Bürostandard, Vorschlag.",
            format_de(DEFAULT_CUT_HEIGHT, 2)
        ),
    );
    sheet
}

/// Schnitt oder Ansicht auf dem kleinsten Blatt, auf das sie passt.
pub fn section_sheet(
    project: &Project,
    geometry: &lot_geom::ModelGeometry,
    view_id: ElementId,
    scale: f64,
    standard: &OfficeStandard,
) -> Sheet {
    let Some(Element::SectionView(view)) = project.elements.get(&view_id) else {
        return Sheet::a4_landscape(format!("{}, Ansicht fehlt", project.name));
    };
    let (lo, hi) = crate::section::section_extent(geometry, view).unwrap_or(([0.0, 0.0], [10.0, 10.0]));
    let pad = 1.0;
    let paper_w = (hi[0] - lo[0] + 2.0 * pad) * 1000.0 / scale;
    let paper_h = (hi[1] - lo[1] + 2.0 * pad) * 1000.0 / scale;
    let (format, width, height) = FORMATS
        .iter()
        .copied()
        .find(|(_, w, h)| paper_w + 2.0 * MARGIN <= *w && paper_h + 2.0 * MARGIN + FOOTER <= *h)
        .unwrap_or(FORMATS[FORMATS.len() - 1]);
    let mut sheet = Sheet {
        title: format!("{}, {}, 1:{}", project.name, view.name, scale),
        width_mm: width,
        height_mm: height,
        objects: Vec::new(),
    };
    frame(&mut sheet);
    let center_model = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0];
    let center_paper = PaperPoint::new(width / 2.0, (height + FOOTER) / 2.0);
    let f = 1000.0 / scale;
    let viewport = Viewport {
        scale,
        origin: PaperPoint::new(center_paper.x - center_model[0] * f, center_paper.y - center_model[1] * f),
    };
    sheet.objects.extend(crate::section::section(project, geometry, view, standard, viewport));
    footer(
        &mut sheet,
        &format!("{}  ·  {}  ·  M 1:{}  ·  {format}", project.name, view.name, scale),
        &format!(
            "Gelände vorläufig {} m. Darstellung nach Bürostandard, Vorschlag.",
            crate::section::format_height(crate::section::GROUND)
        ),
    );
    sheet
}

/// Eine Zelle im Wandknoten-Katalog.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
pub struct CatalogCell {
    pub number: u32,
    pub title: String,
    /// Mitte des Ausschnitts im Modell, in Metern.
    pub center: (f64, f64),
}

/// Katalog: je Blatt A4 quer sechs Zellen (zwei Reihen, drei Spalten), jede
/// mit einem Knoten im Maßstab `scale`.
pub fn catalog_sheets(
    project: &Project,
    storey: ElementId,
    cells: &[CatalogCell],
    scale: f64,
    standard: &OfficeStandard,
    title: &str,
) -> Vec<Sheet> {
    let (width, height) = (297.0, 210.0);
    let (cols, rows) = (3usize, 2usize);
    let cell_w = (width - 2.0 * MARGIN) / cols as f64;
    let cell_h = (height - 2.0 * MARGIN - FOOTER) / rows as f64;
    let pages = cells.len().div_ceil(cols * rows).max(1);
    let mut sheets = Vec::new();
    for page in 0..pages {
        let mut sheet = Sheet {
            title: format!("{title}, Blatt {} von {pages}", page + 1),
            width_mm: width,
            height_mm: height,
            objects: Vec::new(),
        };
        frame(&mut sheet);
        for (slot, cell) in cells.iter().skip(page * cols * rows).take(cols * rows).enumerate() {
            let (col, row) = (slot % cols, slot / cols);
            let x0 = MARGIN + col as f64 * cell_w;
            let y0 = height - MARGIN - (row + 1) as f64 * cell_h;
            sheet.rect(x0, y0, cell_w, cell_h, Pen::mm(0.18));
            sheet.text(x0 + 3.0, y0 + cell_h - 6.0, format!("{}  {}", cell.number, cell.title), 2.5, Anchor::Left);
            // Ausschnitt: alles, was im Fenster der Zelle liegt.
            let f = 1000.0 / scale;
            let half = [(cell_w - 4.0) / 2.0 / f, (cell_h - 12.0) / 2.0 / f];
            let local = crop(project, [cell.center.0, cell.center.1], half);
            let center_paper = PaperPoint::new(x0 + cell_w / 2.0, y0 + (cell_h - 8.0) / 2.0);
            let viewport = Viewport {
                scale,
                origin: PaperPoint::new(center_paper.x - cell.center.0 * f, center_paper.y - cell.center.1 * f),
            };
            let view = FloorPlanView { storey, cut_height: DEFAULT_CUT_HEIGHT, scale };
            sheet.objects.extend(floor_plan(&local, view, standard, viewport));
        }
        footer(
            &mut sheet,
            &format!("{title}  ·  M 1:{scale}  ·  Blatt {} von {pages}", page + 1),
            "Darstellung nach Bürostandard, Vorschlag M1. Prüffrage je Knoten: Würdest du ihn so zeichnen?",
        );
        sheets.push(sheet);
    }
    sheets
}

/// Teilprojekt mit allem, was im Fenster um `center` mit halber Größe `half` liegt.
fn crop(project: &Project, center: [f64; 2], half: [f64; 2]) -> Project {
    let inside = |x: f64, y: f64| (x - center[0]).abs() <= half[0] && (y - center[1]).abs() <= half[1];
    let mut out = Project::new(project.name.clone());
    let mut walls: Vec<ElementId> = Vec::new();
    for (id, e) in &project.elements {
        let keep = match e {
            Element::Storey(_) | Element::WallType(_) | Element::SlabType(_) => true,
            Element::Slab(_)
            | Element::SlabOpening(_)
            | Element::Roof(_)
            | Element::RoofWindow(_)
            | Element::Dormer(_)
            | Element::Stair(_)
            | Element::Railing(_)
            | Element::SectionView(_)
            | Element::Annotation(_)
            | Element::ProjectInfo(_)
            | Element::PlanSheet(_) => false,
            Element::Wall(w) => {
                let k = inside(w.start.x, w.start.y) || inside(w.end.x, w.end.y);
                if k {
                    walls.push(*id);
                }
                k
            }
            Element::Column(c) => inside(c.position.x, c.position.y),
            Element::Space(r) => inside(r.position.x, r.position.y),
            Element::DimensionChain(d) => inside(d.position.x, d.position.y),
            Element::GridLine(g) => inside(g.start.x, g.start.y) && inside(g.end.x, g.end.y),
            Element::Opening(_) => false,
        };
        if keep {
            out.elements.insert(*id, e.clone());
        }
    }
    for (id, e) in &project.elements {
        if let Element::Opening(o) = e
            && walls.contains(&o.host)
        {
            out.elements.insert(*id, e.clone());
        }
    }
    out
}

fn frame(sheet: &mut Sheet) {
    let (w, h) = (sheet.width_mm, sheet.height_mm);
    sheet.objects.push(crate::PlanObject::Line(crate::Line {
        points: rect_points(MARGIN, MARGIN, w - 2.0 * MARGIN, h - 2.0 * MARGIN),
        closed: true,
        pen: Pen::mm(0.35),
        style: crate::LineStyle::Solid,
        color: crate::Color::BLACK,
    }));
}

fn footer(sheet: &mut Sheet, first: &str, second: &str) {
    let (w, y) = (sheet.width_mm, MARGIN);
    sheet.line(vec![PaperPoint::new(MARGIN, y + FOOTER), PaperPoint::new(w - MARGIN, y + FOOTER)], Pen::mm(0.25));
    sheet.text(MARGIN + 3.0, y + 6.5, first, 2.5, Anchor::Left);
    sheet.text(MARGIN + 3.0, y + 2.5, second, 2.0, Anchor::Left);
}
