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
mod roof;
mod stair;

pub use document::{Document, ModelError};
pub use element::{
    BuildingClass, CircularArc, Column, ColumnProfile, DimRef, DimensionChain, Door, Element, ElementId, ElementKind,
    GridLine, Handing, Layer, LayerFunction, Opening, OpeningKind, Param, PlacedView, PlanSheet, Point2, ProjectInfo,
    SectionKind, SectionView, Slab, SlabKind, SlabOpening, SlabType, Space, SpaceUsage, Storey, Top, ViewContent, Wall,
    WallAlignment, WallEnd, WallSide, WallType,
};
pub use operation::Operation;
pub use project::Project;
pub use roof::{Dormer, DormerWindow, EavePlane, Roof, RoofEdge, RoofFace, RoofWindow};
pub use stair::{FlightLayout, Landing, Railing, RailingPath, Stair, StairFlight, StairLayout, Tread};
