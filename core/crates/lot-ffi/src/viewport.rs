//! Die Ansicht der App: Planansicht und 3D, Werkzeuge und Befehle.
//!
//! Zeigerlagen kommen von Swift in Punkten, gemessen von der Mitte der
//! Ansicht, y nach oben. Hier werden sie in Blattmillimeter und Modellmeter
//! umgerechnet und an den Editor (lot-edit) gegeben.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Instant;

use lot_edit::{Command, Editor, Input, Key, Overlay, Response, SnapKind, Tool};
use lot_model::{Point2, Project};
use lot_plan::floor_plan::{DEFAULT_CUT_HEIGHT, FloorPlanView, floor_plan};
use lot_plan::standard::OfficeStandard;
use lot_plan::{PaperPoint, Viewport as PlanViewport};
use lot_render::{OverlayItem, Renderer, UnderlayImage, UnderlayQuad, ViewMode};

use crate::{LotDocument, LotError};

/// Fangradius in Punkten.
const SNAP_POINTS: f64 = 10.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum ViewKind {
    Plan,
    Model,
}

/// Ergebnis eines Ereignisses für die App.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct EventResult {
    /// Die Ansicht muss neu gezeichnet werden.
    pub redraw: bool,
    /// Das Modell hat sich geändert (Eigenschaften, Undo neu lesen).
    pub model_changed: bool,
    /// Meldung für den Benutzer.
    pub error: Option<String>,
}

impl From<Response> for EventResult {
    fn from(r: Response) -> Self {
        Self { redraw: r.redraw, model_changed: r.model_changed, error: r.error }
    }
}

impl EventResult {
    fn redraw() -> Self {
        Self { redraw: true, model_changed: false, error: None }
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct CommandItem {
    pub id: String,
    pub title: String,
    pub shortcut: String,
    pub group: String,
}

/// Tasten für den Editor.
#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum KeyInput {
    Escape,
    Enter,
    Delete,
    Character { text: String },
}

/// Zustand für Werkzeugleiste und Statuszeile.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct ViewState {
    pub view: ViewKind,
    /// Aktives Werkzeug: „select“, „wall“, „grid“ oder „column“.
    pub tool: String,
    pub storey: String,
    pub scale: f64,
    pub wall_type: String,
    pub selection: Vec<u64>,
    pub hint: String,
}

struct State {
    renderer: Renderer,
    document: Option<Arc<LotDocument>>,
    editor: Option<Editor>,
    plan_scale: f64,
    /// Das 3D-Modell ist veraltet. Gerechnet wird erst, wenn die 3D-Ansicht zu sehen ist.
    model_stale: bool,
    /// Die 3D-Kamera wurde schon einmal auf ein nicht leeres Modell eingestellt.
    model_fitted: bool,
    /// Bilder der Unterlagen, nach Datei und Seite (siehe `underlay_key`).
    images: HashMap<String, Arc<UnderlayImage>>,
    /// Linien der Unterlagen aus DXF und DWG, nach Datei.
    vectors: HashMap<String, Arc<lot_io::cad::CadVectors>>,
    /// Bauteile der Referenzmodelle (IFC), nach Datei.
    references: HashMap<String, Arc<Vec<lot_io::ifc::IfcPart>>>,
}

/// Eine Unterlage, deren Bild die App liefern muss (PDF-Seite oder Bild).
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct UnderlayRef {
    pub source: String,
    pub page: u32,
}

fn underlay_key(source: &str, page: u32) -> String {
    format!("{source}#{page}")
}

#[derive(uniffi::Object)]
pub struct Viewport {
    state: Mutex<State>,
}

impl Viewport {
    fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(|e| e.into_inner())
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
        let mut renderer =
            unsafe { Renderer::for_appkit_view(view, width_px, height_px, scale) }.map_err(LotError::from_display)?;
        renderer.set_mode(ViewMode::Plan);
        Ok(Arc::new(Self {
            state: Mutex::new(State {
                renderer,
                document: None,
                editor: None,
                plan_scale: 100.0,
                model_stale: true,
                model_fitted: false,
                images: HashMap::new(),
                vectors: HashMap::new(),
                references: HashMap::new(),
            }),
        }))
    }

    pub fn resize(&self, width_px: u32, height_px: u32, scale: f32) {
        self.lock().renderer.resize(width_px, height_px, scale);
    }

    /// Zeigt ein Dokument. Rechnet alles neu und stellt beide Ansichten auf das ganze Modell ein.
    pub fn set_document(&self, document: Arc<LotDocument>) -> f64 {
        let mut s = self.lock();
        s.editor = Some(Editor::new(document.lock().project()));
        s.document = Some(document);
        s.model_fitted = false;
        s.images.clear();
        s.vectors.clear();
        s.references.clear();
        load_png_underlays(&mut s);
        let ms = rebuild(&mut s, true);
        build_model(&mut s);
        s.renderer.zoom_to_fit();
        s.renderer.plan_zoom_to_fit();
        ms
    }

    /// Bild einer Unterlage, von der App gerendert (PDF-Seite mit PDFKit oder ein Bild).
    /// RGBA mit 8 Bit je Kanal, Zeilen von oben.
    pub fn set_underlay_image(&self, source: String, page: u32, width: u32, height: u32, rgba: Vec<u8>) {
        let mut s = self.lock();
        s.images.insert(underlay_key(&source, page), Arc::new(UnderlayImage { width, height, rgba }));
        rebuild(&mut s, false);
    }

    /// Unterlagen ohne Bild, die die App rendern muss. PNG liest der Kern selbst.
    pub fn missing_underlay_images(&self) -> Vec<UnderlayRef> {
        let mut s = self.lock();
        load_png_underlays(&mut s);
        let Some(doc) = s.document.clone() else { return Vec::new() };
        let doc = doc.lock();
        let mut out: Vec<UnderlayRef> = doc
            .project()
            .elements
            .values()
            .filter_map(|e| match e {
                lot_model::Element::Underlay(u)
                    if !lot_io::cad::is_cad(&u.source) && !s.images.contains_key(&underlay_key(&u.source, u.page)) =>
                {
                    Some(UnderlayRef { source: u.source.clone(), page: u.page })
                }
                _ => None,
            })
            .collect();
        out.dedup();
        out
    }

    /// Legt eine Unterlage aus einer DXF- oder DWG-Datei im Projekt an (vorher
    /// mit `add_attachment` abgelegt). Maßstab nach den Einheiten der Datei;
    /// liegen ihre Koordinaten nahe am Ursprung, bleiben sie, sonst kommt die
    /// Zeichnung in die Mitte der Ansicht.
    pub fn add_cad_underlay(&self, source: String) -> Result<u64, LotError> {
        let mut s = self.lock();
        let (Some(doc), Some(editor)) = (s.document.clone(), s.editor.as_ref()) else {
            return Err(LotError::Failed("Kein Projekt geöffnet".into()));
        };
        let bytes =
            doc.attachment_bytes(&source).ok_or_else(|| LotError::Failed(format!("{source} fehlt im Projekt")))?;
        let vectors = lot_io::cad::read_vectors(&bytes).map_err(LotError::from_display)?;
        let Some((lo, hi)) = vectors.bounds() else {
            return Err(LotError::Failed(format!("{source} enthält keine Linien")));
        };
        let scale = vectors.metres_per_unit();
        let (width, height) = ((hi[0] - lo[0]).max(1e-6), (hi[1] - lo[1]).max(1e-6));
        let origin = if lo[0].abs().max(lo[1].abs()) * scale < 1000.0 {
            Point2::new(lo[0] * scale, lo[1] * scale)
        } else {
            let center = s.renderer.plan_camera().center;
            let m = s.plan_scale / 1000.0;
            Point2::new(center[0] * m - width * scale / 2.0, center[1] * m - height * scale / 2.0)
        };
        let underlay = lot_model::Underlay {
            storey: editor.storey,
            source: source.clone(),
            page: 0,
            origin,
            scale,
            width,
            height,
            rotation: 0.0,
            opacity: 0.45,
        };
        let id = doc.lock().insert(lot_model::Element::Underlay(underlay)).map_err(LotError::from_display)?;
        s.vectors.insert(source, Arc::new(vectors));
        rebuild(&mut s, false);
        Ok(id.0)
    }

    /// Legt ein Referenzmodell aus einer IFC-Datei im Projekt an (vorher mit
    /// `add_attachment` abgelegt). Gibt die Zahl der Bauteile zurück.
    pub fn add_reference_model(&self, source: String) -> Result<u32, LotError> {
        let mut s = self.lock();
        let Some(doc) = s.document.clone() else { return Err(LotError::Failed("Kein Projekt geöffnet".into())) };
        let bytes =
            doc.attachment_bytes(&source).ok_or_else(|| LotError::Failed(format!("{source} fehlt im Projekt")))?;
        let (parts, _) = lot_io::ifc::read_ifc(&bytes).map_err(LotError::from_display)?;
        let count = parts.len() as u32;
        let model = lot_model::ReferenceModel {
            source: source.clone(),
            origin: Point2::new(0.0, 0.0),
            elevation: 0.0,
            rotation: 0.0,
        };
        doc.lock().insert(lot_model::Element::ReferenceModel(model)).map_err(LotError::from_display)?;
        s.references.insert(source, Arc::new(parts));
        rebuild(&mut s, true);
        Ok(count)
    }

    /// Legt eine Unterlage im aktuellen Geschoss an, mittig in der Ansicht.
    /// `width` und `height` in Einheiten der Vorlage (PDF-Punkte oder Bildpunkte),
    /// `scale` in Metern je Einheit (vor dem Kalibrieren geschätzt).
    pub fn add_underlay(
        &self,
        source: String,
        page: u32,
        width: f64,
        height: f64,
        scale: f64,
    ) -> Result<u64, LotError> {
        let mut s = self.lock();
        let (Some(doc), Some(editor)) = (s.document.clone(), s.editor.as_ref()) else {
            return Err(LotError::Failed("Kein Projekt geöffnet".into()));
        };
        let center = s.renderer.plan_camera().center;
        let m = s.plan_scale / 1000.0;
        let origin = Point2::new(center[0] * m - width * scale / 2.0, center[1] * m - height * scale / 2.0);
        let underlay = lot_model::Underlay {
            storey: editor.storey,
            source,
            page,
            origin,
            scale,
            width,
            height,
            rotation: 0.0,
            opacity: 0.45,
        };
        let id = doc.lock().insert(lot_model::Element::Underlay(underlay)).map_err(LotError::from_display)?;
        rebuild(&mut s, false);
        Ok(id.0)
    }

    /// Das Modell hat sich außerhalb der Ansicht geändert (Eigenschaften, Menü).
    /// Rechnet neu und gibt die Zeit in Millisekunden zurück.
    pub fn refresh(&self) -> f64 {
        let mut s = self.lock();
        rebuild(&mut s, true)
    }

    pub fn set_view(&self, view: ViewKind) {
        let mut s = self.lock();
        s.renderer.set_mode(match view {
            ViewKind::Plan => ViewMode::Plan,
            ViewKind::Model => ViewMode::Model,
        });
        if s.model_stale && view == ViewKind::Model {
            build_model(&mut s);
        }
    }

    /// Maßstab der Planansicht (100 für 1:100). Der Zoom bleibt gleich groß auf dem Bildschirm.
    pub fn set_plan_scale(&self, scale: f64) {
        let mut s = self.lock();
        let old = s.plan_scale;
        s.plan_scale = scale;
        let mut camera = s.renderer.plan_camera();
        camera.center = [camera.center[0] * old / scale, camera.center[1] * old / scale];
        camera.px_per_mm *= scale / old;
        s.renderer.set_plan_camera(camera);
        rebuild(&mut s, false);
    }

    pub fn triangle_count(&self) -> u32 {
        self.lock().renderer.triangle_count()
    }

    /// Zwei Finger: in 3D drehen, im Plan verschieben. Bewegung in Punkten.
    pub fn orbit(&self, dx: f32, dy: f32) {
        let mut s = self.lock();
        match s.renderer.mode() {
            ViewMode::Model => s.renderer.camera_mut().orbit(dx, dy),
            ViewMode::Plan => s.renderer.plan_pan(dx, dy),
        }
    }

    pub fn pan(&self, dx: f32, dy: f32) {
        let mut s = self.lock();
        match s.renderer.mode() {
            ViewMode::Model => s.renderer.camera_mut().pan(dx, dy),
            ViewMode::Plan => s.renderer.plan_pan(dx, dy),
        }
    }

    /// Zoomen zum Zeiger. Faktor größer 1 zoomt hinein. `x`, `y`: Lage des
    /// Zeigers in Punkten von der Mitte der Ansicht, y nach oben.
    pub fn zoom(&self, factor: f32, x: f32, y: f32) {
        let mut s = self.lock();
        match s.renderer.mode() {
            ViewMode::Model => s.renderer.camera_mut().zoom(factor, x, y),
            ViewMode::Plan => {
                s.renderer.plan_zoom(factor, x, y);
                update_overlay(&mut s);
            }
        }
    }

    pub fn zoom_to_fit(&self) {
        let mut s = self.lock();
        match s.renderer.mode() {
            ViewMode::Model => s.renderer.zoom_to_fit(),
            ViewMode::Plan => s.renderer.plan_zoom_to_fit(),
        }
    }

    pub fn pointer_moved(&self, x: f32, y: f32, shift: bool) -> EventResult {
        self.edit(x, y, shift, |editor, doc, input| editor.pointer_moved(doc.project(), input))
    }

    pub fn pointer_down(&self, x: f32, y: f32, clicks: u32, shift: bool) -> EventResult {
        self.edit(x, y, shift, |editor, doc, input| editor.pointer_down(doc, input, clicks))
    }

    pub fn pointer_dragged(&self, x: f32, y: f32, shift: bool) -> EventResult {
        self.edit(x, y, shift, |editor, doc, input| editor.pointer_dragged(doc.project(), input))
    }

    pub fn pointer_up(&self) -> EventResult {
        self.edit(0.0, 0.0, false, |editor, doc, _| editor.pointer_up(doc))
    }

    pub fn key(&self, key: KeyInput) -> EventResult {
        let key = match key {
            KeyInput::Escape => Key::Escape,
            KeyInput::Enter => Key::Enter,
            KeyInput::Delete => Key::Delete,
            KeyInput::Character { text } => match text.chars().next() {
                Some(c) => Key::Char(c),
                None => return EventResult::redraw(),
            },
        };
        self.edit(0.0, 0.0, false, |editor, doc, _| editor.key(doc, key))
    }

    /// Befehle für die Befehlspalette: Editor und Ansicht.
    pub fn commands(&self) -> Vec<CommandItem> {
        let s = self.lock();
        let mut list = vec![
            item("view:plan", "Ansicht Grundriss", "1", "Ansicht"),
            item("view:model", "Ansicht 3D", "2", "Ansicht"),
            item("view:fit", "Alles zeigen", "⌘0", "Ansicht"),
        ];
        for scale in [50, 100, 200] {
            let mark = if (s.plan_scale - scale as f64).abs() < 1e-9 { " ✓" } else { "" };
            list.push(item(&format!("scale:{scale}"), &format!("Maßstab 1:{scale}{mark}"), "", "Ansicht"));
        }
        if let (Some(editor), Some(doc)) = (&s.editor, &s.document) {
            list.extend(editor.commands(doc.lock().project()).into_iter().map(|c| CommandItem {
                id: c.id,
                title: c.title,
                shortcut: c.shortcut,
                group: c.group,
            }));
        }
        list
    }

    /// Führt einen Befehl aus der Befehlspalette aus.
    pub fn run_command(&self, id: String) -> EventResult {
        match id.as_str() {
            "view:plan" => {
                self.set_view(ViewKind::Plan);
                return EventResult::redraw();
            }
            "view:model" => {
                self.set_view(ViewKind::Model);
                return EventResult::redraw();
            }
            "view:fit" => {
                self.zoom_to_fit();
                return EventResult::redraw();
            }
            _ => {}
        }
        if let Some(scale) = id.strip_prefix("scale:").and_then(|v| v.parse::<f64>().ok()) {
            self.set_plan_scale(scale);
            return EventResult::redraw();
        }
        let Some(command) = Command::parse(&id) else {
            return EventResult { error: Some(format!("Unbekannter Befehl: {id}")), ..EventResult::redraw() };
        };
        let mut s = self.lock();
        let (Some(doc), Some(mut editor)) = (s.document.clone(), s.editor.take()) else {
            return EventResult::redraw();
        };
        let response = editor.run(&mut doc.lock(), &command);
        s.editor = Some(editor);
        // Wechsel des Geschosses oder Werkzeugs braucht einen neuen Plan, Modelländerungen auch 3D.
        rebuild(&mut s, response.model_changed);
        response.into()
    }

    pub fn state(&self) -> ViewState {
        let s = self.lock();
        let view = match s.renderer.mode() {
            ViewMode::Plan => ViewKind::Plan,
            ViewMode::Model => ViewKind::Model,
        };
        let (storey, wall_type, selection, hint, tool) = match (&s.editor, &s.document) {
            (Some(e), Some(doc)) => {
                let doc = doc.lock();
                let p = doc.project();
                (
                    p.storey(e.storey).map(|st| st.name.clone()).unwrap_or_default(),
                    e.wall_type.and_then(|t| p.wall_type(t)).map(|t| t.name.clone()).unwrap_or_default(),
                    e.selection.iter().map(|id| id.0).collect(),
                    e.hint(),
                    match e.tool {
                        Tool::Select => "select",
                        Tool::Wall => "wall",
                        Tool::ArcWall => "arc",
                        Tool::Grid => "grid",
                        Tool::Column => "column",
                        Tool::Space => "space",
                        Tool::Door => "door",
                        Tool::Window => "window",
                        Tool::SlabOpening => "slabOpening",
                        Tool::Line => "line",
                        Tool::Text => "text",
                        Tool::CalibrateUnderlay => "calibrate",
                        Tool::AlignUnderlay => "align",
                    }
                    .to_string(),
                )
            }
            _ => (String::new(), String::new(), Vec::new(), String::new(), "select".to_string()),
        };
        ViewState { view, tool, storey, scale: s.plan_scale, wall_type, selection, hint }
    }

    /// Zeichnet ein Bild. `false`, wenn das Fenster gerade verdeckt ist; dann erneut versuchen.
    pub fn render(&self) -> Result<bool, LotError> {
        self.lock().renderer.render().map_err(LotError::from_display)
    }
}

impl Viewport {
    /// Gemeinsamer Weg für Zeiger- und Tastenereignisse im Plan.
    fn edit(
        &self,
        x: f32,
        y: f32,
        shift: bool,
        action: impl FnOnce(&mut Editor, &mut lot_model::Document, Input) -> Response,
    ) -> EventResult {
        let mut s = self.lock();
        if s.renderer.mode() != ViewMode::Plan {
            return EventResult { redraw: false, model_changed: false, error: None };
        }
        let (Some(doc), Some(mut editor)) = (s.document.clone(), s.editor.take()) else {
            return EventResult { redraw: false, model_changed: false, error: None };
        };
        let paper = s.renderer.plan_point(x, y);
        let m_per_mm = s.plan_scale / 1000.0;
        let input = Input {
            point: Point2::new(paper[0] * m_per_mm, paper[1] * m_per_mm),
            tolerance: SNAP_POINTS * s.renderer.plan_mm_per_point() * m_per_mm,
            shift,
        };
        let dragging_before = editor.preview(doc.lock().project()).is_some();
        let response = action(&mut editor, &mut doc.lock(), input);
        let dragging = editor.preview(doc.lock().project()).is_some();
        s.editor = Some(editor);
        if response.model_changed {
            rebuild(&mut s, true);
        } else if dragging || dragging_before {
            rebuild(&mut s, false);
        } else if response.redraw {
            update_overlay(&mut s);
        }
        response.into()
    }
}

fn item(id: &str, title: &str, shortcut: &str, group: &str) -> CommandItem {
    CommandItem { id: id.into(), title: title.into(), shortcut: shortcut.into(), group: group.into() }
}

/// Rechnet den Plan neu, und nach einer Modelländerung (`model`) auch 3D,
/// falls die 3D-Ansicht zu sehen ist. Gibt die Zeit in ms zurück.
fn rebuild(s: &mut State, model: bool) -> f64 {
    let start = Instant::now();
    if model {
        s.model_stale = true;
        if s.renderer.mode() == ViewMode::Model {
            build_model(s);
        }
    }
    let Some(doc) = s.document.clone() else { return 0.0 };
    let doc = doc.lock();
    if let Some(editor) = s.editor.as_mut() {
        editor.sync(doc.project());
    }
    let standard = OfficeStandard::default_standard();
    let preview = s.editor.as_ref().and_then(|e| e.preview(doc.project()));
    let project: &Project = preview.as_ref().unwrap_or(doc.project());
    // Gezeigt werden die aktiven Varianten (2.04).
    let shown = project.variant_sets().next().is_some().then(|| project.variant_view());
    let project: &Project = shown.as_ref().unwrap_or(project);
    if let Some(editor) = &s.editor {
        let view = FloorPlanView { storey: editor.storey, cut_height: DEFAULT_CUT_HEIGHT, scale: s.plan_scale };
        let viewport = PlanViewport { scale: s.plan_scale, origin: PaperPoint::new(0.0, 0.0) };
        // Unterlagen aus DXF und DWG als graue Linien unter dem Plan, ihre Endpunkte zum Fangen.
        let mut objects = Vec::new();
        let mut points = Vec::new();
        for (_, u) in project.underlays_on(editor.storey).filter(|(_, u)| lot_io::cad::is_cad(&u.source)) {
            let Some(v) = s.vectors.get(&u.source) else { continue };
            objects.extend(lot_io::cad::underlay_objects(u, v, viewport));
            points.extend(lot_io::cad::underlay_points(u, v));
        }
        // Referenzmodelle, geschnitten in der Schnittebene.
        let z = project.storey(editor.storey).map_or(0.0, |st| st.elevation) + DEFAULT_CUT_HEIGHT;
        let parts = reference_parts(project, &s.references);
        lot_plan::reference::draw_reference_cut(&mut objects, &parts, z, viewport);
        objects.extend(floor_plan(project, view, standard, viewport));
        s.renderer.set_plan(objects);
        let quads = underlay_quads(project, editor.storey, &s.images, viewport);
        s.renderer.set_underlays(&quads);
        if let Some(editor) = s.editor.as_mut() {
            editor.underlay_points = points;
        }
    }
    drop(doc);
    update_overlay(s);
    start.elapsed().as_secs_f64() * 1000.0
}

/// Geometrie aller Referenzmodelle, deren Datei gelesen ist.
fn reference_parts(
    project: &Project,
    references: &HashMap<String, Arc<Vec<lot_io::ifc::IfcPart>>>,
) -> Vec<lot_geom::ElementGeometry> {
    project
        .reference_models()
        .filter_map(|(id, r)| Some(lot_io::ifc::reference_geometry(id, r, references.get(&r.source)?)))
        .flatten()
        .collect()
}

/// Unterlagen des Geschosses, deren Bild vorliegt, an ihrem Platz auf dem Blatt.
pub(crate) fn underlay_quads(
    project: &Project,
    storey: lot_model::ElementId,
    images: &HashMap<String, Arc<UnderlayImage>>,
    viewport: PlanViewport,
) -> Vec<UnderlayQuad> {
    project
        .underlays_on(storey)
        .filter_map(|(_, u)| {
            let image = images.get(&underlay_key(&u.source, u.page))?.clone();
            let corners = u.corners().map(|p| {
                let q = viewport.to_paper(p.x, p.y);
                [q.x, q.y]
            });
            Some(UnderlayQuad { corners, image, opacity: u.opacity as f32 })
        })
        .collect()
}

/// Liest Unterlagen als PNG und aus DXF oder DWG sowie Referenzmodelle aus den Dateien des Projekts.
fn load_png_underlays(s: &mut State) {
    if let Some(doc) = s.document.clone() {
        let ifc: Vec<String> = doc.lock().project().reference_models().map(|(_, r)| r.source.clone()).collect();
        for source in ifc {
            if s.references.contains_key(&source) {
                continue;
            }
            if let Some((parts, _)) = doc.attachment_bytes(&source).and_then(|b| lot_io::ifc::read_ifc(&b).ok()) {
                s.references.insert(source, Arc::new(parts));
            }
        }
        let cad: Vec<String> = doc
            .lock()
            .project()
            .elements
            .values()
            .filter_map(|e| match e {
                lot_model::Element::Underlay(u) if lot_io::cad::is_cad(&u.source) => Some(u.source.clone()),
                _ => None,
            })
            .collect();
        for source in cad {
            if s.vectors.contains_key(&source) {
                continue;
            }
            if let Some(v) = doc.attachment_bytes(&source).and_then(|b| lot_io::cad::read_vectors(&b).ok()) {
                s.vectors.insert(source, Arc::new(v));
            }
        }
    }
    let Some(doc) = s.document.clone() else { return };
    let wanted: Vec<(String, u32)> = doc
        .lock()
        .project()
        .elements
        .values()
        .filter_map(|e| match e {
            lot_model::Element::Underlay(u) if u.source.to_lowercase().ends_with(".png") => {
                Some((u.source.clone(), u.page))
            }
            _ => None,
        })
        .collect();
    for (source, page) in wanted {
        let key = underlay_key(&source, page);
        if s.images.contains_key(&key) {
            continue;
        }
        if let Some(r) = doc.attachment_bytes(&source).and_then(|b| lot_io::raster::decode_png(&b).ok()) {
            s.images.insert(key, Arc::new(UnderlayImage { width: r.width, height: r.height, rgba: r.rgba }));
        }
    }
}

/// Rechnet das 3D-Modell.
fn build_model(s: &mut State) {
    let Some(doc) = s.document.clone() else { return };
    let standard = OfficeStandard::default_standard();
    // Bei Umbauten zeigt das 3D den Zustand nach dem Umbau.
    let built = {
        let guard = doc.lock();
        let project = guard.project();
        if project.has_phases() {
            lot_geom::build(&project.variant_view().phase_view(lot_model::PhaseFilter::Planned))
        } else {
            lot_geom::build(&project.variant_view())
        }
    };
    if let Ok(mut geometry) = built {
        let parts = reference_parts(doc.lock().project(), &s.references);
        geometry.elements.extend(parts);
        s.renderer.set_geometry(&geometry, &|m| standard.material(m).color);
        s.model_stale = false;
        // Beim ersten nicht leeren Modell die Kamera darauf einstellen.
        if !s.model_fitted && !geometry.elements.is_empty() {
            s.renderer.zoom_to_fit();
            s.model_fitted = true;
        }
    }
}

/// Hinweise des Editors über dem Plan, in Blattmillimetern.
fn update_overlay(s: &mut State) {
    let (Some(editor), Some(doc)) = (&s.editor, &s.document) else { return };
    let overlay: Overlay = editor.overlay(doc.lock().project());
    let mm = 1000.0 / s.plan_scale;
    let paper = |p: [f64; 2]| [p[0] * mm, p[1] * mm];
    let blue = linear([0.10, 0.45, 0.95], 1.0);
    let mut items = Vec::new();
    for outline in overlay.selected {
        items.push(OverlayItem::Polygon {
            points: outline.into_iter().map(paper).collect(),
            fill: linear([0.10, 0.45, 0.95], 0.12),
            stroke: blue,
            width_px: 2.0,
        });
    }
    for h in overlay.handles {
        items.push(OverlayItem::Square { center: paper([h.x, h.y]), size_px: 9.0, color: blue });
    }
    if let Some(preview) = overlay.preview {
        items.push(OverlayItem::Polygon {
            points: preview.into_iter().map(paper).collect(),
            fill: linear([0.10, 0.45, 0.95], 0.2),
            stroke: blue,
            width_px: 1.5,
        });
    }
    if let Some((at, text)) = overlay.label {
        items.push(OverlayItem::Text {
            at: paper([at.x, at.y]),
            text,
            height_px: 12.0,
            color: linear([0.05, 0.25, 0.6], 1.0),
        });
    }
    if let Some(snap) = overlay.snap {
        let color = match snap.kind {
            SnapKind::Free => linear([0.5, 0.5, 0.5], 1.0),
            _ => linear([0.95, 0.45, 0.05], 1.0),
        };
        items.push(OverlayItem::Ring {
            center: paper([snap.point.x, snap.point.y]),
            radius_px: 6.0,
            width_px: 2.0,
            color,
        });
    }
    s.renderer.set_overlay(items);
}

fn linear(srgb: [f32; 3], alpha: f32) -> [f32; 4] {
    let f = |c: f32| if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) };
    [f(srgb[0]), f(srgb[1]), f(srgb[2]), alpha]
}
