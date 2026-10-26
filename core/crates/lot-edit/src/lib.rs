//! Bearbeiten im Grundriss: Werkzeuge, Fangen, Auswahl und Befehle.
//!
//! Plattformunabhängig und ohne Fenster testbar. Die App reicht Zeiger- und
//! Tastenereignisse in Modellkoordinaten weiter; jede Änderung am Modell ist
//! eine Operation (CLAUDE.md, Regel 7), Verschieben mit angeschlossenen
//! Wänden eine Gruppe, die ein Undo zurücknimmt.

mod command;
pub mod entry;
pub mod geometry;
pub mod modify;
pub mod select;
pub mod snap;
pub mod tempdims;
pub mod transform;

pub use command::{Command, CommandInfo};
use geometry::{End, as_elements, column_outline, end_point, footprint, moved_elements, moved_endpoint, outline};
use lot_model::{
    Annotation, AnnotationShape, AnnotationView, Column, ColumnProfile, Document, Element, ElementId, GridLine,
    Handing, ModelError, Opening, OpeningKind, Operation, Point2, Project, Space, Storey, Top, Wall, WallAlignment,
    WallSide,
};
pub use select::Candidate;
pub use snap::{Snap, SnapKind};
pub use transform::Transform;

/// Zeigerereignis in Modellkoordinaten.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Input {
    pub point: Point2,
    /// Fangradius in Metern (aus Pixeln und Zoom).
    pub tolerance: f64,
    /// Umschalttaste: beim Zeichnen nur rechte Winkel, bei der Auswahl hinzufügen.
    pub shift: bool,
    /// Wahltaste: Rahmen wird Lasso, Ziehen der Auswahl kopiert.
    pub alt: bool,
}

/// Tasten, die der Editor versteht.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Key {
    Escape,
    Enter,
    Delete,
    /// Nächster Kandidat unter dem Zeiger (ADR 0043).
    Tab,
    /// Pfeiltaste: Auswahl um 1 cm schieben, mit `large` um 10 cm.
    Arrow {
        dx: i8,
        dy: i8,
        large: bool,
    },
    Char(char),
}

/// Was die App nach einem Ereignis tun muss.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Response {
    pub redraw: bool,
    pub model_changed: bool,
    /// Meldung für den Benutzer, wenn eine Änderung nicht ging.
    pub error: Option<String>,
}

impl Response {
    fn redraw() -> Self {
        Self { redraw: true, ..Default::default() }
    }

    fn changed() -> Self {
        Self { redraw: true, model_changed: true, error: None }
    }

    fn from_result(result: Result<(), ModelError>) -> Self {
        match result {
            Ok(()) => Self::changed(),
            Err(e) => Self { redraw: true, model_changed: false, error: Some(e.to_string()) },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Select,
    Wall,
    /// Bogenwand: Anfang, Ende, dann der Stich
    ArcWall,
    /// Achse: zwei Klicks
    Grid,
    /// Stütze: ein Klick
    Column,
    /// Raum: ein Klick in den Bereich
    Space,
    /// Tür: ein Klick auf die Wand, zur Seite, zu der sie aufschlägt
    Door,
    /// Fenster: ein Klick auf die Wand
    Window,
    /// Deckendurchbruch: zwei Ecken eines Rechtecks in der Decke des Geschosses
    SlabOpening,
    /// 2D-Linie: Punkte nacheinander, Doppelklick beendet
    Line,
    /// 2D-Text: ein Klick
    Text,
    /// Unterlage kalibrieren: zwei Punkte, dann der wahre Abstand
    CalibrateUnderlay,
    /// Unterlage ausrichten: zwei Punkte, die waagerecht oder senkrecht liegen sollen
    AlignUnderlay,
    /// Verschieben (MV): Basispunkt, dann Ziel klicken oder Abstand tippen.
    Move,
    /// Kopieren (CO): wie Verschieben, das Original bleibt.
    Copy,
    /// Drehen (RO): Winkel tippen oder Bezugs- und Zielrichtung klicken; Leertaste setzt den Drehpunkt.
    Rotate,
    /// Spiegeln (MM): zwei Punkte der Achse; als Kopie, mit ⇧ ohne.
    Mirror,
    /// Reihe (AR): Basispunkt und Abstand; „3x“ drei Stück im Abstand, „3/“ drei Stück bis zum Punkt.
    Array,
    /// Versetzen (OF): Wand oder Linie anklicken, Kopie zur Seite des Zeigers im getippten Abstand.
    Offset,
    /// Ausrichten (AL): Bezugslinie, dann die Fläche, die daran ausgerichtet wird.
    Align,
    /// Stutzen und Dehnen zur Ecke (TR): zwei Wände, die angeklickten Teile bleiben.
    Trim,
    /// Teilen (SL): Klick auf eine Wand teilt sie dort.
    Split,
}

#[derive(Debug, Clone, PartialEq)]
enum Drag {
    /// Gedrückt, aber noch kaum bewegt: Loslassen ist ein Klick auf `hit`;
    /// Ziehen verschiebt die Auswahl, wenn der Zeiger auf ihr liegt, sonst
    /// entsteht ein Rahmen, mit ⌥ ein Lasso (ADR 0043).
    Pending {
        start: Point2,
        hit: Option<Vec<ElementId>>,
        on_selection: bool,
        shift: bool,
        alt: bool,
        tolerance: f64,
    },
    /// Die ganze Auswahl verschieben, mit `copy` kopieren.
    Selection {
        start: Point2,
        current: Point2,
        copy: bool,
    },
    /// Rahmen: von links nach rechts ganz innen, von rechts nach links auch angeschnitten.
    Frame {
        start: Point2,
        current: Point2,
        shift: bool,
    },
    Lasso {
        points: Vec<Point2>,
        shift: bool,
    },
    Endpoint {
        wall: ElementId,
        end: End,
        current: Point2,
    },
}

/// Was der Renderer über dem Plan zeigt, in Modellkoordinaten.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Overlay {
    /// Umriss der Wand, die gerade gezeichnet wird.
    pub preview: Option<Vec<[f64; 2]>>,
    /// Umrisse der gewählten Elemente.
    pub selected: Vec<Vec<[f64; 2]>>,
    /// Griffe an den Enden gewählter Wände.
    pub handles: Vec<Point2>,
    pub snap: Option<Snap>,
    /// Länge der gezeichneten Wand und wo sie steht.
    pub label: Option<(Point2, String)>,
    /// Umrisse des Kandidaten unter dem Zeiger (Vorhervorhebung).
    pub hovered: Vec<Vec<[f64; 2]>>,
    /// Umrisse der Auswahl, wie sie nach dem laufenden Befehl läge (gestrichelt).
    pub ghosts: Vec<Vec<[f64; 2]>>,
    /// Hilfslinien des laufenden Befehls: Verschiebung, Drehung, Spiegelachse.
    pub guides: Vec<(Point2, Point2)>,
    /// Markierte Punkte: Basispunkt, Drehpunkt.
    pub marks: Vec<Point2>,
    /// Temporäre Maße am gewählten Bauteil: Anfang, Ende, Maßzahl, gerade in Bearbeitung.
    pub temp_dims: Vec<(Point2, Point2, String, bool)>,
    /// Rahmen oder Lasso; `true`, wenn er auch Angeschnittenes wählt.
    pub frame: Option<(Vec<Point2>, bool)>,
    /// Kurzer Text am Zeiger, etwa „2 von 4: Wandkette (3 Wände)“.
    pub tip: Option<(Point2, String)>,
}

pub struct Editor {
    pub storey: ElementId,
    /// Endpunkte der Linien in Unterlagen aus DXF oder DWG, in Modellmetern;
    /// gesetzt von der Ansicht, die die Dateien liest. Der Zeiger rastet daran ein.
    pub underlay_points: Vec<Point2>,
    pub tool: Tool,
    pub wall_type: Option<ElementId>,
    pub alignment: WallAlignment,
    pub selection: Vec<ElementId>,
    chain_start: Option<Point2>,
    /// Endpunkt einer Bogenwand, deren Stich noch fehlt.
    arc_end: Option<Point2>,
    /// Fangradius der letzten Zeigerbewegung, für die Vorschau von Türen und Fenstern.
    hover_tolerance: f64,
    drag: Option<Drag>,
    pointer: Option<Snap>,
    typed: String,
    /// Kandidaten unter dem Zeiger im Auswahlwerkzeug, der gezeigte und wo der Zeiger stand.
    hover: Vec<Candidate>,
    hover_index: usize,
    hover_at: Option<Point2>,
    /// Frühere Auswahlen, die letzte zuletzt (⌘←).
    history: Vec<Vec<ElementId>>,
    /// Getippte Buchstaben eines Kürzels, etwa „W“ vor „WA“.
    shortcut: String,
    /// Letzter Befehl über ein Kürzel, für ⏎ ohne laufende Aktion.
    last_command: Option<Command>,
    /// Drehpunkt für Drehen, anfangs die Mitte der Auswahl.
    pivot: Option<Point2>,
    /// Anzahl für Reihe: „3x“ oder „3/“.
    series: Option<entry::Entry>,
    /// Getippter Abstand für Versetzen.
    offset_distance: Option<f64>,
    /// Erste angeklickte Wand beim Stutzen.
    picked: Option<(ElementId, Point2)>,
    /// Bezugslinie beim Ausrichten.
    align_ref: Option<modify::Line>,
    /// Getippter Winkel, an dem die nächste Wand oder Linie entlangläuft.
    angle_lock: Option<f64>,
    /// Das temporäre Maß, dessen Zahl gerade getippt wird.
    dim_edit: Option<usize>,
    /// Zwischenablage: Stand des Projekts beim Kopieren und die kopierten Elemente.
    clipboard: Option<(Box<Project>, Vec<ElementId>)>,
}

impl Editor {
    pub fn new(project: &Project) -> Self {
        let storey =
            project.storeys().iter().find(|(_, s)| s.elevation >= 0.0).or(project.storeys().first()).map(|(id, _)| *id);
        Self {
            storey: storey.unwrap_or(ElementId(0)),
            underlay_points: Vec::new(),
            tool: Tool::Select,
            wall_type: project.wall_types().next().map(|(id, _)| id),
            alignment: WallAlignment::Center,
            selection: Vec::new(),
            chain_start: None,
            arc_end: None,
            hover_tolerance: 0.1,
            drag: None,
            pointer: None,
            typed: String::new(),
            hover: Vec::new(),
            hover_index: 0,
            hover_at: None,
            history: Vec::new(),
            shortcut: String::new(),
            last_command: None,
            pivot: None,
            series: None,
            offset_distance: None,
            picked: None,
            align_ref: None,
            angle_lock: None,
            dim_edit: None,
            clipboard: None,
        }
    }

    /// Passt den Zustand an ein geändertes Modell an (Undo, Laden): Verschwundene
    /// Elemente fallen aus der Auswahl, ein fehlendes Geschoss wird ersetzt.
    pub fn sync(&mut self, project: &Project) {
        self.selection.retain(|id| project.elements.contains_key(id));
        self.hover.retain(|c| c.ids.iter().all(|id| project.elements.contains_key(id)));
        self.hover_index = self.hover_index.min(self.hover.len().saturating_sub(1));
        if project.storey(self.storey).is_none() {
            *self = Self { tool: self.tool, ..Self::new(project) };
        }
        if self.wall_type.is_none_or(|t| project.wall_type(t).is_none()) {
            self.wall_type = project.wall_types().next().map(|(id, _)| id);
        }
    }

    pub fn pointer_moved(&mut self, project: &Project, input: Input) -> Response {
        match self.tool {
            Tool::ArcWall if self.arc_end.is_some() => {
                // Der Stich folgt dem Zeiger, gerundet auf 1 cm.
                self.pointer = Some(Snap { point: input.point, kind: SnapKind::Free });
                Response::redraw()
            }
            Tool::Wall | Tool::ArcWall | Tool::Grid | Tool::Column | Tool::SlabOpening | Tool::Line => {
                let mut snapped =
                    self.snap(project, self.storey, input.point, input.tolerance, self.chain_start, input.shift, &[]);
                // Ein getippter Winkel hält die Richtung; der Zeiger gibt nur die Länge.
                if let (Some(a), Some(s)) = (self.angle_lock, self.chain_start) {
                    let (sin, cos) = a.to_radians().sin_cos();
                    let t = ((input.point.x - s.x) * cos + (input.point.y - s.y) * sin).max(0.0);
                    let t = (t / snap::ROUND).round() * snap::ROUND;
                    snapped = Snap { point: Point2::new(s.x + cos * t, s.y + sin * t), kind: SnapKind::Angle };
                }
                self.pointer = Some(snapped);
                Response::redraw()
            }
            Tool::Move | Tool::Copy | Tool::Array | Tool::Mirror => {
                self.pointer = Some(self.snap(
                    project,
                    self.storey,
                    input.point,
                    input.tolerance,
                    self.chain_start,
                    input.shift,
                    &[],
                ));
                Response::redraw()
            }
            Tool::Rotate => {
                // Winkelraster um den Drehpunkt.
                let from = if self.chain_start.is_some() { self.pivot } else { None };
                self.pointer =
                    Some(self.snap(project, self.storey, input.point, input.tolerance, from, input.shift, &[]));
                Response::redraw()
            }
            Tool::Offset | Tool::Align | Tool::Trim | Tool::Split => {
                self.pointer = Some(Snap { point: input.point, kind: SnapKind::Free });
                self.hover_tolerance = input.tolerance;
                Response::redraw()
            }
            Tool::Door | Tool::Window | Tool::Text | Tool::CalibrateUnderlay | Tool::AlignUnderlay => {
                self.pointer = Some(Snap { point: input.point, kind: SnapKind::Free });
                self.hover_tolerance = input.tolerance;
                Response::redraw()
            }
            Tool::Select => {
                let shown = project.variant_view();
                let found = select::candidates_at(&shown, self.storey, input.point, input.tolerance);
                let same =
                    found.len() == self.hover.len() && found.iter().zip(&self.hover).all(|(a, b)| a.ids == b.ids);
                if !same {
                    self.hover = found;
                    self.hover_index = 0;
                }
                self.hover_at = Some(input.point);
                Response::redraw()
            }
            Tool::Space => Response::default(),
        }
    }

    pub fn pointer_down(&mut self, doc: &mut Document, input: Input, clicks: u32) -> Response {
        match self.tool {
            Tool::Column => {
                let p = self.snap(doc.project(), self.storey, input.point, input.tolerance, None, false, &[]).point;
                let column = Column {
                    phase: Default::default(),
                    storey: self.storey,
                    position: p,
                    profile: ColumnProfile::Rectangle { width: 0.3, depth: 0.3 },
                    rotation: 0.0,
                    material: "stahlbeton".into(),
                    base_offset: 0.0,
                    top: Top::Storey { offset: 0.0 },
                };
                Response::from_result(doc.insert(Element::Column(column)).map(|_| ()))
            }
            Tool::Door | Tool::Window => match self.placed_opening(doc.project(), input.point, input.tolerance) {
                Some(opening) => Response::from_result(doc.insert(Element::Opening(opening)).map(|_| ())),
                None => Response {
                    error: Some(format!("{} braucht eine Wand mit genug Platz", self.opening_title())),
                    ..Response::redraw()
                },
            },
            Tool::Space => {
                let p = Point2::new((input.point.x / 0.01).round() * 0.01, (input.point.y / 0.01).round() * 0.01);
                let space = self.new_space(doc.project(), p, 0);
                Response::from_result(doc.insert(Element::Space(space)).map(|_| ()))
            }
            Tool::SlabOpening => {
                let p = self.snap(doc.project(), self.storey, input.point, input.tolerance, None, false, &[]).point;
                self.pointer = None;
                match self.chain_start {
                    None => {
                        self.chain_start = Some(p);
                        Response::redraw()
                    }
                    Some(start) => {
                        self.end_chain();
                        self.add_slab_opening(doc, start, p)
                    }
                }
            }
            Tool::Grid => {
                let p = self
                    .snap(doc.project(), self.storey, input.point, input.tolerance, self.chain_start, input.shift, &[])
                    .point;
                self.pointer = None;
                match self.chain_start {
                    None => {
                        self.chain_start = Some(p);
                        Response::redraw()
                    }
                    Some(start) if start.distance(p) >= 0.01 => {
                        self.end_chain();
                        let label = next_grid_label(doc.project(), start, p);
                        Response::from_result(
                            doc.insert(Element::GridLine(GridLine { label, start, end: p })).map(|_| ()),
                        )
                    }
                    Some(_) => Response::redraw(),
                }
            }
            Tool::ArcWall => {
                if clicks >= 2 {
                    self.end_chain();
                    return Response::redraw();
                }
                if let (Some(start), Some(end)) = (self.chain_start, self.arc_end) {
                    self.pointer = None;
                    return self.add_arc_wall(doc, start, end, sagitta_from(start, end, input.point));
                }
                let p = self
                    .snap(doc.project(), self.storey, input.point, input.tolerance, self.chain_start, input.shift, &[])
                    .point;
                self.pointer = None;
                match self.chain_start {
                    None => self.chain_start = Some(p),
                    Some(start) if start.distance(p) >= 0.01 => self.arc_end = Some(p),
                    Some(_) => {}
                }
                Response::redraw()
            }
            Tool::CalibrateUnderlay | Tool::AlignUnderlay => {
                let Some(id) = self.underlay(doc.project()) else {
                    return Response {
                        error: Some("Auf diesem Geschoss liegt keine Unterlage".into()),
                        ..Response::redraw()
                    };
                };
                match (self.chain_start, self.arc_end) {
                    (None, _) => {
                        self.chain_start = Some(input.point);
                        Response::redraw()
                    }
                    (Some(a), None) if self.tool == Tool::AlignUnderlay => {
                        self.end_chain();
                        self.change_underlay(doc, id, "Unterlage ausrichten", |u| u.aligned(a, input.point))
                    }
                    (Some(_), None) => {
                        // Der zweite Punkt; der wahre Abstand wird getippt.
                        self.arc_end = Some(input.point);
                        Response::redraw()
                    }
                    (Some(_), Some(_)) => Response::redraw(),
                }
            }
            Tool::Text => {
                let p = Point2::new((input.point.x / 0.01).round() * 0.01, (input.point.y / 0.01).round() * 0.01);
                let text = Annotation {
                    view: AnnotationView::FloorPlan { storey: self.storey },
                    shape: AnnotationShape::Text { position: p, text: "Text".into(), height: 2.5, rotation: 0.0 },
                    pen: 0.25,
                    dashed: false,
                };
                match doc.insert(Element::Annotation(text)) {
                    Ok(id) => {
                        // Gleich wählen, damit der Text im Inspektor geändert werden kann.
                        self.tool = Tool::Select;
                        self.pointer = None;
                        self.selection = vec![id];
                        Response::changed()
                    }
                    Err(e) => Response { error: Some(e.to_string()), ..Response::redraw() },
                }
            }
            Tool::Line => {
                if clicks >= 2 {
                    self.end_chain();
                    return Response::redraw();
                }
                let p = self
                    .snap(doc.project(), self.storey, input.point, input.tolerance, self.chain_start, input.shift, &[])
                    .point;
                self.pointer = None;
                match self.chain_start {
                    None => {
                        self.chain_start = Some(p);
                        Response::redraw()
                    }
                    Some(start) => self.add_line(doc, start, p),
                }
            }
            Tool::Wall => {
                if clicks >= 2 {
                    self.end_chain();
                    return Response::redraw();
                }
                let p = self
                    .snap(doc.project(), self.storey, input.point, input.tolerance, self.chain_start, input.shift, &[])
                    .point;
                self.pointer = None;
                match self.chain_start {
                    None => {
                        self.chain_start = Some(p);
                        Response::redraw()
                    }
                    Some(start) => self.add_wall(doc, start, p),
                }
            }
            Tool::Move | Tool::Copy | Tool::Array | Tool::Mirror | Tool::Rotate => {
                let from = if self.tool == Tool::Rotate {
                    self.pivot.filter(|_| self.chain_start.is_some())
                } else {
                    self.chain_start
                };
                let p =
                    self.snap(doc.project(), self.storey, input.point, input.tolerance, from, input.shift, &[]).point;
                self.pointer = None;
                match self.chain_start {
                    None => {
                        self.chain_start = Some(p);
                        Response::redraw()
                    }
                    Some(_) => match self.modify_transform(p) {
                        Some(t) => self.finish_modify(doc, t, input.shift),
                        None => Response::redraw(),
                    },
                }
            }
            Tool::Split => {
                let Some(id) = geometry::wall_at(doc.project(), self.storey, input.point, input.tolerance) else {
                    return Response { error: Some("Zum Teilen auf eine Wand klicken".into()), ..Response::redraw() };
                };
                match modify::split_wall_ops(doc.project(), id, input.point) {
                    Ok(ops) => Response::from_result(doc.apply(Operation::Group { label: "Teilen".into(), ops })),
                    Err(e) => Response { error: Some(e), ..Response::redraw() },
                }
            }
            Tool::Trim => {
                let Some(id) = geometry::wall_at(doc.project(), self.storey, input.point, input.tolerance) else {
                    return Response { error: Some("Zum Stutzen auf eine Wand klicken".into()), ..Response::redraw() };
                };
                match self.picked.take() {
                    None => {
                        self.picked = Some((id, input.point));
                        Response::redraw()
                    }
                    Some(first) => match modify::trim_ops(doc.project(), first, (id, input.point)) {
                        Ok(ops) => Response::from_result(doc.apply(Operation::Group { label: "Stutzen".into(), ops })),
                        Err(e) => Response { error: Some(e), ..Response::redraw() },
                    },
                }
            }
            Tool::Align => {
                let Some((id, line)) = modify::line_at(doc.project(), self.storey, input.point, input.tolerance) else {
                    return Response {
                        error: Some("Auf eine Wandfläche, Achse oder Linie klicken".into()),
                        ..Response::redraw()
                    };
                };
                match self.align_ref.take() {
                    None => {
                        self.align_ref = Some(line);
                        Response::redraw()
                    }
                    Some(reference) => match modify::align_delta(reference, line) {
                        Ok(delta) if delta.0.hypot(delta.1) < 1e-9 => Response::redraw(),
                        Ok(delta) => {
                            let changes = moved_elements(doc.project(), &[id], delta);
                            Response::from_result(replace_all(doc, "Ausrichten", changes))
                        }
                        Err(e) => Response { error: Some(e), ..Response::redraw() },
                    },
                }
            }
            Tool::Offset if self.offset_distance.is_none() => Response {
                error: Some("Erst den Abstand tippen und ⏎, dann auf die Wand klicken".into()),
                ..Response::redraw()
            },
            Tool::Offset => match self.offset_target(doc.project(), input.point, input.tolerance) {
                Some((id, delta)) => {
                    let ops =
                        transform::copy_ops(doc.project(), &[id], &Transform::translation(delta.0, delta.1), None);
                    Response::from_result(doc.apply(Operation::Group { label: "Versetzen".into(), ops }))
                }
                None => {
                    Response { error: Some("Auf eine Wand, Achse oder Linie klicken".into()), ..Response::redraw() }
                }
            },
            Tool::Select => {
                let project = doc.project();
                // Griff an einem Ende einer gewählten Wand?
                for &id in &self.selection {
                    let Some(w) = project.wall(id) else { continue };
                    for end in [End::Start, End::End] {
                        if end_point(w, end).distance(input.point) <= input.tolerance {
                            self.drag = Some(Drag::Endpoint { wall: id, end, current: end_point(w, end) });
                            return Response::redraw();
                        }
                    }
                }
                // Klick auf die Zahl eines temporären Maßes: neuen Wert tippen.
                self.dim_edit = None;
                if let Some(i) = self
                    .temp_dims_now(project)
                    .iter()
                    .position(|d| d.middle().distance(input.point) <= input.tolerance * 1.5)
                {
                    self.dim_edit = Some(i);
                    self.typed.clear();
                    return Response::redraw();
                }
                // Bauteile anderer Varianten sind nicht zu sehen und nicht zu treffen.
                let shown = project.variant_view();
                let under = geometry::elements_at(&shown, self.storey, input.point, input.tolerance);
                let on_selection = under.iter().any(|id| self.selection.contains(id));
                let hit = match self.hover_at {
                    Some(at) if at.distance(input.point) <= input.tolerance && !self.hover.is_empty() => {
                        self.hover.get(self.hover_index).map(|c| c.ids.clone())
                    }
                    _ => select::candidates_at(&shown, self.storey, input.point, input.tolerance)
                        .into_iter()
                        .next()
                        .map(|c| c.ids),
                };
                self.drag = Some(Drag::Pending {
                    start: input.point,
                    hit,
                    on_selection,
                    shift: input.shift,
                    alt: input.alt,
                    tolerance: input.tolerance,
                });
                Response::redraw()
            }
        }
    }

    pub fn pointer_dragged(&mut self, project: &Project, input: Input) -> Response {
        match self.drag.clone() {
            Some(Drag::Pending { start, on_selection, shift, alt, tolerance, .. }) => {
                // Erst ab einigen Pixeln ist es ein Ziehen, kein Klick (Trackpad).
                if start.distance(input.point) < tolerance * 0.4 {
                    return Response::default();
                }
                self.drag = Some(if on_selection && !shift {
                    let s = self.snap(project, self.storey, start, tolerance, None, false, &self.selection).point;
                    Drag::Selection { start: s, current: s, copy: alt }
                } else if alt {
                    Drag::Lasso { points: vec![start], shift }
                } else {
                    Drag::Frame { start, current: start, shift }
                });
                self.pointer_dragged(project, input)
            }
            Some(Drag::Frame { .. }) => {
                if let Some(Drag::Frame { current, .. }) = &mut self.drag {
                    *current = input.point;
                }
                Response::redraw()
            }
            Some(Drag::Lasso { .. }) => {
                if let Some(Drag::Lasso { points, .. }) = &mut self.drag
                    && points.last().is_none_or(|q| q.distance(input.point) >= input.tolerance * 0.3)
                {
                    points.push(input.point);
                }
                Response::redraw()
            }
            Some(Drag::Selection { .. }) => {
                let p =
                    self.snap(project, self.storey, input.point, input.tolerance, None, false, &self.selection).point;
                if let Some(Drag::Selection { current, .. }) = &mut self.drag {
                    *current = p;
                }
                Response::redraw()
            }
            Some(Drag::Endpoint { wall, end, .. }) => {
                let other =
                    project.wall(wall).map(|w| end_point(w, if end == End::Start { End::End } else { End::Start }));
                let p =
                    self.snap(project, self.storey, input.point, input.tolerance, other, input.shift, &[wall]).point;
                if let Some(Drag::Endpoint { current, .. }) = &mut self.drag {
                    *current = p;
                }
                Response::redraw()
            }
            None => self.pointer_moved(project, input),
        }
    }

    pub fn pointer_up(&mut self, doc: &mut Document) -> Response {
        let Some(drag) = self.drag.take() else { return Response::default() };
        let (label, changes) = match drag {
            Drag::Pending { hit, shift, .. } => {
                let new = match (hit, shift) {
                    (Some(ids), true) => toggled(&self.selection, &ids),
                    (Some(ids), false) => ids,
                    (None, true) => self.selection.clone(),
                    (None, false) => Vec::new(),
                };
                self.set_selection(new);
                return Response::redraw();
            }
            Drag::Frame { start, current, shift } => {
                let region = select::Region::Rect(start, current);
                let crossing = current.x < start.x;
                let found = select::in_region(&doc.project().variant_view(), self.storey, &region, crossing);
                self.select_found(found, shift);
                return Response::redraw();
            }
            Drag::Lasso { points, shift } => {
                let found = if points.len() >= 3 {
                    select::in_region(&doc.project().variant_view(), self.storey, &select::Region::Lasso(points), false)
                } else {
                    Vec::new()
                };
                self.select_found(found, shift);
                return Response::redraw();
            }
            Drag::Selection { start, current, copy } => {
                let delta = (current.x - start.x, current.y - start.y);
                if delta.0.hypot(delta.1) < 1e-9 {
                    return Response::redraw();
                }
                if copy {
                    return self.copy_selection(doc, Transform::translation(delta.0, delta.1), "Kopieren");
                }
                ("Verschieben", moved_elements(doc.project(), &self.selection, delta))
            }
            Drag::Endpoint { wall, end, current } => {
                if doc.project().wall(wall).is_none_or(|w| end_point(w, end).distance(current) < 1e-9) {
                    return Response::redraw();
                }
                ("Wandende verschieben", as_elements(moved_endpoint(doc.project(), wall, end, current)))
            }
        };
        Response::from_result(replace_all(doc, label, changes))
    }

    /// Stand des Modells während einer Ziehbewegung, für die Vorschau.
    pub fn preview(&self, project: &Project) -> Option<Project> {
        let changes = match self.drag.as_ref()? {
            Drag::Selection { start, current, copy: true } => {
                let t = Transform::translation(current.x - start.x, current.y - start.y);
                let mut p = project.clone();
                for op in transform::copy_ops(project, &self.selection, &t, None) {
                    if let Operation::Insert { id, element } = op {
                        p.elements.insert(id, element);
                    }
                }
                return Some(p);
            }
            Drag::Selection { start, current, .. } => {
                moved_elements(project, &self.selection, (current.x - start.x, current.y - start.y))
            }
            Drag::Endpoint { wall, end, current } => as_elements(moved_endpoint(project, *wall, *end, *current)),
            Drag::Pending { .. } | Drag::Frame { .. } | Drag::Lasso { .. } => return None,
        };
        let mut p = project.clone();
        for (id, e) in changes {
            p.elements.insert(id, e);
        }
        Some(p)
    }

    pub fn key(&mut self, doc: &mut Document, key: Key) -> Response {
        match key {
            Key::Escape => {
                if !self.shortcut.is_empty() {
                    self.shortcut.clear();
                } else if self.dim_edit.take().is_some() || self.drag.take().is_some() {
                } else if self.arc_end.is_some() {
                    self.arc_end = None;
                } else if self.chain_start.is_some() {
                    self.end_chain();
                } else if self.tool != Tool::Select {
                    self.tool = Tool::Select;
                } else {
                    self.set_selection(Vec::new());
                }
                self.typed.clear();
                Response::redraw()
            }
            Key::Tab => {
                if self.tool == Tool::Select && self.hover.len() > 1 {
                    self.hover_index = (self.hover_index + 1) % self.hover.len();
                    return Response::redraw();
                }
                Response::default()
            }
            Key::Arrow { dx, dy, large } => {
                if self.selection.is_empty() || self.drag.is_some() {
                    return Response::default();
                }
                let step = if large { 0.1 } else { 0.01 };
                let delta = (dx as f64 * step, dy as f64 * step);
                let changes = moved_elements(doc.project(), &self.selection, delta);
                Response::from_result(replace_all(doc, "Verschieben", changes))
            }
            Key::Enter if self.typed.is_empty() && self.chain_start.is_none() && self.tool == Tool::Select => {
                // ⏎ ohne laufende Aktion wiederholt den letzten Befehl (wie Revit).
                match self.last_command.clone() {
                    Some(command) => self.run(doc, &command),
                    None => Response::default(),
                }
            }
            Key::Enter if self.dim_edit.is_some() => self.apply_temp_dim(doc),
            Key::Enter if matches!(self.tool, Tool::Move | Tool::Copy | Tool::Array | Tool::Rotate | Tool::Offset) => {
                self.modify_enter(doc)
            }
            Key::Enter => {
                if self.tool == Tool::CalibrateUnderlay
                    && let (Some(a), Some(b), Some(distance)) =
                        (self.chain_start, self.arc_end, parse_length(&self.typed))
                    && let Some(id) = self.underlay(doc.project())
                {
                    self.typed.clear();
                    self.end_chain();
                    return self.change_underlay(doc, id, "Unterlage kalibrieren", |u| u.calibrated(a, b, distance));
                }
                if let (Some(start), Some(end), Some(height)) =
                    (self.chain_start, self.arc_end, parse_length(&self.typed))
                {
                    // Getippter Stich, zur Seite des Zeigers.
                    let side = self.pointer.map_or(1.0, |p| sagitta_from(start, end, p.point).signum());
                    self.typed.clear();
                    return self.add_arc_wall(doc, start, end, side * height);
                }
                if let Some(start) = self.chain_start
                    && !self.typed.is_empty()
                {
                    let typed = std::mem::take(&mut self.typed);
                    let entry = entry::parse(&typed);
                    if let Some(entry::Entry::Angle(a)) = entry {
                        self.angle_lock = Some(a);
                        return Response::redraw();
                    }
                    let Some((dx, dy)) = entry.and_then(|e| self.typed_vector(e, start)) else {
                        return Response {
                            error: Some(format!("Eingabe nicht verstanden: {typed}")),
                            ..Response::redraw()
                        };
                    };
                    let end = Point2::new(start.x + dx, start.y + dy);
                    if self.tool == Tool::ArcWall {
                        self.arc_end = Some(end);
                        return Response::redraw();
                    }
                    if self.tool == Tool::Line {
                        return self.add_line(doc, start, end);
                    }
                    return self.add_wall(doc, start, end);
                }
                self.end_chain();
                Response::redraw()
            }
            Key::Delete => {
                if self.typed.pop().is_some() || self.shortcut.pop().is_some() {
                    return Response::redraw();
                }
                self.delete_selection(doc)
            }
            Key::Char(c) => {
                let numbers = self.chain_start.is_some()
                    || self.dim_edit.is_some()
                    || matches!(self.tool, Tool::Move | Tool::Copy | Tool::Array | Tool::Rotate | Tool::Offset);
                if numbers
                    && ((self.typed.is_empty() && entry::starts(c)) || (!self.typed.is_empty() && entry::accepts(c)))
                {
                    self.typed.push(c);
                    return Response::redraw();
                }
                if c == ' ' && self.tool == Tool::Rotate {
                    // Leertaste setzt den Drehpunkt an den Zeiger (wie Revit).
                    if let Some(p) = self.pointer {
                        self.pivot = Some(p.point);
                        self.chain_start = None;
                    }
                    return Response::redraw();
                }
                if c == ' ' && self.tool == Tool::Select && self.chain_start.is_none() {
                    return self.flip_with_space(doc);
                }
                if c.is_alphabetic() {
                    return self.shortcut_key(doc, c);
                }
                Response::default()
            }
        }
    }

    /// Führt einen Befehl aus der Befehlspalette aus.
    pub fn run(&mut self, doc: &mut Document, command: &Command) -> Response {
        match command {
            Command::SelectTool => {
                self.tool = Tool::Select;
                self.end_chain();
                Response::redraw()
            }
            Command::WallTool => self.set_tool(Tool::Wall),
            Command::ArcWallTool => self.set_tool(Tool::ArcWall),
            Command::GridTool => self.set_tool(Tool::Grid),
            Command::ColumnTool => self.set_tool(Tool::Column),
            Command::SpaceTool => self.set_tool(Tool::Space),
            Command::DoorTool => self.set_tool(Tool::Door),
            Command::WindowTool => self.set_tool(Tool::Window),
            Command::SlabOpeningTool => self.set_tool(Tool::SlabOpening),
            Command::LineTool => self.set_tool(Tool::Line),
            Command::TextTool => self.set_tool(Tool::Text),
            Command::CalibrateUnderlayTool => self.set_tool(Tool::CalibrateUnderlay),
            Command::AlignUnderlayTool => self.set_tool(Tool::AlignUnderlay),
            Command::MoveTool => self.start_modify(doc.project(), Tool::Move),
            Command::CopyTool => self.start_modify(doc.project(), Tool::Copy),
            Command::RotateTool => self.start_modify(doc.project(), Tool::Rotate),
            Command::MirrorTool => self.start_modify(doc.project(), Tool::Mirror),
            Command::ArrayTool => self.start_modify(doc.project(), Tool::Array),
            Command::OffsetTool => self.set_tool(Tool::Offset),
            Command::AlignTool => self.set_tool(Tool::Align),
            Command::TrimTool => self.set_tool(Tool::Trim),
            Command::SplitTool => self.set_tool(Tool::Split),
            Command::FlipDoorSwing => self.change_doors(doc, "Aufschlag wechseln", |d| {
                // Die Bänder bleiben, wo sie sind: mit der Seite wechselt auch DIN links und rechts.
                d.opens_to = d.opens_to.other();
                d.hinge = d.hinge.other();
            }),
            Command::FlipDoorHinge => self.change_doors(doc, "Anschlag wechseln", |d| d.hinge = d.hinge.other()),
            Command::DimensionWalls => self.dimension_walls(doc),
            Command::SlabFromWalls => self.slab_from_walls(doc),
            Command::GableRoofFromWalls => self.gable_roof_from_walls(doc),
            Command::DetectSpaces => self.detect_spaces(doc),
            Command::SetWallType(id) => {
                let id = ElementId(*id);
                if doc.project().wall_type(id).is_none() {
                    return Response { error: Some("Diesen Wandaufbau gibt es nicht".into()), ..Default::default() };
                }
                self.wall_type = Some(id);
                self.change_selected(doc, "Wandaufbau ändern", |w| w.wall_type = id)
            }
            Command::SetAlignment(a) => {
                self.alignment = *a;
                let a = *a;
                self.change_selected(doc, "Bezugslinie ändern", |w| w.alignment = a)
            }
            Command::SetPhase(phase) => {
                let changes: Vec<(ElementId, Element)> = self
                    .selection
                    .iter()
                    .filter_map(|id| {
                        let mut e = doc.project().elements.get(id)?.clone();
                        e.set_phase(*phase).then_some((*id, e))
                    })
                    .collect();
                if changes.is_empty() {
                    return Response::redraw();
                }
                Response::from_result(replace_all(doc, "Bauzustand ändern", changes))
            }
            Command::SetStorey(id) => {
                if doc.project().storey(ElementId(*id)).is_some() {
                    self.storey = ElementId(*id);
                    self.selection.clear();
                    self.end_chain();
                }
                Response::redraw()
            }
            Command::AddStorey => {
                let storeys = doc.project().storeys();
                let (elevation, height) =
                    storeys.last().map_or((0.0, 2.75), |(_, s)| (s.elevation + s.height, s.height));
                let name = format!("{}. OG", storeys.iter().filter(|(_, s)| s.elevation > 0.0).count() + 1);
                match doc.insert(Element::Storey(Storey { name, elevation, height })) {
                    Ok(id) => {
                        self.storey = id;
                        self.selection.clear();
                        Response::changed()
                    }
                    Err(e) => Response { error: Some(e.to_string()), ..Response::redraw() },
                }
            }
            Command::DeleteSelection => self.delete_selection(doc),
            Command::NewVariantSet => {
                let n = doc.project().variant_sets().count() + 1;
                let set = lot_model::VariantSet {
                    name: format!("Varianten {n}"),
                    variants: ["A", "B"]
                        .map(|name| lot_model::Variant { name: name.into(), elements: Vec::new() })
                        .to_vec(),
                    active: 0,
                };
                Response::from_result(doc.insert(Element::VariantSet(set)).map(|_| ()))
            }
            Command::AddSelectionToVariant(set) => {
                let id = ElementId(*set);
                let Some(Element::VariantSet(old)) = doc.project().elements.get(&id).cloned() else {
                    return Response {
                        error: Some("Diese Variantengruppe gibt es nicht".into()),
                        ..Default::default()
                    };
                };
                let mut new = old.clone();
                for v in &mut new.variants {
                    v.elements.retain(|e| !self.selection.contains(e));
                }
                if let Some(v) = new.variants.get_mut(new.active) {
                    v.elements.extend(self.selection.iter().copied().filter(|e| *e != id));
                    v.elements.sort();
                }
                Response::from_result(replace_all(doc, "Auswahl in Variante", vec![(id, Element::VariantSet(new))]))
            }
            Command::SwitchVariant(set) => {
                let id = ElementId(*set);
                let Some(Element::VariantSet(old)) = doc.project().elements.get(&id).cloned() else {
                    return Response {
                        error: Some("Diese Variantengruppe gibt es nicht".into()),
                        ..Default::default()
                    };
                };
                let mut new = old.clone();
                new.active = (old.active + 1) % old.variants.len().max(1);
                self.selection.clear();
                Response::from_result(replace_all(doc, "Variante wechseln", vec![(id, Element::VariantSet(new))]))
            }
            Command::FlipWalls => self.flip_selection(doc),
            Command::SelectSimilar => {
                if self.selection.is_empty() {
                    return Response { error: Some("Erst ein Bauteil wählen, dann SA".into()), ..Response::redraw() };
                }
                let found = select::similar(&doc.project().variant_view(), self.storey, &self.selection);
                self.tool = Tool::Select;
                self.set_selection(found);
                Response::redraw()
            }
            Command::PreviousSelection => {
                if let Some(previous) = self.history.pop() {
                    self.selection = previous;
                    self.sync(doc.project());
                }
                Response::redraw()
            }
            Command::SelectUnderlay => {
                let ids: Vec<ElementId> = doc.project().underlays_on(self.storey).map(|(id, _)| id).collect();
                let next = match self.selection[..] {
                    [one] => ids.iter().position(|u| *u == one).map_or(0, |i| (i + 1) % ids.len().max(1)),
                    _ => 0,
                };
                match ids.get(next) {
                    Some(id) => {
                        self.tool = Tool::Select;
                        self.set_selection(vec![*id]);
                        Response::redraw()
                    }
                    None => Response {
                        error: Some("Auf diesem Geschoss liegt keine Unterlage".into()),
                        ..Response::redraw()
                    },
                }
            }
            Command::CopyToClipboard => {
                if self.selection.is_empty() {
                    return Response::default();
                }
                self.clipboard = Some((Box::new(doc.project().clone()), self.selection.clone()));
                Response::redraw()
            }
            Command::Paste => {
                let Some((source, ids)) = &self.clipboard else {
                    return Response { error: Some("Die Zwischenablage ist leer".into()), ..Response::redraw() };
                };
                // An derselben Stelle, im aktuellen Geschoss (etwa Wände vom EG ins OG).
                let ops = transform::copy_ops_into(
                    source,
                    doc.project(),
                    ids,
                    &Transform::translation(0.0, 0.0),
                    Some(self.storey),
                );
                let new: Vec<ElementId> = ops
                    .iter()
                    .filter_map(|op| match op {
                        Operation::Insert { id, .. } => Some(*id),
                        _ => None,
                    })
                    .collect();
                if ops.is_empty() {
                    return Response::redraw();
                }
                let result = doc.apply(Operation::Group { label: "Einfügen".into(), ops });
                if result.is_ok() {
                    self.tool = Tool::Select;
                    self.set_selection(new);
                }
                Response::from_result(result)
            }
            Command::SetSelection(ids) => {
                let project = doc.project();
                let ids = ids.iter().map(|i| ElementId(*i)).filter(|id| project.elements.contains_key(id)).collect();
                self.set_selection(ids);
                Response::redraw()
            }
            Command::SelectAll => {
                self.tool = Tool::Select;
                let all = select::pickable(&doc.project().variant_view(), self.storey);
                self.set_selection(all);
                Response::redraw()
            }
            Command::Undo => match doc.undo() {
                Ok(_) => {
                    self.sync(doc.project());
                    Response::changed()
                }
                Err(e) => Response { error: Some(e.to_string()), ..Response::redraw() },
            },
            Command::Redo => match doc.redo() {
                Ok(_) => {
                    self.sync(doc.project());
                    Response::changed()
                }
                Err(e) => Response { error: Some(e.to_string()), ..Response::redraw() },
            },
        }
    }

    /// Befehle für die Befehlspalette, passend zum Projekt.
    pub fn commands(&self, project: &Project) -> Vec<CommandInfo> {
        let info = |c: Command, title: &str, group: &str| {
            let k = shortcut_of(&c);
            CommandInfo::new(c, title, k, group)
        };
        let mut list = vec![
            info(Command::SelectTool, "Auswählen", "Werkzeug"),
            info(Command::WallTool, "Wand zeichnen", "Werkzeug"),
            info(Command::ArcWallTool, "Bogenwand zeichnen", "Werkzeug"),
            info(Command::GridTool, "Achse zeichnen", "Werkzeug"),
            info(Command::ColumnTool, "Stütze setzen", "Werkzeug"),
            info(Command::SpaceTool, "Raum setzen", "Werkzeug"),
            info(Command::DoorTool, "Tür setzen", "Werkzeug"),
            info(Command::WindowTool, "Fenster setzen", "Werkzeug"),
            info(Command::SlabOpeningTool, "Deckendurchbruch zeichnen", "Werkzeug"),
            info(Command::LineTool, "Linie zeichnen (2D)", "Werkzeug"),
            info(Command::TextTool, "Text setzen (2D)", "Werkzeug"),
            info(Command::CalibrateUnderlayTool, "Unterlage kalibrieren", "Unterlage"),
            info(Command::AlignUnderlayTool, "Unterlage ausrichten (waagerecht)", "Unterlage"),
            info(Command::SelectUnderlay, "Unterlage wählen (zum Verschieben oder Löschen)", "Unterlage"),
            info(Command::FlipDoorSwing, "Tür: Aufschlag zur anderen Seite", "Auswahl"),
            info(Command::FlipDoorHinge, "Tür: Anschlag wechseln (DIN links, DIN rechts)", "Auswahl"),
            info(Command::DimensionWalls, "Bemaßen: Maßketten an den gewählten Wänden", "Auswahl"),
            info(Command::SlabFromWalls, "Decke aus Wänden (im untersten Geschoss Bodenplatte)", "Geschoss"),
            info(Command::GableRoofFromWalls, "Satteldach über dem Geschoss", "Geschoss"),
            info(Command::DetectSpaces, "Räume erkennen (jeder umschlossene Bereich)", "Werkzeug"),
            CommandInfo::new(Command::SelectAll, "Alles im Geschoss wählen", "⌘A", "Auswahl"),
            info(Command::SelectSimilar, "Gleiche wählen (im Geschoss, zweimal: im Projekt)", "Auswahl"),
            info(Command::MoveTool, "Verschieben", "Bearbeiten"),
            info(Command::CopyTool, "Kopieren", "Bearbeiten"),
            info(Command::RotateTool, "Drehen", "Bearbeiten"),
            info(Command::MirrorTool, "Spiegeln (als Kopie, mit ⇧ ohne)", "Bearbeiten"),
            info(Command::ArrayTool, "Reihe", "Bearbeiten"),
            info(Command::OffsetTool, "Versetzen (parallel kopieren)", "Bearbeiten"),
            info(Command::AlignTool, "Ausrichten", "Bearbeiten"),
            info(Command::TrimTool, "Stutzen und Dehnen zur Ecke", "Bearbeiten"),
            info(Command::SplitTool, "Wand teilen", "Bearbeiten"),
            CommandInfo::new(Command::PreviousSelection, "Vorige Auswahl", "⌘←", "Auswahl"),
            CommandInfo::new(Command::DeleteSelection, "Auswahl löschen", "⌫", "Auswahl"),
            CommandInfo::new(Command::CopyToClipboard, "In die Zwischenablage kopieren", "⌘C", "Bearbeiten"),
            CommandInfo::new(Command::Paste, "Einfügen an derselben Stelle, im aktuellen Geschoss", "⌘V", "Bearbeiten"),
            CommandInfo::new(Command::FlipWalls, "Wand umdrehen (innen und außen tauschen)", "␣", "Auswahl"),
            info(Command::SetPhase(lot_model::Phase::Existing), "Bauzustand: Bestand", "Auswahl"),
            info(Command::SetPhase(lot_model::Phase::Demolished), "Bauzustand: Abbruch", "Auswahl"),
            info(Command::SetPhase(lot_model::Phase::New), "Bauzustand: Neubau", "Auswahl"),
            CommandInfo::new(Command::Undo, "Widerrufen", "⌘Z", "Bearbeiten"),
            CommandInfo::new(Command::Redo, "Wiederholen", "⇧⌘Z", "Bearbeiten"),
        ];
        list.push(CommandInfo::new(Command::NewVariantSet, "Neue Variantengruppe (A und B)", "", "Varianten"));
        for (id, v) in project.variant_sets() {
            let next = v.variants.get((v.active + 1) % v.variants.len().max(1)).map_or("", |x| x.name.as_str());
            list.push(CommandInfo::new(
                Command::SwitchVariant(id.0),
                format!("{}: Variante {} zeigen (jetzt {})", v.name, next, v.active_name()),
                "",
                "Varianten",
            ));
            list.push(CommandInfo::new(
                Command::AddSelectionToVariant(id.0),
                format!("{}: Auswahl nur in Variante {}", v.name, v.active_name()),
                "",
                "Varianten",
            ));
        }
        for (id, t) in project.wall_types() {
            let mark = if Some(id) == self.wall_type { " ✓" } else { "" };
            list.push(CommandInfo::new(Command::SetWallType(id.0), format!("{}{mark}", t.name), "", "Wandaufbau"));
        }
        for (a, title) in [
            (WallAlignment::Center, "Bezugslinie Mitte"),
            (WallAlignment::Exterior, "Bezugslinie Außenkante"),
            (WallAlignment::Interior, "Bezugslinie Innenkante"),
        ] {
            let mark = if a == self.alignment { " ✓" } else { "" };
            list.push(CommandInfo::new(Command::SetAlignment(a), format!("{title}{mark}"), "", "Bezugslinie"));
        }
        for (id, s) in project.storeys() {
            let mark = if id == self.storey { " ✓" } else { "" };
            list.push(CommandInfo::new(Command::SetStorey(id.0), format!("Geschoss {}{mark}", s.name), "", "Geschoss"));
        }
        list.push(CommandInfo::new(Command::AddStorey, "Geschoss hinzufügen", "", "Geschoss"));
        list
    }

    /// Was über dem Plan zu zeichnen ist.
    pub fn overlay(&self, project: &Project) -> Overlay {
        let mut overlay = Overlay::default();
        let shown = self.preview(project);
        let project_now = shown.as_ref().unwrap_or(project);
        for id in &self.selection {
            if let Some(o) = outline(project_now, *id) {
                overlay.selected.push(o);
            }
            if let Some(w) = project_now.wall(*id) {
                overlay.handles.extend([w.start, w.end]);
            }
        }
        for (i, d) in self.temp_dims_now(project).into_iter().enumerate() {
            let editing = self.dim_edit == Some(i);
            let text = if editing && !self.typed.is_empty() { self.typed.clone() } else { select::meters(d.value) };
            overlay.temp_dims.push((d.a, d.b, text, editing));
        }
        if self.tool == Tool::Select {
            match &self.drag {
                None => {
                    if let Some(c) = self.hover.get(self.hover_index) {
                        for id in c.ids.iter().filter(|id| !self.selection.contains(id)) {
                            overlay.hovered.extend(outline(project, *id));
                        }
                        if let Some(at) = self.hover_at {
                            let text = if self.hover.len() > 1 {
                                format!("{} von {}: {}  ⇥", self.hover_index + 1, self.hover.len(), c.label)
                            } else {
                                c.label.clone()
                            };
                            overlay.tip = Some((at, text));
                        }
                    }
                }
                Some(Drag::Frame { start, current, .. }) => {
                    overlay.frame = Some((select::Region::Rect(*start, *current).ring(), current.x < start.x));
                }
                Some(Drag::Lasso { points, .. }) => overlay.frame = Some((points.clone(), false)),
                Some(Drag::Selection { start, current, copy }) => {
                    let d = format_m(start.distance(*current));
                    overlay.tip = Some((*current, if *copy { format!("Kopie · {d}") } else { d }));
                }
                _ => {}
            }
        }
        if self.tool == Tool::Column
            && let Some(p) = self.pointer
        {
            overlay.snap = Some(p);
            overlay.preview = Some(column_outline(&Column {
                phase: Default::default(),
                storey: self.storey,
                position: p.point,
                profile: ColumnProfile::Rectangle { width: 0.3, depth: 0.3 },
                rotation: 0.0,
                material: String::new(),
                base_offset: 0.0,
                top: Top::Storey { offset: 0.0 },
            }));
        }
        if matches!(self.tool, Tool::Door | Tool::Window)
            && let Some(p) = self.pointer
            && let Some(o) = self.placed_opening(project, p.point, self.hover_tolerance)
        {
            overlay.preview = geometry::opening_outline(project, &o);
        }
        if self.tool == Tool::SlabOpening {
            overlay.snap = self.pointer;
            if let (Some(a), Some(p)) = (self.chain_start, self.pointer) {
                let b = p.point;
                overlay.preview = Some(vec![[a.x, a.y], [b.x, a.y], [b.x, b.y], [a.x, b.y]]);
            }
        }
        if matches!(self.tool, Tool::CalibrateUnderlay | Tool::AlignUnderlay)
            && let Some(a) = self.chain_start
            && let Some(b) = self.arc_end.or(self.pointer.map(|p| p.point))
        {
            let (dx, dy) = (b.x - a.x, b.y - a.y);
            let len = dx.hypot(dy).max(1e-9);
            let (nx, ny) = (-dy / len * 0.01, dx / len * 0.01);
            overlay.preview =
                Some(vec![[a.x - nx, a.y - ny], [b.x - nx, b.y - ny], [b.x + nx, b.y + ny], [a.x + nx, a.y + ny]]);
            let mid = Point2::new((a.x + b.x) / 2.0, (a.y + b.y) / 2.0);
            let text =
                if self.typed.is_empty() { format!("jetzt {}", format_m(len)) } else { format!("{} m", self.typed) };
            overlay.label = Some((mid, text));
        }
        if self.tool == Tool::Line {
            overlay.snap = self.pointer;
            if let (Some(start), Some(p)) = (self.chain_start, self.pointer) {
                let (dx, dy) = (p.point.x - start.x, p.point.y - start.y);
                let len = dx.hypot(dy).max(1e-9);
                let (nx, ny) = (-dy / len * 0.01, dx / len * 0.01);
                overlay.preview = Some(vec![
                    [start.x - nx, start.y - ny],
                    [p.point.x - nx, p.point.y - ny],
                    [p.point.x + nx, p.point.y + ny],
                    [start.x + nx, start.y + ny],
                ]);
                let mid = Point2::new((start.x + p.point.x) / 2.0, (start.y + p.point.y) / 2.0);
                let text = if self.typed.is_empty() { format_m(len) } else { format!("{} m", self.typed) };
                overlay.label = Some((mid, text));
            }
        }
        if self.tool == Tool::Grid {
            overlay.snap = self.pointer;
            if let (Some(start), Some(p)) = (self.chain_start, self.pointer) {
                let (dx, dy) = (p.point.x - start.x, p.point.y - start.y);
                let len = dx.hypot(dy).max(1e-9);
                let (nx, ny) = (-dy / len * 0.03, dx / len * 0.03);
                overlay.preview = Some(vec![
                    [start.x - nx, start.y - ny],
                    [p.point.x - nx, p.point.y - ny],
                    [p.point.x + nx, p.point.y + ny],
                    [start.x + nx, start.y + ny],
                ]);
            }
        }
        if self.tool == Tool::ArcWall {
            overlay.snap = self.pointer.filter(|_| self.arc_end.is_none());
            match (self.chain_start, self.arc_end, self.pointer) {
                (Some(start), Some(end), Some(p)) => {
                    if let Some(mut wall) = self.new_wall(start, end) {
                        wall.sagitta = sagitta_from(start, end, p.point);
                        if !self.typed.is_empty() {
                            wall.sagitta = wall.sagitta.signum() * parse_length(&self.typed).unwrap_or(0.0);
                        }
                        overlay.preview = Some(footprint(project, &wall));
                        let text = format!("Stich {}", format_m(wall.sagitta.abs()));
                        overlay.label = Some((wall.point_at(wall.length() / 2.0, 0.0), text));
                    }
                }
                (Some(start), None, Some(p)) => {
                    if let Some(wall) = self.new_wall(start, p.point) {
                        overlay.preview = Some(footprint(project, &wall));
                        let mid = Point2::new((start.x + p.point.x) / 2.0, (start.y + p.point.y) / 2.0);
                        let text =
                            if self.typed.is_empty() { format_m(wall.length()) } else { format!("{} m", self.typed) };
                        overlay.label = Some((mid, text));
                    }
                }
                _ => {}
            }
        }
        self.modify_overlay(project, &mut overlay);
        if self.tool == Tool::Wall {
            overlay.snap = self.pointer;
            if let (Some(start), Some(p)) = (self.chain_start, self.pointer)
                && let Some(wall) = self.new_wall(start, p.point)
            {
                overlay.preview = Some(footprint(project, &wall));
                let mid = Point2::new((start.x + p.point.x) / 2.0, (start.y + p.point.y) / 2.0);
                let text = if self.typed.is_empty() { format_m(wall.length()) } else { format!("{} m", self.typed) };
                overlay.label = Some((mid, text));
            }
        }
        overlay
    }

    /// Vorschau der Bearbeiten-Befehle: Umrisse am Ziel, Hilfslinien, Maße.
    fn modify_overlay(&self, project: &Project, overlay: &mut Overlay) {
        let pointer = self.pointer.map(|p| p.point);
        let moved = |ids: &[ElementId], t: &Transform| -> Vec<Vec<[f64; 2]>> {
            ids.iter()
                .filter_map(|id| outline(project, *id))
                .map(|o| {
                    o.into_iter()
                        .map(|q| {
                            let r = t.point(Point2::new(q[0], q[1]));
                            [r.x, r.y]
                        })
                        .collect()
                })
                .collect()
        };
        match self.tool {
            Tool::Move | Tool::Copy | Tool::Array | Tool::Rotate | Tool::Mirror => {
                overlay.snap = self.pointer;
                if let Some(c) = self.pivot.filter(|_| self.tool == Tool::Rotate) {
                    overlay.marks.push(c);
                }
                let (Some(base), Some(p)) = (self.chain_start, pointer) else {
                    overlay.marks.extend(self.chain_start);
                    return;
                };
                overlay.marks.push(base);
                let Some(t) = self.modify_transform(p) else { return };
                match self.tool {
                    Tool::Array => {
                        let (n, divide) = match self.series {
                            Some(entry::Entry::Count(n)) => (n, false),
                            Some(entry::Entry::Divide(n)) => (n.max(2), true),
                            _ => (2, false),
                        };
                        let o = t.point(Point2::new(0.0, 0.0));
                        let step = if divide { (o.x / (n - 1) as f64, o.y / (n - 1) as f64) } else { (o.x, o.y) };
                        for i in 1..n.min(30) {
                            let ti = Transform::translation(step.0 * i as f64, step.1 * i as f64);
                            overlay.ghosts.extend(moved(&self.selection, &ti));
                        }
                        overlay.guides.push((base, p));
                        overlay.tip = Some((p, format_m(base.distance(p))));
                    }
                    Tool::Rotate => {
                        if let Some(c) = self.pivot {
                            overlay.guides.extend([(c, base), (c, p)]);
                            let a = (p.y - c.y).atan2(p.x - c.x) - (base.y - c.y).atan2(base.x - c.x);
                            overlay.tip = Some((p, format!("{:.2}°", a.to_degrees()).replace('.', ",")));
                        }
                        overlay.ghosts.extend(moved(&self.selection, &t));
                    }
                    Tool::Mirror => {
                        overlay.guides.push((base, p));
                        overlay.ghosts.extend(moved(&self.selection, &t));
                    }
                    _ => {
                        overlay.guides.push((base, p));
                        overlay.ghosts.extend(moved(&self.selection, &t));
                        overlay.tip = Some((p, format_m(base.distance(p))));
                    }
                }
            }
            Tool::Offset => {
                if let Some(p) = pointer
                    && let Some((id, delta)) = self.offset_target(project, p, self.hover_tolerance)
                {
                    overlay.hovered.extend(outline(project, id));
                    overlay.ghosts.extend(moved(&[id], &Transform::translation(delta.0, delta.1)));
                    overlay.tip = Some((p, format_m(delta.0.hypot(delta.1))));
                }
            }
            Tool::Trim | Tool::Split => {
                if let Some((id, _)) = self.picked {
                    overlay.selected.extend(outline(project, id));
                }
                if let Some(p) = pointer
                    && let Some(id) = geometry::wall_at(project, self.storey, p, self.hover_tolerance)
                {
                    overlay.hovered.extend(outline(project, id));
                    if self.tool == Tool::Split
                        && let Some(w) = project.wall(id)
                    {
                        let s = ((w.locate(p).0 / snap::ROUND).round() * snap::ROUND).clamp(0.0, w.length());
                        overlay.marks.push(w.point_at(s, 0.0));
                        overlay.tip = Some((p, format!("{} | {}", format_m(s), format_m(w.length() - s))));
                    }
                }
            }
            Tool::Align => {
                if let Some(r) = self.align_ref {
                    let at = |k: f64| Point2::new(r.point.x + r.dir.0 * k, r.point.y + r.dir.1 * k);
                    overlay.guides.push((at(-30.0), at(30.0)));
                }
                if let Some(p) = pointer
                    && let Some((id, _)) = modify::line_at(project, self.storey, p, self.hover_tolerance)
                {
                    overlay.hovered.extend(outline(project, id));
                }
            }
            _ => {}
        }
    }

    /// Hinweis für die Statuszeile.
    pub fn hint(&self) -> String {
        if !self.shortcut.is_empty() {
            let options: Vec<String> = shortcuts()
                .iter()
                .filter(|(k, _)| k.starts_with(self.shortcut.as_str()))
                .map(|(k, c)| format!("{k} {}", command_title(c)))
                .collect();
            return format!("Kürzel {}…: {}. Esc bricht ab.", self.shortcut, options.join(", "));
        }
        match (self.tool, self.chain_start, &self.drag) {
            (Tool::Wall, None, _) => "Wand: Anfangspunkt klicken. Esc: zurück zum Auswählen.".into(),
            (Tool::Wall, Some(_), _) if !self.typed.is_empty() => {
                format!("Länge {} m, ⏎ übernimmt. ⌫ korrigiert.", self.typed)
            }
            (Tool::Wall, Some(_), _) => {
                "Endpunkt klicken oder Länge tippen. ⇧: nur rechte Winkel. Doppelklick oder Esc: fertig.".into()
            }
            (Tool::ArcWall, None, _) => "Bogenwand: Anfangspunkt klicken. Esc: zurück zum Auswählen.".into(),
            (Tool::ArcWall, Some(_), _) if self.arc_end.is_some() => {
                "Stich ziehen und klicken oder tippen, ⏎ übernimmt. Esc: Endpunkt neu.".into()
            }
            (Tool::ArcWall, Some(_), _) => "Bogenwand: Endpunkt klicken oder Sehne tippen.".into(),
            (Tool::Grid, None, _) => "Achse: Anfangspunkt klicken. Esc: zurück zum Auswählen.".into(),
            (Tool::Grid, Some(_), _) => "Achse: Endpunkt klicken. ⇧: nur rechte Winkel.".into(),
            (Tool::Column, _, _) => "Stütze 30/30: Mittelpunkt klicken. Esc: zurück zum Auswählen.".into(),
            (Tool::Door, _, _) => {
                "Tür: auf die Wand klicken, auf der Seite, zu der sie aufschlägt. Esc: zurück zum Auswählen.".into()
            }
            (Tool::Window, _, _) => "Fenster: auf die Wand klicken. Esc: zurück zum Auswählen.".into(),
            (Tool::SlabOpening, None, _) => {
                "Deckendurchbruch: erste Ecke in der Decke dieses Geschosses klicken. Esc: zurück.".into()
            }
            (Tool::SlabOpening, Some(_), _) => "Deckendurchbruch: gegenüberliegende Ecke klicken.".into(),
            (Tool::Line, None, _) => "Linie: Anfangspunkt klicken. Esc: zurück zum Auswählen.".into(),
            (Tool::Line, Some(_), _) => {
                "Nächsten Punkt klicken oder Länge tippen. ⇧: nur rechte Winkel. Doppelklick oder Esc: fertig.".into()
            }
            (Tool::Text, _, _) => "Text: Stelle klicken, den Text im Inspektor ändern. Esc: zurück.".into(),
            (Tool::CalibrateUnderlay, None, _) => {
                "Unterlage kalibrieren: Anfang einer bekannten Strecke auf der Unterlage klicken.".into()
            }
            (Tool::CalibrateUnderlay, Some(_), _) if self.arc_end.is_none() => "Ende der Strecke klicken.".into(),
            (Tool::CalibrateUnderlay, Some(_), _) if self.typed.is_empty() => {
                "Wahren Abstand in m tippen und ⏎. Esc: neu beginnen.".into()
            }
            (Tool::CalibrateUnderlay, Some(_), _) => format!("Abstand {} m, ⏎ übernimmt.", self.typed),
            (Tool::AlignUnderlay, None, _) => {
                "Unterlage ausrichten: zwei Punkte einer Linie klicken, die waagerecht oder senkrecht liegen soll."
                    .into()
            }
            (Tool::AlignUnderlay, Some(_), _) => "Zweiten Punkt der Linie klicken.".into(),
            (Tool::Space, _, _) => {
                "Raum: in den Bereich klicken, dort steht der Raumstempel. Esc: zurück zum Auswählen.".into()
            }
            (Tool::Move | Tool::Copy | Tool::Array | Tool::Rotate | Tool::Offset, _, _) if !self.typed.is_empty() => {
                format!("Eingabe {}, ⏎ übernimmt. ⌫ korrigiert.", self.typed)
            }
            (Tool::Move, None, _) => "Verschieben: Basispunkt klicken. Esc: abbrechen.".into(),
            (Tool::Move, Some(_), _) => {
                "Verschieben: Ziel klicken oder tippen (3,65 · 3,65<30 · @1,20;0,80) und ⏎. ⇧: nur rechte Winkel.".into()
            }
            (Tool::Copy, None, _) => "Kopieren: Basispunkt klicken. Esc: abbrechen.".into(),
            (Tool::Copy, Some(_), _) => "Kopieren: Ziel klicken oder Abstand tippen und ⏎.".into(),
            (Tool::Array, chain, _) => {
                let n = match self.series {
                    Some(entry::Entry::Count(n)) => format!("{n} Stück im Abstand"),
                    Some(entry::Entry::Divide(n)) => format!("{n} Stück bis zum Punkt"),
                    _ => "2 Stück".into(),
                };
                let step = if chain.is_some() { "Abstand klicken oder tippen" } else { "Basispunkt klicken" };
                format!("Reihe, {n}: {step}. „3x“ drei Stück im Abstand, „3/“ drei Stück bis zum Punkt.")
            }
            (Tool::Rotate, None, _) => {
                "Drehen: Winkel tippen und ⏎, oder Bezugsrichtung klicken. Leertaste: Drehpunkt an den Zeiger.".into()
            }
            (Tool::Rotate, Some(_), _) => "Drehen: Zielrichtung klicken. Esc: abbrechen.".into(),
            (Tool::Mirror, None, _) => "Spiegeln: ersten Punkt der Achse klicken.".into(),
            (Tool::Mirror, Some(_), _) => "Spiegeln: zweiten Punkt der Achse klicken. Als Kopie; mit ⇧ ohne Kopie.".into(),
            (Tool::Offset, _, _) => match self.offset_distance {
                None => "Versetzen: Abstand tippen und ⏎, dann Wand, Achse oder Linie anklicken.".into(),
                Some(d) => format!(
                    "Versetzen um {}: Wand, Achse oder Linie anklicken, Kopie zur Seite des Zeigers. Neuer Abstand: tippen.",
                    format_m(d)
                ),
            },
            (Tool::Align, _, _) if self.align_ref.is_none() => {
                "Ausrichten: Bezugslinie anklicken (Wandfläche, Achse oder Linie).".into()
            }
            (Tool::Align, _, _) => "Ausrichten: die Fläche anklicken, die an die Bezugslinie soll.".into(),
            (Tool::Trim, _, _) if self.picked.is_none() => {
                "Stutzen und Dehnen: erste Wand anklicken, auf dem Teil, der bleibt.".into()
            }
            (Tool::Trim, _, _) => "Stutzen und Dehnen: zweite Wand anklicken; beide enden in der Ecke.".into(),
            (Tool::Split, _, _) => "Teilen: auf die Wand klicken, wo sie geteilt wird.".into(),
            (Tool::Select, _, Some(Drag::Frame { start, current, .. })) if current.x < start.x => {
                "Rahmen nach links: wählt auch Angeschnittenes. ⇧: hinzufügen oder herausnehmen.".into()
            }
            (Tool::Select, _, Some(Drag::Frame { .. })) => {
                "Rahmen nach rechts: wählt, was ganz innen liegt. ⇧: hinzufügen oder herausnehmen.".into()
            }
            (Tool::Select, _, Some(Drag::Lasso { .. })) => "Lasso: wählt, was ganz innen liegt.".into(),
            (Tool::Select, _, Some(Drag::Selection { copy: true, .. })) => "Loslassen setzt die Kopie ab.".into(),
            (Tool::Select, _, Some(_)) => "Loslassen setzt ab. Esc bricht ab.".into(),
            (Tool::Select, _, None) if self.selection.is_empty() => {
                "Klick wählt, Ziehen zieht einen Rahmen, ⌥ ein Lasso, ⇥ blättert. WA Wand, DR Tür, WN Fenster, RM Raum. ⌘K: Befehle."
                    .into()
            }
            (Tool::Select, _, None) if self.dim_edit.is_some() => {
                format!("Neues Maß {} tippen und ⏎. Esc bricht ab.", self.typed)
            }
            (Tool::Select, _, None) => {
                "Ziehen verschiebt, mit ⌥ als Kopie; Maßzahl anklicken ändert den Abstand. Pfeile 1 cm. SA: Gleiche. ⌫ löscht."
                    .into()
            }
        }
    }

    /// Neue Auswahl; die alte kommt in den Verlauf (⌘←).
    fn set_selection(&mut self, new: Vec<ElementId>) {
        self.dim_edit = None;
        if new == self.selection {
            return;
        }
        if !self.selection.is_empty() {
            self.history.push(std::mem::take(&mut self.selection));
            if self.history.len() > 20 {
                self.history.remove(0);
            }
        }
        self.selection = new;
    }

    /// Ergebnis eines Rahmens oder Lassos: ersetzt die Auswahl, mit ⇧ kommt
    /// jedes gefundene Bauteil hinzu oder fällt heraus.
    fn select_found(&mut self, found: Vec<ElementId>, shift: bool) {
        if !shift {
            self.set_selection(found);
            return;
        }
        let mut new = self.selection.clone();
        for id in found {
            match new.iter().position(|s| *s == id) {
                Some(i) => {
                    new.remove(i);
                }
                None => new.push(id),
            }
        }
        self.set_selection(new);
    }

    /// Ein Buchstabe eines Zwei-Buchstaben-Kürzels (ADR 0044).
    fn shortcut_key(&mut self, doc: &mut Document, c: char) -> Response {
        self.shortcut.push(c.to_ascii_uppercase());
        let table = shortcuts();
        if let Some((_, command)) = table.iter().find(|(k, _)| *k == self.shortcut) {
            let command = command.clone();
            self.shortcut.clear();
            self.last_command = Some(command.clone());
            return self.run(doc, &command);
        }
        if !table.iter().any(|(k, _)| k.starts_with(self.shortcut.as_str())) {
            // Kein Kürzel: der letzte Buchstabe beginnt vielleicht ein neues.
            let last = self.shortcut.pop();
            self.shortcut.clear();
            if let Some(l) = last
                && table.iter().any(|(k, _)| k.starts_with(l))
            {
                self.shortcut.push(l);
            }
        }
        Response::redraw()
    }

    /// Leertaste bei gewählten Bauteilen (wie Revit): Wände drehen um, Türen
    /// schalten durch die vier Lagen von Aufschlag und Anschlag (ADR 0047).
    fn flip_with_space(&mut self, doc: &mut Document) -> Response {
        let project = doc.project();
        let has_walls = self.selection.iter().any(|id| project.wall(*id).is_some());
        let has_doors = self.selection.iter().any(|id| {
            matches!(project.elements.get(id), Some(Element::Opening(Opening { kind: OpeningKind::Door(_), .. })))
        });
        if has_walls {
            return self.flip_selection(doc);
        }
        if has_doors {
            // Bänder vorne links, hinten links, hinten rechts, vorne rechts: jede Taste dreht weiter.
            return self.change_doors(doc, "Tür drehen", |d| {
                let (side, at_end) = match (d.opens_to, d.hinge_at_end()) {
                    (WallSide::Left, false) => (WallSide::Left, true),
                    (WallSide::Left, true) => (WallSide::Right, true),
                    (WallSide::Right, true) => (WallSide::Right, false),
                    (WallSide::Right, false) => (WallSide::Left, false),
                };
                d.opens_to = side;
                d.hinge = if (side == WallSide::Left) == at_end { Handing::DinLeft } else { Handing::DinRight };
            });
        }
        Response::default()
    }

    /// Kopiert die Auswahl mit der Abbildung `t`; danach sind die Kopien gewählt.
    fn copy_selection(&mut self, doc: &mut Document, t: Transform, label: &str) -> Response {
        let ops = transform::copy_ops(doc.project(), &self.selection, &t, None);
        if ops.is_empty() {
            return Response::redraw();
        }
        let new: Vec<ElementId> = ops
            .iter()
            .filter_map(|op| match op {
                Operation::Insert { id, .. } => Some(*id),
                _ => None,
            })
            .collect();
        let result = doc.apply(Operation::Group { label: label.into(), ops });
        if result.is_ok() {
            self.set_selection(new);
        }
        Response::from_result(result)
    }

    /// Temporäre Maße, solange genau ein Bauteil gewählt ist und nichts gezogen wird.
    fn temp_dims_now(&self, project: &Project) -> Vec<tempdims::TempDim> {
        match (&self.selection[..], self.tool, &self.drag) {
            ([id], Tool::Select, None) => tempdims::temp_dims(project, self.storey, *id),
            _ => Vec::new(),
        }
    }

    /// Setzt das bearbeitete temporäre Maß auf den getippten Wert; das Bauteil rückt.
    fn apply_temp_dim(&mut self, doc: &mut Document) -> Response {
        let Some(i) = self.dim_edit.take() else { return Response::default() };
        let typed = std::mem::take(&mut self.typed);
        let Some(dim) = self.temp_dims_now(doc.project()).get(i).copied() else { return Response::redraw() };
        let Some(entry::Entry::Length(v)) = entry::parse(&typed) else {
            return Response { error: Some(format!("Eingabe nicht verstanden: {typed}")), ..Response::redraw() };
        };
        let d = v - dim.value;
        let changes = moved_elements(doc.project(), &self.selection, (dim.grow.0 * d, dim.grow.1 * d));
        Response::from_result(replace_all(doc, "Maß ändern", changes))
    }

    /// Startet einen Befehl an der Auswahl (MV, CO, RO, MM, AR); die Auswahl bleibt.
    fn start_modify(&mut self, project: &Project, tool: Tool) -> Response {
        if self.selection.is_empty() {
            let k = match tool {
                Tool::Move => "MV",
                Tool::Copy => "CO",
                Tool::Rotate => "RO",
                Tool::Mirror => "MM",
                _ => "AR",
            };
            return Response { error: Some(format!("Erst auswählen, dann {k}")), ..Response::redraw() };
        }
        self.tool = tool;
        self.end_chain();
        self.series = None;
        self.pivot = if tool == Tool::Rotate { self.selection_center(project) } else { None };
        Response::redraw()
    }

    /// Mitte des umschließenden Rechtecks der Auswahl, auf 1 cm gerundet.
    fn selection_center(&self, project: &Project) -> Option<Point2> {
        let points: Vec<[f64; 2]> = self.selection.iter().filter_map(|id| outline(project, *id)).flatten().collect();
        let (x0, x1) = points.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), p| (a.min(p[0]), b.max(p[0])));
        let (y0, y1) = points.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), p| (a.min(p[1]), b.max(p[1])));
        let r = |v: f64| (v / snap::ROUND).round() * snap::ROUND;
        (x0 <= x1).then(|| Point2::new(r((x0 + x1) / 2.0), r((y0 + y1) / 2.0)))
    }

    /// Abbildung, die der laufende Befehl mit dem Zielpunkt `p` ausführen würde.
    fn modify_transform(&self, p: Point2) -> Option<Transform> {
        match self.tool {
            Tool::Move | Tool::Copy | Tool::Array => {
                let b = self.chain_start?;
                Some(Transform::translation(p.x - b.x, p.y - b.y))
            }
            Tool::Rotate => {
                let (c, r) = (self.pivot?, self.chain_start?);
                let a = (p.y - c.y).atan2(p.x - c.x) - (r.y - c.y).atan2(r.x - c.x);
                let deg = (a.to_degrees() * 100.0).round() / 100.0;
                Some(Transform::rotation(c, deg))
            }
            Tool::Mirror => Transform::mirror(self.chain_start?, p),
            _ => None,
        }
    }

    /// Verschiebung aus einer getippten Eingabe: Länge in Richtung des Zeigers
    /// (oder des getippten Winkels), Länge mit Winkel oder relativ in x und y.
    fn typed_vector(&self, e: entry::Entry, from: Point2) -> Option<(f64, f64)> {
        match e {
            entry::Entry::Length(l) => {
                let (ux, uy) = match self.angle_lock {
                    Some(a) => (a.to_radians().cos(), a.to_radians().sin()),
                    None => {
                        let target = self.pointer.map_or(Point2::new(from.x + 1.0, from.y), |s| s.point);
                        let (dx, dy) = (target.x - from.x, target.y - from.y);
                        let len = dx.hypot(dy);
                        if len > 1e-9 { (dx / len, dy / len) } else { (1.0, 0.0) }
                    }
                };
                Some((ux * l, uy * l))
            }
            entry::Entry::Polar { length, angle } => {
                let (sin, cos) = angle.to_radians().sin_cos();
                Some((cos * length, sin * length))
            }
            entry::Entry::Offset { dx, dy } => Some((dx, dy)),
            _ => None,
        }
    }

    /// ⏎ in Verschieben, Kopieren, Reihe, Drehen und Versetzen.
    fn modify_enter(&mut self, doc: &mut Document) -> Response {
        if self.typed.is_empty() {
            return Response::default();
        }
        let typed = std::mem::take(&mut self.typed);
        let parsed = entry::parse(&typed);
        let not_understood =
            || Response { error: Some(format!("Eingabe nicht verstanden: {typed}")), ..Response::redraw() };
        match (self.tool, parsed) {
            (Tool::Array, Some(e @ (entry::Entry::Count(_) | entry::Entry::Divide(_)))) => {
                self.series = Some(e);
                Response::redraw()
            }
            (Tool::Offset, Some(entry::Entry::Length(d))) => {
                self.offset_distance = Some(d.abs());
                Response::redraw()
            }
            (Tool::Rotate, Some(entry::Entry::Length(a) | entry::Entry::Angle(a))) => match self.pivot {
                Some(c) => self.finish_modify(doc, Transform::rotation(c, a), false),
                None => not_understood(),
            },
            (Tool::Move | Tool::Copy | Tool::Array, Some(e)) => {
                let Some(base) = self.chain_start else {
                    return Response { error: Some("Erst den Basispunkt klicken".into()), ..Response::redraw() };
                };
                match self.typed_vector(e, base) {
                    Some((dx, dy)) => self.finish_modify(doc, Transform::translation(dx, dy), false),
                    None => not_understood(),
                }
            }
            _ => not_understood(),
        }
    }

    /// Führt Verschieben, Kopieren, Reihe, Drehen oder Spiegeln aus; danach ist
    /// das Ergebnis gewählt und das Auswahlwerkzeug aktiv.
    fn finish_modify(&mut self, doc: &mut Document, t: Transform, shift: bool) -> Response {
        let tool = self.tool;
        self.tool = Tool::Select;
        self.end_chain();
        self.pivot = None;
        match tool {
            Tool::Move => {
                let o = t.point(Point2::new(0.0, 0.0));
                let changes = moved_elements(doc.project(), &self.selection, (o.x, o.y));
                Response::from_result(replace_all(doc, "Verschieben", changes))
            }
            Tool::Copy => self.copy_selection(doc, t, "Kopieren"),
            Tool::Mirror if !shift => self.copy_selection(doc, t, "Spiegeln"),
            Tool::Rotate | Tool::Mirror => {
                let ops = transform::transform_ops(doc.project(), &self.selection, &t);
                let label = if tool == Tool::Rotate { "Drehen" } else { "Spiegeln" };
                Response::from_result(doc.apply(Operation::Group { label: label.into(), ops }))
            }
            Tool::Array => {
                let o = t.point(Point2::new(0.0, 0.0));
                self.array(doc, (o.x, o.y))
            }
            _ => Response::redraw(),
        }
    }

    /// Kopien der Auswahl in gleichen Abständen: „3x“ drei Stück im Abstand
    /// `step`, „3/“ drei Stück, das letzte bei `step`. Gezählt wird das Original mit (wie Revit).
    fn array(&mut self, doc: &mut Document, step: (f64, f64)) -> Response {
        let (n, step) = match self.series.take() {
            Some(entry::Entry::Divide(n)) if n >= 2 => (n, (step.0 / (n - 1) as f64, step.1 / (n - 1) as f64)),
            Some(entry::Entry::Count(n)) => (n, step),
            _ => (2, step),
        };
        let mut scratch = doc.project().clone();
        let mut ops = Vec::new();
        let mut new = Vec::new();
        for i in 1..n {
            let t = Transform::translation(step.0 * i as f64, step.1 * i as f64);
            for op in transform::copy_ops(&scratch, &self.selection, &t, None) {
                if let Operation::Insert { id, element } = &op {
                    scratch.elements.insert(*id, element.clone());
                    new.push(*id);
                }
                ops.push(op);
            }
        }
        if ops.is_empty() {
            return Response::redraw();
        }
        let result = doc.apply(Operation::Group { label: "Reihe".into(), ops });
        if result.is_ok() {
            self.set_selection(new);
        }
        Response::from_result(result)
    }

    /// Element und Verschiebung für Versetzen: die Wand, Achse oder Linie unter
    /// dem Zeiger, zur Seite des Zeigers um den getippten Abstand.
    fn offset_target(&self, project: &Project, p: Point2, tolerance: f64) -> Option<(ElementId, (f64, f64))> {
        let d = self.offset_distance?;
        let (id, mut line) = modify::line_at(project, self.storey, p, tolerance)?;
        // Bei Wänden zählt die Mitte zwischen den Flächen.
        if let Some(w) = project.wall(id) {
            let (left, right) = geometry::faces(project, w);
            line.point = w.point_at(0.0, (left + right) / 2.0);
        }
        let (nx, ny) = (-line.dir.1, line.dir.0);
        let t = (p.x - line.point.x) * nx + (p.y - line.point.y) * ny;
        let side = if t >= 0.0 { 1.0 } else { -1.0 };
        Some((id, (nx * d * side, ny * d * side)))
    }

    fn set_tool(&mut self, tool: Tool) -> Response {
        self.tool = tool;
        self.pivot = None;
        self.series = None;
        self.selection.clear();
        self.end_chain();
        Response::redraw()
    }

    fn end_chain(&mut self) {
        self.chain_start = None;
        self.arc_end = None;
        self.typed.clear();
        self.angle_lock = None;
        self.picked = None;
        self.align_ref = None;
    }

    fn new_wall(&self, start: Point2, end: Point2) -> Option<Wall> {
        Some(Wall {
            phase: Default::default(),
            storey: self.storey,
            wall_type: self.wall_type?,
            start,
            end,
            alignment: self.alignment,
            base_offset: 0.0,
            top: Top::Storey { offset: 0.0 },
            sagitta: 0.0,
        })
    }

    fn add_wall(&mut self, doc: &mut Document, start: Point2, end: Point2) -> Response {
        if start.distance(end) < 0.01 {
            return Response::redraw();
        }
        let Some(wall) = self.new_wall(start, end) else {
            return Response { error: Some("Im Projekt gibt es noch keinen Wandaufbau".into()), ..Default::default() };
        };
        let result = doc.insert(Element::Wall(wall)).map(|_| ());
        if result.is_ok() {
            self.chain_start = Some(end);
            self.angle_lock = None;
        }
        Response::from_result(result)
    }

    /// Fangpunkt wie [`snap::snap`], dazu Endpunkte in Unterlagen, wenn kein
    /// Wandende oder Achsschnittpunkt näher liegt.
    #[allow(clippy::too_many_arguments)]
    fn snap(
        &self,
        project: &Project,
        storey: ElementId,
        p: Point2,
        tolerance: f64,
        from: Option<Point2>,
        orthogonal: bool,
        exclude: &[ElementId],
    ) -> Snap {
        let s = snap::snap(project, storey, p, tolerance, from, orthogonal, exclude);
        if matches!(s.kind, SnapKind::Endpoint | SnapKind::GridIntersection) {
            return s;
        }
        self.underlay_points
            .iter()
            .map(|q| (*q, q.distance(p)))
            .filter(|(_, d)| *d <= tolerance)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map_or(s, |(q, _)| Snap { point: q, kind: SnapKind::Underlay })
    }

    /// Die Unterlage, auf die Kalibrieren und Ausrichten wirken: die gewählte,
    /// sonst die zuletzt angelegte des Geschosses.
    fn underlay(&self, project: &Project) -> Option<ElementId> {
        self.selection
            .iter()
            .copied()
            .find(|id| matches!(project.elements.get(id), Some(Element::Underlay(_))))
            .or_else(|| project.underlays_on(self.storey).map(|(id, _)| id).last())
    }

    fn change_underlay(
        &mut self,
        doc: &mut Document,
        id: ElementId,
        label: &str,
        change: impl Fn(&lot_model::Underlay) -> lot_model::Underlay,
    ) -> Response {
        let Some(Element::Underlay(u)) = doc.project().elements.get(&id) else { return Response::redraw() };
        let new = change(u);
        Response::from_result(replace_all(doc, label, vec![(id, Element::Underlay(new))]))
    }

    fn add_line(&mut self, doc: &mut Document, start: Point2, end: Point2) -> Response {
        if start.distance(end) < 0.01 {
            return Response::redraw();
        }
        let line = Annotation {
            view: AnnotationView::FloorPlan { storey: self.storey },
            shape: AnnotationShape::Polyline(vec![start, end]),
            pen: 0.25,
            dashed: false,
        };
        let result = doc.insert(Element::Annotation(line)).map(|_| ());
        if result.is_ok() {
            self.chain_start = Some(end);
            self.angle_lock = None;
        }
        Response::from_result(result)
    }

    fn add_arc_wall(&mut self, doc: &mut Document, start: Point2, end: Point2, sagitta: f64) -> Response {
        let Some(mut wall) = self.new_wall(start, end) else {
            return Response { error: Some("Im Projekt gibt es noch keinen Wandaufbau".into()), ..Default::default() };
        };
        wall.sagitta = sagitta;
        let result = doc.insert(Element::Wall(wall)).map(|_| ());
        if result.is_ok() {
            self.chain_start = Some(end);
            self.arc_end = None;
        }
        Response::from_result(result)
    }

    /// Neuer Raum mit der nächsten freien Nummer im Geschoss. `skip` überspringt
    /// Nummern, wenn mehrere Räume auf einmal entstehen.
    fn new_space(&self, project: &Project, position: Point2, skip: u32) -> Space {
        let below_ground = project.storey(self.storey).is_some_and(|s| s.elevation < -0.01);
        Space {
            phase: Default::default(),
            storey: self.storey,
            position,
            number: next_space_number(project, self.storey, skip),
            name: "Raum".into(),
            usage: lot_model::SpaceUsage::default(),
            living_space: !below_ground,
            unit: String::new(),
        }
    }

    fn detect_spaces(&mut self, doc: &mut Document) -> Response {
        let project = doc.project();
        let finder = lot_geom::spaces::SpaceFinder::new(project, self.storey);
        let ops: Vec<Operation> = finder
            .unclaimed(project, self.storey)
            .into_iter()
            .filter_map(lot_geom::spaces::interior_point)
            .enumerate()
            .map(|(i, p)| {
                let p = Point2::new((p[0] / 0.01).round() * 0.01, (p[1] / 0.01).round() * 0.01);
                Operation::Insert {
                    id: ElementId(project.next_id().0 + i as u64),
                    element: Element::Space(self.new_space(project, p, i as u32)),
                }
            })
            .collect();
        if ops.is_empty() {
            return Response {
                error: Some("Kein umschlossener Bereich ohne Raum gefunden".into()),
                ..Response::redraw()
            };
        }
        Response::from_result(doc.apply(Operation::Group { label: "Räume erkennen".into(), ops }))
    }

    fn opening_title(&self) -> &'static str {
        if self.tool == Tool::Door { "Eine Tür" } else { "Ein Fenster" }
    }

    /// Tür oder Fenster an der Stelle `p` auf einer Wand, mittig zum Zeiger, in
    /// üblichen Rohbaumaßen nach DIN 18100:1983-10 (Vorschlag): Tür 1,01 × 2,135 m,
    /// Fenster 1,26 × 1,385 m mit Brüstung 0,885 m. Die Tür schlägt zur Seite
    /// des Zeigers auf, die Bänder sitzen am näheren Wandende.
    fn placed_opening(&self, project: &Project, p: Point2, tolerance: f64) -> Option<Opening> {
        let host = geometry::wall_at(project, self.storey, p, tolerance)?;
        let wall = project.wall(host)?;
        let (width, height, sill) = if self.tool == Tool::Door { (1.01, 2.135, 0.0) } else { (1.26, 1.385, 0.885) };
        let length = wall.length();
        if length < width + 0.02 {
            return None;
        }
        let (s, t) = wall.locate(p);
        let offset = ((s - width / 2.0) / 0.01).round() * 0.01;
        let offset = offset.clamp(0.0, length - width);
        let kind = if self.tool == Tool::Door {
            let (left, right) = geometry::faces(project, wall);
            let opens_to = if t > (left + right) / 2.0 { WallSide::Left } else { WallSide::Right };
            let mut door = lot_model::Door { opens_to, hinge: Handing::DinLeft };
            if door.hinge_at_end() != (s > length / 2.0) {
                door.hinge = Handing::DinRight;
            }
            OpeningKind::Door(door)
        } else {
            OpeningKind::Window
        };
        Some(Opening { phase: Default::default(), host, offset, width, height, sill_height: sill, kind })
    }

    /// Maßketten außen an den gewählten geraden Wänden (Vorschlag): in 1,00 m
    /// Abstand Wandenden und Laibungen, in 1,70 m das Gesamtmaß. Ohne
    /// Öffnungen nur das Gesamtmaß in 1,00 m.
    fn dimension_walls(&mut self, doc: &mut Document) -> Response {
        use lot_model::{DimRef, DimensionChain, WallEnd};
        let project = doc.project();
        let mut ops = Vec::new();
        let mut next = project.next_id().0;
        for id in &self.selection {
            let Some(wall) = project.wall(*id) else { continue };
            if wall.is_curved() {
                return Response {
                    error: Some("Bogenwände lassen sich noch nicht bemaßen".into()),
                    ..Response::redraw()
                };
            }
            let (left, _) = geometry::faces(project, wall);
            let ends = |end| DimRef::WallEnd { wall: *id, end, side: WallSide::Left };
            let mut openings: Vec<(ElementId, f64)> =
                project.openings_in(*id).map(|(oid, o)| (oid, o.offset)).collect();
            openings.sort_by(|a, b| a.1.total_cmp(&b.1));
            let mut chains = Vec::new();
            if !openings.is_empty() {
                let mut refs = vec![ends(WallEnd::Start)];
                for (oid, _) in &openings {
                    refs.push(DimRef::Jamb { opening: *oid, end: WallEnd::Start, side: WallSide::Left });
                    refs.push(DimRef::Jamb { opening: *oid, end: WallEnd::End, side: WallSide::Left });
                }
                refs.push(ends(WallEnd::End));
                chains.push((1.0, refs));
            }
            let total = if openings.is_empty() { 1.0 } else { 1.7 };
            chains.push((total, vec![ends(WallEnd::Start), ends(WallEnd::End)]));
            let (dx, dy) = wall.direction();
            for (distance, refs) in chains {
                let element = Element::DimensionChain(DimensionChain {
                    storey: self.storey,
                    position: wall.point_at(0.0, left + distance),
                    direction: Point2::new(dx, dy),
                    refs,
                });
                ops.push(Operation::Insert { id: ElementId(next), element });
                next += 1;
            }
        }
        if ops.is_empty() {
            return Response { error: Some("Keine Wand gewählt".into()), ..Response::redraw() };
        }
        Response::from_result(doc.apply(Operation::Group { label: "Bemaßen".into(), ops }))
    }

    /// Decke mit Oberkante auf OKFF des aktuellen Geschosses. Sie liegt auf den
    /// Wänden des Geschosses darunter und folgt der Außenkante ihrer Tragschicht;
    /// im untersten Geschoss wird sie zur Bodenplatte unter den eigenen Wänden.
    fn slab_from_walls(&mut self, doc: &mut Document) -> Response {
        use lot_model::{Slab, SlabKind};
        let project = doc.project();
        let storeys = project.storeys();
        let Some(index) = storeys.iter().position(|(id, _)| *id == self.storey) else { return Response::default() };
        let (support, kind) =
            if index == 0 { (self.storey, SlabKind::Base) } else { (storeys[index - 1].0, SlabKind::Floor) };
        let types: Vec<_> = project.slab_types().collect();
        let base_type = types.iter().find(|(_, t)| t.name.contains("Bodenplatte")).or(types.last()).map(|(id, _)| *id);
        let floor_type =
            types.iter().find(|(_, t)| !t.name.contains("Bodenplatte")).or(types.first()).map(|(id, _)| *id);
        let Some(slab_type) = (if kind == SlabKind::Base { base_type } else { floor_type }) else {
            return Response {
                error: Some("Im Projekt gibt es noch keinen Deckenaufbau".into()),
                ..Response::redraw()
            };
        };
        let outlines = lot_geom::slabs::slab_outlines(project, support);
        if outlines.is_empty() {
            return Response {
                error: Some("Für eine Decke fehlen geschlossene Wände im Geschoss darunter".into()),
                ..Response::redraw()
            };
        }
        let first = project.next_id().0;
        let ops = outlines
            .into_iter()
            .enumerate()
            .map(|(i, ring)| Operation::Insert {
                id: ElementId(first + i as u64),
                element: Element::Slab(Slab {
                    phase: Default::default(),
                    storey: self.storey,
                    slab_type,
                    outline: ring.into_iter().map(|p| Point2::new(p[0], p[1])).collect(),
                    top_offset: 0.0,
                    kind,
                }),
            })
            .collect();
        Response::from_result(doc.apply(Operation::Group { label: "Decke aus Wänden".into(), ops }))
    }

    /// Rechteckiger Durchbruch in der Decke des aktuellen Geschosses, in der die erste Ecke liegt.
    fn add_slab_opening(&mut self, doc: &mut Document, a: Point2, b: Point2) -> Response {
        if (a.x - b.x).abs() < 0.05 || (a.y - b.y).abs() < 0.05 {
            return Response::redraw();
        }
        let project = doc.project();
        let host = project.slabs_on(self.storey).find(|(_, s)| {
            let ring: Vec<[f64; 2]> = s.outline.iter().map(|p| [p.x, p.y]).collect();
            lot_geom::Area::from_ring(&ring).contains([a.x, a.y])
        });
        let Some((host, _)) = host else {
            return Response {
                error: Some("Hier liegt keine Decke dieses Geschosses (Werkzeug, Decke aus Wänden)".into()),
                ..Response::redraw()
            };
        };
        let outline = vec![a, Point2::new(b.x, a.y), b, Point2::new(a.x, b.y)];
        let opening = lot_model::SlabOpening { phase: Default::default(), host, outline };
        Response::from_result(doc.insert(Element::SlabOpening(opening)).map(|_| ()))
    }

    /// Satteldach über dem aktuellen Geschoss (Vorschlag): Umriss an den
    /// Außenkanten der Wände, Traufen an den Seiten parallel zur längsten Kante,
    /// 40° Neigung, 50 cm Traufüberstand, 30 cm am Ortgang. Die Dachhaut liegt
    /// so hoch, dass die Unterseite an der Außenkante auf der höchsten
    /// Außenwand mit fester Höhe (Kniestock) aufliegt, sonst auf Geschosshöhe.
    /// Wände bis zum nächsten Geschoss reichen danach bis unter das Dach.
    fn gable_roof_from_walls(&mut self, doc: &mut Document) -> Response {
        use lot_model::{Roof, RoofEdge, Top};
        let project = doc.project();
        let Some(storey) = project.storey(self.storey) else { return Response::default() };
        let Some(roof_type) = project
            .slab_types()
            .find(|(_, t)| t.name.contains("dach"))
            .or_else(|| project.slab_types().last())
            .map(|(id, _)| id)
        else {
            return Response { error: Some("Im Projekt gibt es noch keinen Dachaufbau".into()), ..Response::redraw() };
        };
        let finder = lot_geom::spaces::SpaceFinder::new(project, self.storey);
        let outer: Vec<_> = finder.footprint.rings().into_iter().filter(|r| lot_geom::signed_area(r) > 0.0).collect();
        let [ring] = &outer[..] else {
            return Response {
                error: Some("Für ein Dach braucht das Geschoss genau einen geschlossenen Umriss aus Wänden".into()),
                ..Response::redraw()
            };
        };
        let outline: Vec<Point2> = ring.iter().map(|p| Point2::new(p[0], p[1])).collect();
        let n = outline.len();
        let dir = |i: usize| {
            let (a, b) = (outline[i], outline[(i + 1) % n]);
            let len = a.distance(b).max(1e-12);
            ((b.x - a.x) / len, (b.y - a.y) / len, len)
        };
        let longest = (0..n).max_by(|a, b| dir(*a).2.total_cmp(&dir(*b).2)).unwrap_or(0);
        let (lx, ly, _) = dir(longest);
        let slope: f64 = 40.0;
        let edges: Vec<RoofEdge> = (0..n)
            .map(|i| {
                let (x, y, _) = dir(i);
                if (x * ly - y * lx).abs() < 0.05 {
                    RoofEdge::Eave { slope, overhang: 0.5 }
                } else {
                    RoofEdge::Verge { overhang: 0.3 }
                }
            })
            .collect();
        let knee = project
            .walls_on(self.storey)
            .filter_map(|(_, w)| match w.top {
                Top::Height(h) => Some(h + w.base_offset),
                _ => None,
            })
            .fold(f64::NEG_INFINITY, f64::max);
        let knee = if knee.is_finite() { knee } else { storey.height };
        let thickness = project.slab_type(roof_type).map_or(0.0, |t| t.thickness());
        let roof = Roof {
            phase: Default::default(),
            storey: self.storey,
            roof_type,
            outline,
            edges,
            eave_height: knee + thickness / slope.to_radians().cos(),
        };
        let mut ops = vec![Operation::Insert { id: project.next_id(), element: Element::Roof(roof) }];
        for (id, w) in project.walls_on(self.storey) {
            if let Top::Storey { offset } = w.top {
                let mut n = w.clone();
                n.top = Top::Roof { offset };
                ops.push(Operation::Replace { id, old: Element::Wall(w.clone()), new: Element::Wall(n) });
            }
        }
        Response::from_result(doc.apply(Operation::Group { label: "Satteldach".into(), ops }))
    }

    fn change_doors(&mut self, doc: &mut Document, label: &str, change: impl Fn(&mut lot_model::Door)) -> Response {
        let changes: Vec<(ElementId, Element)> = self
            .selection
            .iter()
            .filter_map(|id| match doc.project().elements.get(id) {
                Some(Element::Opening(o)) => match o.kind {
                    OpeningKind::Door(mut d) => {
                        change(&mut d);
                        Some((*id, Element::Opening(Opening { kind: OpeningKind::Door(d), ..o.clone() })))
                    }
                    _ => None,
                },
                _ => None,
            })
            .collect();
        if changes.is_empty() {
            return Response { error: Some("Keine Tür gewählt".into()), ..Response::redraw() };
        }
        Response::from_result(replace_all(doc, label, changes))
    }

    fn change_selected(&mut self, doc: &mut Document, label: &str, change: impl Fn(&mut Wall)) -> Response {
        if self.selection.is_empty() {
            return Response::redraw();
        }
        let changes: Vec<(ElementId, Element)> = self
            .selection
            .iter()
            .filter_map(|id| {
                let mut w = doc.project().wall(*id)?.clone();
                change(&mut w);
                Some((*id, Element::Wall(w)))
            })
            .collect();
        Response::from_result(replace_all(doc, label, changes))
    }

    fn delete_selection(&mut self, doc: &mut Document) -> Response {
        if self.selection.is_empty() {
            return Response::default();
        }
        let project = doc.project();
        let mut ops = Vec::new();
        for id in &self.selection {
            for (oid, _) in project.openings_in(*id) {
                ops.push(Operation::Delete { id: oid, element: project.elements[&oid].clone() });
            }
        }
        for id in &self.selection {
            if let Some(e) = project.elements.get(id) {
                ops.push(Operation::Delete { id: *id, element: e.clone() });
            }
        }
        let result = doc.apply(Operation::Group { label: "Löschen".into(), ops });
        if result.is_ok() {
            self.selection.clear();
        }
        Response::from_result(result)
    }

    fn flip_selection(&mut self, doc: &mut Document) -> Response {
        let project = doc.project();
        let mut ops = Vec::new();
        for id in &self.selection {
            let Some(w) = project.wall(*id) else { continue };
            let mut n = w.clone();
            std::mem::swap(&mut n.start, &mut n.end);
            // Der Bogen bleibt, wo er ist: links und rechts tauschen auch hier.
            n.sagitta = -w.sagitta;
            ops.push(Operation::Replace { id: *id, old: Element::Wall(w.clone()), new: Element::Wall(n) });
            // Öffnungen bleiben an ihrem Ort: Abstand nun vom anderen Ende.
            for (oid, o) in project.openings_in(*id) {
                let mut m = o.clone();
                m.offset = w.length() - o.offset - o.width;
                ops.push(Operation::Replace { id: oid, old: Element::Opening(o.clone()), new: Element::Opening(m) });
            }
        }
        if ops.is_empty() {
            return Response::default();
        }
        Response::from_result(doc.apply(Operation::Group { label: "Wand umdrehen".into(), ops }))
    }
}

/// Zwei-Buchstaben-Kürzel wie in Revit (ADR 0044), dazu eigene für
/// Werkzeuge, die Revit so nicht hat (BW, DB, KA).
pub fn shortcuts() -> Vec<(&'static str, Command)> {
    vec![
        ("MD", Command::SelectTool),
        ("WA", Command::WallTool),
        ("BW", Command::ArcWallTool),
        ("GR", Command::GridTool),
        ("CL", Command::ColumnTool),
        ("RM", Command::SpaceTool),
        ("DR", Command::DoorTool),
        ("WN", Command::WindowTool),
        ("DB", Command::SlabOpeningTool),
        ("LI", Command::LineTool),
        ("TX", Command::TextTool),
        ("KA", Command::CalibrateUnderlayTool),
        ("DI", Command::DimensionWalls),
        ("SA", Command::SelectSimilar),
        ("DE", Command::DeleteSelection),
        ("MV", Command::MoveTool),
        ("CO", Command::CopyTool),
        ("CC", Command::CopyTool),
        ("RO", Command::RotateTool),
        ("MM", Command::MirrorTool),
        ("AR", Command::ArrayTool),
        ("OF", Command::OffsetTool),
        ("AL", Command::AlignTool),
        ("TR", Command::TrimTool),
        ("SL", Command::SplitTool),
    ]
}

/// Kurzer Name eines Befehls für den Hinweis zu Kürzeln.
fn command_title(command: &Command) -> &'static str {
    match command {
        Command::SelectTool => "Auswählen",
        Command::WallTool => "Wand",
        Command::ArcWallTool => "Bogenwand",
        Command::GridTool => "Achse",
        Command::ColumnTool => "Stütze",
        Command::SpaceTool => "Raum",
        Command::DoorTool => "Tür",
        Command::WindowTool => "Fenster",
        Command::SlabOpeningTool => "Deckendurchbruch",
        Command::LineTool => "Linie",
        Command::TextTool => "Text",
        Command::CalibrateUnderlayTool => "Unterlage kalibrieren",
        Command::DimensionWalls => "Bemaßen",
        Command::SelectSimilar => "Gleiche wählen",
        Command::DeleteSelection => "Löschen",
        Command::MoveTool => "Verschieben",
        Command::CopyTool => "Kopieren",
        Command::RotateTool => "Drehen",
        Command::MirrorTool => "Spiegeln",
        Command::ArrayTool => "Reihe",
        Command::OffsetTool => "Versetzen",
        Command::AlignTool => "Ausrichten",
        Command::TrimTool => "Stutzen und Dehnen",
        Command::SplitTool => "Teilen",
        _ => "",
    }
}

/// Kürzel eines Befehls zur Anzeige, sonst leer.
fn shortcut_of(command: &Command) -> &'static str {
    shortcuts().into_iter().find(|(_, c)| c == command).map_or("", |(k, _)| k)
}

/// Auswahl nach einem ⇧-Klick: Ist alles schon gewählt, fällt es heraus, sonst kommt es hinzu.
fn toggled(selection: &[ElementId], ids: &[ElementId]) -> Vec<ElementId> {
    if ids.iter().all(|id| selection.contains(id)) {
        selection.iter().copied().filter(|s| !ids.contains(s)).collect()
    } else {
        let mut v = selection.to_vec();
        v.extend(ids.iter().copied().filter(|id| !selection.contains(id)));
        v
    }
}

/// Ersetzt mehrere Elemente als eine Operation.
fn replace_all(doc: &mut Document, label: &str, changes: Vec<(ElementId, Element)>) -> Result<(), ModelError> {
    let project = doc.project();
    let ops: Vec<Operation> = changes
        .into_iter()
        .filter_map(|(id, new)| {
            let old = project.elements.get(&id)?.clone();
            (old != new).then_some(Operation::Replace { id, old, new })
        })
        .collect();
    if ops.is_empty() {
        return Ok(());
    }
    doc.apply(Operation::Group { label: label.into(), ops })
}

/// Nächste freie Achsbezeichnung: senkrechte Achsen mit Zahlen (1, 2, ...),
/// waagerechte mit Buchstaben (A, B, ...).
pub(crate) fn next_grid_label(project: &Project, a: Point2, b: Point2) -> String {
    let used: Vec<String> = project.grid_lines().map(|(_, g)| g.label.clone()).collect();
    let vertical = (b.y - a.y).abs() > (b.x - a.x).abs();
    if vertical {
        (1..).map(|n: u32| n.to_string()).find(|l| !used.contains(l)).unwrap_or_default()
    } else {
        ('A'..='Z').map(String::from).find(|l| !used.contains(l)).unwrap_or_else(|| "?".into())
    }
}

/// Stich für eine Bogenwand von `start` nach `end` durch den Zeiger `p`: sein
/// Abstand von der Sehne (links positiv), auf 1 cm gerundet, höchstens ein Halbkreis.
fn sagitta_from(start: Point2, end: Point2, p: Point2) -> f64 {
    let chord = start.distance(end);
    if chord < 1e-9 {
        return 0.0;
    }
    let (dx, dy) = ((end.x - start.x) / chord, (end.y - start.y) / chord);
    let t = -(p.x - start.x) * dy + (p.y - start.y) * dx;
    ((t / snap::ROUND).round() * snap::ROUND).clamp(-chord / 2.0, chord / 2.0)
}

/// Nächste Raumnummer im Geschoss: Ebene, Punkt, laufende Nummer, etwa „0.03“
/// im Erdgeschoss, „1.01“ im ersten Obergeschoss, „-1.02“ im Keller. Die Ebene
/// zählt ab dem untersten Geschoss mit OKFF auf oder über ±0,00.
pub(crate) fn next_space_number(project: &Project, storey: ElementId, skip: u32) -> String {
    let storeys = project.storeys();
    let ground = storeys.iter().position(|(_, s)| s.elevation >= -0.01).unwrap_or(0) as i64;
    let level = storeys.iter().position(|(id, _)| *id == storey).map_or(0, |i| i as i64 - ground);
    let prefix = format!("{level}.");
    let highest = project
        .spaces_on(storey)
        .filter_map(|(_, r)| r.number.strip_prefix(&prefix)?.parse::<u32>().ok())
        .max()
        .unwrap_or(0);
    format!("{prefix}{:02}", highest + 1 + skip)
}

/// Getippte Länge in Metern, Komma oder Punkt als Dezimalzeichen.
fn parse_length(text: &str) -> Option<f64> {
    let v: f64 = text.replace(',', ".").parse().ok()?;
    (v > 0.0).then_some(v)
}

/// Länge im deutschen Format, „4,50 m“.
pub fn format_m(v: f64) -> String {
    format!("{v:.2} m").replace('.', ",")
}

#[cfg(test)]
mod tests;
