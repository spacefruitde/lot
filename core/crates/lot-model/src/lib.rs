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
mod project;

pub use document::{Document, ModelError};
pub use element::{
    CircularArc, Column, ColumnProfile, DimRef, DimensionChain, Door, Element, ElementId, ElementKind, GridLine,
    Handing, Layer, LayerFunction, Opening, OpeningKind, Param, Point2, Space, SpaceUsage, Storey, Top, Wall,
    WallAlignment, WallEnd, WallSide, WallType,
};
pub use operation::Operation;
pub use project::Project;
