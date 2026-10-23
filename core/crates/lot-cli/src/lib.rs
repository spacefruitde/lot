//! Ausgaben eines Testprojekts: welche Blätter und Bilder `lot-cli render`
//! erzeugt. Beschrieben in `testprojekte/<name>/ausgaben.ron`; fehlt die
//! Datei, entsteht das M0-Prüfblatt.
//!
//! Dieselbe Beschreibung nutzen die Golden-Tests (`tests/golden.rs`).

use std::path::Path;

use anyhow::{Context, Result};
use lot_model::{ElementId, Project};
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
    /// Grundriss eines Geschosses.
    FloorPlan { name: String, storey: u64, scale: f64 },
    /// Katalog von Ausschnitten, etwa der Wandknoten.
    Catalog { name: String, storey: u64, scale: f64, title: String, cells: Vec<CatalogCell> },
    /// 3D-Ansicht als Bild, ohne Fenster gerechnet.
    View3d { name: String },
    /// Planansicht der App als Bild, ohne Fenster gerechnet.
    PlanView { name: String, storey: u64, scale: f64 },
    /// Flächenaufstellung nach DIN 277 und WoFlV, als Blatt und als CSV.
    AreaTable { name: String },
}

impl Output {
    pub fn name(&self) -> &str {
        match self {
            Output::M0Sheet { name }
            | Output::FloorPlan { name, .. }
            | Output::Catalog { name, .. }
            | Output::View3d { name }
            | Output::PlanView { name, .. }
            | Output::AreaTable { name } => name,
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
        Output::FloorPlan { storey, scale, .. } => {
            vec![floor_plan_sheet(project, ElementId(*storey), *scale, standard)]
        }
        Output::Catalog { storey, scale, title, cells, .. } => {
            catalog_sheets(project, ElementId(*storey), cells, *scale, standard, title)
        }
        Output::AreaTable { .. } => lot_plan::area_table::area_sheets(&project.name, &lot_rules::area_report(project)),
        Output::View3d { .. } | Output::PlanView { .. } => Vec::new(),
    }
}

/// Dateinamen der Seiten: `name` bei einer Seite, sonst `name-1`, `name-2`, ...
pub fn page_names(name: &str, pages: usize) -> Vec<String> {
    if pages == 1 { vec![name.to_string()] } else { (1..=pages).map(|i| format!("{name}-{i}")).collect() }
}
