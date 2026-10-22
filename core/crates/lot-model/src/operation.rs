use serde::{Deserialize, Serialize};

use crate::{Element, ElementId, Param};

/// Eine Änderung am Modell. Jede Operation trägt genug Information, um sich
/// selbst umzukehren. Das Protokoll aller Operationen ist die Grundlage für
/// Undo/Redo, den späteren Versionsverlauf und das Arbeiten mit mehreren Personen.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Operation {
    Insert {
        id: ElementId,
        element: Element,
    },
    /// `element` ist der Zustand vor dem Löschen, damit sich die Operation umkehren lässt.
    Delete {
        id: ElementId,
        element: Element,
    },
    SetParam {
        id: ElementId,
        param: Param,
        old: f64,
        new: f64,
    },
    /// Ersetzt ein Element als Ganzes, etwa beim Verschieben einer Wand.
    Replace {
        id: ElementId,
        old: Element,
        new: Element,
    },
    /// Mehrere Operationen, die als eine gelten, etwa eine Wand verschieben
    /// und die angeschlossenen Wände mitziehen. Ein Undo nimmt alle zurück.
    Group {
        label: String,
        ops: Vec<Operation>,
    },
}

impl Operation {
    pub fn inverse(&self) -> Operation {
        match self {
            Operation::Insert { id, element } => Operation::Delete { id: *id, element: element.clone() },
            Operation::Delete { id, element } => Operation::Insert { id: *id, element: element.clone() },
            Operation::SetParam { id, param, old, new } => {
                Operation::SetParam { id: *id, param: *param, old: *new, new: *old }
            }
            Operation::Replace { id, old, new } => Operation::Replace { id: *id, old: new.clone(), new: old.clone() },
            Operation::Group { label, ops } => {
                Operation::Group { label: label.clone(), ops: ops.iter().rev().map(Operation::inverse).collect() }
            }
        }
    }

    /// Die Elemente, die die Operation betrifft.
    pub fn targets(&self) -> Vec<ElementId> {
        match self {
            Operation::Insert { id, .. }
            | Operation::Delete { id, .. }
            | Operation::SetParam { id, .. }
            | Operation::Replace { id, .. } => vec![*id],
            Operation::Group { ops, .. } => ops.iter().flat_map(Operation::targets).collect(),
        }
    }
}
