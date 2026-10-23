//! Regeln und Mengen (PLAN.md, Abschnitt 4: „Regeln und Prüfungen“).
//!
//! Jede Regel nennt ihre Quelle mit Norm oder Verordnung, Abschnitt und Fassung
//! (CLAUDE.md, Regel 8). Bisher:
//!
//! - [`areas`]: Flächen nach DIN 277:2021-08 (4.01) und Wohnfläche nach der
//!   Wohnflächenverordnung (WoFlV) vom 25.11.2003 (4.02), dazu der Brutto-Rauminhalt (4.03)

pub mod areas;

pub use areas::{AreaReport, RoomArea, StoreyAreas, area_report};
