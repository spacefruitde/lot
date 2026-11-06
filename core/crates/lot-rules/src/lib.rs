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
//! - [`accessible`]: Barrierefreiheit nach DIN 18040-2 (5.03)
//! - [`escape`]: Rettungswege nach MBO §§ 33 bis 35 (5.05)
//! - [`land_use`]: GRZ und GFZ nach §§ 19, 20 BauNVO
//! - [`schedule`]: Tür- und Fensterliste (4.05)
//! - [`report`]: Prüfbericht aus allen Prüfungen

pub mod accessible;
pub mod areas;
pub mod building;
pub mod escape;
pub mod falls;
pub mod land_use;
pub mod landesrecht;
pub mod report;
pub mod rooms;
pub mod schedule;
pub mod setbacks;
pub mod stairs;

pub use areas::{AreaReport, SpaceArea, StoreyAreas, area_report};
pub use building::{BuildingClassResult, building_class};
pub use landesrecht::StateRules;
pub use report::{CheckReport, Finding, ReportSection, Status, check_project};
pub use schedule::{OpeningRow, OpeningType, opening_schedule};
pub use setbacks::{SetbackArea, SetbackReport, setbacks};
pub use stairs::{StairCheck, check_stairs};
