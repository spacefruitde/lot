use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Eindeutige Kennung eines Elements innerhalb eines Projekts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ElementId(pub u64);

/// Punkt in der Grundrissebene, in Metern.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point2 {
    pub x: f64,
    pub y: f64,
}

impl Point2 {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn distance(self, other: Point2) -> f64 {
        (other.x - self.x).hypot(other.y - self.y)
    }
}

/// Ein Projekt: alle Elemente mit ihren Parametern.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub elements: BTreeMap<ElementId, Element>,
}

impl Project {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), elements: BTreeMap::new() }
    }

    /// Nächste freie Kennung. Wird aus dem Bestand abgeleitet und nicht gespeichert.
    pub fn next_id(&self) -> ElementId {
        ElementId(self.elements.keys().last().map_or(1, |id| id.0 + 1))
    }

    pub fn wall(&self, id: ElementId) -> Option<&Wall> {
        match self.elements.get(&id) {
            Some(Element::Wall(w)) => Some(w),
            _ => None,
        }
    }

    /// Alle Öffnungen, die in der Wand `host` sitzen.
    pub fn openings_in(&self, host: ElementId) -> impl Iterator<Item = (ElementId, &Opening)> {
        self.elements.iter().filter_map(move |(id, e)| match e {
            Element::Opening(o) if o.host == host => Some((*id, o)),
            _ => None,
        })
    }

    /// Beispielprojekt für M0: eine 5 m lange Wand mit einem Fenster.
    /// Entspricht `testprojekte/m0-wand/projekt.ron`.
    pub fn sample() -> Self {
        let mut p = Project::new("M0 Wand mit Fenster");
        p.elements.insert(
            ElementId(1),
            Element::Wall(Wall {
                start: Point2::new(0.0, 0.0),
                end: Point2::new(5.0, 0.0),
                thickness: 0.3,
                height: 2.6,
            }),
        );
        p.elements.insert(
            ElementId(2),
            Element::Opening(Opening { host: ElementId(1), offset: 1.75, width: 1.5, height: 1.3, sill_height: 0.9 }),
        );
        p
    }
}

/// Bauteil im Modell. Die Varianten folgen den IFC-Klassen (siehe docs/glossar.md).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Element {
    Wall(Wall),
    Opening(Opening),
}

/// Art eines Elements, ohne Daten.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementKind {
    Wall,
    Opening,
}

impl Element {
    pub fn kind(&self) -> ElementKind {
        match self {
            Element::Wall(_) => ElementKind::Wall,
            Element::Opening(_) => ElementKind::Opening,
        }
    }

    /// Wert eines Parameters, falls das Element ihn hat.
    pub fn param(&self, param: Param) -> Option<f64> {
        match (self, param) {
            (Element::Wall(w), Param::WallLength) => Some(w.length()),
            (Element::Wall(w), Param::WallThickness) => Some(w.thickness),
            (Element::Wall(w), Param::WallHeight) => Some(w.height),
            (Element::Opening(o), Param::OpeningOffset) => Some(o.offset),
            (Element::Opening(o), Param::OpeningWidth) => Some(o.width),
            (Element::Opening(o), Param::OpeningHeight) => Some(o.height),
            (Element::Opening(o), Param::OpeningSillHeight) => Some(o.sill_height),
            _ => None,
        }
    }

    /// Setzt einen Parameter. Gibt `false` zurück, wenn das Element ihn nicht hat.
    pub(crate) fn set_param(&mut self, param: Param, value: f64) -> bool {
        match (self, param) {
            (Element::Wall(w), Param::WallLength) => w.set_length(value),
            (Element::Wall(w), Param::WallThickness) => w.thickness = value,
            (Element::Wall(w), Param::WallHeight) => w.height = value,
            (Element::Opening(o), Param::OpeningOffset) => o.offset = value,
            (Element::Opening(o), Param::OpeningWidth) => o.width = value,
            (Element::Opening(o), Param::OpeningHeight) => o.height = value,
            (Element::Opening(o), Param::OpeningSillHeight) => o.sill_height = value,
            _ => return false,
        }
        true
    }
}

/// Wand (IfcWall). Die Achse läuft von `start` nach `end`, die Dicke liegt
/// symmetrisch zur Achse. Schichtaufbau folgt in M1.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Wall {
    pub start: Point2,
    pub end: Point2,
    pub thickness: f64,
    pub height: f64,
}

impl Wall {
    pub fn length(&self) -> f64 {
        self.start.distance(self.end)
    }

    /// Ändert die Länge, der Anfangspunkt bleibt fest.
    fn set_length(&mut self, length: f64) {
        let current = self.length();
        let (dx, dy) = if current > 0.0 {
            ((self.end.x - self.start.x) / current, (self.end.y - self.start.y) / current)
        } else {
            (1.0, 0.0)
        };
        self.end = Point2::new(self.start.x + dx * length, self.start.y + dy * length);
    }
}

/// Öffnung in einer Wand (IfcOpeningElement). In M0 eine rechteckige
/// Fensteröffnung ohne Rahmen, Laibung oder Anschlag.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Opening {
    /// Die Wand, in der die Öffnung sitzt.
    pub host: ElementId,
    /// Abstand vom Wandanfang bis zur Kante der Öffnung, entlang der Achse.
    pub offset: f64,
    pub width: f64,
    pub height: f64,
    /// Brüstungshöhe über Unterkante Wand. Geschosse und Fertigfußboden folgen in M1.
    pub sill_height: f64,
}

/// Parameter, die sich über eine Operation ändern lassen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Param {
    WallLength,
    WallThickness,
    WallHeight,
    OpeningOffset,
    OpeningWidth,
    OpeningHeight,
    OpeningSillHeight,
}
