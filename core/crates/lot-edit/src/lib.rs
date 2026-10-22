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
use geometry::{End, as_elements, end_point, footprint, moved_endpoint, moved_walls, wall_at};
use lot_model::{
    Document, Element, ElementId, ModelError, Operation, Point2, Project, Storey, Top, Wall, WallAlignment,
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
}

#[derive(Debug, Clone, PartialEq)]
enum Drag {
    Walls { start: Point2, current: Point2 },
    Endpoint { wall: ElementId, end: End, current: Point2 },
}

/// Was der Renderer über dem Plan zeigt, in Modellkoordinaten.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Overlay {
    /// Umriss der Wand, die gerade gezeichnet wird.
    pub preview: Option<Vec<[f64; 2]>>,
    /// Umrisse der gewählten Wände.
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
            drag: None,
            pointer: None,
            typed: String::new(),
        }
    }

    /// Passt den Zustand an ein geändertes Modell an (Undo, Laden): Verschwundene
    /// Elemente fallen aus der Auswahl, ein fehlendes Geschoss wird ersetzt.
    pub fn sync(&mut self, project: &Project) {
        self.selection.retain(|id| project.wall(*id).is_some());
        if project.storey(self.storey).is_none() {
            *self = Self { tool: self.tool, ..Self::new(project) };
        }
        if self.wall_type.is_none_or(|t| project.wall_type(t).is_none()) {
            self.wall_type = project.wall_types().next().map(|(id, _)| id);
        }
    }

    pub fn pointer_moved(&mut self, project: &Project, input: Input) -> Response {
        match self.tool {
            Tool::Wall => {
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
            Tool::Select => Response::default(),
        }
    }

    pub fn pointer_down(&mut self, doc: &mut Document, input: Input, clicks: u32) -> Response {
        match self.tool {
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
                match wall_at(project, self.storey, input.point, input.tolerance) {
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
                        self.drag = Some(Drag::Walls { start, current: start });
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
            Some(Drag::Walls { current, .. }) => {
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
            Drag::Walls { start, current } => {
                let delta = (current.x - start.x, current.y - start.y);
                if delta.0.hypot(delta.1) < 1e-9 {
                    return Response::redraw();
                }
                ("Wände verschieben", moved_walls(doc.project(), &self.selection, delta))
            }
            Drag::Endpoint { wall, end, current } => {
                if doc.project().wall(wall).is_none_or(|w| end_point(w, end).distance(current) < 1e-9) {
                    return Response::redraw();
                }
                ("Wandende verschieben", moved_endpoint(doc.project(), wall, end, current))
            }
        };
        Response::from_result(replace_all(doc, label, as_elements(changes)))
    }

    /// Stand des Modells während einer Ziehbewegung, für die Vorschau.
    pub fn preview(&self, project: &Project) -> Option<Project> {
        let changes = match self.drag.as_ref()? {
            Drag::Walls { start, current } => {
                moved_walls(project, &self.selection, (current.x - start.x, current.y - start.y))
            }
            Drag::Endpoint { wall, end, current } => moved_endpoint(project, *wall, *end, *current),
        };
        let mut p = project.clone();
        for (id, w) in changes {
            p.elements.insert(id, Element::Wall(w));
        }
        Some(p)
    }

    pub fn key(&mut self, doc: &mut Document, key: Key) -> Response {
        match key {
            Key::Escape => {
                if self.drag.take().is_some() {
                } else if self.chain_start.is_some() {
                    self.end_chain();
                } else if self.tool == Tool::Wall {
                    self.tool = Tool::Select;
                } else {
                    self.selection.clear();
                }
                self.typed.clear();
                Response::redraw()
            }
            Key::Enter => {
                if let (Some(start), Some(length)) = (self.chain_start, parse_length(&self.typed)) {
                    let target = self.pointer.map_or(Point2::new(start.x + 1.0, start.y), |s| s.point);
                    let (dx, dy) = (target.x - start.x, target.y - start.y);
                    let len = dx.hypot(dy);
                    let (ux, uy) = if len > 1e-9 { (dx / len, dy / len) } else { (1.0, 0.0) };
                    self.typed.clear();
                    let end = Point2::new(start.x + ux * length, start.y + uy * length);
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
            Command::WallTool => {
                self.tool = Tool::Wall;
                self.selection.clear();
                Response::redraw()
            }
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
                self.selection = doc.project().walls_on(self.storey).map(|(id, _)| id).collect();
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
            CommandInfo::new(Command::SelectAll, "Alle Wände im Geschoss wählen", "⌘A", "Auswahl"),
            CommandInfo::new(Command::DeleteSelection, "Auswahl löschen", "⌫", "Auswahl"),
            CommandInfo::new(Command::FlipWalls, "Wand umdrehen (innen und außen tauschen)", "F", "Auswahl"),
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
            if let Some(w) = project_now.wall(*id) {
                overlay.selected.push(footprint(project_now, w));
                overlay.handles.extend([w.start, w.end]);
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
            (Tool::Select, _, Some(_)) => "Loslassen setzt die Wand ab. Esc bricht ab.".into(),
            (Tool::Select, _, None) if self.selection.is_empty() => {
                "Wand anklicken zum Wählen. W: Wand zeichnen. ⌘K: Befehle.".into()
            }
            (Tool::Select, _, None) => {
                "Ziehen verschiebt, Enden ziehen ändert die Länge. ⌫ löscht. F dreht um. ⌘K: Befehle.".into()
            }
        }
    }

    fn end_chain(&mut self) {
        self.chain_start = None;
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
