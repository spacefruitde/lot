//! Bauwerksmodell von Lot: Elemente, Parameter, Operationen, Undo/Redo.
//!
//! Gespeichert werden nur Parameter und Bezüge, nie berechnete Geometrie
//! (CLAUDE.md, Regel 6). Jede Änderung am Modell läuft als [`Operation`]
//! über [`Document::apply`] (Regel 7). Darauf bauen Undo/Redo und später
//! Versionsverlauf und Mehrbenutzer auf.
//!
//! Einheiten: Längen in Metern als `f64`, Koordinaten rechtshändig mit z nach
//! oben (siehe ADR 0002).

mod document;
mod element;
mod operation;

pub use document::{Document, ModelError};
pub use element::{Element, ElementId, ElementKind, Opening, Param, Point2, Project, Wall};
pub use operation::Operation;
