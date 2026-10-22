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
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn distance(self, other: Point2) -> f64 {
        (other.x - self.x).hypot(other.y - self.y)
    }
}

/// Bauteil oder Bezug im Modell. Die Varianten folgen den IFC-Klassen
/// (siehe docs/glossar.md).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Element {
    Storey(Storey),
    GridLine(GridLine),
    WallType(WallType),
    Wall(Wall),
    Opening(Opening),
    Column(Column),
}

/// Art eines Elements, ohne Daten.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ElementKind {
    Storey,
    GridLine,
    WallType,
    Wall,
    Opening,
    Column,
}

impl Element {
    pub fn kind(&self) -> ElementKind {
        match self {
            Element::Storey(_) => ElementKind::Storey,
            Element::GridLine(_) => ElementKind::GridLine,
            Element::WallType(_) => ElementKind::WallType,
            Element::Wall(_) => ElementKind::Wall,
            Element::Opening(_) => ElementKind::Opening,
            Element::Column(_) => ElementKind::Column,
        }
    }

    /// Wert eines Parameters, falls das Element ihn hat.
    pub fn param(&self, param: Param) -> Option<f64> {
        match (self, param) {
            (Element::Wall(w), Param::WallLength) => Some(w.length()),
            (Element::Wall(w), Param::WallHeight) => match w.top {
                Top::Height(h) => Some(h),
                Top::Storey { .. } => None,
            },
            (Element::Wall(w), Param::WallTopOffset) => match w.top {
                Top::Storey { offset } => Some(offset),
                Top::Height(_) => None,
            },
            (Element::Wall(w), Param::BaseOffset) => Some(w.base_offset),
            (Element::Column(c), Param::BaseOffset) => Some(c.base_offset),
            (Element::Opening(o), Param::OpeningOffset) => Some(o.offset),
            (Element::Opening(o), Param::OpeningWidth) => Some(o.width),
            (Element::Opening(o), Param::OpeningHeight) => Some(o.height),
            (Element::Opening(o), Param::OpeningSillHeight) => Some(o.sill_height),
            (Element::Storey(s), Param::StoreyElevation) => Some(s.elevation),
            (Element::Storey(s), Param::StoreyHeight) => Some(s.height),
            (Element::Column(c), Param::ColumnWidth) => match c.profile {
                ColumnProfile::Rectangle { width, .. } => Some(width),
                ColumnProfile::Circle { diameter } => Some(diameter),
            },
            (Element::Column(c), Param::ColumnDepth) => match c.profile {
                ColumnProfile::Rectangle { depth, .. } => Some(depth),
                ColumnProfile::Circle { .. } => None,
            },
            _ => None,
        }
    }

    /// Setzt einen Parameter. Gibt `false` zurück, wenn das Element ihn nicht hat.
    pub(crate) fn set_param(&mut self, param: Param, value: f64) -> bool {
        if self.param(param).is_none() {
            return false;
        }
        match (self, param) {
            (Element::Wall(w), Param::WallLength) => w.set_length(value),
            (Element::Wall(w), Param::WallHeight) => w.top = Top::Height(value),
            (Element::Wall(w), Param::WallTopOffset) => w.top = Top::Storey { offset: value },
            (Element::Wall(w), Param::BaseOffset) => w.base_offset = value,
            (Element::Column(c), Param::BaseOffset) => c.base_offset = value,
            (Element::Opening(o), Param::OpeningOffset) => o.offset = value,
            (Element::Opening(o), Param::OpeningWidth) => o.width = value,
            (Element::Opening(o), Param::OpeningHeight) => o.height = value,
            (Element::Opening(o), Param::OpeningSillHeight) => o.sill_height = value,
            (Element::Storey(s), Param::StoreyElevation) => s.elevation = value,
            (Element::Storey(s), Param::StoreyHeight) => s.height = value,
            (Element::Column(c), Param::ColumnWidth) => match &mut c.profile {
                ColumnProfile::Rectangle { width, .. } => *width = value,
                ColumnProfile::Circle { diameter } => *diameter = value,
            },
            (Element::Column(c), Param::ColumnDepth) => {
                if let ColumnProfile::Rectangle { depth, .. } = &mut c.profile {
                    *depth = value;
                }
            }
            _ => return false,
        }
        true
    }
}

/// Geschoss (IfcBuildingStorey).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Storey {
    pub name: String,
    /// Höhe der Oberkante Fertigfußboden (OKFF) über ±0,00, in Metern.
    pub elevation: f64,
    /// Geschosshöhe: von OKFF bis OKFF des Geschosses darüber.
    pub height: f64,
}

/// Achse eines Achsrasters (IfcGridAxis).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GridLine {
    /// Beschriftung im Achskreis, etwa „A“ oder „1“.
    pub label: String,
    pub start: Point2,
    pub end: Point2,
}

/// Wandaufbau (IfcWallType): Schichten von außen nach innen.
///
/// „Außen“ ist die linke Seite in Zeichenrichtung der Wand. Wer ein Haus im
/// Uhrzeigersinn zeichnet, hat die Außenseite also immer außen.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WallType {
    pub name: String,
    pub layers: Vec<Layer>,
}

impl WallType {
    pub fn thickness(&self) -> f64 {
        self.layers.iter().map(|l| l.thickness).sum()
    }
}

/// Schicht eines Wandaufbaus (IfcMaterialLayer).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Layer {
    /// Schlüssel des Materials im Bürostandard, etwa „kalksandstein“.
    pub material: String,
    pub thickness: f64,
    pub function: LayerFunction,
}

/// Funktion einer Schicht. Sie bestimmt, wie Schichten an Wandanschlüssen
/// ineinandergreifen (siehe [`LayerFunction::priority`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LayerFunction {
    /// Tragschicht
    Structure,
    /// Dämmung
    Insulation,
    /// Luftschicht
    AirGap,
    /// Installationsebene
    Service,
    /// Putz oder Bekleidung
    Finish,
}

impl LayerFunction {
    /// Rang an Anschlüssen, 1 ist der höchste. Eine Schicht läuft durch
    /// Schichten mit niedrigerem Rang hindurch und endet an der ersten Schicht
    /// mit gleichem oder höherem Rang.
    pub fn priority(self) -> u8 {
        match self {
            LayerFunction::Structure => 1,
            LayerFunction::Insulation => 2,
            LayerFunction::AirGap => 3,
            LayerFunction::Service => 4,
            LayerFunction::Finish => 5,
        }
    }
}

/// Lage der Bezugslinie einer Wand im Aufbau.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum WallAlignment {
    /// Mitte des Gesamtaufbaus
    #[default]
    Center,
    /// Außenkante (linke Seite in Zeichenrichtung)
    Exterior,
    /// Innenkante (rechte Seite in Zeichenrichtung)
    Interior,
}

/// Oberkante eines Bauteils.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Top {
    /// Feste Höhe über der Unterkante.
    Height(f64),
    /// Bis OKFF des darüberliegenden Geschosses (Geschosshöhe), plus Versatz.
    Storey { offset: f64 },
}

/// Wand (IfcWall). Die Bezugslinie läuft von `start` nach `end`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Wall {
    pub storey: ElementId,
    pub wall_type: ElementId,
    pub start: Point2,
    pub end: Point2,
    #[serde(default)]
    pub alignment: WallAlignment,
    /// Unterkante über OKFF des Geschosses.
    #[serde(default)]
    pub base_offset: f64,
    pub top: Top,
}

impl Wall {
    pub fn length(&self) -> f64 {
        self.start.distance(self.end)
    }

    /// Einheitsvektor in Zeichenrichtung.
    pub fn direction(&self) -> (f64, f64) {
        let len = self.length();
        if len > 0.0 { ((self.end.x - self.start.x) / len, (self.end.y - self.start.y) / len) } else { (1.0, 0.0) }
    }

    /// Ändert die Länge, der Anfangspunkt bleibt fest.
    fn set_length(&mut self, length: f64) {
        let (dx, dy) = self.direction();
        self.end = Point2::new(self.start.x + dx * length, self.start.y + dy * length);
    }

    /// Lage der Schichten quer zur Bezugslinie, als (links, rechts) je Schicht,
    /// von außen nach innen. Positive Werte liegen links der Bezugslinie.
    pub fn layer_offsets(&self, wall_type: &WallType) -> Vec<(f64, f64)> {
        let total = wall_type.thickness();
        let mut left = match self.alignment {
            WallAlignment::Center => total / 2.0,
            WallAlignment::Exterior => 0.0,
            WallAlignment::Interior => total,
        };
        wall_type
            .layers
            .iter()
            .map(|l| {
                let right = left - l.thickness;
                let pair = (left, right);
                left = right;
                pair
            })
            .collect()
    }
}

/// Öffnung in einer Wand (IfcOpeningElement). Rechteckig, ohne Rahmen,
/// Laibung oder Anschlag; Türen und Fenster folgen in M2.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Opening {
    /// Die Wand, in der die Öffnung sitzt.
    pub host: ElementId,
    /// Abstand vom Wandanfang bis zur Kante der Öffnung, entlang der Bezugslinie.
    pub offset: f64,
    pub width: f64,
    pub height: f64,
    /// Brüstungshöhe über Unterkante Wand.
    pub sill_height: f64,
}

/// Stütze (IfcColumn).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Column {
    pub storey: ElementId,
    /// Mittelpunkt des Querschnitts.
    pub position: Point2,
    pub profile: ColumnProfile,
    /// Drehung des Querschnitts gegen den Uhrzeigersinn, in Grad.
    #[serde(default)]
    pub rotation: f64,
    /// Schlüssel des Materials im Bürostandard.
    pub material: String,
    #[serde(default)]
    pub base_offset: f64,
    pub top: Top,
}

/// Querschnitt einer Stütze.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ColumnProfile {
    Rectangle { width: f64, depth: f64 },
    Circle { diameter: f64 },
}

/// Parameter, die sich über eine Operation ändern lassen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Param {
    WallLength,
    /// Feste Höhe; nur bei Wänden mit `Top::Height`.
    WallHeight,
    /// Versatz der Oberkante; nur bei Wänden, die am Geschoss hängen.
    WallTopOffset,
    /// Unterkante über OKFF, für Wände und Stützen.
    BaseOffset,
    OpeningOffset,
    OpeningWidth,
    OpeningHeight,
    OpeningSillHeight,
    StoreyElevation,
    StoreyHeight,
    /// Breite einer Rechteckstütze oder Durchmesser einer Rundstütze.
    ColumnWidth,
    ColumnDepth,
}
