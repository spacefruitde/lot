//! Schnittstelle des Kerns für die Swift-App, erzeugt mit UniFFI.
//!
//! Die Swift-Seite sieht zwei Objekte:
//!
//! - [`LotDocument`]: ein geöffnetes Projekt mit Parametern, Undo/Redo, Speichern und Laden
//! - [`Viewport`]: die 3D-Ansicht, die über wgpu in einen NSView der App zeichnet
//!
//! Die Swift-Anbindung entsteht mit `app-mac/scripts/build-core.sh`.

use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Instant;

use lot_model::{Document, ElementId, Param, Project};
use lot_render::Renderer;

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

const ALL_PARAMS: [ParamKind; 12] = [
    ParamKind::WallLength,
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
    fn lock(&self) -> MutexGuard<'_, Document> {
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

    /// Alle änderbaren Parameter aller Elemente, nach Element geordnet.
    pub fn parameters(&self) -> Vec<ParamValue> {
        let doc = self.lock();
        let mut out = Vec::new();
        for (id, element) in &doc.project().elements {
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

    pub fn can_undo(&self) -> bool {
        self.lock().can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.lock().can_redo()
    }
}

/// Die 3D-Ansicht.
#[derive(uniffi::Object)]
pub struct Viewport {
    renderer: Mutex<Renderer>,
}

impl Viewport {
    fn lock(&self) -> MutexGuard<'_, Renderer> {
        self.renderer.lock().unwrap_or_else(|e| e.into_inner())
    }
}

#[uniffi::export]
impl Viewport {
    /// Erzeugt die Ansicht für einen NSView der App.
    ///
    /// `ns_view` ist die Adresse eines NSView, dessen Schicht ein CAMetalLayer ist.
    /// Der View muss leben, solange dieses Objekt lebt, und alle Aufrufe kommen
    /// vom Main-Thread. Größen in Pixeln, `scale` in Pixeln je Punkt.
    #[uniffi::constructor]
    pub fn new(ns_view: u64, width_px: u32, height_px: u32, scale: f32) -> Result<Arc<Self>, LotError> {
        let view = std::ptr::NonNull::new(ns_view as usize as *mut std::ffi::c_void)
            .ok_or_else(|| LotError::Failed("Kein View übergeben".into()))?;
        // SAFETY: Die App übergibt einen gültigen NSView und hält ihn am Leben (siehe oben).
        let renderer =
            unsafe { Renderer::for_appkit_view(view, width_px, height_px, scale) }.map_err(LotError::from_display)?;
        Ok(Arc::new(Self { renderer: Mutex::new(renderer) }))
    }

    pub fn resize(&self, width_px: u32, height_px: u32, scale: f32) {
        self.lock().resize(width_px, height_px, scale);
    }

    /// Rechnet die Geometrie des Dokuments und lädt sie auf die Grafikkarte.
    /// Gibt die benötigte Zeit in Millisekunden zurück.
    pub fn show(&self, document: Arc<LotDocument>) -> Result<f64, LotError> {
        let start = Instant::now();
        let geometry = lot_geom::build(document.lock().project()).map_err(LotError::from_display)?;
        let standard = lot_plan::standard::OfficeStandard::default_standard();
        self.lock().set_geometry(&geometry, &|m| standard.material(m).color);
        Ok(start.elapsed().as_secs_f64() * 1000.0)
    }

    pub fn triangle_count(&self) -> u32 {
        self.lock().triangle_count()
    }

    /// Drehen um das Ziel. Verschiebung in Punkten, wie sie das Trackpad liefert.
    pub fn orbit(&self, dx: f32, dy: f32) {
        self.lock().camera_mut().orbit(dx, dy);
    }

    pub fn pan(&self, dx: f32, dy: f32) {
        self.lock().camera_mut().pan(dx, dy);
    }

    /// Zoomen zum Zeiger. Faktor größer 1 zoomt hinein. `x`, `y`: Lage des
    /// Zeigers in Punkten vom Mittelpunkt der Ansicht, y nach oben.
    pub fn zoom(&self, factor: f32, x: f32, y: f32) {
        self.lock().camera_mut().zoom(factor, x, y);
    }

    pub fn zoom_to_fit(&self) {
        self.lock().zoom_to_fit();
    }

    /// Zeichnet ein Bild. `false`, wenn das Fenster gerade verdeckt ist; dann erneut versuchen.
    pub fn render(&self) -> Result<bool, LotError> {
        self.lock().render().map_err(LotError::from_display)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn value(doc: &LotDocument, kind: ParamKind) -> f64 {
        doc.parameters().into_iter().find(|p| p.kind == kind).unwrap().value
    }

    #[test]
    fn sample_has_all_parameters() {
        let doc = LotDocument::new_sample();
        // Geschoss: Höhe und Lage; Wand: Länge, Höhe, Versatz; Öffnung: vier Maße.
        assert_eq!(doc.parameters().len(), 2 + 3 + 4);
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
