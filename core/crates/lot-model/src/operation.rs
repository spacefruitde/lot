use serde::{Deserialize, Serialize};

use crate::{Element, ElementId, Param};

/// Eine Änderung am Modell. Jede Operation trägt genug Information, um sich
/// selbst umzukehren. Das Protokoll aller Operationen ist die Grundlage für
/// Undo/Redo, den späteren Versionsverlauf und das Arbeiten mit mehreren Personen.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Operation {
    Insert { id: ElementId, element: Element },
    /// `element` ist der Zustand vor dem Löschen, damit sich die Operation umkehren lässt.
    Delete { id: ElementId, element: Element },
    SetParam { id: ElementId, param: Param, old: f64, new: f64 },
}

impl Operation {
    pub fn inverse(&self) -> Operation {
        match self {
            Operation::Insert { id, element } => Operation::Delete {
                id: *id,
                element: element.clone(),
            },
            Operation::Delete { id, element } => Operation::Insert {
                id: *id,
                element: element.clone(),
            },
            Operation::SetParam { id, param, old, new } => Operation::SetParam {
                id: *id,
                param: *param,
                old: *new,
                new: *old,
            },
        }
    }

    /// Das Element, das die Operation betrifft.
    pub fn target(&self) -> ElementId {
        match self {
            Operation::Insert { id, .. } | Operation::Delete { id, .. } | Operation::SetParam { id, .. } => *id,
        }
    }
}
