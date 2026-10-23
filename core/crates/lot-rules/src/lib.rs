//! Regeln und Mengen (PLAN.md, Abschnitt 4: „Regeln und Prüfungen“).
//!
//! Jede Regel nennt ihre Quelle mit Norm oder Verordnung, Abschnitt und Fassung
//! (CLAUDE.md, Regel 8). Bisher:
//!
//! - [`areas`]: Flächen nach DIN 277:2021-08 (4.01) und Wohnfläche nach der
//!   Wohnflächenverordnung (WoFlV) vom 25.11.2003 (4.02), dazu der Brutto-Rauminhalt (4.03)
//! - [`stairs`]: Treppen nach DIN 18065:2020-08 (1.07)
//! - [`landesrecht`]: Landesrecht als Daten je Bundesland, Voreinstellung MBO
//! - [`building`]: Umriss, Gelände, Oberkante und Gebäudeklasse nach MBO § 2 Abs. 3
//! - [`setbacks`]: Abstandsflächen nach MBO § 6 (5.01)

pub mod areas;
pub mod building;
pub mod landesrecht;
pub mod setbacks;
pub mod stairs;

pub use areas::{AreaReport, SpaceArea, StoreyAreas, area_report};
pub use building::{BuildingClassResult, building_class};
pub use landesrecht::StateRules;
pub use setbacks::{SetbackArea, SetbackReport, setbacks};
pub use stairs::{StairCheck, check_stairs};
