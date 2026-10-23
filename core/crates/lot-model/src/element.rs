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
    Space(Space),
    DimensionChain(DimensionChain),
    SlabType(SlabType),
    Slab(Slab),
    SlabOpening(SlabOpening),
    Roof(crate::Roof),
    RoofWindow(crate::RoofWindow),
    Dormer(crate::Dormer),
    Stair(crate::Stair),
    Railing(crate::Railing),
    SectionView(SectionView),
    Annotation(crate::Annotation),
    Underlay(crate::Underlay),
    Terrain(crate::Terrain),
    ProjectInfo(ProjectInfo),
    PlanSheet(PlanSheet),
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
    Space,
    DimensionChain,
    SlabType,
    Slab,
    SlabOpening,
    Roof,
    RoofWindow,
    Dormer,
    Stair,
    Railing,
    SectionView,
    Annotation,
    Underlay,
    Terrain,
    ProjectInfo,
    PlanSheet,
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
            Element::Space(_) => ElementKind::Space,
            Element::DimensionChain(_) => ElementKind::DimensionChain,
            Element::SlabType(_) => ElementKind::SlabType,
            Element::Slab(_) => ElementKind::Slab,
            Element::SlabOpening(_) => ElementKind::SlabOpening,
            Element::Roof(_) => ElementKind::Roof,
            Element::RoofWindow(_) => ElementKind::RoofWindow,
            Element::Dormer(_) => ElementKind::Dormer,
            Element::Stair(_) => ElementKind::Stair,
            Element::Railing(_) => ElementKind::Railing,
            Element::SectionView(_) => ElementKind::SectionView,
            Element::Annotation(_) => ElementKind::Annotation,
            Element::Underlay(_) => ElementKind::Underlay,
            Element::Terrain(_) => ElementKind::Terrain,
            Element::ProjectInfo(_) => ElementKind::ProjectInfo,
            Element::PlanSheet(_) => ElementKind::PlanSheet,
        }
    }

    /// Bauzustand (2.05); `None` bei Elementen ohne Bauzustand (Geschosse, Typen, Ansichten).
    pub fn phase(&self) -> Option<Phase> {
        Some(match self {
            Element::Wall(w) => w.phase,
            Element::Opening(o) => o.phase,
            Element::Column(c) => c.phase,
            Element::Slab(s) => s.phase,
            Element::SlabOpening(o) => o.phase,
            Element::Stair(s) => s.phase,
            Element::Railing(r) => r.phase,
            Element::Roof(r) => r.phase,
            Element::Space(r) => r.phase,
            _ => return None,
        })
    }

    /// Setzt den Bauzustand (2.05). `false`, wenn das Element keinen hat.
    pub fn set_phase(&mut self, phase: Phase) -> bool {
        match self {
            Element::Wall(w) => w.phase = phase,
            Element::Opening(o) => o.phase = phase,
            Element::Column(c) => c.phase = phase,
            Element::Slab(s) => s.phase = phase,
            Element::SlabOpening(o) => o.phase = phase,
            Element::Stair(s) => s.phase = phase,
            Element::Railing(r) => r.phase = phase,
            Element::Roof(r) => r.phase = phase,
            Element::Space(r) => r.phase = phase,
            _ => return false,
        }
        true
    }

    /// Wert eines Parameters, falls das Element ihn hat.
    pub fn param(&self, param: Param) -> Option<f64> {
        match (self, param) {
            (Element::Wall(w), Param::WallLength) => Some(w.length()),
            (Element::Wall(w), Param::WallSagitta) => Some(w.sagitta),
            (Element::Wall(w), Param::WallHeight) => match w.top {
                Top::Height(h) => Some(h),
                Top::Storey { .. } | Top::Roof { .. } => None,
            },
            (Element::Wall(w), Param::WallTopOffset) => match w.top {
                Top::Storey { offset } => Some(offset),
                Top::Height(_) | Top::Roof { .. } => None,
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
            (Element::Roof(r), Param::RoofEaveHeight) => Some(r.eave_height),
            (Element::Roof(r), Param::RoofSlope) => r.edges.iter().find_map(|e| match e {
                crate::RoofEdge::Eave { slope, .. } => Some(*slope),
                crate::RoofEdge::Verge { .. } => None,
            }),
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
            (Element::Wall(w), Param::WallSagitta) => w.sagitta = value,
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
            (Element::Roof(r), Param::RoofEaveHeight) => r.eave_height = value,
            (Element::Roof(r), Param::RoofSlope) => {
                for e in &mut r.edges {
                    if let crate::RoofEdge::Eave { slope, .. } = e {
                        *slope = value;
                    }
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
    /// Bis unter das Dach desselben Geschosses, plus Versatz (Giebelwände, Wände im Dachgeschoss).
    Roof { offset: f64 },
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
    /// Stich (Pfeilhöhe) einer gebogenen Wand: Abstand der Bogenmitte von der
    /// Sehne zwischen `start` und `end`, positiv nach links. 0 bei geraden Wänden.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub sagitta: f64,
    /// Bestand, Abbruch oder Neubau (2.05).
    #[serde(default, skip_serializing_if = "Phase::is_new")]
    pub phase: Phase,
}

fn is_zero(v: &f64) -> bool {
    *v == 0.0
}

/// Kreisbogen der Bezugslinie einer gebogenen Wand.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CircularArc {
    pub center: Point2,
    pub radius: f64,
    /// Winkel des Anfangspunkts im Bogenmaß.
    pub start_angle: f64,
    /// Überstrichener Winkel, positiv gegen den Uhrzeigersinn.
    pub sweep: f64,
}

impl CircularArc {
    pub fn length(&self) -> f64 {
        self.radius * self.sweep.abs()
    }

    /// Vorzeichen der Querlage: +1, wenn links der Zeichenrichtung außen liegt
    /// (Bogen im Uhrzeigersinn), sonst -1.
    fn outward_sign(&self) -> f64 {
        if self.sweep < 0.0 { 1.0 } else { -1.0 }
    }

    /// Radius einer Linie im Abstand `across` von der Bezugslinie (links positiv).
    pub fn radius_at(&self, across: f64) -> f64 {
        self.radius + self.outward_sign() * across
    }

    fn angle_at(&self, along: f64) -> f64 {
        self.start_angle + self.sweep * along / self.length()
    }

    /// Punkt im Abstand `along` vom Anfang, gemessen auf dem Bogen, und
    /// `across` quer dazu (links der Laufrichtung positiv).
    pub fn point_at(&self, along: f64, across: f64) -> Point2 {
        let angle = self.angle_at(along);
        let r = self.radius_at(across);
        Point2::new(self.center.x + r * angle.cos(), self.center.y + r * angle.sin())
    }

    /// Umkehrung von [`CircularArc::point_at`]. Die Unstetigkeit liegt gegenüber der Bogenmitte.
    pub fn locate(&self, p: Point2) -> (f64, f64) {
        use std::f64::consts::{PI, TAU};
        let angle = (p.y - self.center.y).atan2(p.x - self.center.x);
        let half = self.sweep.abs() / 2.0;
        let phi = (self.sweep.signum() * (angle - self.start_angle) - half + PI).rem_euclid(TAU) - PI + half;
        (phi * self.radius, self.outward_sign() * (p.distance(self.center) - self.radius))
    }

    /// Laufrichtung im Abstand `along` vom Anfang.
    pub fn tangent_at(&self, along: f64) -> (f64, f64) {
        let angle = self.angle_at(along);
        let sign = self.sweep.signum();
        (-sign * angle.sin(), sign * angle.cos())
    }
}

impl Wall {
    /// Länge entlang der Bezugslinie, bei gebogenen Wänden die Bogenlänge.
    pub fn length(&self) -> f64 {
        self.arc().map_or_else(|| self.chord(), |a| a.length())
    }

    /// Abstand zwischen Anfang und Ende.
    pub fn chord(&self) -> f64 {
        self.start.distance(self.end)
    }

    pub fn is_curved(&self) -> bool {
        self.arc().is_some()
    }

    /// Bogen der Bezugslinie; `None` bei geraden Wänden.
    pub fn arc(&self) -> Option<CircularArc> {
        let c = self.chord();
        if self.sagitta.abs() < 1e-6 || c < 1e-9 {
            return None;
        }
        let s = self.sagitta;
        let radius = (c * c / 4.0 + s * s) / (2.0 * s.abs());
        let (dx, dy) = self.direction();
        let (nx, ny) = (-dy, dx);
        let mid = ((self.start.x + self.end.x) / 2.0, (self.start.y + self.end.y) / 2.0);
        let k = s - s.signum() * radius;
        let center = Point2::new(mid.0 + nx * k, mid.1 + ny * k);
        let start_angle = (self.start.y - center.y).atan2(self.start.x - center.x);
        let sweep = -s.signum() * 4.0 * (2.0 * s.abs() / c).atan();
        Some(CircularArc { center, radius, start_angle, sweep })
    }

    /// Einheitsvektor in Zeichenrichtung, bei gebogenen Wänden entlang der Sehne.
    pub fn direction(&self) -> (f64, f64) {
        let len = self.chord();
        if len > 0.0 { ((self.end.x - self.start.x) / len, (self.end.y - self.start.y) / len) } else { (1.0, 0.0) }
    }

    /// Punkt im Abstand `along` vom Anfang, gemessen entlang der Bezugslinie,
    /// und `across` quer dazu (links positiv).
    pub fn point_at(&self, along: f64, across: f64) -> Point2 {
        match self.arc() {
            Some(a) => a.point_at(along, across),
            None => {
                let (dx, dy) = self.direction();
                Point2::new(self.start.x + dx * along - dy * across, self.start.y + dy * along + dx * across)
            }
        }
    }

    /// Lage eines Punkts zur Bezugslinie: (entlang ab Anfang, quer, links positiv).
    /// Umkehrung von [`Wall::point_at`].
    pub fn locate(&self, p: Point2) -> (f64, f64) {
        match self.arc() {
            Some(a) => a.locate(p),
            None => {
                let (dx, dy) = self.direction();
                let (px, py) = (p.x - self.start.x, p.y - self.start.y);
                (px * dx + py * dy, -px * dy + py * dx)
            }
        }
    }

    /// Richtung der Bezugslinie im Abstand `along` vom Anfang.
    pub fn tangent_at(&self, along: f64) -> (f64, f64) {
        self.arc().map_or_else(|| self.direction(), |a| a.tangent_at(along))
    }

    /// Ändert die Länge, der Anfangspunkt bleibt fest. Ein Bogen behält seinen
    /// Öffnungswinkel, Sehne und Stich wachsen mit.
    fn set_length(&mut self, length: f64) {
        let k = length / self.length().max(1e-12);
        self.end =
            Point2::new(self.start.x + (self.end.x - self.start.x) * k, self.start.y + (self.end.y - self.start.y) * k);
        self.sagitta *= k;
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

/// Öffnung in einer Wand (IfcOpeningElement), rechteckig. Maße sind
/// Rohbaumaße; eine Tür oder ein Fenster füllt die Öffnung (IfcDoor, IfcWindow).
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
    #[serde(default, skip_serializing_if = "OpeningKind::is_plain")]
    pub kind: OpeningKind,
    /// Bestand, Abbruch oder Neubau (2.05).
    #[serde(default, skip_serializing_if = "Phase::is_new")]
    pub phase: Phase,
}

/// Was in einer Öffnung sitzt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum OpeningKind {
    /// Nichts: Durchgang oder Durchbruch.
    #[default]
    Plain,
    Door(Door),
    Window,
}

impl OpeningKind {
    pub fn is_plain(&self) -> bool {
        *self == OpeningKind::Plain
    }
}

/// Drehflügeltür.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Door {
    /// Seite der Wand, zu der die Tür aufschlägt, bezogen auf die Zeichenrichtung der Wand.
    pub opens_to: WallSide,
    /// Anschlag nach DIN 107:1974-04: von der Seite gesehen, zu der die Tür
    /// aufschlägt, sitzen die Bänder links (DIN links) oder rechts (DIN rechts).
    pub hinge: Handing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WallSide {
    Left,
    Right,
}

impl WallSide {
    pub fn other(self) -> Self {
        match self {
            WallSide::Left => WallSide::Right,
            WallSide::Right => WallSide::Left,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Handing {
    DinLeft,
    DinRight,
}

impl Handing {
    pub fn other(self) -> Self {
        match self {
            Handing::DinLeft => Handing::DinRight,
            Handing::DinRight => Handing::DinLeft,
        }
    }
}

impl Door {
    /// Ob die Bänder am Ende der Öffnung sitzen, das in Zeichenrichtung der Wand
    /// hinten liegt (bei `offset + width`), sonst vorne (bei `offset`).
    ///
    /// Wer auf der linken Seite der Wand steht und auf sie blickt, hat die
    /// Zeichenrichtung zur Linken; auf der rechten Seite zur Rechten.
    pub fn hinge_at_end(&self) -> bool {
        matches!((self.opens_to, self.hinge), (WallSide::Left, Handing::DinLeft) | (WallSide::Right, Handing::DinRight))
    }
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
    /// Bestand, Abbruch oder Neubau (2.05).
    #[serde(default, skip_serializing_if = "Phase::is_new")]
    pub phase: Phase,
}

/// Raum (IfcSpace). Gespeichert wird nur ein Punkt im Raum; die Fläche ergibt
/// sich aus den umgebenden Wänden (CLAUDE.md, Regel 6).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Space {
    pub storey: ElementId,
    /// Ein Punkt im Raum, zugleich Lage des Raumstempels.
    pub position: Point2,
    /// Raumnummer, etwa „0.01“.
    #[serde(default)]
    pub number: String,
    pub name: String,
    #[serde(default)]
    pub usage: SpaceUsage,
    /// Ob der Raum zur Wohnfläche zählt. Zubehörräume wie Keller, Heizungs- und
    /// Abstellräume außerhalb der Wohnung zählen nicht (WoFlV vom 25.11.2003, § 2 Abs. 3).
    #[serde(default = "yes", skip_serializing_if = "is_true")]
    pub living_space: bool,
    /// Bestand, Abbruch oder Neubau (2.05); abgebrochene Räume fehlen im Umbauplan.
    #[serde(default, skip_serializing_if = "Phase::is_new")]
    pub phase: Phase,
}

fn yes() -> bool {
    true
}

fn is_true(v: &bool) -> bool {
    *v
}

/// Nutzungsgruppe nach DIN 277:2021-08. Die Netto-Raumfläche (NRF) teilt sich
/// in Nutzungsfläche (NUF 1 bis 7), Technikfläche (TF) und Verkehrsfläche (VF).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum SpaceUsage {
    /// NUF 1 Wohnen und Aufenthalt
    #[default]
    Living,
    /// NUF 2 Büroarbeit
    Office,
    /// NUF 3 Produktion, Hand- und Maschinenarbeit, Forschung und Entwicklung
    Production,
    /// NUF 4 Lagern, Verteilen und Verkaufen
    Storage,
    /// NUF 5 Bildung, Unterricht und Kultur
    Education,
    /// NUF 6 Heilen und Pflegen
    Care,
    /// NUF 7 Sonstige Nutzungen (etwa Sanitärräume, Abstellräume)
    Other,
    /// TF Technikfläche
    Technical,
    /// VF Verkehrsfläche
    Circulation,
}

impl SpaceUsage {
    pub const ALL: [SpaceUsage; 9] = [
        SpaceUsage::Living,
        SpaceUsage::Office,
        SpaceUsage::Production,
        SpaceUsage::Storage,
        SpaceUsage::Education,
        SpaceUsage::Care,
        SpaceUsage::Other,
        SpaceUsage::Technical,
        SpaceUsage::Circulation,
    ];

    /// Kurzzeichen nach DIN 277:2021-08, etwa „NUF 1“ oder „VF“.
    pub fn code(self) -> &'static str {
        match self {
            SpaceUsage::Living => "NUF 1",
            SpaceUsage::Office => "NUF 2",
            SpaceUsage::Production => "NUF 3",
            SpaceUsage::Storage => "NUF 4",
            SpaceUsage::Education => "NUF 5",
            SpaceUsage::Care => "NUF 6",
            SpaceUsage::Other => "NUF 7",
            SpaceUsage::Technical => "TF",
            SpaceUsage::Circulation => "VF",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            SpaceUsage::Living => "Wohnen und Aufenthalt",
            SpaceUsage::Office => "Büroarbeit",
            SpaceUsage::Production => "Produktion, Hand- und Maschinenarbeit, Forschung",
            SpaceUsage::Storage => "Lagern, Verteilen und Verkaufen",
            SpaceUsage::Education => "Bildung, Unterricht und Kultur",
            SpaceUsage::Care => "Heilen und Pflegen",
            SpaceUsage::Other => "Sonstige Nutzungen",
            SpaceUsage::Technical => "Technikfläche",
            SpaceUsage::Circulation => "Verkehrsfläche",
        }
    }

    /// Ob die Fläche zur Nutzungsfläche (NUF) gehört.
    pub fn is_usable(self) -> bool {
        !matches!(self, SpaceUsage::Technical | SpaceUsage::Circulation)
    }
}

/// Maßkette (3.05). Gespeichert werden die Bezüge, nicht die Maße: Ändert sich
/// eine Wand oder Öffnung, ändert sich die Kette mit (assoziativ).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DimensionChain {
    pub storey: ElementId,
    /// Ein Punkt auf der Maßlinie.
    pub position: Point2,
    /// Richtung der Maßlinie; muss nicht normiert sein.
    pub direction: Point2,
    /// Gemessen wird zwischen den Bezügen, in der Reihenfolge entlang der Maßlinie.
    pub refs: Vec<DimRef>,
}

/// Ein Bezugspunkt einer Maßkette.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DimRef {
    /// Wandende auf einer Wandseite; an einer L-Ecke die äußere Ecke beider Wände.
    WallEnd { wall: ElementId, end: WallEnd, side: WallSide },
    /// Laibung einer Öffnung auf einer Wandseite. `End` ist die Laibung bei `offset + width`.
    Jamb { opening: ElementId, end: WallEnd, side: WallSide },
    /// Fläche einer Wand, für Maße quer zur Wand.
    WallFace { wall: ElementId, side: WallSide },
    /// Achse.
    Grid(ElementId),
    /// Fester Punkt.
    Point(Point2),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WallEnd {
    Start,
    End,
}

/// Deckenaufbau (IfcSlabType): Schichten von oben nach unten, etwa Estrich,
/// Trittschalldämmung, Rohdecke, Putz.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SlabType {
    pub name: String,
    pub layers: Vec<Layer>,
}

impl SlabType {
    pub fn thickness(&self) -> f64 {
        self.layers.iter().map(|l| l.thickness).sum()
    }
}

/// Decke oder Bodenplatte (IfcSlab). Die Oberkante liegt auf OKFF des
/// Geschosses plus `top_offset`, die Schichten reichen nach unten.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Slab {
    pub storey: ElementId,
    pub slab_type: ElementId,
    /// Umriss in Metern, Richtung egal.
    pub outline: Vec<Point2>,
    #[serde(default)]
    pub top_offset: f64,
    #[serde(default)]
    pub kind: SlabKind,
    /// Bestand, Abbruch oder Neubau (2.05).
    #[serde(default, skip_serializing_if = "Phase::is_new")]
    pub phase: Phase,
}

/// Durchbruch in einer Decke (IfcOpeningElement), etwa für eine Treppe oder einen Schacht.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SlabOpening {
    /// Die Decke, in der der Durchbruch sitzt.
    pub host: ElementId,
    /// Umriss in Metern, Richtung egal.
    pub outline: Vec<Point2>,
    /// Bestand, Abbruch oder Neubau (2.05).
    #[serde(default, skip_serializing_if = "Phase::is_new")]
    pub phase: Phase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SlabKind {
    /// Geschossdecke
    #[default]
    Floor,
    /// Bodenplatte des untersten Geschosses
    Base,
}

/// Schnitt oder Ansicht (3.02): eine senkrechte Ebene durch die Linie von
/// `start` nach `end`, mit Blick zur Seite `look` bis `depth` tief. Eine
/// Ansicht ist ein Schnitt, dessen Ebene außerhalb des Gebäudes liegt.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SectionView {
    /// Etwa „Schnitt A-A“ oder „Ansicht Süd“.
    pub name: String,
    /// Kennbuchstabe an der Schnittlinie im Grundriss, etwa „A“; leer bei Ansichten.
    #[serde(default)]
    pub mark: String,
    pub start: Point2,
    pub end: Point2,
    pub look: WallSide,
    /// Blicktiefe in Metern.
    pub depth: f64,
    #[serde(default)]
    pub kind: SectionKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SectionKind {
    #[default]
    Section,
    Elevation,
}

impl SectionView {
    /// Blickrichtung (Einheitsvektor) und Richtung, die im Bild nach rechts zeigt.
    pub fn frame(&self) -> ((f64, f64), (f64, f64)) {
        let len = self.start.distance(self.end).max(1e-12);
        let d = ((self.end.x - self.start.x) / len, (self.end.y - self.start.y) / len);
        let n = match self.look {
            WallSide::Left => (-d.1, d.0),
            WallSide::Right => (d.1, -d.0),
        };
        (n, (n.1, -n.0))
    }
}

/// Bauzustand eines Bauteils (2.05). Im Bauantrag nach Bauvorlagenverordnung
/// farbig: Bestand grau, Abbruch gelb, Neubau rot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Phase {
    /// Bleibt, wie es ist.
    Existing,
    /// Bestand, der abgebrochen wird.
    Demolished,
    #[default]
    New,
}

impl Phase {
    pub fn is_new(&self) -> bool {
        *self == Phase::New
    }

    /// Steht das Bauteil im Zustand `state`?
    pub fn shown_in(self, state: PhaseFilter) -> bool {
        match state {
            PhaseFilter::All => true,
            PhaseFilter::Existing => self != Phase::New,
            PhaseFilter::Planned => self != Phase::Demolished,
        }
    }
}

/// Welcher Bauzustand gezeigt wird.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PhaseFilter {
    /// Alles, Bestand, Abbruch und Neubau farbig (Umbauplan).
    #[default]
    All,
    /// Bestand vor dem Umbau, einschließlich dessen, was abgebrochen wird.
    Existing,
    /// Zustand nach dem Umbau: Bestand und Neubau.
    Planned,
}

/// Angaben für den Plankopf, einmal je Projekt.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ProjectInfo {
    #[serde(default)]
    pub client: String,
    /// Bauort, etwa Straße und Ort.
    #[serde(default)]
    pub site: String,
    /// Entwurfsverfasser (Büro).
    #[serde(default)]
    pub architect: String,
    /// Gebäudeart, bestimmt etwa die Grenzwerte für Treppen.
    #[serde(default)]
    pub building: BuildingClass,
}

/// Gebäudeart nach DIN 18065:2020-08, Tabelle 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum BuildingClass {
    /// Sonstige Gebäude (strengere Grenzwerte).
    #[default]
    General,
    /// Wohngebäude mit nicht mehr als zwei Wohnungen, und innerhalb von Wohnungen.
    SmallResidential,
}

/// Planblatt (3.07) mit Plankopf und darauf platzierten Ansichten.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlanSheet {
    /// Plannummer, etwa „101“.
    pub number: String,
    /// Planinhalt, etwa „Grundrisse“.
    pub title: String,
    /// Blattformat nach DIN EN ISO 216, quer: „A4“ bis „A0“.
    pub format: String,
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub index: String,
    pub views: Vec<PlacedView>,
}

/// Eine Ansicht auf einem Planblatt: Inhalt, Maßstab und Mitte auf dem Blatt in mm.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlacedView {
    pub content: ViewContent,
    pub scale: f64,
    pub center: Point2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViewContent {
    /// Grundriss eines Geschosses.
    FloorPlan { storey: ElementId },
    /// Schnitt oder Ansicht (Element `SectionView`).
    Section { view: ElementId },
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
    /// Stich einer gebogenen Wand, 0 macht sie gerade.
    WallSagitta,
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
    /// Traufhöhe eines Dachs über OKFF.
    RoofEaveHeight,
    /// Neigung aller Traufen eines Dachs, in Grad.
    RoofSlope,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn arc_wall(sagitta: f64) -> Wall {
        Wall {
            phase: Default::default(),
            storey: ElementId(1),
            wall_type: ElementId(2),
            start: Point2::new(0.0, 0.0),
            end: Point2::new(4.0, 0.0),
            alignment: WallAlignment::Center,
            base_offset: 0.0,
            top: Top::Height(2.5),
            sagitta,
        }
    }

    fn close(a: Point2, b: Point2) -> bool {
        a.distance(b) < 1e-9
    }

    #[test]
    fn semicircle_bulges_to_the_left() {
        let w = arc_wall(2.0);
        let a = w.arc().unwrap();
        assert!(close(a.center, Point2::new(2.0, 0.0)));
        assert!((a.radius - 2.0).abs() < 1e-12);
        assert!((w.length() - std::f64::consts::PI * 2.0).abs() < 1e-9);
        // Bogenmitte liegt 2 m links der Sehne.
        assert!(close(w.point_at(w.length() / 2.0, 0.0), Point2::new(2.0, 2.0)));
        assert!(close(w.point_at(w.length(), 0.0), w.end));
        // Links ist außen: 0,1 m links der Bogenmitte liegt weiter oben.
        assert!(close(w.point_at(w.length() / 2.0, 0.1), Point2::new(2.0, 2.1)));
        // Am Anfang zeigt die Tangente nach oben.
        let (tx, ty) = w.tangent_at(0.0);
        assert!(tx.abs() < 1e-9 && (ty - 1.0).abs() < 1e-9);
    }

    #[test]
    fn locate_inverts_point_at() {
        for sagitta in [0.0, 0.5, -0.8, 2.0] {
            let w = arc_wall(sagitta);
            for (along, across) in [(0.0, 0.0), (1.3, 0.12), (w.length(), -0.2), (-0.4, 0.05), (w.length() + 0.3, 0.0)]
            {
                let (s, t) = w.locate(w.point_at(along, across));
                assert!(
                    (s - along).abs() < 1e-9 && (t - across).abs() < 1e-9,
                    "{sagitta}: {along}, {across} -> {s}, {t}"
                );
            }
        }
    }

    #[test]
    fn set_length_keeps_the_angle() {
        let mut w = arc_wall(1.0);
        let sweep = w.arc().unwrap().sweep;
        w.set_length(w.length() * 1.5);
        assert!((w.arc().unwrap().sweep - sweep).abs() < 1e-12);
        assert!((w.sagitta - 1.5).abs() < 1e-12);
        assert!(close(w.start, Point2::new(0.0, 0.0)));
    }

    #[test]
    fn din_handing_depends_on_the_opening_side() {
        // Wand nach Osten gezeichnet, links ist Norden. Wer von Norden auf die Wand
        // blickt, hat Osten (das Ende der Öffnung) zur Linken.
        let door = |opens_to, hinge| Door { opens_to, hinge };
        assert!(door(WallSide::Left, Handing::DinLeft).hinge_at_end());
        assert!(!door(WallSide::Left, Handing::DinRight).hinge_at_end());
        assert!(!door(WallSide::Right, Handing::DinLeft).hinge_at_end());
        assert!(door(WallSide::Right, Handing::DinRight).hinge_at_end());
    }
}
