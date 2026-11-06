//! Sichtbarkeit (ADR 0050): die Kette Bürostandard, Vorlage, Ansicht und
//! Bauteil auflösen, jede Einstellung mit ihrer Herkunft, und anwenden:
//! Bauzustand, Regeln, in der Ansicht ausgeblendete Bauteile und das
//! vorübergehende Ausblenden und Isolieren (H, I), das nie gedruckt wird.

use std::collections::{BTreeMap, BTreeSet};

use lot_model::{
    Category, Detail, Element, ElementId, PhaseFilter, Project, Rule, RuleAction, ViewContent, ViewSettings,
};

use crate::Color;
use crate::standard::OfficeStandard;

/// Woher eine Einstellung kommt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    Office,
    Template,
    View,
}

impl Origin {
    /// Etwa „aus Vorlage Bauantrag 1:100“.
    pub fn text(self, template: Option<&str>) -> String {
        match (self, template) {
            (Origin::Office, _) => "aus dem Bürostandard".into(),
            (Origin::Template, Some(t)) => format!("aus Vorlage {t}"),
            (Origin::Template, None) => "aus der Vorlage".into(),
            (Origin::View, _) => "in dieser Ansicht gesetzt".into(),
        }
    }
}

/// Ein Wert und woher er kommt.
#[derive(Debug, Clone, PartialEq)]
pub struct Setting<T> {
    pub value: T,
    pub origin: Origin,
}

/// Eine Regel, wie sie in der Ansicht gilt.
#[derive(Debug, Clone, PartialEq)]
pub struct LookRule {
    pub rule: Rule,
    /// Thema der Regel; `None` bei eigenen Regeln der Ansicht.
    pub theme: Option<String>,
    pub origin: Origin,
    /// Die Ansicht schaltet Regeln aus dem Thema ab.
    pub on: bool,
}

/// Wie eine Ansicht aussieht, mit Herkunft jeder Einstellung.
#[derive(Debug, Clone, PartialEq)]
pub struct Look {
    pub view: ViewContent,
    pub template: Option<String>,
    pub scale: Setting<f64>,
    pub detail: Setting<Detail>,
    pub phase: Setting<PhaseFilter>,
    pub theme: Setting<Option<String>>,
    pub cut_height: Setting<f64>,
    pub faded_below: Setting<bool>,
    /// Als Präsentationsplan zeichnen (Thema oder Inhalt „Präsentation“).
    pub presentation: bool,
    pub rules: Vec<LookRule>,
    /// In dieser Ansicht ausgeblendete Bauteile.
    pub hidden: Vec<ElementId>,
}

/// Die Kette für eine Ansicht auflösen.
pub fn look(project: &Project, standard: &OfficeStandard, view: ViewContent) -> Look {
    let own = project.view_settings(view).map(|(_, s)| s.clone()).unwrap_or_else(|| ViewSettings::new(view));
    let template = own.template.as_deref().and_then(|n| standard.template(n));
    let d = &standard.views;
    let pick = |own: Option<f64>, from_template: Option<f64>, office: f64| match (own, from_template) {
        (Some(v), _) => Setting { value: v, origin: Origin::View },
        (None, Some(v)) => Setting { value: v, origin: Origin::Template },
        (None, None) => Setting { value: office, origin: Origin::Office },
    };
    let office_scale = if view == ViewContent::SitePlan { d.site_scale } else { d.scale };
    let scale = pick(own.scale, template.and_then(|t| t.scale).map(|b| b.scale), office_scale);
    let detail = match (own.detail, template.and_then(|t| t.scale).and_then(|b| b.detail)) {
        (Some(v), _) => Setting { value: v, origin: Origin::View },
        (None, Some(v)) => Setting { value: v, origin: Origin::Template },
        (None, None) => Setting { value: standard.style_for(scale.value).detail, origin: Origin::Office },
    };
    let phase = match (own.phase, template.and_then(|t| t.phase)) {
        (Some(v), _) => Setting { value: v, origin: Origin::View },
        (None, Some(v)) => Setting { value: v, origin: Origin::Template },
        (None, None) => Setting { value: d.phase, origin: Origin::Office },
    };
    // Ein leerer Name in der Ansicht heißt ausdrücklich: kein Thema.
    let theme = match (&own.theme, template.and_then(|t| t.theme.clone())) {
        (Some(v), _) => Setting { value: (!v.is_empty()).then(|| v.clone()), origin: Origin::View },
        (None, Some(v)) => Setting { value: Some(v), origin: Origin::Template },
        (None, None) => Setting { value: None, origin: Origin::Office },
    };
    let cut_height = pick(own.cut_height, None, d.cut_height);
    let faded_below = match own.faded_below {
        Some(v) => Setting { value: v, origin: Origin::View },
        None => Setting { value: d.faded_below, origin: Origin::Office },
    };
    let theme_def = theme.value.as_deref().and_then(|n| standard.theme(n));
    let mut rules: Vec<LookRule> = theme_def
        .map(|t| {
            t.rules
                .iter()
                .map(|r| LookRule {
                    rule: r.clone(),
                    theme: Some(t.name.clone()),
                    origin: theme.origin,
                    on: !own.rules_off.contains(&r.name),
                })
                .collect()
        })
        .unwrap_or_default();
    rules.extend(own.rules.iter().map(|r| LookRule { rule: r.clone(), theme: None, origin: Origin::View, on: true }));
    Look {
        view,
        template: own.template.clone().filter(|_| template.is_some()),
        scale,
        detail,
        phase,
        presentation: theme_def.is_some_and(|t| t.presentation) || matches!(view, ViewContent::Presentation { .. }),
        theme,
        cut_height,
        faded_below,
        rules,
        hidden: own.hidden.clone(),
    }
}

/// Vorübergehend ausgeblendet oder isoliert (H, I): nur am Bildschirm, nie im Druck.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Temporary {
    pub hidden: BTreeSet<ElementId>,
    /// Isoliert: nur diese Bauteile (und ihre Wände) bleiben zu sehen.
    pub isolated: Option<BTreeSet<ElementId>>,
}

impl Temporary {
    pub fn is_active(&self) -> bool {
        !self.hidden.is_empty() || self.isolated.is_some()
    }
}

/// Farben und Detailgrad, mit denen eine Ansicht zeichnet.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Paint {
    pub detail: Option<Detail>,
    /// Füllfarbe je Bauteil nach den Regeln.
    pub colors: BTreeMap<ElementId, Color>,
}

/// Grau für die Regelwirkung „grau“.
pub const RULE_GRAY: Color = Color::gray(0.82);

/// Was eine Ansicht zeigt.
#[derive(Debug, Clone)]
pub struct Shown {
    /// Das Projekt ohne Ausgeblendetes, im Bauzustand der Ansicht.
    pub project: Project,
    pub paint: Paint,
    /// Je Regel in [`Look::rules`]: wie viele Bauteile sie trifft.
    pub counts: Vec<usize>,
    /// Wie viele Bauteile ausgeblendet sind: durch Regeln, in der Ansicht, vorübergehend.
    pub hidden_by_rules: usize,
}

/// Die erste eingeschaltete Regel, die das Bauteil trifft.
pub fn rule_for(project: &Project, look: &Look, id: ElementId) -> Option<usize> {
    look.rules.iter().position(|r| r.on && r.rule.matches(project, id))
}

/// Wendet die Ansicht an. Mit `temporary` die Wirkung von H und I (nicht im Druck).
pub fn apply(project: &Project, look: &Look, temporary: &Temporary) -> Shown {
    let phased = project.phase_view(look.phase.value);
    let mut hidden: BTreeSet<ElementId> = BTreeSet::new();
    let mut paint = Paint { detail: Some(look.detail.value), colors: BTreeMap::new() };
    let mut counts = vec![0; look.rules.len()];
    for (id, e) in &phased.elements {
        if Category::of(e).is_none() {
            continue;
        }
        let Some(i) = rule_for(&phased, look, *id) else { continue };
        counts[i] += 1;
        match look.rules[i].rule.then {
            RuleAction::Hide => {
                hidden.insert(*id);
            }
            RuleAction::Gray => {
                paint.colors.insert(*id, RULE_GRAY);
            }
            RuleAction::Color([r, g, b]) => {
                paint.colors.insert(*id, Color::rgb8(r, g, b));
            }
        }
    }
    let hidden_by_rules = hidden.len();
    hidden.extend(look.hidden.iter().copied());
    hidden.extend(temporary.hidden.iter().copied());
    if let Some(keep) = &temporary.isolated {
        // Isolierte Öffnungen behalten ihre Wand, sonst ließen sie sich nicht zeichnen.
        let hosts: BTreeSet<ElementId> = keep
            .iter()
            .filter_map(|id| match phased.elements.get(id) {
                Some(Element::Opening(o)) => Some(o.host),
                Some(Element::SlabOpening(o)) => Some(o.host),
                _ => None,
            })
            .collect();
        for (id, e) in &phased.elements {
            if Category::of(e).is_some() && !keep.contains(id) && !hosts.contains(id) {
                hidden.insert(*id);
            }
        }
    }
    Shown { project: phased.without(&hidden), paint, counts, hidden_by_rules }
}

/// Warum ein Bauteil in einer Ansicht nicht zu sehen ist.
#[derive(Debug, Clone, PartialEq)]
pub enum Hidden {
    /// Es liegt in einem anderen Geschoss.
    OtherStorey(ElementId),
    /// Der Bauzustand der Ansicht zeigt es nicht.
    Phase(PhaseFilter, Origin),
    /// Eine Regel blendet es aus: Stelle in [`Look::rules`].
    Rule(usize),
    /// In dieser Ansicht ausgeblendet.
    InView,
    /// Vorübergehend ausgeblendet (H).
    Temporary,
    /// Andere Bauteile sind isoliert (I).
    NotIsolated,
    /// Es gehört zu einer Variante, die nicht aktiv ist: die Gruppe.
    OtherVariant(ElementId),
    /// Es beginnt über der Schnitthöhe des Grundrisses: seine Unterkante über OKFF.
    AboveCut(f64),
}

/// Geschoss eines Bauteils, soweit es eins hat.
pub fn storey_of(project: &Project, id: ElementId) -> Option<ElementId> {
    Some(match project.elements.get(&id)? {
        Element::Wall(w) => w.storey,
        Element::Opening(o) => project.wall(o.host)?.storey,
        Element::Column(c) => c.storey,
        Element::Space(s) => s.storey,
        Element::Slab(s) => s.storey,
        Element::SlabOpening(o) => return storey_of(project, o.host),
        Element::Roof(r) => r.storey,
        Element::DimensionChain(d) => d.storey,
        Element::Underlay(u) => u.storey,
        Element::Annotation(a) => match a.view {
            lot_model::AnnotationView::FloorPlan { storey } => storey,
            _ => return None,
        },
        _ => return None,
    })
}

/// Alle Gründe, warum `id` in der Ansicht fehlt; leer, wenn es zu sehen ist.
pub fn why_hidden(project: &Project, look: &Look, temporary: &Temporary, id: ElementId) -> Vec<Hidden> {
    let mut out = Vec::new();
    let Some(e) = project.elements.get(&id) else { return out };
    for (set, v) in project.variant_sets() {
        let active = v.variants.get(v.active).map(|a| a.elements.contains(&id)).unwrap_or(false);
        if !active && v.variants.iter().any(|a| a.elements.contains(&id)) {
            out.push(Hidden::OtherVariant(set));
        }
    }
    if let ViewContent::FloorPlan { storey } | ViewContent::Presentation { storey } = look.view {
        match storey_of(project, id) {
            Some(s) if s != storey => out.push(Hidden::OtherStorey(s)),
            _ => {}
        }
        let base = match e {
            Element::Wall(w) => Some(w.base_offset),
            Element::Column(c) => Some(c.base_offset),
            _ => None,
        };
        if let Some(b) = base.filter(|b| *b > look.cut_height.value) {
            out.push(Hidden::AboveCut(b));
        }
    }
    if e.phase().is_some_and(|p| !p.shown_in(look.phase.value)) {
        out.push(Hidden::Phase(look.phase.value, look.phase.origin));
    }
    if let Some(i) = rule_for(project, look, id).filter(|i| look.rules[*i].rule.then == RuleAction::Hide) {
        out.push(Hidden::Rule(i));
    }
    if look.hidden.contains(&id) {
        out.push(Hidden::InView);
    }
    if temporary.hidden.contains(&id) {
        out.push(Hidden::Temporary);
    }
    if let Some(keep) = &temporary.isolated
        && !keep.contains(&id)
    {
        out.push(Hidden::NotIsolated);
    }
    out
}

#[cfg(test)]
mod tests {
    use lot_model::{Condition, Phase, Property, ScaleBlock, ViewTemplate};

    use super::*;

    fn standard() -> &'static OfficeStandard {
        OfficeStandard::builtin()
    }

    fn sample_with(settings: Option<ViewSettings>) -> Project {
        let mut p = Project::sample();
        if let Some(s) = settings {
            let id = p.next_id();
            p.elements.insert(id, Element::ViewSettings(s));
        }
        p
    }

    const PLAN: ViewContent = ViewContent::FloorPlan { storey: ElementId(1) };

    #[test]
    fn builtin_templates_and_themes_are_there() {
        let names: Vec<&str> = standard().templates.iter().map(|t| t.name.as_str()).collect();
        for n in [
            "Entwurf 1:200",
            "Bauantrag 1:100",
            "Werkplan 1:50",
            "Präsentation",
            "Umbauplan",
            "Flächenplan",
            "Brandschutzplan",
        ] {
            assert!(names.contains(&n), "{n} fehlt");
        }
        // Jedes Thema einer Vorlage gibt es.
        for t in &standard().templates {
            if let Some(theme) = &t.theme {
                assert!(standard().theme(theme).is_some(), "{theme}");
            }
        }
    }

    #[test]
    fn the_chain_shows_where_each_value_comes_from() {
        // Ohne Einstellungen: alles aus dem Bürostandard.
        let l = look(&sample_with(None), standard(), PLAN);
        assert_eq!((l.scale.value, l.scale.origin), (100.0, Origin::Office));
        assert_eq!(l.detail.value, Detail::Medium);
        assert_eq!(l.phase.origin, Origin::Office);
        // Vorlage Werkplan: Maßstab aus der Vorlage, Detailgrad dazu aus dem Bürostandard.
        let mut s = ViewSettings::new(PLAN);
        s.template = Some("Werkplan 1:50".into());
        let l = look(&sample_with(Some(s.clone())), standard(), PLAN);
        assert_eq!((l.scale.value, l.scale.origin), (50.0, Origin::Template));
        assert_eq!((l.detail.value, l.detail.origin), (Detail::Fine, Origin::Office));
        assert_eq!(l.phase.origin, Origin::Office, "Werkplan hat keinen Baustein Bauzustand");
        assert_eq!(l.scale.origin.text(l.template.as_deref()), "aus Vorlage Werkplan 1:50");
        // Die Ansicht setzt den Detailgrad selbst.
        s.detail = Some(Detail::Coarse);
        let l = look(&sample_with(Some(s)), standard(), PLAN);
        assert_eq!((l.detail.value, l.detail.origin), (Detail::Coarse, Origin::View));
        // Unbekannte Vorlage zählt nicht.
        let mut s = ViewSettings::new(PLAN);
        s.template = Some("Gibt es nicht".into());
        assert_eq!(look(&sample_with(Some(s)), standard(), PLAN).template, None);
    }

    #[test]
    fn a_template_block_controls_only_its_fields() {
        let mut st = standard().clone();
        st.templates.push(ViewTemplate {
            name: "Nur Maßstab".into(),
            scale: Some(ScaleBlock { scale: 200.0, detail: Some(Detail::Medium) }),
            phase: None,
            theme: None,
        });
        let mut s = ViewSettings::new(PLAN);
        s.template = Some("Nur Maßstab".into());
        s.phase = Some(PhaseFilter::Planned);
        let l = look(&sample_with(Some(s)), &st, PLAN);
        assert_eq!((l.detail.value, l.detail.origin), (Detail::Medium, Origin::Template));
        assert_eq!((l.phase.value, l.phase.origin), (PhaseFilter::Planned, Origin::View));
    }

    #[test]
    fn theme_rules_paint_hide_and_count() {
        let mut p = sample_with(None);
        let wall = p.walls().next().unwrap().0;
        if let Some(Element::Wall(w)) = p.elements.get_mut(&wall) {
            w.props.fire_rating = Some("F90".into());
        }
        let mut s = ViewSettings::new(PLAN);
        s.template = Some("Brandschutzplan".into());
        let id = p.next_id();
        p.elements.insert(id, Element::ViewSettings(s.clone()));
        let l = look(&p, standard(), PLAN);
        assert_eq!(l.theme.value.as_deref(), Some("Brandschutz"));
        let shown = apply(&p, &l, &Temporary::default());
        assert_eq!(shown.paint.colors.get(&wall), Some(&Color::rgb8(226, 35, 26)));
        let f90 = l.rules.iter().position(|r| r.rule.name == "F90").unwrap();
        assert_eq!(shown.counts[f90], 1);
        // Eigene Regel der Ansicht: Öffnungen ausblenden; gezählt und mit Grund.
        s.rules.push(Rule {
            name: "Öffnungen aus".into(),
            when: vec![Condition::Kind(Category::Opening)],
            then: RuleAction::Hide,
        });
        p.elements.insert(id, Element::ViewSettings(s.clone()));
        let l = look(&p, standard(), PLAN);
        let shown = apply(&p, &l, &Temporary::default());
        let window = ElementId(4);
        assert!(!shown.project.elements.contains_key(&window));
        assert_eq!(shown.hidden_by_rules, 1);
        let i = l.rules.iter().position(|r| r.rule.name == "Öffnungen aus").unwrap();
        assert_eq!(why_hidden(&p, &l, &Temporary::default(), window), vec![Hidden::Rule(i)]);
        // Abgeschaltet gilt die Regel des Themas nicht mehr.
        s.rules_off.push("F90".into());
        p.elements.insert(id, Element::ViewSettings(s));
        let l = look(&p, standard(), PLAN);
        let shown = apply(&p, &l, &Temporary::default());
        assert_eq!(shown.paint.colors.get(&wall), Some(&RULE_GRAY), "nun greift „Übrige Wände grau“");
        assert!(l.rules.iter().any(|r| r.rule.name == "F90" && !r.on));
        let _ = Property::FireRating;
    }

    #[test]
    fn painted_walls_are_filled_in_their_color() {
        let p = Project::sample();
        let wall = p.walls().next().unwrap().0;
        let red = Color::rgb8(226, 35, 26);
        let paint = Paint { detail: Some(Detail::Fine), colors: [(wall, red)].into() };
        let view = crate::floor_plan::FloorPlanView { storey: ElementId(1), cut_height: 1.0, scale: 100.0 };
        let viewport = crate::Viewport { scale: 100.0, origin: crate::PaperPoint::new(0.0, 0.0) };
        let objects = crate::floor_plan::floor_plan_with(&p, view, standard(), viewport, &paint);
        let filled = |c: Color| {
            objects.iter().any(|o| matches!(o, crate::PlanObject::Area(a) if a.fill == crate::Fill::Solid(c)))
        };
        assert!(filled(red));
        // Ohne Farbe: wie bisher, keine rote Fläche.
        let plain = crate::floor_plan::floor_plan(&p, view, standard(), viewport);
        assert!(!plain.iter().any(|o| matches!(o, crate::PlanObject::Area(a) if a.fill == crate::Fill::Solid(red))));
    }

    #[test]
    fn hidden_in_view_temporary_and_isolated_have_reasons() {
        let p = sample_with(None);
        let wall = p.walls().next().unwrap().0;
        let window = ElementId(4);
        let l = look(&p, standard(), PLAN);
        let temp = Temporary { hidden: [window].into(), isolated: None };
        let shown = apply(&p, &l, &temp);
        assert!(!shown.project.elements.contains_key(&window));
        assert_eq!(why_hidden(&p, &l, &temp, window), vec![Hidden::Temporary]);
        // Isoliertes Fenster behält seine Wand.
        let temp = Temporary { hidden: Default::default(), isolated: Some([window].into()) };
        let shown = apply(&p, &l, &temp);
        assert!(shown.project.elements.contains_key(&window) && shown.project.elements.contains_key(&wall));
        // Im Bestand fehlt ein Neubau.
        let mut s = ViewSettings::new(PLAN);
        s.phase = Some(PhaseFilter::Existing);
        s.hidden = vec![wall];
        let p = sample_with(Some(s));
        let l = look(&p, standard(), PLAN);
        let reasons = why_hidden(&p, &l, &Temporary::default(), wall);
        assert!(reasons.contains(&Hidden::Phase(PhaseFilter::Existing, Origin::View)));
        assert!(reasons.contains(&Hidden::InView));
        assert!(Phase::New.shown_in(PhaseFilter::Planned));
    }
}
