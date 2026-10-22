//! Dateien lesen und schreiben.
//!
//! - Projektdatei `.lot` als SQLite-Datenbank ([`project_file`])
//! - Testprojekte als lesbare RON-Textdateien ([`testproject`])
//! - Planblätter als PDF, SVG und PNG ([`pdf`], [`svg`], [`png`])

pub mod pdf;
pub mod plan_font;
pub mod png;
pub mod project_file;
pub mod svg;
pub mod testproject;

#[derive(Debug, thiserror::Error)]
pub enum IoError {
    #[error("Datenbankfehler: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Datei: {0}")]
    File(#[from] std::io::Error),
    #[error("Element nicht lesbar: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Testprojekt nicht lesbar: {0}")]
    Ron(#[from] ron::error::SpannedError),
    #[error("Testprojekt nicht schreibbar: {0}")]
    RonWrite(#[from] ron::Error),
    #[error("Keine Lot-Projektdatei")]
    NotALotFile,
    #[error("Die Datei hat Formatversion {found}, diese Version von Lot kennt nur bis {supported}")]
    UnsupportedVersion { found: i64, supported: i64 },
    #[error("PDF-Ausgabe: {0}")]
    Pdf(String),
    #[error("PNG-Ausgabe: {0}")]
    Png(String),
    #[error("Das Projekt in der Datei ist ungültig: {0}")]
    Model(#[from] lot_model::ModelError),
}
