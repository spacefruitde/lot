//! Sichtbarkeit (ADR 0050): Einstellungen je Ansicht, Ansichtsvorlagen aus
//! Bausteinen, Themen und Regeln nach Eigenschaften.
//!
//! Die Kette ist sichtbar: Bürostandard, dann Ansichtsvorlage, dann Ansicht,
//! dann Bauteil. Eine Ansicht speichert nur, was sie selbst anders setzt;
//! fehlt ein Wert, gilt der der Vorlage, sonst der des Bürostandards.
//! Vorlagen und Themen stehen im Bürostandard (ADR 0040), die Einstellungen
//! einer Ansicht im Projekt.

use serde::{Deserialize, Serialize};

use crate::{Element, ElementId, OpeningKind, Phase, PhaseFilter, Project, Property, ViewContent};

/// Detailgrad einer Ansicht.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Detail {
    /// Wände als gefüllte Fläche, keine Schichten.
    Coarse,
    /// Tragschicht mit Muster, weitere Schichten nur als Linie.
    Medium,
    /// Alle Schichten mit Materialmuster.
    Fine,
}

impl Detail {
    pub const ALL: [Detail; 3] = [Detail::Coarse, Detail::Medium, Detail::Fine];

    pub fn title(self) -> &'static str {
        match self {
            Detail::Coarse => "grob",
            Detail::Medium => "mittel",
            Detail::Fine => "fein",
        }
    }
}

impl PhaseFilter {
    pub const ALL: [PhaseFilter; 3] = [PhaseFilter::All, PhaseFilter::Existing, PhaseFilter::Planned];

    pub fn title(self) -> &'static str {
        match self {
            PhaseFilter::All => "Umbauplan",
            PhaseFilter::Existing => "Bestand",
            PhaseFilter::Planned => "Neuzustand",
        }
    }
}

/// Bauteilart, nach der eine Regel unterscheidet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Category {
    Wall,
    Door,
    Window,
    /// Durchgang oder Nische.
    Opening,
    Column,
    Space,
    Slab,
    Roof,
    Stair,
    Railing,
    Dimension,
    /// 2D-Linie, -Fläche, -Text.
    Annotation,
    Grid,
}

impl Category {
    pub fn of(element: &Element) -> Option<Category> {
        Some(match element {
            Element::Wall(_) => Category::Wall,
            Element::Opening(o) => match o.kind {
                OpeningKind::Door(_) => Category::Door,
                OpeningKind::Window => Category::Window,
                OpeningKind::Plain => Category::Opening,
            },
            Element::Column(_) => Category::Column,
            Element::Space(_) => Category::Space,
            Element::Slab(_) | Element::SlabOpening(_) => Category::Slab,
            Element::Roof(_) | Element::RoofWindow(_) | Element::Dormer(_) => Category::Roof,
            Element::Stair(_) | Element::Elevator(_) => Category::Stair,
            Element::Railing(_) => Category::Railing,
            Element::DimensionChain(_) => Category::Dimension,
            Element::Annotation(_) => Category::Annotation,
            Element::GridLine(_) => Category::Grid,
            _ => return None,
        })
    }

    pub fn title(self) -> &'static str {
        match self {
            Category::Wall => "Wände",
            Category::Door => "Türen",
            Category::Window => "Fenster",
            Category::Opening => "Durchgänge und Nischen",
            Category::Column => "Stützen",
            Category::Space => "Räume",
            Category::Slab => "Decken",
            Category::Roof => "Dächer",
            Category::Stair => "Treppen und Aufzüge",
            Category::Railing => "Geländer",
            Category::Dimension => "Maßketten",
            Category::Annotation => "2D-Elemente",
            Category::Grid => "Achsen",
        }
    }
}

/// Eine Bedingung einer Regel; eine Regel trifft, wenn alle zutreffen.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Condition {
    Kind(Category),
    Phase(Phase),
    /// Die Eigenschaft hat genau diesen Wert, etwa Brandschutz „F90“.
    Property(Property, String),
    /// Die Eigenschaft hat irgendeinen Wert.
    HasProperty(Property),
    /// Der Name des Wand- oder Deckenaufbaus enthält diesen Text.
    TypeName(String),
    /// Nutzung des Raums nach DIN 277, etwa „VF“; „NUF“ trifft alle Nutzungsflächen.
    Usage(String),
}

impl Condition {
    pub fn matches(&self, project: &Project, id: ElementId) -> bool {
        let Some(e) = project.elements.get(&id) else { return false };
        match self {
            Condition::Kind(k) => Category::of(e) == Some(*k),
            Condition::Phase(p) => e.phase() == Some(*p),
            Condition::Property(p, v) => project.property(id, *p).is_some_and(|x| x.trim() == v.trim()),
            Condition::HasProperty(p) => project.property(id, *p).is_some_and(|x| !x.trim().is_empty()),
            Condition::TypeName(t) => {
                let name = match e {
                    Element::Wall(w) => project.wall_type(w.wall_type).map(|t| t.name.as_str()),
                    Element::Slab(s) => project.slab_type(s.slab_type).map(|t| t.name.as_str()),
                    Element::Roof(r) => project.slab_type(r.roof_type).map(|t| t.name.as_str()),
                    _ => None,
                };
                name.is_some_and(|n| n.to_lowercase().contains(&t.to_lowercase()))
            }
            Condition::Usage(u) => match e {
                Element::Space(s) => s.usage.code().starts_with(u.trim()),
                _ => false,
            },
        }
    }
}

/// Was eine Regel mit den Bauteilen macht, die sie trifft.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RuleAction {
    /// Ausblenden; die Ansicht zählt, wie viele.
    Hide,
    /// Grau zeigen.
    Gray,
    /// Farbig füllen, sRGB.
    Color([u8; 3]),
}

/// Eine Regel nach Eigenschaften: etwa „Abbruch ausblenden“ oder „F90 rot“.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rule {
    pub name: String,
    pub when: Vec<Condition>,
    pub then: RuleAction,
}

impl Rule {
    pub fn matches(&self, project: &Project, id: ElementId) -> bool {
        !self.when.is_empty() && self.when.iter().all(|c| c.matches(project, id))
    }
}

/// Baustein Maßstab einer Vorlage, mit Detailgrad; ohne Detailgrad gilt der
/// des Bürostandards für diesen Maßstab.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ScaleBlock {
    pub scale: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<Detail>,
}

/// Ansichtsvorlage aus Bausteinen (ADR 0050). Ein Baustein, der fehlt,
/// steuert nichts; dort gilt der Bürostandard oder die Ansicht.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ViewTemplate {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<ScaleBlock>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phase: Option<PhaseFilter>,
    /// Name eines Themas im Bürostandard.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
}

/// Thema einer Ansicht: Regeln, die zusammen eine Aussage machen, etwa
/// Brandschutz oder Flächen; als Präsentationsplan mit dunklen Wänden.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    #[serde(default)]
    pub presentation: bool,
    #[serde(default)]
    pub rules: Vec<Rule>,
}

/// Einstellungen einer Ansicht: nur, was sie selbst setzt (ADR 0050).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ViewSettings {
    pub view: ViewContent,
    /// Name der Ansichtsvorlage im Bürostandard.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<Detail>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phase: Option<PhaseFilter>,
    /// Name eines Themas; leer heißt ausdrücklich keins.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
    /// Schnitthöhe über OKFF, nur in Grundrissen.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cut_height: Option<f64>,
    /// Geschoss darunter blass (ADR 0049), nur in Grundrissen.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub faded_below: Option<bool>,
    /// Eigene Regeln der Ansicht, nach denen aus Thema und Vorlage.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rules: Vec<Rule>,
    /// Regeln aus dem Thema, die in dieser Ansicht nicht gelten, nach Namen.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rules_off: Vec<String>,
    /// Bauteile, die in dieser Ansicht ausgeblendet sind (die Stufe „Bauteil“).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hidden: Vec<ElementId>,
}

impl ViewSettings {
    pub fn new(view: ViewContent) -> Self {
        ViewSettings {
            view,
            template: None,
            scale: None,
            detail: None,
            phase: None,
            theme: None,
            cut_height: None,
            faded_below: None,
            rules: Vec::new(),
            rules_off: Vec::new(),
            hidden: Vec::new(),
        }
    }

    /// Setzt die Ansicht nichts selbst, braucht sie kein Element.
    pub fn is_empty(&self) -> bool {
        *self == ViewSettings::new(self.view)
    }
}

impl Project {
    /// Einstellungen einer Ansicht, wenn sie welche hat.
    pub fn view_settings(&self, view: ViewContent) -> Option<(ElementId, &ViewSettings)> {
        self.elements.iter().find_map(|(id, e)| match e {
            Element::ViewSettings(s) if s.view == view => Some((*id, s)),
            _ => None,
        })
    }

    /// Wert einer Eigenschaft an einem Bauteil: sein eigener, sonst der seines
    /// Typs (ADR 0048). Nur Texteigenschaften.
    pub fn property(&self, id: ElementId, p: Property) -> Option<String> {
        let (own, type_id) = match self.elements.get(&id)? {
            Element::Wall(w) => (&w.props, Some(w.wall_type)),
            Element::Slab(s) => (&s.props, Some(s.slab_type)),
            Element::Opening(o) => (&o.props, o.opening_type),
            _ => return None,
        };
        if let Some(v) = own.get(p) {
            return Some(v.clone());
        }
        match self.elements.get(&type_id?)? {
            Element::WallType(t) => t.props.get(p).cloned(),
            Element::SlabType(t) => t.props.get(p).cloned(),
            Element::OpeningType(t) => t.props.get(p).cloned(),
            _ => None,
        }
    }

    /// Das Projekt ohne die Elemente `ids`; Öffnungen und Durchbrüche in
    /// entfernten Bauteilen fallen mit weg.
    pub fn without(&self, ids: &std::collections::BTreeSet<ElementId>) -> Project {
        let mut out = self.clone();
        out.elements.retain(|id, _| !ids.contains(id));
        let gone = |host: &ElementId| ids.contains(host) || !self.elements.contains_key(host);
        out.elements.retain(|_, e| match e {
            Element::Opening(o) => !gone(&o.host),
            Element::SlabOpening(o) => !gone(&o.host),
            Element::RoofWindow(w) => !gone(&w.roof),
            Element::Dormer(d) => !gone(&d.roof),
            _ => true,
        });
        out
    }
}
