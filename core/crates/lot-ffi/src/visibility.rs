//! Sichtbarkeit für die App (ADR 0050): Darstellung der gezeigten Ansicht mit
//! Herkunft jeder Einstellung, Regeln mit Zähler, und „Warum sehe ich das nicht?“.

use lot_model::{Detail, PhaseFilter};
use lot_plan::standard::OfficeStandard;
use lot_plan::views::{Look, Origin};

/// Darstellung der gezeigten Ansicht. Je Einstellung: Wert, woher er kommt
/// („aus Vorlage Bauantrag 1:100“) und ob die Ansicht ihn selbst setzt.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct LookInfo {
    /// Name der Vorlage oder leer.
    pub template: String,
    pub templates: Vec<String>,
    pub themes: Vec<String>,
    pub scale: f64,
    pub scale_origin: String,
    pub scale_own: bool,
    /// Index in [`detail_levels`].
    pub detail: u32,
    pub detail_origin: String,
    pub detail_own: bool,
    /// Index in [`phase_filters`].
    pub phase: u32,
    pub phase_origin: String,
    pub phase_own: bool,
    /// Name des Themas oder leer.
    pub theme: String,
    pub theme_origin: String,
    pub theme_own: bool,
    /// Nur Grundrisse haben Schnitthöhe und Geschoss darunter.
    pub is_plan: bool,
    pub cut_height: f64,
    pub cut_height_origin: String,
    pub cut_height_own: bool,
    pub faded_below: bool,
    pub faded_below_origin: String,
    pub faded_below_own: bool,
    pub rules: Vec<RuleInfo>,
    /// In der Ansicht ausgeblendete Bauteile (Stufe „Bauteil“).
    pub hidden: Vec<u64>,
    /// Wie viele Bauteile Regeln ausblenden.
    pub hidden_by_rules: u32,
}

/// Eine Regel der Ansicht mit ihrer Wirkung.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct RuleInfo {
    pub name: String,
    /// Etwa „Thema Brandschutz, aus Vorlage Brandschutzplan“ oder „Regel dieser Ansicht“.
    pub origin: String,
    pub on: bool,
    /// Eigene Regel der Ansicht: löschen statt abschalten.
    pub own: bool,
    /// Etwa „12 Bauteile ausgeblendet“.
    pub count: String,
    /// Füllfarbe als sRGB, bei „ausblenden“ leer.
    pub color: Vec<u8>,
}

/// Ein gewähltes Bauteil, das die Ansicht nicht zeigt, mit den Gründen.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct HiddenInfo {
    pub element: u64,
    pub label: String,
    pub reasons: Vec<ReasonInfo>,
}

/// Ein Grund mit Lösung: `fix` ist ein Befehl oder `go:` und eine Ansicht.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct ReasonInfo {
    pub text: String,
    pub fix_title: String,
    pub fix: String,
}

pub(crate) fn look_info(look: &Look, counts: &[usize], hidden_by_rules: usize) -> LookInfo {
    let standard = OfficeStandard::default_standard();
    let t = look.template.as_deref();
    let origin = |o: Origin| o.text(t);
    LookInfo {
        template: look.template.clone().unwrap_or_default(),
        templates: standard.templates.iter().map(|x| x.name.clone()).collect(),
        themes: standard.themes.iter().map(|x| x.name.clone()).collect(),
        scale: look.scale.value,
        scale_origin: origin(look.scale.origin),
        scale_own: look.scale.origin == Origin::View,
        detail: Detail::ALL.iter().position(|d| *d == look.detail.value).unwrap_or(0) as u32,
        detail_origin: origin(look.detail.origin),
        detail_own: look.detail.origin == Origin::View,
        phase: PhaseFilter::ALL.iter().position(|p| *p == look.phase.value).unwrap_or(0) as u32,
        phase_origin: origin(look.phase.origin),
        phase_own: look.phase.origin == Origin::View,
        theme: look.theme.value.clone().unwrap_or_default(),
        theme_origin: origin(look.theme.origin),
        theme_own: look.theme.origin == Origin::View,
        is_plan: matches!(look.view, lot_model::ViewContent::FloorPlan { .. }),
        cut_height: look.cut_height.value,
        cut_height_origin: origin(look.cut_height.origin),
        cut_height_own: look.cut_height.origin == Origin::View,
        faded_below: look.faded_below.value,
        faded_below_origin: origin(look.faded_below.origin),
        faded_below_own: look.faded_below.origin == Origin::View,
        rules: look
            .rules
            .iter()
            .zip(counts)
            .map(|(r, n)| RuleInfo {
                name: r.rule.name.clone(),
                origin: match &r.theme {
                    Some(theme) => format!("Thema {theme}, {}", origin(r.origin)),
                    None => "Regel dieser Ansicht".into(),
                },
                on: r.on,
                own: r.theme.is_none(),
                count: if r.on { lot_edit::visibility::count_text(r.rule.then, *n) } else { "abgeschaltet".into() },
                color: match r.rule.then {
                    lot_model::RuleAction::Color(c) => c.to_vec(),
                    lot_model::RuleAction::Gray => lot_plan::views::RULE_GRAY.bytes().to_vec(),
                    lot_model::RuleAction::Hide => Vec::new(),
                },
            })
            .collect(),
        hidden: look.hidden.iter().map(|id| id.0).collect(),
        hidden_by_rules: hidden_by_rules as u32,
    }
}

/// Detailgrade für die Auswahl im Inspektor: grob, mittel, fein.
#[uniffi::export]
pub fn detail_levels() -> Vec<String> {
    Detail::ALL.iter().map(|d| d.title().to_string()).collect()
}

/// Bauzustände für die Auswahl: Umbauplan, Bestand, Neuzustand.
#[uniffi::export]
pub fn phase_filters() -> Vec<String> {
    PhaseFilter::ALL.iter().map(|p| p.title().to_string()).collect()
}

/// RON-Schreibweise für `set_view_field`: Detailgrad nach Index, `None` übernimmt.
#[uniffi::export]
pub fn detail_field(index: Option<u32>) -> String {
    match index.and_then(|i| Detail::ALL.get(i as usize)) {
        Some(d) => format!("Detail(Some({d:?}))"),
        None => "Detail(None)".into(),
    }
}

/// RON-Schreibweise für `set_view_field`: Bauzustand nach Index, `None` übernimmt.
#[uniffi::export]
pub fn phase_field(index: Option<u32>) -> String {
    match index.and_then(|i| PhaseFilter::ALL.get(i as usize)) {
        Some(p) => format!("Phase(Some({p:?}))"),
        None => "Phase(None)".into(),
    }
}
