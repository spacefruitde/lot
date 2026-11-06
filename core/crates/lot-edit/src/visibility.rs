//! Sichtbarkeit bearbeiten (ADR 0050): Einstellungen einer Ansicht setzen und
//! zurücksetzen, Vorlage wählen, Regeln ein- und ausschalten, Bauteile in
//! einer Ansicht ausblenden, und „Warum sehe ich das nicht?“ mit einer
//! Lösung je Grund.

use lot_model::{
    Detail, Element, ElementId, Operation, PhaseFilter, Project, Rule, RuleAction, ViewContent, ViewSettings,
};
use lot_plan::standard::OfficeStandard;
use lot_plan::views::{self, Hidden, Look, Temporary};
use serde::{Deserialize, Serialize};

use crate::Command;
use crate::navigation::ViewRef;

/// Eine Einstellung einer Ansicht; `None` übernimmt sie wieder aus Vorlage oder Bürostandard.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ViewField {
    Scale(Option<f64>),
    Detail(Option<Detail>),
    Phase(Option<PhaseFilter>),
    /// Name eines Themas; `Some("")` heißt ausdrücklich keins.
    Theme(Option<String>),
    CutHeight(Option<f64>),
    FadedBelow(Option<bool>),
}

impl ViewField {
    pub fn parse(text: &str) -> Option<ViewField> {
        ron::from_str(text).ok()
    }
}

/// Ändert die Einstellungen einer Ansicht; legt sie an, wenn es noch keine
/// gibt, und löscht sie, wenn die Ansicht danach nichts mehr selbst setzt.
pub fn change_ops(project: &Project, view: ViewContent, change: impl FnOnce(&mut ViewSettings)) -> Vec<Operation> {
    match project.view_settings(view) {
        Some((id, old)) => {
            let mut new = old.clone();
            change(&mut new);
            if new == *old {
                Vec::new()
            } else if new.is_empty() {
                vec![Operation::Delete { id, element: Element::ViewSettings(old.clone()) }]
            } else {
                vec![Operation::Replace {
                    id,
                    old: Element::ViewSettings(old.clone()),
                    new: Element::ViewSettings(new),
                }]
            }
        }
        None => {
            let mut new = ViewSettings::new(view);
            change(&mut new);
            if new.is_empty() {
                return Vec::new();
            }
            vec![Operation::Insert { id: project.next_id(), element: Element::ViewSettings(new) }]
        }
    }
}

pub fn set_field_ops(project: &Project, view: ViewContent, field: &ViewField) -> Vec<Operation> {
    change_ops(project, view, |s| match field.clone() {
        ViewField::Scale(v) => s.scale = v,
        ViewField::Detail(v) => s.detail = v,
        ViewField::Phase(v) => s.phase = v,
        ViewField::Theme(v) => s.theme = v,
        ViewField::CutHeight(v) => s.cut_height = v,
        ViewField::FadedBelow(v) => s.faded_below = v,
    })
}

/// Wählt eine Vorlage. Was ihre Bausteine steuern, setzt die Ansicht danach
/// nicht mehr selbst, damit die Vorlage wirkt (Abweichen ist danach bewusst).
pub fn set_template_ops(
    project: &Project,
    standard: &OfficeStandard,
    view: ViewContent,
    name: Option<&str>,
) -> Result<Vec<Operation>, String> {
    let template = match name {
        Some(n) => Some(standard.template(n).ok_or_else(|| format!("Die Vorlage „{n}“ gibt es nicht"))?),
        None => None,
    };
    Ok(change_ops(project, view, |s| {
        s.template = template.map(|t| t.name.clone());
        if let Some(t) = template {
            if let Some(b) = t.scale {
                s.scale = None;
                if b.detail.is_some() {
                    s.detail = None;
                }
            }
            if t.phase.is_some() {
                s.phase = None;
            }
            if t.theme.is_some() {
                s.theme = None;
                s.rules_off.clear();
            }
        }
    }))
}

/// Schaltet eine Regel des Themas in dieser Ansicht ab oder wieder ein.
pub fn set_rule_on_ops(project: &Project, view: ViewContent, name: &str, on: bool) -> Vec<Operation> {
    change_ops(project, view, |s| {
        s.rules_off.retain(|n| n != name);
        if !on {
            s.rules_off.push(name.to_string());
        }
    })
}

pub fn add_rule_ops(project: &Project, view: ViewContent, rule: &Rule) -> Vec<Operation> {
    change_ops(project, view, |s| {
        s.rules.retain(|r| r.name != rule.name);
        s.rules.push(rule.clone());
    })
}

pub fn remove_rule_ops(project: &Project, view: ViewContent, name: &str) -> Vec<Operation> {
    change_ops(project, view, |s| s.rules.retain(|r| r.name != name))
}

/// Bauteile in dieser Ansicht ausblenden (die Stufe „Bauteil“ der Kette).
pub fn hide_ops(project: &Project, view: ViewContent, ids: &[ElementId]) -> Vec<Operation> {
    change_ops(project, view, |s| {
        for id in ids {
            if !s.hidden.contains(id) {
                s.hidden.push(*id);
            }
        }
    })
}

/// Ausgeblendete wieder zeigen; leer zeigt alle.
pub fn show_ops(project: &Project, view: ViewContent, ids: &[ElementId]) -> Vec<Operation> {
    change_ops(project, view, |s| {
        if ids.is_empty() {
            s.hidden.clear();
        } else {
            s.hidden.retain(|id| !ids.contains(id));
        }
    })
}

/// Wie ein Grund behoben wird.
#[derive(Debug, Clone, PartialEq)]
pub enum Fix {
    Run(Command),
    Go(ViewRef),
}

/// Ein Grund, warum ein Bauteil fehlt, in Worten und mit Lösung.
#[derive(Debug, Clone, PartialEq)]
pub struct Reason {
    pub text: String,
    /// Knopftitel und was er tut.
    pub fix: Option<(String, Fix)>,
}

/// „Warum sehe ich das nicht?“ (ADR 0050): Gründe mit einer Lösung je Grund.
/// Leer, wenn das Bauteil in der Ansicht zu sehen ist.
pub fn reasons(project: &Project, look: &Look, temporary: &Temporary, id: ElementId) -> Vec<Reason> {
    let view = look.view;
    let template = look.template.as_deref();
    views::why_hidden(project, look, temporary, id)
        .into_iter()
        .map(|h| match h {
            Hidden::OtherStorey(s) => Reason {
                text: format!("liegt im Geschoss {}", project.storey(s).map_or("?", |st| st.name.as_str())),
                fix: Some(("Dorthin".into(), Fix::Go(ViewRef::FloorPlan(s.0)))),
            },
            Hidden::Phase(filter, origin) => Reason {
                text: format!("Bauzustand: die Ansicht zeigt {} ({})", filter.title(), origin.text(template)),
                fix: Some((
                    "Umbauplan zeigen".into(),
                    Fix::Run(Command::SetViewField(view, ViewField::Phase(Some(PhaseFilter::All)))),
                )),
            },
            Hidden::Rule(i) => {
                let r = &look.rules[i];
                let from = match &r.theme {
                    Some(t) => format!("Thema {t}, {}", r.origin.text(template)),
                    None => "Regel dieser Ansicht".into(),
                };
                let fix = match &r.theme {
                    Some(_) => Command::SetRuleOn(view, r.rule.name.clone(), false),
                    None => Command::RemoveViewRule(view, r.rule.name.clone()),
                };
                Reason {
                    text: format!("Regel „{}“ blendet es aus ({from})", r.rule.name),
                    fix: Some(("Regel abschalten".into(), Fix::Run(fix))),
                }
            }
            Hidden::InView => Reason {
                text: "in dieser Ansicht ausgeblendet".into(),
                fix: Some(("Wieder zeigen".into(), Fix::Run(Command::ShowInView(view, vec![id.0])))),
            },
            Hidden::Temporary => Reason {
                text: "vorübergehend ausgeblendet (H)".into(),
                fix: Some(("Alles zeigen".into(), Fix::Run(Command::TempReset))),
            },
            Hidden::NotIsolated => Reason {
                text: "andere Bauteile sind isoliert (I)".into(),
                fix: Some(("Alles zeigen".into(), Fix::Run(Command::TempReset))),
            },
            Hidden::OtherVariant(set) => Reason {
                text: match project.elements.get(&set) {
                    Some(Element::VariantSet(v)) => format!("gehört zu einer anderen Variante der Gruppe {}", v.name),
                    _ => "gehört zu einer anderen Variante".into(),
                },
                fix: Some(("Variante wechseln".into(), Fix::Run(Command::SwitchVariant(set.0)))),
            },
            Hidden::AboveCut(base) => Reason {
                text: format!(
                    "beginnt {} über OKFF, über der Schnitthöhe {}",
                    crate::format_m(base),
                    crate::format_m(look.cut_height.value)
                ),
                fix: Some((
                    "Schnitthöhe darüber".into(),
                    Fix::Run(Command::SetViewField(
                        view,
                        ViewField::CutHeight(Some(((base + 0.1) * 100.0).round() / 100.0)),
                    )),
                )),
            },
        })
        .collect()
}

/// Wie viele Bauteile eine Regel trifft, in Worten: „12 Bauteile ausgeblendet“.
pub fn count_text(action: RuleAction, count: usize) -> String {
    let n = if count == 1 { "1 Bauteil".to_string() } else { format!("{count} Bauteile") };
    match action {
        RuleAction::Hide => format!("{n} ausgeblendet"),
        RuleAction::Gray => format!("{n} grau"),
        RuleAction::Color(_) => format!("{n} farbig"),
    }
}
