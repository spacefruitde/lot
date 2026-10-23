//! Schnittstelle des Kerns für die Swift-App, erzeugt mit UniFFI.
//!
//! Die Swift-Seite sieht zwei Objekte:
//!
//! - [`LotDocument`]: ein geöffnetes Projekt mit Parametern, Undo/Redo, Speichern und Laden
//! - [`Viewport`]: Planansicht und 3D-Ansicht, die über wgpu in einen NSView
//!   der App zeichnen, mit Werkzeugen und Befehlen (`viewport.rs`)
//!
//! Die Swift-Anbindung entsteht mit `app-mac/scripts/build-core.sh`.

mod viewport;

use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};

use lot_model::{Document, ElementId, Param, Project};
pub use viewport::Viewport;

uniffi::setup_scaffolding!();

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum LotError {
    #[error("{0}")]
    Failed(String),
}

impl LotError {
    fn from_display(e: impl std::fmt::Display) -> Self {
        LotError::Failed(e.to_string())
    }
}

/// Parameter, die die App ändern kann. Spiegelt `lot_model::Param`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, uniffi::Enum)]
pub enum ParamKind {
    WallLength,
    WallSagitta,
    WallHeight,
    WallTopOffset,
    BaseOffset,
    OpeningOffset,
    OpeningWidth,
    OpeningHeight,
    OpeningSillHeight,
    StoreyElevation,
    StoreyHeight,
    ColumnWidth,
    ColumnDepth,
}

impl From<ParamKind> for Param {
    fn from(k: ParamKind) -> Self {
        match k {
            ParamKind::WallLength => Param::WallLength,
            ParamKind::WallSagitta => Param::WallSagitta,
            ParamKind::WallHeight => Param::WallHeight,
            ParamKind::WallTopOffset => Param::WallTopOffset,
            ParamKind::BaseOffset => Param::BaseOffset,
            ParamKind::OpeningOffset => Param::OpeningOffset,
            ParamKind::OpeningWidth => Param::OpeningWidth,
            ParamKind::OpeningHeight => Param::OpeningHeight,
            ParamKind::OpeningSillHeight => Param::OpeningSillHeight,
            ParamKind::StoreyElevation => Param::StoreyElevation,
            ParamKind::StoreyHeight => Param::StoreyHeight,
            ParamKind::ColumnWidth => Param::ColumnWidth,
            ParamKind::ColumnDepth => Param::ColumnDepth,
        }
    }
}

const ALL_PARAMS: [ParamKind; 13] = [
    ParamKind::WallLength,
    ParamKind::WallSagitta,
    ParamKind::WallHeight,
    ParamKind::WallTopOffset,
    ParamKind::BaseOffset,
    ParamKind::OpeningOffset,
    ParamKind::OpeningWidth,
    ParamKind::OpeningHeight,
    ParamKind::OpeningSillHeight,
    ParamKind::StoreyElevation,
    ParamKind::StoreyHeight,
    ParamKind::ColumnWidth,
    ParamKind::ColumnDepth,
];

/// Ein Parameterwert eines Elements, in Metern.
#[derive(Debug, Clone, uniffi::Record)]
pub struct ParamValue {
    pub element: u64,
    pub kind: ParamKind,
    pub value: f64,
}

#[derive(uniffi::Object)]
pub struct LotDocument {
    inner: Mutex<Document>,
}

impl LotDocument {
    pub(crate) fn lock(&self) -> MutexGuard<'_, Document> {
        // Ein Panic in einem anderen Aufruf soll die App nicht dauerhaft blockieren.
        self.inner.lock().unwrap_or_else(|e| e.into_inner())
    }
}

#[uniffi::export]
impl LotDocument {
    /// Beispielprojekt aus M0: eine Wand mit Fenster.
    #[uniffi::constructor]
    pub fn new_sample() -> Arc<Self> {
        Arc::new(Self { inner: Mutex::new(Document::new(Project::sample())) })
    }

    /// Neues, leeres Projekt mit Erdgeschoss und üblichen Wandaufbauten.
    #[uniffi::constructor]
    pub fn new_empty() -> Arc<Self> {
        Arc::new(Self { inner: Mutex::new(Document::new(Project::template("Neues Projekt"))) })
    }

    /// Testprojekt aus einem Ordner mit `projekt.ron` (zum Ausprobieren, etwa das EFH).
    #[uniffi::constructor]
    pub fn open_testproject(path: String) -> Result<Arc<Self>, LotError> {
        let project = lot_io::testproject::load(Path::new(&path)).map_err(LotError::from_display)?;
        Ok(Arc::new(Self { inner: Mutex::new(Document::new(project)) }))
    }

    #[uniffi::constructor]
    pub fn open(path: String) -> Result<Arc<Self>, LotError> {
        let doc = lot_io::project_file::load(Path::new(&path)).map_err(LotError::from_display)?;
        Ok(Arc::new(Self { inner: Mutex::new(doc) }))
    }

    pub fn save(&self, path: String) -> Result<(), LotError> {
        lot_io::project_file::save(&self.lock(), Path::new(&path)).map_err(LotError::from_display)
    }

    pub fn name(&self) -> String {
        self.lock().project().name.clone()
    }

    /// Alle änderbaren Parameter der Elemente `ids`, oder aller Elemente, wenn `ids` leer ist.
    pub fn parameters(&self, ids: Vec<u64>) -> Vec<ParamValue> {
        let doc = self.lock();
        let mut out = Vec::new();
        for (id, element) in &doc.project().elements {
            if !ids.is_empty() && !ids.contains(&id.0) {
                continue;
            }
            for kind in ALL_PARAMS {
                if let Some(value) = element.param(kind.into()) {
                    out.push(ParamValue { element: id.0, kind, value });
                }
            }
        }
        out
    }

    pub fn set_param(&self, element: u64, kind: ParamKind, value: f64) -> Result<(), LotError> {
        self.lock().set_param(ElementId(element), kind.into(), value).map_err(LotError::from_display)
    }

    pub fn undo(&self) -> Result<bool, LotError> {
        self.lock().undo().map_err(LotError::from_display)
    }

    pub fn redo(&self) -> Result<bool, LotError> {
        self.lock().redo().map_err(LotError::from_display)
    }

    /// Bezeichnung eines Elements für die Anzeige, etwa „Wand · Außenwand WDVS 36“.
    pub fn element_label(&self, element: u64) -> String {
        let doc = self.lock();
        let p = doc.project();
        match p.elements.get(&ElementId(element)) {
            Some(lot_model::Element::Wall(w)) => {
                format!("Wand · {}", p.wall_type(w.wall_type).map_or("?", |t| t.name.as_str()))
            }
            Some(lot_model::Element::Opening(_)) => "Öffnung".into(),
            Some(lot_model::Element::Storey(s)) => format!("Geschoss {}", s.name),
            Some(lot_model::Element::Column(_)) => "Stütze".into(),
            Some(lot_model::Element::Room(r)) => format!("Raum {} {}", r.number, r.name).trim_end().to_string(),
            Some(lot_model::Element::GridLine(g)) => format!("Achse {}", g.label),
            Some(lot_model::Element::WallType(t)) => format!("Wandaufbau {}", t.name),
            None => String::new(),
        }
    }

    pub fn can_undo(&self) -> bool {
        self.lock().can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.lock().can_redo()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn value(doc: &LotDocument, kind: ParamKind) -> f64 {
        doc.parameters(Vec::new()).into_iter().find(|p| p.kind == kind).unwrap().value
    }

    #[test]
    fn sample_has_all_parameters() {
        let doc = LotDocument::new_sample();
        // Geschoss: Höhe und Lage; Wand: Länge, Stich, Höhe, Versatz; Öffnung: vier Maße.
        assert_eq!(doc.parameters(Vec::new()).len(), 2 + 4 + 4);
        assert_eq!(doc.parameters(vec![4]).len(), 4);
        assert_eq!(value(&doc, ParamKind::OpeningWidth), 1.5);
    }

    #[test]
    fn change_undo_redo() {
        let doc = LotDocument::new_sample();
        doc.set_param(4, ParamKind::OpeningWidth, 2.0).unwrap();
        assert!(doc.can_undo());
        assert!(doc.undo().unwrap());
        assert_eq!(value(&doc, ParamKind::OpeningWidth), 1.5);
        assert!(doc.redo().unwrap());
        assert_eq!(value(&doc, ParamKind::OpeningWidth), 2.0);
    }

    #[test]
    fn invalid_value_gives_german_message() {
        let doc = LotDocument::new_sample();
        let LotError::Failed(message) = doc.set_param(4, ParamKind::OpeningWidth, 21.5).unwrap_err();
        assert!(message.contains("ragt seitlich aus der Wand"), "{message}");
        assert_eq!(value(&doc, ParamKind::OpeningWidth), 1.5);
    }

    #[test]
    fn save_and_open() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("wand.lot").to_string_lossy().into_owned();
        let doc = LotDocument::new_sample();
        doc.set_param(3, ParamKind::WallHeight, 2.75).unwrap();
        doc.save(path.clone()).unwrap();
        let opened = LotDocument::open(path).unwrap();
        assert_eq!(value(&opened, ParamKind::WallHeight), 2.75);
        assert_eq!(opened.name(), "M0 Wand mit Fenster");
    }

    #[test]
    fn opening_missing_file_fails_cleanly() {
        assert!(LotDocument::open("/gibt/es/nicht.lot".into()).is_err());
    }
}
