//! Ausgaben eines Testprojekts: welche Blätter und Bilder `lot-cli render`
//! erzeugt. Beschrieben in `testprojekte/<name>/ausgaben.ron`; fehlt die
//! Datei, entsteht das M0-Prüfblatt.
//!
//! Dieselbe Beschreibung nutzen die Golden-Tests (`tests/golden.rs`).

use std::path::Path;

use anyhow::{Context, Result};
use lot_model::{ElementId, PhaseFilter, Project};
use lot_plan::Sheet;
use lot_plan::sheets::{CatalogCell, catalog_sheets, floor_plan_sheet};
use lot_plan::standard::OfficeStandard;
use serde::Deserialize;

/// Dateiname der Ausgabenbeschreibung in einem Testprojektordner.
pub const OUTPUTS_FILE: &str = "ausgaben.ron";

/// Eine Ausgabe. `name` ist der Dateiname ohne Endung.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub enum Output {
    /// Das M0-Prüfblatt mit Kalibrierung und Grundriss.
    M0Sheet { name: String },
    /// Grundriss eines Geschosses; `phase` wählt Umbauplan (farbig), Bestand oder
    /// Neuzustand. Mit `cad` auch als DXF und DWG in Metern (6.04).
    FloorPlan {
        name: String,
        storey: u64,
        scale: f64,
        #[serde(default)]
        phase: PhaseFilter,
        #[serde(default)]
        cad: bool,
    },
    /// Katalog von Ausschnitten, etwa der Wandknoten.
    Catalog { name: String, storey: u64, scale: f64, title: String, cells: Vec<CatalogCell> },
    /// 3D-Ansicht als Bild, ohne Fenster gerechnet.
    View3d { name: String },
    /// Planansicht der App als Bild, ohne Fenster gerechnet.
    PlanView { name: String, storey: u64, scale: f64 },
    /// Flächenaufstellung nach DIN 277 und WoFlV, als Blatt und als CSV; bei
    /// Umbauten im gewählten Bauzustand (sonst alles).
    AreaTable {
        name: String,
        #[serde(default)]
        phase: PhaseFilter,
    },
    /// Schnitt oder Ansicht (Element `view` im Projekt); mit `cad` auch als DXF
    /// und DWG in Metern, x entlang der Schnittlinie, y als Höhe.
    Section {
        name: String,
        view: u64,
        scale: f64,
        #[serde(default)]
        phase: PhaseFilter,
        #[serde(default)]
        cad: bool,
    },
    /// Alle Planblätter des Projekts, eine Seite je Blatt; mit `cad` je Blatt
    /// auch als DXF und DWG in Millimetern.
    PlanSheets {
        name: String,
        #[serde(default)]
        cad: bool,
    },
}

impl Output {
    pub fn name(&self) -> &str {
        match self {
            Output::M0Sheet { name }
            | Output::FloorPlan { name, .. }
            | Output::Catalog { name, .. }
            | Output::View3d { name }
            | Output::PlanView { name, .. }
            | Output::AreaTable { name, .. }
            | Output::Section { name, .. }
            | Output::PlanSheets { name, .. } => name,
        }
    }
}

/// Liest die Ausgabenbeschreibung eines Testprojekts.
pub fn outputs(project_dir: &Path) -> Result<Vec<Output>> {
    let file = project_dir.join(OUTPUTS_FILE);
    if !file.is_file() {
        return Ok(vec![Output::M0Sheet { name: "m0-pruefblatt".into() }]);
    }
    let text = std::fs::read_to_string(&file).with_context(|| format!("{} nicht lesbar", file.display()))?;
    ron::from_str(&text).with_context(|| format!("{} nicht lesbar", file.display()))
}

/// Die Blätter einer Ausgabe. Leer für Ausgaben ohne Blätter (3D-Bild).
pub fn sheets(project: &Project, output: &Output, source: &str) -> Vec<Sheet> {
    let standard = OfficeStandard::default_standard();
    match output {
        Output::M0Sheet { .. } => vec![lot_plan::m0::sheet(project, source)],
        Output::FloorPlan { storey, scale, phase, .. } => {
            vec![floor_plan_sheet(&phase_project(project, *phase), ElementId(*storey), *scale, standard)]
        }
        Output::Catalog { storey, scale, title, cells, .. } => {
            catalog_sheets(project, ElementId(*storey), cells, *scale, standard, title)
        }
        Output::AreaTable { phase, .. } => {
            let project = phase_project(project, *phase);
            lot_plan::area_table::area_sheets(&project.name, &lot_rules::area_report(&project))
        }
        Output::Section { view, scale, phase, .. } => {
            let project = phase_project(project, *phase);
            match geometry(&project) {
                Ok(geometry) => {
                    vec![lot_plan::sheets::section_sheet(&project, &geometry, ElementId(*view), *scale, standard)]
                }
                Err(_) => Vec::new(),
            }
        }
        Output::PlanSheets { .. } => match geometry(project) {
            Ok(geometry) => lot_plan::layout::plan_sheets(project, &geometry, standard),
            Err(_) => Vec::new(),
        },
        Output::View3d { .. } | Output::PlanView { .. } => Vec::new(),
    }
}

/// Das Projekt im gewählten Bauzustand; Bestand und Neuzustand stehen im Titel.
pub fn phase_project(project: &Project, phase: PhaseFilter) -> Project {
    let mut p = project.phase_view(phase);
    if project.has_phases() {
        match phase {
            PhaseFilter::All => {}
            PhaseFilter::Existing => p.name = format!("{} (Bestand)", project.name),
            PhaseFilter::Planned => p.name = format!("{} (Neuzustand)", project.name),
        }
    }
    p
}

/// 3D-Geometrie für Schnitte und Planblätter; im Umbauplan ohne die Öffnungen,
/// die die Wand nicht durchbrechen (siehe `Project::without_phase_openings`).
pub fn geometry(project: &Project) -> Result<lot_geom::ModelGeometry, lot_geom::GeomError> {
    if project.has_phases() { lot_geom::build(&project.without_phase_openings()) } else { lot_geom::build(project) }
}

/// DXF- und DWG-Dateien einer Ausgabe, falls sie welche verlangt: (Dateiname, Inhalt).
pub fn cad_files(project: &Project, output: &Output) -> Result<Vec<(String, Vec<u8>)>> {
    use lot_io::cad::{CadUnits, dwg_bytes, dxf_bytes, to_cad};
    let standard = OfficeStandard::default_standard();
    let origin = lot_plan::Viewport { scale: 100.0, origin: lot_plan::PaperPoint::new(0.0, 0.0) };
    let mut docs: Vec<(String, lot_io::cad::CadDocument)> = Vec::new();
    match output {
        Output::FloorPlan { name, storey, scale, phase, cad: true } => {
            let project = phase_project(project, *phase);
            let view = lot_plan::floor_plan::FloorPlanView {
                storey: ElementId(*storey),
                cut_height: lot_plan::floor_plan::DEFAULT_CUT_HEIGHT,
                scale: *scale,
            };
            let viewport = lot_plan::Viewport { scale: *scale, ..origin };
            let layers = lot_plan::floor_plan::floor_plan_layers(&project, view, standard, viewport);
            docs.push((name.clone(), to_cad(&layers, scale / 1000.0, CadUnits::Meters)?));
        }
        Output::Section { name, view, scale, phase, cad: true } => {
            let project = phase_project(project, *phase);
            if let (Ok(geometry), Some(lot_model::Element::SectionView(v))) =
                (geometry(&project), project.elements.get(&ElementId(*view)))
            {
                let viewport = lot_plan::Viewport { scale: *scale, ..origin };
                let objects = lot_plan::section::section(&project, &geometry, v, standard, viewport);
                docs.push((name.clone(), to_cad(&[("A_Schnitt", objects)], scale / 1000.0, CadUnits::Meters)?));
            }
        }
        Output::PlanSheets { name, cad: true } => {
            if let Ok(geometry) = geometry(project) {
                let sheets = lot_plan::layout::plan_sheets(project, &geometry, standard);
                for (sheet, page) in sheets.into_iter().zip(page_names(name, project.plan_sheets().len())) {
                    docs.push((page, to_cad(&[("A_Planblatt", sheet.objects)], 1.0, CadUnits::Millimeters)?));
                }
            }
        }
        _ => {}
    }
    let mut files = Vec::new();
    for (name, doc) in docs {
        files.push((format!("{name}.dxf"), dxf_bytes(&doc)?));
        files.push((format!("{name}.dwg"), dwg_bytes(&doc)?));
    }
    Ok(files)
}

/// Dateinamen der Seiten: `name` bei einer Seite, sonst `name-1`, `name-2`, ...
pub fn page_names(name: &str, pages: usize) -> Vec<String> {
    if pages == 1 { vec![name.to_string()] } else { (1..=pages).map(|i| format!("{name}-{i}")).collect() }
}
