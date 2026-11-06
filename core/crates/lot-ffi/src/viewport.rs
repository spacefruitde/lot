//! Die Ansicht der App: Planansicht und 3D, Werkzeuge und Befehle.
//!
//! Zeigerlagen kommen von Swift in Punkten, gemessen von der Mitte der
//! Ansicht, y nach oben. Hier werden sie in Blattmillimeter und Modellmeter
//! umgerechnet und an den Editor (lot-edit) gegeben.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Instant;

use lot_edit::navigation::{self as nav, History, Place, Schedule, Target, ViewRef};
use lot_edit::{Command, Editor, Input, Key, Overlay, Response, SnapKind, Tool};
use lot_model::{Point2, Project};
use lot_plan::floor_plan::FloorPlanView;
use lot_plan::standard::OfficeStandard;
use lot_plan::{PaperPoint, PlanObject, Viewport as PlanViewport};
use lot_render::{OverlayItem, PlanCamera, Renderer, UnderlayImage, UnderlayQuad, ViewMode};

use crate::visibility::{HiddenInfo, LookInfo, ReasonInfo, look_info};
use crate::{LotDocument, LotError};

/// Fangradius in Punkten.
const SNAP_POINTS: f64 = 10.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum ViewKind {
    /// Grundriss, der einzige, in dem gezeichnet wird.
    Plan,
    Model,
    Section,
    SitePlan,
    Sheet,
    Schedule,
}

/// Abstand der Seiten einer Liste untereinander, in mm.
const PAGE_GAP: f64 = 20.0;

/// Ergebnis eines Ereignisses für die App.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct EventResult {
    /// Die Ansicht muss neu gezeichnet werden.
    pub redraw: bool,
    /// Das Modell hat sich geändert (Eigenschaften, Undo neu lesen).
    pub model_changed: bool,
    /// Meldung für den Benutzer.
    pub error: Option<String>,
    /// Diese Ansicht öffnen (Kennung eines `ViewRef`), etwa nach Doppelklick auf eine Schnittmarke.
    pub open: Option<String>,
}

impl From<Response> for EventResult {
    fn from(r: Response) -> Self {
        Self { redraw: r.redraw, model_changed: r.model_changed, error: r.error, open: r.open.map(|v| v.id()) }
    }
}

impl EventResult {
    fn redraw() -> Self {
        Self { redraw: true, model_changed: false, error: None, open: None }
    }

    fn none() -> Self {
        Self { redraw: false, model_changed: false, error: None, open: None }
    }
}

/// Die gewählte Ansicht auf einem Planblatt, für den Inspektor.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct PlacedInfo {
    pub sheet: u64,
    pub index: u32,
    /// Kennung der Ansicht, etwa `Section(12)`.
    pub view: String,
    pub title: String,
    pub scale: f64,
    /// Liegt die Ansicht auf mehreren Blättern, ist dies ihre Heimat.
    pub home: bool,
    /// Die Ansicht liegt auf mehreren Blättern und ist ein Schnitt: Heimat wählbar.
    pub can_be_home: bool,
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
    Tab,
    /// Pfeiltaste, `large` mit ⇧.
    Arrow {
        dx: i8,
        dy: i8,
        large: bool,
    },
    Character {
        text: String,
    },
}

/// Der gewählte Anschluss für den Inspektor (ADR 0045).
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct JunctionInfo {
    /// Etwa „Ecke“ oder „T-Stoß“.
    pub title: String,
    /// Warum es so aussieht, etwa „Kalksandstein schneidet Innenputz“.
    pub lines: Vec<String>,
    pub arms: Vec<JunctionArmInfo>,
}

/// Eine Wand an einem Anschluss.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct JunctionArmInfo {
    pub wall: u64,
    pub label: String,
    /// Etwa „läuft durch (Ausnahme)“.
    pub role: String,
    /// 0 automatisch, 1 läuft durch, 2 nicht verbinden.
    pub rule: u32,
    /// Befehle für die drei Regeln; leer bei der Wand, die an einem T-Stoß durchläuft.
    pub commands: Vec<String>,
}

/// Zustand für Werkzeugleiste und Statuszeile.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct ViewState {
    pub view: ViewKind,
    /// Kennung der gezeigten Ansicht, etwa `FloorPlan(3)` (ADR 0049).
    pub shown: String,
    /// Etwa „Grundriss EG“ oder „Schnitt A-A“.
    pub title: String,
    pub can_go_back: bool,
    pub can_go_forward: bool,
    /// Im Grundriss gibt es ein Geschoss darunter.
    pub has_below: bool,
    /// Das Geschoss darunter ist blass zu sehen.
    pub faded_below: bool,
    /// Vorübergehend ausgeblendet oder isoliert (H, I): etwa „3 ausgeblendet“; leer, wenn nichts.
    pub temporary: String,
    /// Druckbild: nur, was gedruckt wird (ADR 0050).
    pub print_view: bool,
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
    /// Hinweise aus dem Prüfbericht im Plan zeigen; der Bericht wird nach
    /// Modelländerungen neu gerechnet.
    show_findings: bool,
    findings: Option<lot_rules::CheckReport>,
    /// Was die Ansicht zeigt (ADR 0049).
    shown: ViewRef,
    history: History,
    /// Kamera je Ansicht; alle Grundrisse teilen eine, damit der Geschosswechsel an der Stelle bleibt.
    cameras: HashMap<ViewRef, PlanCamera>,
    /// Vorübergehend Ausgeblendetes (H, I) der übrigen Ansichten; das der gezeigten hält der Editor.
    temps: HashMap<ViewRef, lot_plan::views::Temporary>,
    /// Druckbild: ohne Hinweise, Fangpunkte, Unterlagen, blasses Geschoss und ohne H und I.
    print_view: bool,
    /// Darstellung der gezeigten Ansicht mit der Wirkung jeder Regel (ADR 0050).
    look: Option<(lot_plan::views::Look, Vec<usize>, usize)>,
    /// Gerechnetes Modell, wie die gezeigte Ansicht es zeigt (Schnitt, Lageplan).
    shown_geometry: Option<Arc<lot_geom::ModelGeometry>>,
    /// Gerechnetes Modell für Schnitte, Lageplan und Blätter; verfällt mit jeder Modelländerung.
    geometry: Option<Arc<lot_geom::ModelGeometry>>,
    /// Auf dem gezeigten Blatt: Umrisse der Ansichten, die gewählte und ein laufendes Ziehen.
    extents: Vec<(usize, PaperPoint, PaperPoint)>,
    placed: Option<usize>,
    placed_drag: Option<PlacedDrag>,
}

/// Eine Ansicht auf dem Blatt wird gezogen: Stelle, Start und jetzt in Blatt-mm.
#[derive(Debug, Clone, Copy)]
struct PlacedDrag {
    index: usize,
    start: [f64; 2],
    current: [f64; 2],
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
                show_findings: false,
                findings: None,
                shown: ViewRef::FloorPlan(0),
                history: History::default(),
                cameras: HashMap::new(),
                temps: HashMap::new(),
                print_view: false,
                look: None,
                shown_geometry: None,
                geometry: None,
                extents: Vec::new(),
                placed: None,
                placed_drag: None,
            }),
        }))
    }

    /// Hinweise aus dem Prüfbericht im Grundriss ein- oder ausblenden.
    pub fn set_show_findings(&self, show: bool) {
        let mut s = self.lock();
        s.show_findings = show;
        rebuild(&mut s, false);
    }

    /// Zeigt einen Ort: wechselt ins Geschoss und schiebt den Plan dorthin.
    pub fn focus(&self, storey: u64, x: f64, y: f64) {
        let mut s = self.lock();
        show(&mut s, ViewRef::FloorPlan(storey), true);
        let f = 1000.0 / s.plan_scale;
        let mut camera = s.renderer.plan_camera();
        camera.center = [x * f, y * f];
        s.renderer.set_plan_camera(camera);
    }

    pub fn resize(&self, width_px: u32, height_px: u32, scale: f32) {
        self.lock().renderer.resize(width_px, height_px, scale);
    }

    /// Zeigt ein Dokument. Rechnet alles neu und stellt beide Ansichten auf das ganze Modell ein.
    pub fn set_document(&self, document: Arc<LotDocument>) -> f64 {
        let mut s = self.lock();
        let editor = Editor::new(document.lock().project());
        s.shown = ViewRef::FloorPlan(editor.storey.0);
        s.editor = Some(editor);
        s.document = Some(document);
        s.history = History::default();
        s.cameras.clear();
        s.geometry = None;
        s.placed = None;
        s.placed_drag = None;
        s.renderer.set_mode(ViewMode::Plan);
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

    /// ⌘1 Grundriss des aktuellen Geschosses, ⌘2 3D; die übrigen Ansichten öffnet `go`.
    pub fn set_view(&self, view: ViewKind) {
        let mut s = self.lock();
        let target = match view {
            ViewKind::Model => ViewRef::Model,
            _ => ViewRef::FloorPlan(s.editor.as_ref().map_or(0, |e| e.storey.0)),
        };
        show(&mut s, target, true);
    }

    /// Springt zu einer Ansicht oder einem Element (Kennung eines `Target` oder `ViewRef`), mit Verlauf.
    pub fn go(&self, target: String) -> EventResult {
        let mut s = self.lock();
        let target = match (Target::parse(&target), ViewRef::parse(&target)) {
            (Some(t), _) => t,
            (None, Some(v)) => Target::View(v),
            _ => return EventResult { error: Some(format!("Unbekanntes Ziel: {target}")), ..EventResult::redraw() },
        };
        match target {
            Target::View(v) => {
                if !show(&mut s, v, true) {
                    return EventResult {
                        error: Some("Diese Ansicht gibt es nicht mehr".into()),
                        ..EventResult::redraw()
                    };
                }
            }
            Target::Element(id) => {
                let Some(doc) = s.document.clone() else { return EventResult::none() };
                let at = match doc.lock().project().elements.get(&lot_model::ElementId(id)) {
                    Some(lot_model::Element::Space(r)) => Some((r.storey, r.position)),
                    _ => None,
                };
                let Some((storey, p)) = at else { return EventResult::none() };
                show(&mut s, ViewRef::FloorPlan(storey.0), true);
                if let Some(editor) = s.editor.as_mut() {
                    editor.selection = vec![lot_model::ElementId(id)];
                }
                let f = 1000.0 / s.plan_scale;
                let mut camera = s.renderer.plan_camera();
                camera.center = [p.x * f, p.y * f];
                s.renderer.set_plan_camera(camera);
                update_overlay(&mut s);
            }
        }
        EventResult::redraw()
    }

    /// Zurück (⌘[) an die vorige Ansicht und Stelle.
    pub fn back(&self) -> EventResult {
        let mut s = self.lock();
        let current = place(&s);
        if let Some(p) = s.history.back(current) {
            restore(&mut s, p);
        }
        EventResult::redraw()
    }

    /// Vorwärts (⌘]).
    pub fn forward(&self) -> EventResult {
        let mut s = self.lock();
        let current = place(&s);
        if let Some(p) = s.history.forward(current) {
            restore(&mut s, p);
        }
        EventResult::redraw()
    }

    /// Ein Geschoss höher (⌘↑) oder tiefer (⌘↓); außerhalb eines Grundrisses nichts.
    pub fn step_storey(&self, up: bool) -> EventResult {
        let mut s = self.lock();
        let ViewRef::FloorPlan(storey) = s.shown else { return EventResult::none() };
        let Some(doc) = s.document.clone() else { return EventResult::none() };
        let next = nav::storey_step(doc.lock().project(), lot_model::ElementId(storey), up);
        match next {
            Some(id) => {
                show(&mut s, ViewRef::FloorPlan(id.0), true);
                EventResult::redraw()
            }
            None => EventResult::none(),
        }
    }

    /// Geschoss darunter im aktuellen Grundriss blass zeigen oder nicht; eine Einstellung der Ansicht.
    pub fn set_faded_below(&self, show_it: bool) -> EventResult {
        self.set_view_field(format!("FadedBelow(Some({show_it}))"))
    }

    /// Eine Einstellung der gezeigten Ansicht setzen oder übernehmen (ADR 0050);
    /// `field` ist die RON-Schreibweise eines `ViewField`, etwa `Scale(Some(50.0))`.
    pub fn set_view_field(&self, field: String) -> EventResult {
        let Some(field) = lot_edit::visibility::ViewField::parse(&field) else {
            return EventResult { error: Some(format!("Unbekannte Einstellung: {field}")), ..EventResult::redraw() };
        };
        let mut s = self.lock();
        let Some(target) = s.shown.content() else { return EventResult::none() };
        run_editor(&mut s, &Command::SetViewField(target, field))
    }

    /// Ansichtsvorlage der gezeigten Ansicht; `None` für keine.
    pub fn set_view_template(&self, name: Option<String>) -> EventResult {
        let mut s = self.lock();
        let Some(target) = s.shown.content() else { return EventResult::none() };
        run_editor(&mut s, &Command::SetViewTemplate(target, name))
    }

    /// Regel des Themas in der gezeigten Ansicht ein- oder abschalten; eigene Regeln der Ansicht löschen.
    pub fn set_rule_on(&self, name: String, on: bool, own: bool) -> EventResult {
        let mut s = self.lock();
        let Some(target) = s.shown.content() else { return EventResult::none() };
        let command = if own { Command::RemoveViewRule(target, name) } else { Command::SetRuleOn(target, name, on) };
        run_editor(&mut s, &command)
    }

    /// Gewählte Bauteile in der gezeigten Ansicht ausblenden.
    pub fn hide_in_view(&self) -> EventResult {
        let mut s = self.lock();
        let Some(target) = s.shown.content() else { return EventResult::none() };
        run_editor(&mut s, &Command::HideInView(target))
    }

    /// In der gezeigten Ansicht Ausgeblendete wieder zeigen; leer zeigt alle.
    pub fn show_in_view(&self, ids: Vec<u64>) -> EventResult {
        let mut s = self.lock();
        let Some(target) = s.shown.content() else { return EventResult::none() };
        run_editor(&mut s, &Command::ShowInView(target, ids))
    }

    /// Druckbild ein oder aus.
    pub fn set_print_view(&self, on: bool) {
        let mut s = self.lock();
        s.print_view = on;
        rebuild(&mut s, false);
    }

    /// Darstellung der gezeigten Ansicht mit Herkunft jeder Einstellung; `None` bei 3D, Blättern und Listen.
    pub fn look(&self) -> Option<LookInfo> {
        let s = self.lock();
        let (look, counts, hidden_by_rules) = s.look.as_ref()?;
        Some(look_info(look, counts, *hidden_by_rules))
    }

    /// Warum gewählte Bauteile in der gezeigten Ansicht fehlen, mit Lösung je Grund.
    pub fn hidden_reasons(&self, ids: Vec<u64>) -> Vec<HiddenInfo> {
        let s = self.lock();
        let (Some((look, _, _)), Some(doc), Some(editor)) = (&s.look, &s.document, &s.editor) else {
            return Vec::new();
        };
        let doc = doc.lock();
        let project = doc.project();
        ids.into_iter()
            .filter_map(|id| {
                let id = lot_model::ElementId(id);
                let reasons = lot_edit::visibility::reasons(project, look, &editor.temp, id);
                (!reasons.is_empty()).then(|| HiddenInfo {
                    element: id.0,
                    label: lot_edit::select::label(project, id),
                    reasons: reasons
                        .into_iter()
                        .map(|r| {
                            let (fix_title, fix) = match r.fix {
                                Some((t, lot_edit::visibility::Fix::Run(c))) => (t, c.id()),
                                Some((t, lot_edit::visibility::Fix::Go(v))) => (t, format!("go:{}", v.id())),
                                None => (String::new(), String::new()),
                            };
                            ReasonInfo { text: r.text, fix_title, fix }
                        })
                        .collect(),
                })
            })
            .collect()
    }

    /// Auswahl von außen setzen, etwa aus der Ansicht nebenan (gemeinsame Auswahl, ADR 0049).
    pub fn set_selection(&self, ids: Vec<u64>) {
        let mut s = self.lock();
        if let Some(editor) = s.editor.as_mut() {
            let ids: Vec<lot_model::ElementId> = ids.into_iter().map(lot_model::ElementId).collect();
            if editor.selection != ids {
                editor.selection = ids;
                editor.junction = None;
            }
        }
        update_overlay(&mut s);
    }

    /// Eine Ansicht aus der Seitenleiste auf das gezeigte Blatt fallen lassen.
    /// `x`, `y` wie beim Zeiger: Punkte von der Mitte der Ansicht.
    pub fn drop_view(&self, view: String, x: f32, y: f32) -> EventResult {
        let mut s = self.lock();
        let ViewRef::Sheet(sheet) = s.shown else {
            return EventResult {
                error: Some("Ansichten lassen sich nur auf ein Planblatt legen".into()),
                ..EventResult::redraw()
            };
        };
        let Some(view) = ViewRef::parse(&view) else { return EventResult::none() };
        let p = s.renderer.plan_point(x, y);
        let result = run_editor(
            &mut s,
            &Command::PlaceView(sheet, view, (p[0] * 10.0).round() / 10.0, (p[1] * 10.0).round() / 10.0),
        );
        if result.error.is_none()
            && let Some(doc) = s.document.clone()
            && let Some(lot_model::Element::PlanSheet(sh)) =
                doc.lock().project().elements.get(&lot_model::ElementId(sheet))
        {
            s.placed = sh.views.len().checked_sub(1);
        }
        update_overlay(&mut s);
        result
    }

    /// Die gewählte Ansicht auf dem gezeigten Blatt.
    pub fn placed_info(&self) -> Option<PlacedInfo> {
        let s = self.lock();
        let (ViewRef::Sheet(sheet), Some(index)) = (s.shown, s.placed) else { return None };
        let doc = s.document.clone()?;
        let doc = doc.lock();
        let project = doc.project();
        let Some(lot_model::Element::PlanSheet(sh)) = project.elements.get(&lot_model::ElementId(sheet)) else {
            return None;
        };
        let placed = sh.views.get(index)?;
        let view = ViewRef::of_content(placed.content);
        let many = nav::placements(project, view).len() > 1;
        Some(PlacedInfo {
            sheet,
            index: index as u32,
            view: view.id(),
            title: view.title(project),
            scale: placed.scale,
            home: many && nav::home(project, view) == Some((lot_model::ElementId(sheet), index)),
            can_be_home: many && matches!(view, ViewRef::Section(_)),
        })
    }

    /// Maßstab der gezeigten Ansicht (100 für 1:100), eine Einstellung der Ansicht (ADR 0050).
    pub fn set_plan_scale(&self, scale: f64) -> EventResult {
        self.set_view_field(format!("Scale(Some({scale:?}))"))
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
                if matches!(s.shown, ViewRef::FloorPlan(_)) {
                    update_overlay(&mut s);
                }
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

    pub fn pointer_moved(&self, x: f32, y: f32, shift: bool, alt: bool) -> EventResult {
        self.edit(x, y, shift, alt, |editor, doc, input| editor.pointer_moved(doc.project(), input))
    }

    pub fn pointer_down(&self, x: f32, y: f32, clicks: u32, shift: bool, alt: bool) -> EventResult {
        let shown = self.lock().shown;
        match shown {
            ViewRef::Section(_) => self.section_click(x, y, shift),
            ViewRef::Sheet(_) => self.sheet_down(x, y, clicks),
            _ => self.edit(x, y, shift, alt, |editor, doc, input| editor.pointer_down(doc, input, clicks)),
        }
    }

    pub fn pointer_dragged(&self, x: f32, y: f32, shift: bool, alt: bool) -> EventResult {
        let mut s = self.lock();
        if let ViewRef::Sheet(_) = s.shown {
            let p = s.renderer.plan_point(x, y);
            if let Some(d) = s.placed_drag.as_mut() {
                d.current = p;
            }
            update_overlay(&mut s);
            return EventResult::redraw();
        }
        drop(s);
        self.edit(x, y, shift, alt, |editor, doc, input| editor.pointer_dragged(doc.project(), input))
    }

    pub fn pointer_up(&self) -> EventResult {
        let mut s = self.lock();
        if let ViewRef::Sheet(sheet) = s.shown {
            let Some(d) = s.placed_drag.take() else { return EventResult::none() };
            let (dx, dy) = (d.current[0] - d.start[0], d.current[1] - d.start[1]);
            if dx.hypot(dy) < 0.5 {
                update_overlay(&mut s);
                return EventResult::redraw();
            }
            let Some(doc) = s.document.clone() else { return EventResult::none() };
            let center = match doc.lock().project().elements.get(&lot_model::ElementId(sheet)) {
                Some(lot_model::Element::PlanSheet(sh)) => sh.views.get(d.index).map(|v| v.center),
                _ => None,
            };
            let Some(c) = center else { return EventResult::none() };
            // Auf ganze Millimeter, damit Blätter ordentlich bleiben.
            let command = Command::MovePlacedView(sheet, d.index as u32, (c.x + dx).round(), (c.y + dy).round());
            return run_editor(&mut s, &command);
        }
        drop(s);
        self.edit(0.0, 0.0, false, false, |editor, doc, _| editor.pointer_up(doc))
    }

    pub fn key(&self, key: KeyInput) -> EventResult {
        {
            let mut s = self.lock();
            match (s.shown, &key) {
                (ViewRef::FloorPlan(_), _) => {}
                (ViewRef::Section(_), KeyInput::Delete) => return run_editor(&mut s, &Command::DeleteSelection),
                (ViewRef::Section(_), KeyInput::Character { text }) if text.eq_ignore_ascii_case("h") => {
                    return run_editor(&mut s, &Command::TempHide);
                }
                (ViewRef::Section(_), KeyInput::Character { text }) if text.eq_ignore_ascii_case("i") => {
                    return run_editor(&mut s, &Command::TempIsolate);
                }
                (ViewRef::Section(_), KeyInput::Escape) => {
                    if let Some(e) = s.editor.as_mut() {
                        e.selection.clear();
                    }
                    update_overlay(&mut s);
                    return EventResult::redraw();
                }
                (ViewRef::Sheet(sheet), KeyInput::Delete) => {
                    let Some(i) = s.placed.take() else { return EventResult::none() };
                    return run_editor(&mut s, &Command::RemovePlacedView(sheet, i as u32));
                }
                (ViewRef::Sheet(_), KeyInput::Escape) => {
                    s.placed = None;
                    s.placed_drag = None;
                    update_overlay(&mut s);
                    return EventResult::redraw();
                }
                _ => return EventResult::none(),
            }
        }
        let key = match key {
            KeyInput::Escape => Key::Escape,
            KeyInput::Enter => Key::Enter,
            KeyInput::Delete => Key::Delete,
            KeyInput::Tab => Key::Tab,
            KeyInput::Arrow { dx, dy, large } => Key::Arrow { dx, dy, large },
            KeyInput::Character { text } => match text.chars().next() {
                Some(c) => Key::Char(c),
                None => return EventResult::redraw(),
            },
        };
        self.edit(0.0, 0.0, false, false, |editor, doc, _| editor.key(doc, key))
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
        let mark = if s.print_view { " ✓" } else { "" };
        list.push(item("view:print", &format!("Druckbild{mark}"), "⌥⌘P", "Ansicht"));
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
            return self.set_plan_scale(scale);
        }
        if id == "view:print" {
            let on = !self.lock().print_view;
            self.set_print_view(on);
            return EventResult::redraw();
        }
        let Some(command) = Command::parse(&id) else {
            return EventResult { error: Some(format!("Unbekannter Befehl: {id}")), ..EventResult::redraw() };
        };
        let mut s = self.lock();
        // Werkzeuge zeichnen nur im Grundriss: aus anderen Ansichten geht es dorthin.
        if command.id().ends_with("Tool")
            && !matches!(s.shown, ViewRef::FloorPlan(_))
            && let Some(storey) = s.editor.as_ref().map(|e| e.storey.0)
        {
            show(&mut s, ViewRef::FloorPlan(storey), true);
        }
        run_editor(&mut s, &command)
    }

    /// Art der gezeigten Ansicht, ohne den übrigen Zustand: für jedes Zeigerereignis.
    pub fn kind(&self) -> ViewKind {
        view_kind(self.lock().shown)
    }

    pub fn state(&self) -> ViewState {
        let s = self.lock();
        let view = view_kind(s.shown);
        let (title, has_below) = match &s.document {
            Some(doc) => {
                let doc = doc.lock();
                let below = match s.shown {
                    ViewRef::FloorPlan(id) => {
                        nav::storey_step(doc.project(), lot_model::ElementId(id), false).is_some()
                    }
                    _ => false,
                };
                (s.shown.title(doc.project()), below)
            }
            None => (String::new(), false),
        };
        let faded_below = has_below && s.look.as_ref().is_some_and(|(l, _, _)| l.faded_below.value);
        let temporary = match s.editor.as_ref().map(|e| &e.temp) {
            Some(t) if t.isolated.is_some() => "isoliert · I zeigt wieder alles".to_string(),
            Some(t) if !t.hidden.is_empty() => format!("{} ausgeblendet · H zeigt wieder alles", t.hidden.len()),
            _ => String::new(),
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
                        Tool::AlignUnderlay => "alignUnderlay",
                        Tool::Move => "move",
                        Tool::Copy => "copy",
                        Tool::Rotate => "rotate",
                        Tool::Mirror => "mirror",
                        Tool::Array => "array",
                        Tool::Offset => "offset",
                        Tool::Align => "align",
                        Tool::Trim => "trim",
                        Tool::Split => "split",
                        Tool::WallRect => "wallRect",
                        Tool::PickDirection => "direction",
                        Tool::Section => "section",
                        Tool::Elevation => "elevation",
                        Tool::Stair => "stair",
                        Tool::RoofEdge => "roofEdge",
                        Tool::Drain => "drain",
                        Tool::Overflow => "overflow",
                    }
                    .to_string(),
                )
            }
            _ => (String::new(), String::new(), Vec::new(), String::new(), "select".to_string()),
        };
        ViewState {
            view,
            shown: s.shown.id(),
            title,
            can_go_back: s.history.can_go_back(),
            can_go_forward: s.history.can_go_forward(),
            has_below,
            faded_below,
            temporary,
            print_view: s.print_view,
            tool,
            storey,
            scale: s.plan_scale,
            wall_type,
            selection,
            hint,
        }
    }

    /// Der gewählte Anschluss mit Erklärung und Ausnahmen; `None`, wenn keiner gewählt ist.
    pub fn junction(&self) -> Option<JunctionInfo> {
        let s = self.lock();
        let (Some(editor), Some(doc)) = (&s.editor, &s.document) else { return None };
        let doc = doc.lock();
        let project = doc.project();
        let j = editor.selected_junction(project)?;
        Some(junction_info(project, editor.storey, &j))
    }

    /// Zeichnet ein Bild. `false`, wenn das Fenster gerade verdeckt ist; dann erneut versuchen.
    pub fn render(&self) -> Result<bool, LotError> {
        self.lock().renderer.render().map_err(LotError::from_display)
    }
}

impl Viewport {
    /// Klick im Schnitt: wählt das Bauteil, das man dort sieht; mit ⇧ dazu oder weg.
    fn section_click(&self, x: f32, y: f32, shift: bool) -> EventResult {
        let mut s = self.lock();
        let ViewRef::Section(id) = s.shown else { return EventResult::none() };
        let p = s.renderer.plan_point(x, y);
        let m = s.plan_scale / 1000.0;
        let (Some(geometry), Some(doc)) = (s.shown_geometry.clone(), s.document.clone()) else {
            return EventResult::none();
        };
        let picked = match doc.lock().project().elements.get(&lot_model::ElementId(id)) {
            Some(lot_model::Element::SectionView(v)) => {
                lot_plan::section::section_pick(&geometry, v, [p[0] * m, p[1] * m])
            }
            _ => None,
        };
        if let Some(editor) = s.editor.as_mut() {
            editor.junction = None;
            match (picked, shift) {
                (Some(id), true) => {
                    if let Some(i) = editor.selection.iter().position(|x| *x == id) {
                        editor.selection.remove(i);
                    } else {
                        editor.selection.push(id);
                    }
                }
                (Some(id), false) => editor.selection = vec![id],
                (None, true) => {}
                (None, false) => editor.selection.clear(),
            }
        }
        update_overlay(&mut s);
        EventResult::redraw()
    }

    /// Klick auf dem Blatt: wählt die Ansicht darunter und beginnt das Ziehen; Doppelklick öffnet sie.
    fn sheet_down(&self, x: f32, y: f32, clicks: u32) -> EventResult {
        let mut s = self.lock();
        let ViewRef::Sheet(sheet) = s.shown else { return EventResult::none() };
        let p = s.renderer.plan_point(x, y);
        let hit = s
            .extents
            .iter()
            .rev()
            .find(|(_, lo, hi)| p[0] >= lo.x && p[0] <= hi.x && p[1] >= lo.y && p[1] <= hi.y)
            .map(|(i, _, _)| *i);
        s.placed = hit;
        s.placed_drag = hit.map(|index| PlacedDrag { index, start: p, current: p });
        let mut result = EventResult::redraw();
        if clicks >= 2
            && let Some(i) = hit
            && let Some(doc) = s.document.clone()
            && let Some(lot_model::Element::PlanSheet(sh)) =
                doc.lock().project().elements.get(&lot_model::ElementId(sheet))
            && let Some(placed) = sh.views.get(i)
        {
            s.placed_drag = None;
            result.open = Some(ViewRef::of_content(placed.content).id());
        }
        update_overlay(&mut s);
        result
    }

    /// Gemeinsamer Weg für Zeiger- und Tastenereignisse im Plan.
    fn edit(
        &self,
        x: f32,
        y: f32,
        shift: bool,
        alt: bool,
        action: impl FnOnce(&mut Editor, &mut lot_model::Document, Input) -> Response,
    ) -> EventResult {
        let mut s = self.lock();
        if !matches!(s.shown, ViewRef::FloorPlan(_)) {
            return EventResult::none();
        }
        let (Some(doc), Some(mut editor)) = (s.document.clone(), s.editor.take()) else {
            return EventResult::none();
        };
        let paper = s.renderer.plan_point(x, y);
        let m_per_mm = s.plan_scale / 1000.0;
        let input = Input {
            point: Point2::new(paper[0] * m_per_mm, paper[1] * m_per_mm),
            tolerance: SNAP_POINTS * s.renderer.plan_mm_per_point() * m_per_mm,
            shift,
            alt,
        };
        let dragging_before = editor.preview(doc.lock().project()).is_some();
        let temp_before = editor.temp.clone();
        let response = action(&mut editor, &mut doc.lock(), input);
        let dragging = editor.preview(doc.lock().project()).is_some();
        let temp_changed = editor.temp != temp_before;
        s.editor = Some(editor);
        if response.model_changed || temp_changed {
            rebuild(&mut s, true);
        } else if dragging || dragging_before {
            rebuild(&mut s, false);
        } else if response.redraw {
            update_overlay(&mut s);
        }
        response.into()
    }
}

fn view_kind(view: ViewRef) -> ViewKind {
    match view {
        ViewRef::FloorPlan(_) => ViewKind::Plan,
        ViewRef::Model => ViewKind::Model,
        ViewRef::Section(_) => ViewKind::Section,
        ViewRef::SitePlan => ViewKind::SitePlan,
        ViewRef::Sheet(_) => ViewKind::Sheet,
        ViewRef::Schedule(_) => ViewKind::Schedule,
    }
}

fn item(id: &str, title: &str, shortcut: &str, group: &str) -> CommandItem {
    CommandItem { id: id.into(), title: title.into(), shortcut: shortcut.into(), group: group.into() }
}

/// Rechnet die gezeigte Ansicht neu, und nach einer Modelländerung (`model`)
/// auch 3D, falls die 3D-Ansicht zu sehen ist. Gibt die Zeit in ms zurück.
fn rebuild(s: &mut State, model: bool) -> f64 {
    let start = Instant::now();
    if model {
        s.findings = None;
        s.model_stale = true;
        s.geometry = None;
        if s.renderer.mode() == ViewMode::Model {
            build_model(s);
        }
    }
    let Some(doc) = s.document.clone() else { return 0.0 };
    if let Some(editor) = s.editor.as_mut() {
        editor.sync(doc.lock().project());
        editor.scale = s.plan_scale;
    }
    sync_shown(s, &doc);
    let needs_geometry = matches!(
        s.shown,
        ViewRef::Section(_) | ViewRef::SitePlan | ViewRef::Sheet(_) | ViewRef::Schedule(Schedule::Checks)
    );
    let geometry = if needs_geometry { ensure_geometry(s) } else { None };
    let doc = doc.lock();
    let standard = OfficeStandard::default_standard();
    let preview = s.editor.as_ref().and_then(|e| e.preview(doc.project()));
    let project: &Project = preview.as_ref().unwrap_or(doc.project());
    // Gezeigt werden die aktiven Varianten (2.04).
    let shown = project.variant_sets().next().is_some().then(|| project.variant_view());
    let project: &Project = shown.as_ref().unwrap_or(project);
    // Darstellung der Ansicht (ADR 0050); Grundrisse, Schnitte und der Lageplan zeichnen in ihrem Maßstab.
    let look = s.shown.content().map(|c| lot_plan::views::look(project, standard, c));
    if let Some(l) = &look {
        set_scale(s, l.scale.value);
    }
    s.look = None;
    s.shown_geometry = None;
    match (s.shown, look) {
        (ViewRef::FloorPlan(_), Some(look)) => floor_plan_objects(s, project, standard, look),
        (ViewRef::Model, _) => {}
        (view, look) => {
            let objects = view_objects(s, project, view, geometry.as_deref(), standard, look);
            s.renderer.set_plan(objects);
            s.renderer.set_underlays(&[]);
        }
    }
    drop(doc);
    update_overlay(s);
    start.elapsed().as_secs_f64() * 1000.0
}

/// Grundriss des Editor-Geschosses mit Unterlagen, Referenzmodellen, Hinweisen
/// und, falls gewünscht, dem Geschoss darunter blass.
fn floor_plan_objects(s: &mut State, project: &Project, standard: &OfficeStandard, look: lot_plan::views::Look) {
    let Some(editor) = &s.editor else { return };
    let storey = editor.storey;
    let temp = if s.print_view { Default::default() } else { editor.temp.clone() };
    let shown = lot_plan::views::apply(project, &look, &temp);
    let hidden = hidden_ids(project, &shown.project);
    let view = FloorPlanView { storey, cut_height: look.cut_height.value, scale: s.plan_scale };
    let viewport = PlanViewport { scale: s.plan_scale, origin: PaperPoint::new(0.0, 0.0) };
    let mut objects = Vec::new();
    // Geschoss darunter blass, ganz unten (ADR 0049); nicht im Druckbild.
    if look.faded_below.value
        && !s.print_view
        && let Some(below) = nav::storey_step(project, storey, false)
    {
        objects.extend(lot_plan::floor_plan::faded_storey(&shown.project, below, standard, viewport));
    }
    let print = s.print_view;
    let project = &shown.project;
    // Unterlagen aus DXF und DWG als graue Linien unter dem Plan, ihre Endpunkte zum Fangen.
    let mut points = Vec::new();
    for (_, u) in project.underlays_on(storey).filter(|(_, u)| lot_io::cad::is_cad(&u.source) && !print) {
        let Some(v) = s.vectors.get(&u.source) else { continue };
        objects.extend(lot_io::cad::underlay_objects(u, v, viewport));
        points.extend(lot_io::cad::underlay_points(u, v));
    }
    // Referenzmodelle, geschnitten in der Schnittebene.
    let z = project.storey(storey).map_or(0.0, |st| st.elevation) + look.cut_height.value;
    if !print {
        let parts = reference_parts(project, &s.references);
        lot_plan::reference::draw_reference_cut(&mut objects, &parts, z, viewport);
    }
    if look.presentation {
        objects.extend(lot_plan::presentation::presentation_plan(project, storey, s.plan_scale, standard, viewport));
    } else {
        objects.extend(lot_plan::floor_plan::floor_plan_with(project, view, standard, viewport, &shown.paint));
    }
    if s.show_findings && !print {
        if s.findings.is_none()
            && let Ok(geometry) = lot_geom::build(project)
        {
            s.findings = Some(lot_rules::check_project(project, &geometry));
        }
        if let Some(report) = &s.findings {
            lot_plan::findings::draw_findings(&mut objects, report, storey, viewport);
        }
    }
    s.renderer.set_plan(objects);
    let quads = if print { Vec::new() } else { underlay_quads(project, storey, &s.images, viewport) };
    s.renderer.set_underlays(&quads);
    if let Some(editor) = s.editor.as_mut() {
        editor.underlay_points = points;
        editor.hidden = hidden;
    }
    s.look = Some((look, shown.counts, shown.hidden_by_rules));
}

/// Bauteile des Projekts, die die Ansicht nicht zeigt; sie lassen sich nicht anklicken.
fn hidden_ids(project: &Project, shown: &Project) -> std::collections::BTreeSet<lot_model::ElementId> {
    project
        .elements
        .iter()
        .filter(|(id, e)| lot_model::Category::of(e).is_some() && !shown.elements.contains_key(id))
        .map(|(id, _)| *id)
        .collect()
}

/// Maßstab der gezeigten Ansicht; der Zoom bleibt auf dem Bildschirm gleich groß.
fn set_scale(s: &mut State, scale: f64) {
    let old = s.plan_scale;
    if (old - scale).abs() < 1e-9 || scale <= 0.0 {
        return;
    }
    s.plan_scale = scale;
    let mut camera = s.renderer.plan_camera();
    camera.center = [camera.center[0] * old / scale, camera.center[1] * old / scale];
    camera.px_per_mm *= scale / old;
    s.renderer.set_plan_camera(camera);
}

/// Schnitt, Lageplan, Planblatt oder Liste als Planobjekte.
fn view_objects(
    s: &mut State,
    project: &Project,
    view: ViewRef,
    geometry: Option<&lot_geom::ModelGeometry>,
    standard: &OfficeStandard,
    look: Option<lot_plan::views::Look>,
) -> Vec<PlanObject> {
    s.extents.clear();
    let origin = PaperPoint::new(0.0, 0.0);
    // Schnitt und Lageplan, wie die Ansicht sie zeigt (ADR 0050).
    if let (Some(look), Some(g), ViewRef::Section(_) | ViewRef::SitePlan) = (look, geometry, view) {
        let temp = match (&s.editor, s.print_view) {
            (Some(e), false) => e.temp.clone(),
            _ => Default::default(),
        };
        let shown = lot_plan::views::apply(project, &look, &temp);
        let visible = Arc::new(lot_plan::layout::visible_geometry(g, project, &shown.project));
        let viewport = PlanViewport { scale: s.plan_scale, origin };
        let objects = match view {
            ViewRef::Section(id) => match shown.project.elements.get(&lot_model::ElementId(id)) {
                Some(lot_model::Element::SectionView(v)) => {
                    lot_plan::section::section_with(&shown.project, &visible, v, standard, viewport, &shown.paint)
                }
                _ => Vec::new(),
            },
            _ => lot_plan::site_plan::site_plan(&shown.project, &visible, standard, viewport),
        };
        if let Some(editor) = s.editor.as_mut() {
            editor.hidden = hidden_ids(project, &shown.project);
        }
        s.shown_geometry = Some(visible);
        s.look = Some((look, shown.counts, shown.hidden_by_rules));
        return objects;
    }
    match view {
        ViewRef::Section(_) | ViewRef::SitePlan => Vec::new(),
        ViewRef::Sheet(id) => match (project.elements.get(&lot_model::ElementId(id)), geometry) {
            (Some(lot_model::Element::PlanSheet(sheet)), Some(g)) => {
                s.extents = lot_plan::layout::placed_extents(project, g, sheet, standard);
                pages(vec![lot_plan::layout::plan_sheet(project, g, sheet, standard)])
            }
            _ => Vec::new(),
        },
        ViewRef::Schedule(Schedule::Areas) => {
            pages(lot_plan::area_table::area_sheets(&project.name, &lot_rules::area_report(project)))
        }
        ViewRef::Schedule(Schedule::Openings) => {
            let (rows, types) = lot_rules::opening_schedule(project);
            pages(lot_plan::table::schedule_sheets(&project.name, &rows, &types))
        }
        ViewRef::Schedule(Schedule::Checks) => geometry
            .map(|g| pages(lot_plan::table::report_sheets(&project.name, &lot_rules::check_project(project, g))))
            .unwrap_or_default(),
        ViewRef::FloorPlan(_) | ViewRef::Model => Vec::new(),
    }
}

/// Seiten untereinander, jede mit ihrem Papierrand in Grau.
fn pages(sheets: Vec<lot_plan::Sheet>) -> Vec<PlanObject> {
    let mut out = Vec::new();
    let mut y = 0.0;
    for sheet in sheets {
        let dy = y;
        out.push(PlanObject::Line(lot_plan::Line {
            points: lot_plan::rect_points(0.0, dy, sheet.width_mm, sheet.height_mm),
            closed: true,
            pen: lot_plan::Pen::mm(0.25),
            style: lot_plan::LineStyle::Solid,
            color: lot_plan::Color::gray(0.6),
        }));
        out.extend(sheet.objects.into_iter().map(|mut o| {
            o.translate(0.0, dy);
            o
        }));
        y -= sheet.height_mm + PAGE_GAP;
    }
    out
}

/// Gerechnetes Modell der gezeigten Varianten, einmal je Modellstand.
fn ensure_geometry(s: &mut State) -> Option<Arc<lot_geom::ModelGeometry>> {
    if s.geometry.is_none() {
        let doc = s.document.clone()?;
        let built = lot_geom::build(&doc.lock().project().variant_view()).ok()?;
        s.geometry = Some(Arc::new(built));
    }
    s.geometry.clone()
}

/// Hält die gezeigte Ansicht stimmig mit dem Modell und dem Editor: Wechselt
/// der Editor das Geschoss (Menü, neues Geschoss), folgt der Grundriss; gibt
/// es die Ansicht nicht mehr (Löschen, Widerrufen), geht es zum Grundriss.
fn sync_shown(s: &mut State, doc: &LotDocument) {
    let guard = doc.lock();
    let project = guard.project();
    s.history.retain(project);
    let storey = s.editor.as_ref().map_or(0, |e| e.storey.0);
    let target = match s.shown {
        ViewRef::FloorPlan(x) if x != storey => Some(ViewRef::FloorPlan(storey)),
        v if !v.exists(project) => Some(ViewRef::FloorPlan(storey)),
        _ => None,
    };
    if let ViewRef::Sheet(id) = s.shown
        && let Some(lot_model::Element::PlanSheet(sheet)) = project.elements.get(&lot_model::ElementId(id))
        && s.placed.is_some_and(|i| i >= sheet.views.len())
    {
        s.placed = None;
    }
    drop(guard);
    if let Some(t) = target {
        let from = place(s);
        s.history.leave(from, t);
        s.shown = t;
        s.placed = None;
        if s.renderer.mode() == ViewMode::Model {
            s.renderer.set_mode(ViewMode::Plan);
        }
    }
}

/// Alle Grundrisse teilen eine Kamera; die übrigen Ansichten haben je ihre.
fn camera_key(view: ViewRef) -> ViewRef {
    match view {
        ViewRef::FloorPlan(_) => ViewRef::FloorPlan(0),
        v => v,
    }
}

/// Wo die Ansicht gerade steht, für den Verlauf.
fn place(s: &State) -> Place {
    let camera = (s.shown != ViewRef::Model).then(|| {
        let c = s.renderer.plan_camera();
        [c.center[0], c.center[1], c.px_per_mm]
    });
    Place { view: s.shown, camera }
}

/// Zeigt eine Ansicht; mit `record` merkt sich der Verlauf die bisherige.
/// `false`, wenn es die Ansicht nicht gibt.
fn show(s: &mut State, view: ViewRef, record: bool) -> bool {
    let Some(doc) = s.document.clone() else { return false };
    if !view.exists(doc.lock().project()) {
        return false;
    }
    if record {
        let from = place(s);
        s.history.leave(from, view);
    }
    if s.shown != ViewRef::Model {
        let camera = s.renderer.plan_camera();
        s.cameras.insert(camera_key(s.shown), camera);
    }
    let same_camera = camera_key(s.shown) == camera_key(view);
    // H und I gelten je Ansicht.
    if let Some(editor) = s.editor.as_mut() {
        let temp = std::mem::take(&mut editor.temp);
        let old = s.shown;
        s.temps.insert(old, temp);
        if let Some(editor) = s.editor.as_mut() {
            editor.temp = s.temps.remove(&view).unwrap_or_default();
        }
    }
    s.shown = view;
    s.placed = None;
    s.placed_drag = None;
    if let ViewRef::FloorPlan(storey) = view
        && let Some(mut editor) = s.editor.take()
    {
        if editor.storey.0 != storey {
            editor.run(&mut doc.lock(), &Command::SetStorey(storey));
        }
        s.editor = Some(editor);
    }
    s.renderer.set_mode(if view == ViewRef::Model { ViewMode::Model } else { ViewMode::Plan });
    if view == ViewRef::Model && s.model_stale {
        build_model(s);
    }
    rebuild(s, false);
    if view != ViewRef::Model && !same_camera {
        match s.cameras.get(&camera_key(view)) {
            Some(c) => s.renderer.set_plan_camera(*c),
            None => s.renderer.plan_zoom_to_fit(),
        }
        update_overlay(s);
    }
    true
}

/// Geht an einen Ort aus dem Verlauf: Ansicht und Kamera.
fn restore(s: &mut State, place: Place) {
    if !show(s, place.view, false) {
        return;
    }
    if let Some([x, y, px]) = place.camera {
        s.renderer.set_plan_camera(PlanCamera { center: [x, y], px_per_mm: px });
        update_overlay(s);
    }
}

/// Führt einen Befehl im Editor aus und rechnet die Ansicht neu.
fn run_editor(s: &mut State, command: &Command) -> EventResult {
    let (Some(doc), Some(mut editor)) = (s.document.clone(), s.editor.take()) else {
        return EventResult::redraw();
    };
    let response = editor.run(&mut doc.lock(), command);
    s.editor = Some(editor);
    // Wechsel des Geschosses oder Werkzeugs braucht einen neuen Plan, Modelländerungen auch 3D.
    rebuild(s, response.model_changed);
    response.into()
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

/// Was über der gezeigten Ansicht liegt: Hinweise des Editors im Grundriss,
/// die Auswahl im Schnitt, die gewählte Ansicht auf dem Blatt.
fn update_overlay(s: &mut State) {
    match s.shown {
        ViewRef::FloorPlan(_) => plan_overlay(s),
        ViewRef::Section(id) => section_overlay(s, id),
        ViewRef::Sheet(_) => sheet_overlay(s),
        _ => s.renderer.set_overlay(Vec::new()),
    }
}

/// Die gemeinsame Auswahl im Schnitt, blau wie im Grundriss.
fn section_overlay(s: &mut State, id: u64) {
    let mut items = Vec::new();
    if let (Some(editor), Some(doc), Some(geometry)) = (&s.editor, &s.document, &s.shown_geometry)
        && !editor.selection.is_empty()
        && let Some(lot_model::Element::SectionView(v)) = doc.lock().project().elements.get(&lot_model::ElementId(id))
    {
        let f = 1000.0 / s.plan_scale;
        for ring in lot_plan::section::section_outlines(geometry, v, &editor.selection) {
            items.push(OverlayItem::Polygon {
                points: ring.into_iter().map(|p| [p[0] * f, p[1] * f]).collect(),
                fill: linear([0.10, 0.45, 0.95], 0.18),
                stroke: linear([0.10, 0.45, 0.95], 1.0),
                width_px: 2.0,
            });
        }
    }
    s.renderer.set_overlay(items);
}

/// Die gewählte Ansicht auf dem Blatt; beim Ziehen gestrichelt, wo sie hinkommt.
fn sheet_overlay(s: &mut State) {
    let blue = linear([0.10, 0.45, 0.95], 1.0);
    let mut items = Vec::new();
    let rect = |lo: PaperPoint, hi: PaperPoint, dx: f64, dy: f64| {
        vec![[lo.x + dx, lo.y + dy], [hi.x + dx, lo.y + dy], [hi.x + dx, hi.y + dy], [lo.x + dx, hi.y + dy]]
    };
    if let Some(i) = s.placed
        && let Some((_, lo, hi)) = s.extents.iter().find(|(j, _, _)| *j == i)
    {
        items.push(OverlayItem::Polygon {
            points: rect(*lo, *hi, 0.0, 0.0),
            fill: linear([0.10, 0.45, 0.95], 0.06),
            stroke: blue,
            width_px: 2.0,
        });
        if let Some(d) = s.placed_drag {
            let (dx, dy) = (d.current[0] - d.start[0], d.current[1] - d.start[1]);
            if dx.hypot(dy) >= 0.5 {
                items.push(OverlayItem::Path {
                    points: rect(*lo, *hi, dx, dy),
                    closed: true,
                    color: blue,
                    width_px: 1.5,
                    dash_px: Some((6.0, 4.0)),
                });
            }
        }
    }
    s.renderer.set_overlay(items);
}

/// Hinweise des Editors über dem Plan, in Blattmillimetern.
fn plan_overlay(s: &mut State) {
    let (Some(editor), Some(doc)) = (&s.editor, &s.document) else { return };
    let overlay: Overlay = editor.overlay(doc.lock().project());
    let mm = 1000.0 / s.plan_scale;
    let paper = |p: [f64; 2]| [p[0] * mm, p[1] * mm];
    let blue = linear([0.10, 0.45, 0.95], 1.0);
    let mut items = Vec::new();
    // Vorhervorhebung unter der Auswahl, dünn und heller.
    for outline in overlay.hovered {
        items.push(OverlayItem::Polygon {
            points: outline.into_iter().map(paper).collect(),
            fill: linear([0.10, 0.45, 0.95], 0.06),
            stroke: linear([0.35, 0.62, 1.0], 1.0),
            width_px: 1.5,
        });
    }
    // Wo die Auswahl nach dem laufenden Befehl läge, gestrichelt.
    for ghost in overlay.ghosts {
        items.push(OverlayItem::Path {
            points: ghost.into_iter().map(paper).collect(),
            closed: true,
            color: blue,
            width_px: 1.2,
            dash_px: Some((5.0, 3.0)),
        });
    }
    for (a, b) in overlay.guides {
        items.push(OverlayItem::Path {
            points: vec![paper([a.x, a.y]), paper([b.x, b.y])],
            closed: false,
            color: linear([0.95, 0.45, 0.05], 1.0),
            width_px: 1.0,
            dash_px: Some((4.0, 3.0)),
        });
    }
    // Bezugspunkte (Q) mit ihrer Nummer.
    for (p, n) in overlay.references {
        let at = paper([p.x, p.y]);
        items.push(OverlayItem::Square { center: at, size_px: 7.0, color: linear([0.95, 0.45, 0.05], 1.0) });
        items.push(OverlayItem::Label {
            at,
            offset_px: [6.0, 4.0],
            text: n,
            height_px: 8.0,
            color: linear([1.0, 1.0, 1.0], 1.0),
            background: linear([0.95, 0.45, 0.05], 1.0),
        });
    }
    // Ansichtslinien und der Schnitt beim Zeichnen: gestrichelt, Pfeil in Blickrichtung.
    for [a, b, tip] in overlay.view_lines {
        let (pa, pb, pt) = (paper([a.x, a.y]), paper([b.x, b.y]), paper([tip.x, tip.y]));
        items.push(OverlayItem::Path {
            points: vec![pa, pb],
            closed: false,
            color: blue,
            width_px: 1.5,
            dash_px: Some((8.0, 4.0)),
        });
        let mid = [(pa[0] + pb[0]) / 2.0, (pa[1] + pb[1]) / 2.0];
        items.push(OverlayItem::Path {
            points: vec![mid, pt],
            closed: false,
            color: blue,
            width_px: 1.5,
            dash_px: None,
        });
        items.push(OverlayItem::Ring { center: pt, radius_px: 3.0, width_px: 2.0, color: blue });
    }
    for m in overlay.marks {
        items.push(OverlayItem::Ring {
            center: paper([m.x, m.y]),
            radius_px: 4.0,
            width_px: 2.0,
            color: linear([0.95, 0.45, 0.05], 1.0),
        });
    }
    if let Some((points, crossing)) = overlay.frame {
        let points: Vec<[f64; 2]> = points.into_iter().map(|p| paper([p.x, p.y])).collect();
        // Wie in Revit: durchgezogen wählt ganz innen, gestrichelt auch Angeschnittenes.
        let tint = if crossing { [0.15, 0.65, 0.3] } else { [0.10, 0.45, 0.95] };
        items.push(OverlayItem::Polygon {
            points: points.clone(),
            fill: linear(tint, 0.08),
            stroke: [0.0; 4],
            width_px: 0.0,
        });
        items.push(OverlayItem::Path {
            points,
            closed: true,
            color: linear(tint, 1.0),
            width_px: 1.2,
            dash_px: crossing.then_some((6.0, 4.0)),
        });
    }
    for outline in overlay.selected {
        items.push(OverlayItem::Polygon {
            points: outline.into_iter().map(paper).collect(),
            fill: linear([0.10, 0.45, 0.95], 0.12),
            stroke: blue,
            width_px: 2.0,
        });
    }
    // Gewählter Anschluss: ein Kreis um den Knoten.
    if let Some((p, r)) = overlay.junction {
        let points: Vec<[f64; 2]> = (0..48)
            .map(|i| {
                let a = i as f64 / 48.0 * std::f64::consts::TAU;
                paper([p.x + r * a.cos(), p.y + r * a.sin()])
            })
            .collect();
        items.push(OverlayItem::Polygon {
            points,
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
    // Kanten gewählter Decken: blau, wo sie einer Wand folgen, orange, wo sie frei sind (ADR 0051).
    for (a, b, followed) in overlay.edge_marks {
        let color = if followed { blue } else { linear([0.95, 0.45, 0.05], 1.0) };
        items.push(OverlayItem::Path {
            points: vec![paper([a.x, a.y]), paper([b.x, b.y])],
            closed: false,
            color,
            width_px: 4.0,
            dash_px: if followed { None } else { Some((6.0, 4.0)) },
        });
    }
    if let Some((at, text)) = overlay.label {
        match overlay.verdict {
            // Ampel nach DIN 18065 am Zeiger.
            Some(ok) => items.push(OverlayItem::Label {
                at: paper([at.x, at.y]),
                offset_px: [14.0, 10.0],
                text,
                height_px: 10.0,
                color: linear([1.0, 1.0, 1.0], 1.0),
                background: if ok { linear([0.15, 0.6, 0.25], 1.0) } else { linear([0.85, 0.2, 0.15], 1.0) },
            }),
            None => items.push(OverlayItem::Text {
                at: paper([at.x, at.y]),
                text,
                height_px: 12.0,
                color: linear([0.05, 0.25, 0.6], 1.0),
            }),
        }
    }
    // Temporäre Maße: dünne Linie mit Endstrichen, die Zahl zum Anklicken auf einem Feld.
    let dim_color = linear([0.10, 0.35, 0.80], 1.0);
    for (a, b, text, editing) in overlay.temp_dims {
        let (pa, pb) = (paper([a.x, a.y]), paper([b.x, b.y]));
        items.push(OverlayItem::Path {
            points: vec![pa, pb],
            closed: false,
            color: dim_color,
            width_px: 1.0,
            dash_px: None,
        });
        let len = ((pb[0] - pa[0]).powi(2) + (pb[1] - pa[1]).powi(2)).sqrt().max(1e-9);
        let (nx, ny) = (-(pb[1] - pa[1]) / len, (pb[0] - pa[0]) / len);
        for p in [pa, pb] {
            let k = 1.2;
            items.push(OverlayItem::Path {
                points: vec![[p[0] - nx * k, p[1] - ny * k], [p[0] + nx * k, p[1] + ny * k]],
                closed: false,
                color: dim_color,
                width_px: 1.0,
                dash_px: None,
            });
        }
        let middle = [(pa[0] + pb[0]) / 2.0, (pa[1] + pb[1]) / 2.0];
        items.push(OverlayItem::Label {
            at: middle,
            offset_px: [-12.0, -5.0],
            text,
            height_px: 10.0,
            color: if editing { linear([1.0, 1.0, 1.0], 1.0) } else { dim_color },
            background: if editing { dim_color } else { linear([1.0, 1.0, 1.0], 0.95) },
        });
    }
    if let Some((at, text)) = overlay.tip {
        items.push(OverlayItem::Label {
            at: paper([at.x, at.y]),
            offset_px: [14.0, -26.0],
            text,
            height_px: 9.0,
            color: linear([0.12, 0.12, 0.14], 1.0),
            background: linear([1.0, 1.0, 1.0], 0.92),
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

/// Anschluss in Worten: welche Wand wie anschließt und welche Schicht welche schneidet.
fn junction_info(project: &Project, storey: lot_model::ElementId, j: &lot_geom::Junction) -> JunctionInfo {
    use lot_geom::{ArmRole, Cause};
    use lot_model::JoinRule;
    let standard = OfficeStandard::default_standard();
    let title = if j.arms.iter().any(|a| a.end.is_none()) {
        "T-Stoß".to_string()
    } else if j.arms.len() == 2 {
        "Ecke".to_string()
    } else {
        format!("Knoten aus {} Wänden", j.arms.len())
    };
    let mut lines = Vec::new();
    for c in lot_geom::contacts(project, storey, j) {
        let (a, b) = (standard.material(&c.first.material).name, standard.material(&c.second.material).name);
        let line = if c.cuts { format!("{a} schneidet {b}") } else { format!("{a} stößt an {b}") };
        if !lines.contains(&line) {
            lines.push(line);
        }
    }
    let arms = j
        .arms
        .iter()
        .map(|a| {
            let role = match a.role {
                ArmRole::Mitre => "auf Gehrung",
                ArmRole::Through => "läuft durch",
                ArmRole::Butts => "stößt an, Schichten greifen ein",
                ArmRole::Separate => "stößt stumpf an, nicht verbunden",
            };
            let cause = match a.cause {
                Some(Cause::Exception) => " (Ausnahme)",
                Some(Cause::Phase) => " (Neubau an Bestand)",
                Some(Cause::Height) => " (höhere Wand)",
                None => "",
            };
            let rule = a.end.and_then(|e| project.wall(a.wall).map(|w| w.joins[e.index()]));
            JunctionArmInfo {
                wall: a.wall.0,
                label: lot_edit::select::label(project, a.wall),
                role: format!("{role}{cause}"),
                rule: match rule {
                    Some(JoinRule::Through) => 1,
                    Some(JoinRule::Butt) => 2,
                    _ => 0,
                },
                commands: match a.end {
                    Some(e) => [JoinRule::Auto, JoinRule::Through, JoinRule::Butt]
                        .into_iter()
                        .map(|r| Command::SetJoinRule(a.wall.0, e, r).id())
                        .collect(),
                    None => Vec::new(),
                },
            }
        })
        .collect();
    JunctionInfo { title, lines, arms }
}

fn linear(srgb: [f32; 3], alpha: f32) -> [f32; 4] {
    let f = |c: f32| if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) };
    [f(srgb[0]), f(srgb[1]), f(srgb[2]), alpha]
}
