use crate::{ColumnProfile, Element, ElementId, Operation, Param, Project, Top};

/// Kleinste Maße, die das Modell zulässt, in Metern.
const MIN_LENGTH: f64 = 0.01;
/// Toleranz beim Vergleich von Parameterwerten, in Metern.
const TOLERANCE: f64 = 1e-9;

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ModelError {
    #[error("Element {0:?} gibt es nicht")]
    UnknownElement(ElementId),
    #[error("Element {0:?} gibt es schon")]
    DuplicateElement(ElementId),
    #[error("Element {id:?} hat den Parameter {param:?} nicht")]
    UnknownParam { id: ElementId, param: Param },
    #[error("Der Modellstand passt nicht zur Operation")]
    Conflict,
    #[error("Ungültiger Wert: {0}")]
    Invalid(String),
}

/// Ein geöffnetes Projekt mit Operationsprotokoll und Undo/Redo.
///
/// Das Protokoll ist nur anhängend: Auch Undo und Redo erscheinen darin als
/// eigene Operationen.
#[derive(Debug, Clone)]
pub struct Document {
    project: Project,
    log: Vec<Operation>,
    undo: Vec<Operation>,
    redo: Vec<Operation>,
}

impl Document {
    pub fn new(project: Project) -> Self {
        Self { project, log: Vec::new(), undo: Vec::new(), redo: Vec::new() }
    }

    /// Öffnet ein Projekt mit einem schon vorhandenen Protokoll, etwa aus einer Datei.
    pub fn with_log(project: Project, log: Vec<Operation>) -> Self {
        Self { project, log, undo: Vec::new(), redo: Vec::new() }
    }

    pub fn project(&self) -> &Project {
        &self.project
    }

    pub fn log(&self) -> &[Operation] {
        &self.log
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Führt eine Operation des Benutzers aus. Leert den Redo-Stapel.
    pub fn apply(&mut self, op: Operation) -> Result<(), ModelError> {
        self.execute(&op)?;
        self.undo.push(op);
        self.redo.clear();
        Ok(())
    }

    /// Fügt ein neues Element ein und gibt seine Kennung zurück.
    pub fn insert(&mut self, element: Element) -> Result<ElementId, ModelError> {
        let id = self.project.next_id();
        self.apply(Operation::Insert { id, element })?;
        Ok(id)
    }

    /// Ersetzt ein Element als Ganzes.
    pub fn replace(&mut self, id: ElementId, new: Element) -> Result<(), ModelError> {
        let old = self.project.elements.get(&id).ok_or(ModelError::UnknownElement(id))?.clone();
        if old == new {
            return Ok(());
        }
        self.apply(Operation::Replace { id, old, new })
    }

    /// Löscht ein Element.
    pub fn delete(&mut self, id: ElementId) -> Result<(), ModelError> {
        let element = self.project.elements.get(&id).ok_or(ModelError::UnknownElement(id))?.clone();
        self.apply(Operation::Delete { id, element })
    }

    /// Setzt einen Parameter auf einen neuen Wert.
    pub fn set_param(&mut self, id: ElementId, param: Param, value: f64) -> Result<(), ModelError> {
        let element = self.project.elements.get(&id).ok_or(ModelError::UnknownElement(id))?;
        let old = element.param(param).ok_or(ModelError::UnknownParam { id, param })?;
        if (old - value).abs() <= TOLERANCE {
            return Ok(());
        }
        self.apply(Operation::SetParam { id, param, old, new: value })
    }

    pub fn undo(&mut self) -> Result<bool, ModelError> {
        let Some(op) = self.undo.pop() else {
            return Ok(false);
        };
        if let Err(e) = self.execute(&op.inverse()) {
            self.undo.push(op);
            return Err(e);
        }
        self.redo.push(op);
        Ok(true)
    }

    pub fn redo(&mut self) -> Result<bool, ModelError> {
        let Some(op) = self.redo.pop() else {
            return Ok(false);
        };
        if let Err(e) = self.execute(&op) {
            self.redo.push(op);
            return Err(e);
        }
        self.undo.push(op);
        Ok(true)
    }

    /// Prüft und führt eine Operation aus und hängt sie ans Protokoll.
    /// Bei einem Fehler bleibt das Modell unverändert.
    fn execute(&mut self, op: &Operation) -> Result<(), ModelError> {
        let mut next = self.project.clone();
        apply_to(&mut next, op)?;
        validate(&next)?;
        self.project = next;
        self.log.push(op.clone());
        Ok(())
    }
}

/// Wendet eine Operation auf einen Projektstand an, ohne zu prüfen, ob das
/// Ergebnis gültig ist.
fn apply_to(project: &mut Project, op: &Operation) -> Result<(), ModelError> {
    match op {
        Operation::Insert { id, element } => {
            if project.elements.contains_key(id) {
                return Err(ModelError::DuplicateElement(*id));
            }
            project.elements.insert(*id, element.clone());
        }
        Operation::Delete { id, element } => {
            match project.elements.get(id) {
                Some(found) if found == element => {}
                Some(_) => return Err(ModelError::Conflict),
                None => return Err(ModelError::UnknownElement(*id)),
            }
            project.elements.remove(id);
        }
        Operation::SetParam { id, param, old, new } => {
            let element = project.elements.get_mut(id).ok_or(ModelError::UnknownElement(*id))?;
            let found = element.param(*param).ok_or(ModelError::UnknownParam { id: *id, param: *param })?;
            if (found - old).abs() > TOLERANCE {
                return Err(ModelError::Conflict);
            }
            element.set_param(*param, *new);
        }
        Operation::Replace { id, old, new } => {
            let element = project.elements.get_mut(id).ok_or(ModelError::UnknownElement(*id))?;
            if element != old {
                return Err(ModelError::Conflict);
            }
            if element.kind() != new.kind() {
                return Err(ModelError::Invalid("Ein Element kann seine Art nicht ändern".into()));
            }
            *element = new.clone();
        }
        Operation::Group { ops, .. } => {
            for op in ops {
                apply_to(project, op)?;
            }
        }
    }
    Ok(())
}

impl Project {
    /// Prüft, ob der Projektstand sinnvoll ist: Bezüge zeigen auf passende
    /// Elemente, Maße sind positiv, Öffnungen passen in ihre Wand. Dieselben
    /// Regeln gelten für jede Operation, für geladene Dateien und für Testprojekte.
    pub fn validate(&self) -> Result<(), ModelError> {
        validate(self)
    }
}

fn invalid(message: impl Into<String>) -> ModelError {
    ModelError::Invalid(message.into())
}

fn validate(project: &Project) -> Result<(), ModelError> {
    for (id, element) in &project.elements {
        match element {
            Element::Storey(s) => {
                if s.height < MIN_LENGTH {
                    return Err(invalid(format!(
                        "Geschoss {}: Die Geschosshöhe muss mindestens 1 cm betragen",
                        s.name
                    )));
                }
            }
            Element::GridLine(g) => {
                if g.start.distance(g.end) < MIN_LENGTH {
                    return Err(invalid(format!("Achse {}: Anfang und Ende fallen zusammen", g.label)));
                }
            }
            Element::WallType(t) => {
                if t.layers.is_empty() {
                    return Err(invalid(format!("Wandaufbau {}: Es fehlt mindestens eine Schicht", t.name)));
                }
                if t.layers.iter().any(|l| l.thickness < 0.001) {
                    return Err(invalid(format!("Wandaufbau {}: Jede Schicht muss mindestens 1 mm dick sein", t.name)));
                }
            }
            Element::Wall(w) => {
                if project.storey(w.storey).is_none() {
                    return Err(invalid(format!("Wand {}: Das Geschoss fehlt", id.0)));
                }
                if project.wall_type(w.wall_type).is_none() {
                    return Err(invalid(format!("Wand {}: Der Wandaufbau fehlt", id.0)));
                }
                if w.length() < MIN_LENGTH {
                    return Err(invalid("Eine Wand muss mindestens 1 cm lang sein"));
                }
                if w.sagitta.abs() > w.chord() / 2.0 + TOLERANCE {
                    return Err(invalid(
                        "Der Stich einer Bogenwand darf höchstens die halbe Sehne betragen (Halbkreis)",
                    ));
                }
                if let (Some(arc), Some(t)) = (w.arc(), project.wall_type(w.wall_type)) {
                    let offsets = w.layer_offsets(t);
                    let inner = arc.radius_at(offsets[0].0).min(arc.radius_at(offsets[offsets.len() - 1].1));
                    if inner < MIN_LENGTH {
                        return Err(invalid("Der Bogen ist zu eng für die Wanddicke"));
                    }
                }
                check_height(project, w.storey, w.base_offset, w.top, "Wand")?;
            }
            Element::Opening(o) => {
                let wall = project
                    .wall(o.host)
                    .ok_or_else(|| invalid(format!("Die Öffnung sitzt in Element {}, das keine Wand ist", o.host.0)))?;
                if o.width < MIN_LENGTH || o.height < MIN_LENGTH {
                    return Err(invalid("Breite und Höhe der Öffnung müssen mindestens 1 cm betragen"));
                }
                if o.offset < 0.0 || o.offset + o.width > wall.length() + TOLERANCE {
                    return Err(invalid("Die Öffnung ragt seitlich aus der Wand"));
                }
                let overlaps = project.openings_in(o.host).any(|(other_id, other)| {
                    other_id != *id
                        && o.offset < other.offset + other.width - TOLERANCE
                        && other.offset < o.offset + o.width - TOLERANCE
                });
                if overlaps {
                    return Err(invalid("Die Öffnung überschneidet sich mit einer anderen in derselben Wand"));
                }
                let (bottom, top) = project.wall_extent(wall).unwrap_or((0.0, 0.0));
                if o.sill_height < 0.0 || o.sill_height + o.height > top - bottom + TOLERANCE {
                    return Err(invalid("Die Öffnung ragt oben oder unten aus der Wand"));
                }
            }
            Element::SlabType(t) => {
                if t.layers.is_empty() || t.layers.iter().any(|l| l.thickness < 0.001) {
                    return Err(invalid(format!(
                        "Deckenaufbau {}: Jede Schicht muss mindestens 1 mm dick sein",
                        t.name
                    )));
                }
            }
            Element::Slab(s) => {
                if project.storey(s.storey).is_none() {
                    return Err(invalid("Decke: Das Geschoss fehlt"));
                }
                if project.slab_type(s.slab_type).is_none() {
                    return Err(invalid("Decke: Der Deckenaufbau fehlt"));
                }
                let n = s.outline.len();
                let area: f64 = (0..n)
                    .map(|i| {
                        let (a, b) = (s.outline[i], s.outline[(i + 1) % n]);
                        a.x * b.y - b.x * a.y
                    })
                    .sum::<f64>()
                    / 2.0;
                if n < 3 || area.abs() < 0.01 {
                    return Err(invalid("Eine Decke braucht einen Umriss mit mindestens 0,01 m²"));
                }
            }
            Element::ProjectInfo(_) => {}
            Element::PlanSheet(s) => {
                if !["A4", "A3", "A2", "A1", "A0"].contains(&s.format.as_str()) {
                    return Err(invalid("Planblatt: Format A4 bis A0"));
                }
                if s.views.iter().any(|v| v.scale <= 0.0) {
                    return Err(invalid("Planblatt: Der Maßstab muss größer als null sein"));
                }
            }
            Element::SectionView(v) => {
                if v.start.distance(v.end) < MIN_LENGTH || v.depth <= 0.0 {
                    return Err(invalid("Schnitt: Linie und Blicktiefe müssen größer als null sein"));
                }
            }
            Element::Roof(r) => {
                if project.storey(r.storey).is_none() {
                    return Err(invalid("Dach: Das Geschoss fehlt"));
                }
                if project.slab_type(r.roof_type).is_none() {
                    return Err(invalid("Dach: Der Dachaufbau fehlt"));
                }
                if r.outline.len() < 3 || r.edges.len() != r.outline.len() {
                    return Err(invalid("Dach: Jede Umrisskante braucht eine Angabe, Traufe oder Giebel"));
                }
                if !r.is_convex() {
                    return Err(invalid(
                        "Dächer über verwinkelten Grundrissen folgen später; der Umriss muss konvex sein",
                    ));
                }
                for e in &r.edges {
                    if let crate::RoofEdge::Eave { slope, .. } = e
                        && !(0.0..75.0).contains(slope)
                    {
                        return Err(invalid("Dach: Die Neigung muss zwischen 0° und 75° liegen"));
                    }
                    if e.overhang() < 0.0 {
                        return Err(invalid("Dach: Der Überstand darf nicht negativ sein"));
                    }
                }
            }
            Element::SlabOpening(o) => {
                if !matches!(project.elements.get(&o.host), Some(Element::Slab(_))) {
                    return Err(invalid("Der Durchbruch sitzt nicht in einer Decke"));
                }
                if o.outline.len() < 3 {
                    return Err(invalid("Ein Durchbruch braucht mindestens drei Eckpunkte"));
                }
            }
            Element::DimensionChain(d) => {
                if project.storey(d.storey).is_none() {
                    return Err(invalid("Maßkette: Das Geschoss fehlt"));
                }
                if d.direction.x.hypot(d.direction.y) < 1e-9 {
                    return Err(invalid("Maßkette: Die Richtung fehlt"));
                }
            }
            Element::Space(r) => {
                if project.storey(r.storey).is_none() {
                    return Err(invalid(format!("Raum {}: Das Geschoss fehlt", r.name)));
                }
                if r.name.trim().is_empty() {
                    return Err(invalid("Ein Raum braucht einen Namen"));
                }
            }
            Element::Column(c) => {
                if project.storey(c.storey).is_none() {
                    return Err(invalid(format!("Stütze {}: Das Geschoss fehlt", id.0)));
                }
                let too_small = match c.profile {
                    ColumnProfile::Rectangle { width, depth } => width < MIN_LENGTH || depth < MIN_LENGTH,
                    ColumnProfile::Circle { diameter } => diameter < MIN_LENGTH,
                };
                if too_small {
                    return Err(invalid("Der Querschnitt einer Stütze muss mindestens 1 cm messen"));
                }
                check_height(project, c.storey, c.base_offset, c.top, "Stütze")?;
            }
        }
    }
    Ok(())
}

fn check_height(project: &Project, storey: ElementId, base: f64, top: Top, what: &str) -> Result<(), ModelError> {
    let (bottom, top) = project.vertical_extent(storey, base, top).unwrap_or((0.0, 0.0));
    if top - bottom < MIN_LENGTH {
        return Err(invalid(format!("Die {what} muss mindestens 1 cm hoch sein")));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Opening, Point2, Wall, WallAlignment};

    const WALL: ElementId = ElementId(3);
    const OPENING: ElementId = ElementId(4);

    fn doc() -> Document {
        Document::new(Project::sample())
    }

    fn width(d: &Document) -> Option<f64> {
        d.project().elements[&OPENING].param(Param::OpeningWidth)
    }

    #[test]
    fn set_param_and_undo_redo() {
        let mut d = doc();
        d.set_param(OPENING, Param::OpeningWidth, 2.0).unwrap();
        assert_eq!(width(&d), Some(2.0));
        assert!(d.undo().unwrap());
        assert_eq!(width(&d), Some(1.5));
        assert!(d.redo().unwrap());
        assert_eq!(width(&d), Some(2.0));
        // Ausführen, Undo, Redo: drei Einträge im Protokoll.
        assert_eq!(d.log().len(), 3);
    }

    #[test]
    fn new_operation_clears_redo() {
        let mut d = doc();
        d.set_param(WALL, Param::WallHeight, 3.0).unwrap();
        d.undo().unwrap();
        assert!(d.can_redo());
        d.set_param(WALL, Param::BaseOffset, 0.1).unwrap();
        assert!(!d.can_redo());
    }

    #[test]
    fn invalid_change_leaves_model_untouched() {
        let mut d = doc();
        let before = d.project().clone();
        // Die Öffnung (1,75 + 1,5 = 3,25 m) passt nicht mehr in eine 3 m lange Wand.
        let err = d.set_param(WALL, Param::WallLength, 3.0).unwrap_err();
        assert!(matches!(err, ModelError::Invalid(_)));
        assert_eq!(d.project(), &before);
        assert!(d.log().is_empty());
        assert!(!d.can_undo());
    }

    #[test]
    fn wall_length_keeps_direction() {
        let mut d = doc();
        let mut wall = d.project().wall(WALL).unwrap().clone();
        wall.start = Point2::new(1.0, 1.0);
        wall.end = Point2::new(4.0, 5.0);
        d.replace(WALL, Element::Wall(wall)).unwrap();
        d.set_param(WALL, Param::WallLength, 10.0).unwrap();
        let w = d.project().wall(WALL).unwrap();
        assert!((w.end.x - 7.0).abs() < 1e-12 && (w.end.y - 9.0).abs() < 1e-12);
    }

    #[test]
    fn stale_operation_is_a_conflict() {
        let mut d = doc();
        let op = Operation::SetParam { id: OPENING, param: Param::OpeningWidth, old: 9.0, new: 1.0 };
        assert!(matches!(d.apply(op), Err(ModelError::Conflict)));
    }

    #[test]
    fn cannot_delete_wall_with_openings() {
        let mut d = doc();
        assert!(matches!(d.delete(WALL), Err(ModelError::Invalid(_))));
    }

    #[test]
    fn cannot_delete_storey_in_use() {
        let mut d = doc();
        assert!(matches!(d.delete(ElementId(1)), Err(ModelError::Invalid(_))));
    }

    #[test]
    fn opening_needs_wall_host() {
        let mut d = doc();
        let err = d
            .insert(Element::Opening(Opening {
                host: ElementId(2),
                offset: 0.0,
                width: 1.0,
                height: 1.0,
                sill_height: 0.0,
                kind: crate::OpeningKind::Plain,
            }))
            .unwrap_err();
        assert!(matches!(err, ModelError::Invalid(_)));
    }

    #[test]
    fn group_is_undone_as_one() {
        let mut d = doc();
        let mut moved = d.project().wall(WALL).unwrap().clone();
        moved.start.y += 1.0;
        moved.end.y += 1.0;
        let group = Operation::Group {
            label: "Wand verschieben".into(),
            ops: vec![
                Operation::Replace { id: WALL, old: d.project().elements[&WALL].clone(), new: Element::Wall(moved) },
                Operation::SetParam { id: OPENING, param: Param::OpeningWidth, old: 1.5, new: 1.0 },
            ],
        };
        d.apply(group).unwrap();
        assert_eq!(d.project().wall(WALL).unwrap().start.y, 1.0);
        assert_eq!(width(&d), Some(1.0));
        d.undo().unwrap();
        assert_eq!(d.project(), &Project::sample());
    }

    #[test]
    fn failing_group_changes_nothing() {
        let mut d = doc();
        let group = Operation::Group {
            label: "t".into(),
            ops: vec![
                Operation::SetParam { id: OPENING, param: Param::OpeningWidth, old: 1.5, new: 1.0 },
                Operation::SetParam { id: OPENING, param: Param::OpeningWidth, old: 9.0, new: 1.0 },
            ],
        };
        assert!(d.apply(group).is_err());
        assert_eq!(d.project(), &Project::sample());
    }

    #[test]
    fn wall_on_storey_follows_storey_height() {
        let mut d = doc();
        let mut wall = d.project().wall(WALL).unwrap().clone();
        wall.top = Top::Storey { offset: -0.2 };
        d.replace(WALL, Element::Wall(wall)).unwrap();
        let w = d.project().wall(WALL).unwrap();
        assert_eq!(d.project().wall_extent(w), Some((0.0, 2.55)));
        d.set_param(ElementId(1), Param::StoreyHeight, 3.0).unwrap();
        let w = d.project().wall(WALL).unwrap();
        assert_eq!(d.project().wall_extent(w), Some((0.0, 2.8)));
    }

    #[test]
    fn layer_offsets_follow_alignment() {
        let p = Project::sample();
        let t = crate::WallType {
            name: "t".into(),
            layers: vec![
                crate::Layer { material: "a".into(), thickness: 0.1, function: crate::LayerFunction::Insulation },
                crate::Layer { material: "b".into(), thickness: 0.2, function: crate::LayerFunction::Structure },
            ],
        };
        let mut w: Wall = p.wall(WALL).unwrap().clone();
        w.alignment = WallAlignment::Exterior;
        assert_eq!(w.layer_offsets(&t), vec![(0.0, -0.1), (-0.1, -0.30000000000000004)]);
        w.alignment = WallAlignment::Interior;
        assert_eq!(w.layer_offsets(&t)[0], (0.30000000000000004, 0.20000000000000004));
        w.alignment = WallAlignment::Center;
        assert_eq!(w.layer_offsets(&t)[0].0, 0.15000000000000002);
    }
}
