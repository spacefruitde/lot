//! Landesrecht als Daten je Bundesland (CLAUDE.md, Regel 8; PLAN.md,
//! Abschnitt 4: „Regeln und Prüfungen“).
//!
//! Jedes Land ist eine RON-Datei in `landesrecht/`. Solange Bens Bundesland
//! fehlt, gilt die Musterbauordnung (`mbo.ron`). Werte und Paragrafen stehen
//! in der Datei, die Prüfungen lesen sie nur.

use std::sync::OnceLock;

use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct StateRules {
    /// Kürzel nach ISO 3166-2 ohne „DE-“, oder „MBO“.
    pub code: String,
    pub name: String,
    pub setbacks: SetbackRules,
    pub building_classes: BuildingClassRules,
    pub escape: EscapeRules,
    pub accessible: AccessibleRules,
    pub land_use: LandUseRules,
    pub submission: SubmissionRules,
}

/// Abstandsflächen (5.01).
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct SetbackRules {
    pub source: String,
    /// Tiefe als Anteil der Wandhöhe H.
    pub factor: f64,
    /// Mindesttiefe in m.
    pub minimum: f64,
    /// Feste Tiefe für kleine Wohngebäude, falls das Land sie kennt.
    #[serde(default)]
    pub small_buildings: Option<SmallBuildings>,
    /// Dachneigung in Grad, ab der die Dachhöhe voll zur Wandhöhe zählt.
    pub roof_full_above: f64,
    /// Dachneigung in Grad, ab der ein Drittel der Dachhöhe zählt.
    pub roof_third_above: f64,
    /// Ob Abstandsflächen auf öffentlichen Flächen bis zu deren Mitte liegen dürfen.
    pub public_to_center: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub struct SmallBuildings {
    /// Höchste Gebäudeklasse, für die die Regel gilt.
    pub max_class: u8,
    /// Höchstens so viele oberirdische Geschosse.
    pub max_storeys: usize,
    pub depth: f64,
}

/// Gebäudeklassen.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct BuildingClassRules {
    pub source: String,
    /// Höhengrenze der Gebäudeklassen 1 bis 3, in m.
    pub low: f64,
    /// Höhengrenze der Gebäudeklasse 4, in m.
    pub medium: f64,
    pub small_units: usize,
    pub small_area: f64,
    pub unit_area: f64,
    /// Ab dieser Höhe der Deckenoberkante über dem Gelände im Mittel ist ein Geschoss oberirdisch.
    pub above_ground: f64,
}

/// Rettungswege (5.05).
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct EscapeRules {
    pub max_distance: f64,
    pub distance_source: String,
    pub ladder_height: f64,
    pub ladder_source: String,
    pub window_width: f64,
    pub window_height: f64,
    pub window_sill: f64,
    pub window_source: String,
    pub stair_window_area: f64,
    pub smoke_vent_area: f64,
    pub stair_source: String,
}

/// Wann Wohnungen barrierefrei sein müssen (5.03).
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct AccessibleRules {
    /// Ab so vielen Wohnungen im Gebäude.
    pub min_dwellings: usize,
    pub source: String,
}

/// Maß der baulichen Nutzung: Vollgeschosse für die Geschossfläche.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct LandUseRules {
    pub full_storey_height: f64,
    pub full_storey_share: f64,
    pub source: String,
}

/// Bauvorlagen (5.02).
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct SubmissionRules {
    pub source: String,
    /// Kleinster Maßstab des Lageplans, als Maßstabszahl.
    pub site_plan_scale: f64,
    /// Maßstab der Bauzeichnungen.
    pub drawing_scale: f64,
}

/// Alle Länder, für die Daten vorliegen.
fn all() -> &'static [StateRules] {
    static RULES: OnceLock<Vec<StateRules>> = OnceLock::new();
    RULES.get_or_init(|| {
        [include_str!("../landesrecht/mbo.ron")]
            .iter()
            .map(|text| ron::from_str(text).expect("eingebautes Landesrecht ist lesbar"))
            .collect()
    })
}

impl StateRules {
    /// Das Landesrecht zum Kürzel, etwa „NW“. Fehlt es, gilt die Musterbauordnung;
    /// das zweite Feld sagt dann, dass ersetzt wurde.
    pub fn for_state(code: &str) -> (&'static StateRules, bool) {
        let code = code.trim().to_uppercase();
        match all().iter().find(|r| r.code == code) {
            Some(r) => (r, false),
            None => (all().iter().find(|r| r.code == "MBO").expect("MBO vorhanden"), !code.is_empty()),
        }
    }

    /// Kürzel der Länder mit eigenen Daten.
    pub fn codes() -> Vec<&'static str> {
        all().iter().map(|r| r.code.as_str()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mbo_loads_and_unknown_state_falls_back() {
        let (mbo, replaced) = StateRules::for_state("");
        assert_eq!(mbo.code, "MBO");
        assert!(!replaced);
        assert_eq!(mbo.setbacks.factor, 0.4);
        assert_eq!(mbo.setbacks.minimum, 3.0);
        let (r, replaced) = StateRules::for_state("xx");
        assert_eq!(r.code, "MBO");
        assert!(replaced);
        assert!(StateRules::codes().contains(&"MBO"));
    }
}
