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
    WallThickness,
    WallHeight,
    OpeningOffset,
    OpeningWidth,
    OpeningHeight,
    OpeningSillHeight,
}

impl From<ParamKind> for Param {
    fn from(k: ParamKind) -> Self {
        match k {
            ParamKind::WallLength => Param::WallLength,
            ParamKind::WallThickness => Param::WallThickness,
            ParamKind::WallHeight => Param::WallHeight,
            ParamKind::OpeningOffset => Param::OpeningOffset,
            ParamKind::OpeningWidth => Param::OpeningWidth,
            ParamKind::OpeningHeight => Param::OpeningHeight,
            ParamKind::OpeningSillHeight => Param::OpeningSillHeight,
        }
    }
}

const ALL_PARAMS: [ParamKind; 7] = [
    ParamKind::WallLength,
    ParamKind::WallThickness,
    ParamKind::WallHeight,
    ParamKind::OpeningOffset,
    ParamKind::OpeningWidth,
    ParamKind::OpeningHeight,
    ParamKind::OpeningSillHeight,
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
        self.lock()
            .set_param(ElementId(element), kind.into(), value)
            .map_err(LotError::from_display)
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
        let renderer = unsafe { Renderer::for_appkit_view(view, width_px, height_px, scale) }.map_err(LotError::from_display)?;
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
        self.lock().set_geometry(&geometry);
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

    /// Faktor größer 1 zoomt hinein.
    pub fn zoom(&self, factor: f32) {
        self.lock().camera_mut().zoom(factor);
    }

    pub fn zoom_to_fit(&self) {
        self.lock().zoom_to_fit();
    }

    /// Zeichnet ein Bild. `false`, wenn das Fenster gerade verdeckt ist; dann erneut versuchen.
    pub fn render(&self) -> Result<bool, LotError> {
        self.lock().render().map_err(LotError::from_display)
    }
}
