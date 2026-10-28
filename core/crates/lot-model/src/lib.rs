//! Bauwerksmodell von Lot: Elemente, Parameter, Operationen, Undo/Redo.
//!
//! Gespeichert werden nur Parameter und Bezüge, nie berechnete Geometrie
//! (CLAUDE.md, Regel 6). Jede Änderung am Modell läuft als [`Operation`]
//! über [`Document::apply`] (Regel 7). Darauf bauen Undo/Redo und später
//! Versionsverlauf und Mehrbenutzer auf.
//!
//! Einheiten: Längen in Metern als `f64`, Koordinaten rechtshändig mit z nach
//! oben (siehe ADR 0002).

mod annotation;
mod document;
mod element;
mod elevator;
mod operation;
mod plot;
mod project;
mod roof;
mod stair;
mod terrain;
mod underlay;
mod variant;

pub use annotation::{Annotation, AnnotationShape, AnnotationView};
pub use document::{Document, ModelError};
pub use element::{
    BuildingClass, CircularArc, Column, ColumnProfile, DimRef, DimensionChain, Door, Element, ElementId, ElementKind,
    GridLine, Handing, JoinRule, Layer, LayerFunction, Opening, OpeningKind, OpeningType, Param, Phase, PhaseFilter,
    PlacedView, PlanSheet, Point2, ProjectInfo, Recess, SectionKind, SectionView, Slab, SlabKind, SlabOpening,
    SlabType, Space, SpaceUsage, Storey, Top, ViewContent, Wall, WallAlignment, WallEnd, WallSide, WallType,
};
pub use elevator::Elevator;
pub use operation::Operation;
pub use plot::{Plot, PublicEdge, PublicKind};
pub use project::{Project, rough_sill_changes};
pub use roof::{Dormer, DormerWindow, EavePlane, Roof, RoofEdge, RoofFace, RoofWindow};
pub use stair::{FlightLayout, Landing, Railing, RailingPath, Stair, StairFlight, StairLayout, Tread};
pub use terrain::{Terrain, TerrainPoint};
pub use underlay::{ReferenceModel, Underlay, pdf_point_scale};
pub use variant::{Variant, VariantSet};
