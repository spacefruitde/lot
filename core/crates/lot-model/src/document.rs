use crate::{Element, ElementId, Operation, Param, Project};

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
    #[error("Der Modellstand passt nicht zur Operation (erwartet {expected}, gefunden {found})")]
    Conflict { expected: f64, found: f64 },
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
        Self {
            project,
            log: Vec::new(),
            undo: Vec::new(),
            redo: Vec::new(),
        }
    }

    /// Öffnet ein Projekt mit einem schon vorhandenen Protokoll, etwa aus einer Datei.
    pub fn with_log(project: Project, log: Vec<Operation>) -> Self {
        Self {
            project,
            log,
            undo: Vec::new(),
            redo: Vec::new(),
        }
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
        match op {
            Operation::Insert { id, element } => {
                if next.elements.contains_key(id) {
                    return Err(ModelError::DuplicateElement(*id));
                }
                next.elements.insert(*id, element.clone());
            }
            Operation::Delete { id, .. } => {
                if next.elements.remove(id).is_none() {
                    return Err(ModelError::UnknownElement(*id));
                }
                if next.openings_in(*id).next().is_some() {
                    return Err(ModelError::Invalid(
                        "Die Wand enthält noch Öffnungen".into(),
                    ));
                }
            }
            Operation::SetParam { id, param, old, new } => {
                let element = next.elements.get_mut(id).ok_or(ModelError::UnknownElement(*id))?;
                let found = element.param(*param).ok_or(ModelError::UnknownParam { id: *id, param: *param })?;
                if (found - old).abs() > TOLERANCE {
                    return Err(ModelError::Conflict { expected: *old, found });
                }
                element.set_param(*param, *new);
            }
        }
        validate(&next)?;
        self.project = next;
        self.log.push(op.clone());
        Ok(())
    }
}

impl Project {
    /// Prüft, ob der Projektstand geometrisch sinnvoll ist. Dieselben Regeln
    /// gelten für jede Operation, für geladene Dateien und für Testprojekte.
    pub fn validate(&self) -> Result<(), ModelError> {
        validate(self)
    }
}

fn validate(project: &Project) -> Result<(), ModelError> {
    for element in project.elements.values() {
        match element {
            Element::Wall(w) => {
                if w.length() < MIN_LENGTH || w.thickness < MIN_LENGTH || w.height < MIN_LENGTH {
                    return Err(ModelError::Invalid(
                        "Länge, Dicke und Höhe der Wand müssen mindestens 1 cm betragen".into(),
                    ));
                }
            }
            Element::Opening(o) => {
                let wall = project.wall(o.host).ok_or_else(|| {
                    ModelError::Invalid(format!("Die Öffnung sitzt in Element {:?}, das keine Wand ist", o.host))
                })?;
                if o.width < MIN_LENGTH || o.height < MIN_LENGTH {
                    return Err(ModelError::Invalid(
                        "Breite und Höhe der Öffnung müssen mindestens 1 cm betragen".into(),
                    ));
                }
                if o.offset < 0.0 || o.offset + o.width > wall.length() + TOLERANCE {
                    return Err(ModelError::Invalid("Die Öffnung ragt seitlich aus der Wand".into()));
                }
                if o.sill_height < 0.0 || o.sill_height + o.height > wall.height + TOLERANCE {
                    return Err(ModelError::Invalid("Die Öffnung ragt oben oder unten aus der Wand".into()));
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Opening, Point2, Wall};

    fn doc() -> Document {
        Document::new(Project::sample())
    }

    #[test]
    fn set_param_and_undo_redo() {
        let mut d = doc();
        d.set_param(ElementId(2), Param::OpeningWidth, 2.0).unwrap();
        assert_eq!(d.project().elements[&ElementId(2)].param(Param::OpeningWidth), Some(2.0));
        assert!(d.undo().unwrap());
        assert_eq!(d.project().elements[&ElementId(2)].param(Param::OpeningWidth), Some(1.5));
        assert!(d.redo().unwrap());
        assert_eq!(d.project().elements[&ElementId(2)].param(Param::OpeningWidth), Some(2.0));
        // Ausführen, Undo, Redo: drei Einträge im Protokoll.
        assert_eq!(d.log().len(), 3);
    }

    #[test]
    fn new_operation_clears_redo() {
        let mut d = doc();
        d.set_param(ElementId(1), Param::WallHeight, 3.0).unwrap();
        d.undo().unwrap();
        assert!(d.can_redo());
        d.set_param(ElementId(1), Param::WallThickness, 0.24).unwrap();
        assert!(!d.can_redo());
    }

    #[test]
    fn invalid_change_leaves_model_untouched() {
        let mut d = doc();
        let before = d.project().clone();
        // Die Öffnung (1,75 + 1,5 = 3,25 m) passt nicht mehr in eine 3 m lange Wand.
        let err = d.set_param(ElementId(1), Param::WallLength, 3.0).unwrap_err();
        assert!(matches!(err, ModelError::Invalid(_)));
        assert_eq!(d.project(), &before);
        assert!(d.log().is_empty());
        assert!(!d.can_undo());
    }

    #[test]
    fn wall_length_keeps_direction() {
        let mut d = Document::new(Project::new("t"));
        let id = d
            .insert(Element::Wall(Wall {
                start: Point2::new(1.0, 1.0),
                end: Point2::new(4.0, 5.0),
                thickness: 0.2,
                height: 2.5,
            }))
            .unwrap();
        d.set_param(id, Param::WallLength, 10.0).unwrap();
        let w = d.project().wall(id).unwrap();
        assert!((w.end.x - 7.0).abs() < 1e-12 && (w.end.y - 9.0).abs() < 1e-12);
    }

    #[test]
    fn stale_operation_is_a_conflict() {
        let mut d = doc();
        let op = Operation::SetParam { id: ElementId(2), param: Param::OpeningWidth, old: 9.0, new: 1.0 };
        assert!(matches!(d.apply(op), Err(ModelError::Conflict { .. })));
    }

    #[test]
    fn cannot_delete_wall_with_openings() {
        let mut d = doc();
        let wall = d.project().elements[&ElementId(1)].clone();
        let err = d.apply(Operation::Delete { id: ElementId(1), element: wall }).unwrap_err();
        assert!(matches!(err, ModelError::Invalid(_)));
    }

    #[test]
    fn opening_needs_wall_host() {
        let mut d = doc();
        let err = d
            .insert(Element::Opening(Opening { host: ElementId(2), offset: 0.0, width: 1.0, height: 1.0, sill_height: 0.0 }))
            .unwrap_err();
        assert!(matches!(err, ModelError::Invalid(_)));
    }
}
