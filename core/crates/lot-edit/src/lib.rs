//! Bearbeiten im Grundriss: Werkzeuge, Fangen, Auswahl und Befehle.
//!
//! Plattformunabhängig und ohne Fenster testbar. Die App reicht Zeiger- und
//! Tastenereignisse in Modellkoordinaten weiter; jede Änderung am Modell ist
//! eine Operation (CLAUDE.md, Regel 7), Verschieben mit angeschlossenen
//! Wänden eine Gruppe, die ein Undo zurücknimmt.

mod command;
pub mod geometry;
pub mod snap;

pub use command::{Command, CommandInfo};
use geometry::{
    End, as_elements, column_outline, element_at, end_point, footprint, moved_elements, moved_endpoint, outline,
};
use lot_model::{
    Column, ColumnProfile, Document, Element, ElementId, GridLine, Handing, ModelError, Opening, OpeningKind,
    Operation, Point2, Project, Room, Storey, Top, Wall, WallAlignment, WallSide,
};
pub use snap::{Snap, SnapKind};

/// Zeigerereignis in Modellkoordinaten.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Input {
    pub point: Point2,
    /// Fangradius in Metern (aus Pixeln und Zoom).
    pub tolerance: f64,
    /// Umschalttaste: beim Zeichnen nur rechte Winkel, bei der Auswahl hinzufügen.
    pub shift: bool,
}

/// Tasten, die der Editor versteht.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Key {
    Escape,
    Enter,
    Delete,
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
    Room,
    /// Tür: ein Klick auf die Wand, zur Seite, zu der sie aufschlägt
    Door,
    /// Fenster: ein Klick auf die Wand
    Window,
}

#[derive(Debug, Clone, PartialEq)]
enum Drag {
    /// Die ganze Auswahl verschieben.
    Selection {
        start: Point2,
        current: Point2,
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
}

pub struct Editor {
    pub storey: ElementId,
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
}

impl Editor {
    pub fn new(project: &Project) -> Self {
        let storey =
            project.storeys().iter().find(|(_, s)| s.elevation >= 0.0).or(project.storeys().first()).map(|(id, _)| *id);
        Self {
            storey: storey.unwrap_or(ElementId(0)),
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
        }
    }

    /// Passt den Zustand an ein geändertes Modell an (Undo, Laden): Verschwundene
    /// Elemente fallen aus der Auswahl, ein fehlendes Geschoss wird ersetzt.
    pub fn sync(&mut self, project: &Project) {
        self.selection.retain(|id| project.elements.contains_key(id));
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
            Tool::Wall | Tool::ArcWall | Tool::Grid | Tool::Column => {
                self.pointer = Some(snap::snap(
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
            Tool::Door | Tool::Window => {
                self.pointer = Some(Snap { point: input.point, kind: SnapKind::Free });
                self.hover_tolerance = input.tolerance;
                Response::redraw()
            }
            Tool::Room | Tool::Select => Response::default(),
        }
    }

    pub fn pointer_down(&mut self, doc: &mut Document, input: Input, clicks: u32) -> Response {
        match self.tool {
            Tool::Column => {
                let p = snap::snap(doc.project(), self.storey, input.point, input.tolerance, None, false, &[]).point;
                let column = Column {
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
            Tool::Room => {
                let p = Point2::new((input.point.x / 0.01).round() * 0.01, (input.point.y / 0.01).round() * 0.01);
                let room = self.new_room(doc.project(), p, 0);
                Response::from_result(doc.insert(Element::Room(room)).map(|_| ()))
            }
            Tool::Grid => {
                let p = snap::snap(
                    doc.project(),
                    self.storey,
                    input.point,
                    input.tolerance,
                    self.chain_start,
                    input.shift,
                    &[],
                )
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
                let p = snap::snap(
                    doc.project(),
                    self.storey,
                    input.point,
                    input.tolerance,
                    self.chain_start,
                    input.shift,
                    &[],
                )
                .point;
                self.pointer = None;
                match self.chain_start {
                    None => self.chain_start = Some(p),
                    Some(start) if start.distance(p) >= 0.01 => self.arc_end = Some(p),
                    Some(_) => {}
                }
                Response::redraw()
            }
            Tool::Wall => {
                if clicks >= 2 {
                    self.end_chain();
                    return Response::redraw();
                }
                let p = snap::snap(
                    doc.project(),
                    self.storey,
                    input.point,
                    input.tolerance,
                    self.chain_start,
                    input.shift,
                    &[],
                )
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
                match element_at(project, self.storey, input.point, input.tolerance) {
                    Some(id) => {
                        if input.shift {
                            if let Some(pos) = self.selection.iter().position(|s| *s == id) {
                                self.selection.remove(pos);
                            } else {
                                self.selection.push(id);
                            }
                        } else if !self.selection.contains(&id) {
                            self.selection = vec![id];
                        }
                        let start = snap::snap(
                            project,
                            self.storey,
                            input.point,
                            input.tolerance,
                            None,
                            false,
                            &self.selection,
                        )
                        .point;
                        self.drag = Some(Drag::Selection { start, current: start });
                    }
                    None => {
                        if !input.shift {
                            self.selection.clear();
                        }
                    }
                }
                Response::redraw()
            }
        }
    }

    pub fn pointer_dragged(&mut self, project: &Project, input: Input) -> Response {
        match &mut self.drag {
            Some(Drag::Selection { current, .. }) => {
                *current =
                    snap::snap(project, self.storey, input.point, input.tolerance, None, false, &self.selection).point;
                Response::redraw()
            }
            Some(Drag::Endpoint { wall, end, current }) => {
                let other =
                    project.wall(*wall).map(|w| end_point(w, if *end == End::Start { End::End } else { End::Start }));
                *current =
                    snap::snap(project, self.storey, input.point, input.tolerance, other, input.shift, &[*wall]).point;
                Response::redraw()
            }
            None => self.pointer_moved(project, input),
        }
    }

    pub fn pointer_up(&mut self, doc: &mut Document) -> Response {
        let Some(drag) = self.drag.take() else { return Response::default() };
        let (label, changes) = match drag {
            Drag::Selection { start, current } => {
                let delta = (current.x - start.x, current.y - start.y);
                if delta.0.hypot(delta.1) < 1e-9 {
                    return Response::redraw();
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
            Drag::Selection { start, current } => {
                moved_elements(project, &self.selection, (current.x - start.x, current.y - start.y))
            }
            Drag::Endpoint { wall, end, current } => as_elements(moved_endpoint(project, *wall, *end, *current)),
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
                if self.drag.take().is_some() {
                } else if self.arc_end.is_some() {
                    self.arc_end = None;
                } else if self.chain_start.is_some() {
                    self.end_chain();
                } else if self.tool != Tool::Select {
                    self.tool = Tool::Select;
                } else {
                    self.selection.clear();
                }
                self.typed.clear();
                Response::redraw()
            }
            Key::Enter => {
                if let (Some(start), Some(end), Some(height)) =
                    (self.chain_start, self.arc_end, parse_length(&self.typed))
                {
                    // Getippter Stich, zur Seite des Zeigers.
                    let side = self.pointer.map_or(1.0, |p| sagitta_from(start, end, p.point).signum());
                    self.typed.clear();
                    return self.add_arc_wall(doc, start, end, side * height);
                }
                if let (Some(start), Some(length)) = (self.chain_start, parse_length(&self.typed)) {
                    let target = self.pointer.map_or(Point2::new(start.x + 1.0, start.y), |s| s.point);
                    let (dx, dy) = (target.x - start.x, target.y - start.y);
                    let len = dx.hypot(dy);
                    let (ux, uy) = if len > 1e-9 { (dx / len, dy / len) } else { (1.0, 0.0) };
                    self.typed.clear();
                    let end = Point2::new(start.x + ux * length, start.y + uy * length);
                    if self.tool == Tool::ArcWall {
                        self.arc_end = Some(end);
                        return Response::redraw();
                    }
                    return self.add_wall(doc, start, end);
                }
                self.end_chain();
                Response::redraw()
            }
            Key::Delete => {
                if self.typed.pop().is_some() {
                    return Response::redraw();
                }
                self.delete_selection(doc)
            }
            Key::Char(c) => {
                if self.chain_start.is_some() && (c.is_ascii_digit() || c == ',' || c == '.') {
                    self.typed.push(c);
                    return Response::redraw();
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
            Command::RoomTool => self.set_tool(Tool::Room),
            Command::DoorTool => self.set_tool(Tool::Door),
            Command::WindowTool => self.set_tool(Tool::Window),
            Command::FlipDoorSwing => self.change_doors(doc, "Aufschlag wechseln", |d| {
                // Die Bänder bleiben, wo sie sind: mit der Seite wechselt auch DIN links und rechts.
                d.opens_to = d.opens_to.other();
                d.hinge = d.hinge.other();
            }),
            Command::FlipDoorHinge => self.change_doors(doc, "Anschlag wechseln", |d| d.hinge = d.hinge.other()),
            Command::DetectRooms => self.detect_rooms(doc),
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
            Command::FlipWalls => self.flip_selection(doc),
            Command::SelectAll => {
                self.tool = Tool::Select;
                let p = doc.project();
                self.selection = p
                    .walls_on(self.storey)
                    .map(|(id, _)| id)
                    .chain(p.columns_on(self.storey).map(|(id, _)| id))
                    .chain(p.rooms_on(self.storey).map(|(id, _)| id))
                    .collect();
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
        let mut list = vec![
            CommandInfo::new(Command::SelectTool, "Auswählen", "V", "Werkzeug"),
            CommandInfo::new(Command::WallTool, "Wand zeichnen", "W", "Werkzeug"),
            CommandInfo::new(Command::ArcWallTool, "Bogenwand zeichnen", "B", "Werkzeug"),
            CommandInfo::new(Command::GridTool, "Achse zeichnen", "A", "Werkzeug"),
            CommandInfo::new(Command::ColumnTool, "Stütze setzen", "S", "Werkzeug"),
            CommandInfo::new(Command::RoomTool, "Raum setzen", "R", "Werkzeug"),
            CommandInfo::new(Command::DoorTool, "Tür setzen", "T", "Werkzeug"),
            CommandInfo::new(Command::WindowTool, "Fenster setzen", "F", "Werkzeug"),
            CommandInfo::new(Command::FlipDoorSwing, "Tür: Aufschlag zur anderen Seite", "", "Auswahl"),
            CommandInfo::new(Command::FlipDoorHinge, "Tür: Anschlag wechseln (DIN links, DIN rechts)", "", "Auswahl"),
            CommandInfo::new(Command::DetectRooms, "Räume erkennen (jeder umschlossene Bereich)", "", "Werkzeug"),
            CommandInfo::new(Command::SelectAll, "Alles im Geschoss wählen", "⌘A", "Auswahl"),
            CommandInfo::new(Command::DeleteSelection, "Auswahl löschen", "⌫", "Auswahl"),
            CommandInfo::new(Command::FlipWalls, "Wand umdrehen (innen und außen tauschen)", "⇧F", "Auswahl"),
            CommandInfo::new(Command::Undo, "Widerrufen", "⌘Z", "Bearbeiten"),
            CommandInfo::new(Command::Redo, "Wiederholen", "⇧⌘Z", "Bearbeiten"),
        ];
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
        if self.tool == Tool::Column
            && let Some(p) = self.pointer
        {
            overlay.snap = Some(p);
            overlay.preview = Some(column_outline(&Column {
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

    /// Hinweis für die Statuszeile.
    pub fn hint(&self) -> String {
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
            (Tool::Room, _, _) => {
                "Raum: in den Bereich klicken, dort steht der Raumstempel. Esc: zurück zum Auswählen.".into()
            }
            (Tool::Select, _, Some(_)) => "Loslassen setzt ab. Esc bricht ab.".into(),
            (Tool::Select, _, None) if self.selection.is_empty() => {
                "Anklicken wählt. W: Wand, B: Bogenwand, T: Tür, F: Fenster, R: Raum, A: Achse, S: Stütze. ⌘K: Befehle."
                    .into()
            }
            (Tool::Select, _, None) => {
                "Ziehen verschiebt, Enden ziehen ändert die Länge. ⌫ löscht. ⇧F dreht um. ⌘K: Befehle.".into()
            }
        }
    }

    fn set_tool(&mut self, tool: Tool) -> Response {
        self.tool = tool;
        self.selection.clear();
        self.end_chain();
        Response::redraw()
    }

    fn end_chain(&mut self) {
        self.chain_start = None;
        self.arc_end = None;
        self.typed.clear();
    }

    fn new_wall(&self, start: Point2, end: Point2) -> Option<Wall> {
        Some(Wall {
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
    fn new_room(&self, project: &Project, position: Point2, skip: u32) -> Room {
        let below_ground = project.storey(self.storey).is_some_and(|s| s.elevation < -0.01);
        Room {
            storey: self.storey,
            position,
            number: next_room_number(project, self.storey, skip),
            name: "Raum".into(),
            usage: lot_model::RoomUsage::default(),
            living_space: !below_ground,
        }
    }

    fn detect_rooms(&mut self, doc: &mut Document) -> Response {
        let project = doc.project();
        let finder = lot_geom::rooms::RoomFinder::new(project, self.storey);
        let ops: Vec<Operation> = finder
            .unclaimed(project, self.storey)
            .into_iter()
            .filter_map(lot_geom::rooms::interior_point)
            .enumerate()
            .map(|(i, p)| {
                let p = Point2::new((p[0] / 0.01).round() * 0.01, (p[1] / 0.01).round() * 0.01);
                Operation::Insert {
                    id: ElementId(project.next_id().0 + i as u64),
                    element: Element::Room(self.new_room(project, p, i as u32)),
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
        Some(Opening { host, offset, width, height, sill_height: sill, kind })
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
fn next_grid_label(project: &Project, a: Point2, b: Point2) -> String {
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
fn next_room_number(project: &Project, storey: ElementId, skip: u32) -> String {
    let storeys = project.storeys();
    let ground = storeys.iter().position(|(_, s)| s.elevation >= -0.01).unwrap_or(0) as i64;
    let level = storeys.iter().position(|(id, _)| *id == storey).map_or(0, |i| i as i64 - ground);
    let prefix = format!("{level}.");
    let highest = project
        .rooms_on(storey)
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
