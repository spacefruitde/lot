//! Treppen (1.07) und Geländer (1.08).
//!
//! Gespeichert werden Antritt, Richtung, Laufbreite, Auftritt und je Lauf die
//! Zahl der Steigungen. Stufen, Podeste, Lauflinie und die Steigungshöhe
//! werden abgeleitet (CLAUDE.md, Regel 6).

use serde::{Deserialize, Serialize};

use crate::{ElementId, Point2, WallSide};

/// Treppe (IfcStair) aus geraden Läufen, zwischen zwei Läufen liegt ein
/// Podest. Sie beginnt auf OKFF ihres Geschosses und endet auf OKFF des
/// Geschosses darüber; die Steigungshöhe ist die Geschosshöhe geteilt durch
/// die Zahl aller Steigungen.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stair {
    pub storey: ElementId,
    /// Antritt: Mitte der Vorderkante der ersten Stufe.
    pub start: Point2,
    /// Laufrichtung des ersten Laufs, die Länge zählt nicht.
    pub direction: Point2,
    /// Laufbreite in m, von Seite zu Seite des Laufs.
    pub width: f64,
    /// Auftritt in m.
    pub tread: f64,
    pub flights: Vec<StairFlight>,
    /// Dicke der Laufplatte und der Podeste in m, senkrecht zur Unterseite.
    #[serde(default = "default_thickness")]
    pub thickness: f64,
    /// Baurechtlich notwendige Treppe; bestimmt die Grenzwerte der Prüfung.
    #[serde(default = "default_true")]
    pub necessary: bool,
    /// Bestand, Abbruch oder Neubau (2.05).
    #[serde(default, skip_serializing_if = "crate::Phase::is_new")]
    pub phase: crate::Phase,
    /// Spindeltreppe (ADR 0051): die Stufen laufen um die Spindel; `start`
    /// gibt die Richtung des Antritts vom Mittelpunkt aus.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spiral: Option<Spiral>,
    /// Treppenauge bei halbgewendelten Läufen: Abstand der Läufe in m (ADR 0051).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub eye: f64,
}

fn is_zero(v: &f64) -> bool {
    *v == 0.0
}

/// Spindeltreppe: Mittelpunkt, Außenradius, Spindelradius, Drehsinn nach oben.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Spiral {
    pub center: Point2,
    /// Radius der äußeren Begrenzung der Laufbreite, in m.
    pub radius: f64,
    /// Radius der Spindel, in m.
    pub spindle: f64,
    #[serde(default)]
    pub clockwise: bool,
}

fn default_thickness() -> f64 {
    0.18
}

fn default_true() -> bool {
    true
}

/// Ein gerader Treppenlauf.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct StairFlight {
    /// Steigungen dieses Laufs; die letzte führt auf das Podest oder den Austritt.
    pub risers: u32,
    /// Podest nach diesem Lauf; beim letzten Lauf ohne Bedeutung.
    #[serde(default)]
    pub landing: Landing,
}

/// Podest zwischen zwei Läufen und wie es danach weitergeht.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Landing {
    /// Zwischenpodest, gleiche Richtung.
    #[default]
    Straight,
    /// Viertelpodest, danach 90° nach links.
    Left,
    /// Viertelpodest, danach 90° nach rechts.
    Right,
    /// Halbpodest, der nächste Lauf liegt links daneben und läuft zurück.
    BackLeft,
    /// Halbpodest, der nächste Lauf liegt rechts daneben und läuft zurück.
    BackRight,
    /// Viertelgewendelt nach links: Wendelstufen statt Podest (ADR 0051).
    WinderLeft,
    WinderRight,
    /// Halbgewendelt, der nächste Lauf liegt links daneben und läuft zurück.
    WinderBackLeft,
    WinderBackRight,
}

impl Landing {
    /// Wendelstufen statt Podest.
    pub fn is_winder(self) -> bool {
        matches!(self, Landing::WinderLeft | Landing::WinderRight | Landing::WinderBackLeft | Landing::WinderBackRight)
    }

    /// Das Podest, dessen Platz die Wendelung einnimmt.
    fn as_landing(self) -> Landing {
        match self {
            Landing::WinderLeft => Landing::Left,
            Landing::WinderRight => Landing::Right,
            Landing::WinderBackLeft => Landing::BackLeft,
            Landing::WinderBackRight => Landing::BackRight,
            l => l,
        }
    }
}

/// Lage eines Laufs, abgeleitet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FlightLayout {
    /// Mitte der Vorderkante der ersten Stufe.
    pub start: Point2,
    /// Laufrichtung, Länge 1.
    pub dir: Point2,
    pub risers: u32,
    /// Höhe des Antritts über OKFF des Geschosses.
    pub base: f64,
}

/// Eine Stufe von oben gesehen: Umriss (vorne rechts, vorne links, hinten
/// links, hinten rechts) und Höhe der Trittfläche über OKFF.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tread {
    pub outline: [Point2; 4],
    pub top: f64,
}

/// Abgeleitete Form einer Treppe für eine Geschosshöhe.
#[derive(Debug, Clone, PartialEq)]
pub struct StairLayout {
    pub width: f64,
    pub tread: f64,
    /// Steigungshöhe in m.
    pub rise: f64,
    pub flights: Vec<FlightLayout>,
    /// Podeste: Umriss und Höhe der Oberfläche über OKFF.
    pub landings: Vec<(Vec<Point2>, f64)>,
    /// Bei gewendelten und Spindeltreppen alle Stufen; dann sind `flights` und
    /// `landings` leer (ADR 0051).
    pub steps: Vec<Step>,
    /// Lauflinie bei gewendelten und Spindeltreppen.
    pub walking: Vec<Point2>,
}

/// Eine Stufe beliebiger Form (Wendel- oder Spindelstufe).
#[derive(Debug, Clone, PartialEq)]
pub struct Step {
    pub outline: Vec<Point2>,
    /// Höhe der Trittfläche über OKFF.
    pub top: f64,
    /// Vorderkante von der inneren zur äußeren Begrenzung.
    pub front: (Point2, Point2),
    /// Hinterkante: Vorderkante der nächsten Stufe oder des Austritts.
    pub back: (Point2, Point2),
    /// Auftritt an der inneren Begrenzung der Laufbreite, als Sehne (DIN 18065:2020-08, 6.2.2).
    pub inner: f64,
}

impl Stair {
    pub fn risers(&self) -> u32 {
        self.flights.iter().map(|f| f.risers).sum()
    }

    /// Form der Treppe bei der Geschosshöhe `height`.
    pub fn layout(&self, height: f64) -> StairLayout {
        self.layout_with(height, crate::stair_winders::MIN_INNER_GENERAL)
    }

    /// Wie [`Stair::layout`]; Wendelstufen werden so weit verzogen, dass ihr
    /// Auftritt an der inneren Begrenzung mindestens `min_inner` beträgt.
    pub fn layout_with(&self, height: f64, min_inner: f64) -> StairLayout {
        if let Some(spiral) = self.spiral {
            // Spindeltreppen: 10 cm innen in sonstigen Gebäuden, in Wohngebäuden keine Anforderung (6.2.4).
            let spiral_min = if min_inner >= crate::stair_winders::MIN_INNER_GENERAL - 1e-9 {
                crate::stair_winders::MIN_INNER_GENERAL
            } else {
                0.0
            };
            return self.spiral_layout(spiral, height, spiral_min);
        }
        if self.flights.iter().any(|f| f.landing.is_winder()) {
            return self.winder_layout(height, min_inner);
        }
        let rise = height / self.risers().max(1) as f64;
        let h = self.width / 2.0;
        let w = self.width;
        let len = self.direction.x.hypot(self.direction.y).max(1e-12);
        let mut dir = Point2::new(self.direction.x / len, self.direction.y / len);
        let mut start = self.start;
        let mut base = 0.0;
        let mut flights = Vec::new();
        let mut landings = Vec::new();
        for (i, f) in self.flights.iter().enumerate() {
            flights.push(FlightLayout { start, dir, risers: f.risers, base });
            let end = add(start, dir, (f.risers.max(1) - 1) as f64 * self.tread);
            base += f.risers as f64 * rise;
            if i + 1 == self.flights.len() {
                break;
            }
            let left = Point2::new(-dir.y, dir.x);
            let (outline, next_start, next_dir) = match f.landing.as_landing() {
                Landing::Straight => (rect(end, dir, left, w, -h, h), add(end, dir, w), dir),
                Landing::Left => (rect(end, dir, left, w, -h, h), add(add(end, dir, h), left, h), left),
                Landing::Right => {
                    let right = Point2::new(-left.x, -left.y);
                    (rect(end, dir, left, w, -h, h), add(add(end, dir, h), right, h), right)
                }
                Landing::BackLeft => {
                    (rect(end, dir, left, w, -h, h + w), add(end, left, w), Point2::new(-dir.x, -dir.y))
                }
                Landing::BackRight => {
                    (rect(end, dir, left, w, -h - w, h), add(end, left, -w), Point2::new(-dir.x, -dir.y))
                }
                _ => (rect(end, dir, left, w, -h, h), add(end, dir, w), dir),
            };
            landings.push((outline, base));
            start = next_start;
            dir = next_dir;
        }
        StairLayout {
            width: self.width,
            tread: self.tread,
            rise,
            flights,
            landings,
            steps: Vec::new(),
            walking: Vec::new(),
        }
    }
}

impl StairLayout {
    /// Seite einer gewendelten oder Spindeltreppe, um `inset` nach innen gerückt:
    /// Punkte an jeder Vorderkante und am Austritt, mit der Höhe der Stufenvorderkante.
    pub fn steps_side(&self, outer: bool, inset: f64) -> Vec<(Point2, f64)> {
        let pick = |(i, o): (Point2, Point2)| {
            let (from, to) = if outer { (o, i) } else { (i, o) };
            let len = from.distance(to).max(1e-12);
            Point2::new(from.x + (to.x - from.x) / len * inset, from.y + (to.y - from.y) / len * inset)
        };
        let mut out: Vec<(Point2, f64)> = self.steps.iter().map(|s| (pick(s.front), s.top)).collect();
        if let Some(last) = self.steps.last() {
            out.push((pick(last.back), last.top + self.rise));
        }
        out
    }

    /// Stufen eines Laufs, von unten nach oben. Ein Lauf mit n Steigungen hat
    /// n - 1 Stufen; die letzte Steigung führt auf das Podest oder den Austritt.
    pub fn treads(&self, flight: &FlightLayout) -> Vec<Tread> {
        let h = self.width / 2.0;
        let left = Point2::new(-flight.dir.y, flight.dir.x);
        (1..flight.risers)
            .map(|k| {
                let front = add(flight.start, flight.dir, (k - 1) as f64 * self.tread);
                let back = add(flight.start, flight.dir, k as f64 * self.tread);
                Tread {
                    outline: [add(front, left, -h), add(front, left, h), add(back, left, h), add(back, left, -h)],
                    top: flight.base + k as f64 * self.rise,
                }
            })
            .collect()
    }

    /// Ende eines Laufs: Mitte der Vorderkante des Podests oder des Austritts.
    pub fn end(&self, flight: &FlightLayout) -> Point2 {
        add(flight.start, flight.dir, (flight.risers.max(1) - 1) as f64 * self.tread)
    }

    /// Lauflinie vom Antritt bis zum Austritt, in der Mitte der Läufe. Über
    /// Viertel- und Halbpodeste läuft sie im Bogen um die innere Ecke
    /// beziehungsweise um die Mitte zwischen den Läufen.
    pub fn walking_line(&self) -> Vec<Point2> {
        let mut points: Vec<Point2> = Vec::new();
        for (i, f) in self.flights.iter().enumerate() {
            points.push(f.start);
            let end = self.end(f);
            points.push(end);
            let Some(next) = self.flights.get(i + 1) else { continue };
            let turn = f.dir.x * next.dir.y - f.dir.y * next.dir.x;
            let dot = f.dir.x * next.dir.x + f.dir.y * next.dir.y;
            if turn.abs() < 1e-9 && dot > 0.0 {
                continue;
            }
            // Mittelpunkt des Bogens: bei der Wendung um 90° die innere Ecke des
            // Podests auf Höhe des Laufendes, bei 180° die Mitte zwischen den Läufen.
            let center = if dot < 0.0 {
                Point2::new((end.x + next.start.x) / 2.0, (end.y + next.start.y) / 2.0)
            } else {
                inner_corner(end, f.dir, next.start)
            };
            let (a0, a1) = (angle(center, end), angle(center, next.start));
            let mut sweep = a1 - a0;
            let side = if turn.abs() > 1e-9 { turn.signum() } else { side_of(f.dir, end, next.start) };
            if side > 0.0 && sweep < 0.0 {
                sweep += std::f64::consts::TAU;
            }
            if side < 0.0 && sweep > 0.0 {
                sweep -= std::f64::consts::TAU;
            }
            let r = center.distance(end);
            let steps = 12;
            for k in 1..steps {
                let a = a0 + sweep * k as f64 / steps as f64;
                points.push(Point2::new(center.x + r * a.cos(), center.y + r * a.sin()));
            }
        }
        points.dedup_by(|a, b| a.distance(*b) < 1e-9);
        points
    }

    /// Länge eines Laufs entlang der Lauflinie.
    pub fn flight_length(&self, flight: &FlightLayout) -> f64 {
        (flight.risers.max(1) - 1) as f64 * self.tread
    }

    /// Seite eines Laufs, um `inset` nach innen gerückt: Anfangs- und
    /// Endpunkt, jeweils mit der Höhe der Stufenvorderkante darüber.
    pub fn flight_side(&self, flight: &FlightLayout, side: WallSide, inset: f64) -> [(Point2, f64); 2] {
        let s = match side {
            WallSide::Left => 1.0,
            WallSide::Right => -1.0,
        };
        let left = Point2::new(-flight.dir.y, flight.dir.x);
        let offset = s * (self.width / 2.0 - inset);
        let a = add(flight.start, left, offset);
        let b = add(self.end(flight), left, offset);
        [(a, flight.base + self.rise), (b, flight.base + flight.risers as f64 * self.rise)]
    }
}

/// Geländer (IfcRailing): Handlauf mit senkrechten Stäben.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Railing {
    pub storey: ElementId,
    pub path: RailingPath,
    /// Höhe der Oberkante des Handlaufs in m, über OKFF oder über der
    /// Stufenvorderkante.
    pub height: f64,
    /// Bestand, Abbruch oder Neubau (2.05).
    #[serde(default, skip_serializing_if = "crate::Phase::is_new")]
    pub phase: crate::Phase,
    /// Umwehrung eines Treppenlochs: folgt seinem Rand, ohne die Seite des Austritts (ADR 0051).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub around: Option<ElementId>,
}

/// Verlauf eines Geländers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RailingPath {
    /// Frei verlegt auf OKFF, etwa an der Kante eines Deckendurchbruchs.
    Points(Vec<Point2>),
    /// An einer Seite eines Treppenlaufs (links oder rechts in Gehrichtung nach oben).
    Flight { stair: ElementId, flight: usize, side: WallSide },
    /// Über alle Stufen einer gewendelten oder Spindeltreppe, an der äußeren
    /// oder der inneren Begrenzung (ADR 0051).
    Steps { stair: ElementId, outer: bool },
}

/// Innere Ecke eines Viertelpodests: auf Höhe des Laufendes, auf der Seite des nächsten Laufs.
fn inner_corner(end: Point2, dir: Point2, next_start: Point2) -> Point2 {
    // Der nächste Antritt liegt eine halbe Laufbreite vor und neben dem Laufende.
    let ahead = (next_start.x - end.x) * dir.x + (next_start.y - end.y) * dir.y;
    Point2::new(next_start.x - dir.x * ahead, next_start.y - dir.y * ahead)
}

fn angle(center: Point2, p: Point2) -> f64 {
    (p.y - center.y).atan2(p.x - center.x)
}

/// Links (+1) oder rechts (-1) von der Laufrichtung liegt der nächste Lauf.
fn side_of(dir: Point2, end: Point2, next_start: Point2) -> f64 {
    let cross = dir.x * (next_start.y - end.y) - dir.y * (next_start.x - end.x);
    cross.signum()
}

pub(crate) fn add(p: Point2, d: Point2, t: f64) -> Point2 {
    Point2::new(p.x + d.x * t, p.y + d.y * t)
}

/// Rechteck ab `origin`: `depth` in Richtung `dir`, quer von `from` bis `to` entlang `left`.
fn rect(origin: Point2, dir: Point2, left: Point2, depth: f64, from: f64, to: f64) -> Vec<Point2> {
    vec![
        add(origin, left, from),
        add(origin, left, to),
        add(add(origin, left, to), dir, depth),
        add(add(origin, left, from), dir, depth),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn u_stair() -> Stair {
        Stair {
            phase: Default::default(),
            storey: ElementId(1),
            start: Point2::new(0.5, 0.0),
            direction: Point2::new(0.0, 1.0),
            width: 1.0,
            tread: 0.27,
            flights: vec![
                StairFlight { risers: 8, landing: Landing::BackRight },
                StairFlight { risers: 8, landing: Landing::Straight },
            ],
            thickness: 0.18,
            necessary: true,
            spiral: None,
            eye: 0.0,
        }
    }

    #[test]
    fn rise_follows_storey_height() {
        let l = u_stair().layout(2.75);
        assert!((l.rise - 2.75 / 16.0).abs() < 1e-12);
        assert_eq!(l.flights.len(), 2);
        assert!((l.flights[1].base - 1.375).abs() < 1e-12);
    }

    #[test]
    fn u_turn_landing_and_second_flight() {
        let l = u_stair().layout(2.75);
        // Erster Lauf: 7 Stufen zu 27 cm, Podest 1 m tief, 2 m breit nach rechts.
        let end = l.end(&l.flights[0]);
        assert!((end.y - 1.89).abs() < 1e-9);
        let (outline, top) = &l.landings[0];
        let xs: Vec<f64> = outline.iter().map(|p| p.x).collect();
        assert!((xs.iter().cloned().fold(f64::MAX, f64::min) - 0.0).abs() < 1e-9);
        assert!((xs.iter().cloned().fold(f64::MIN, f64::max) - 2.0).abs() < 1e-9);
        assert!((top - 1.375).abs() < 1e-12);
        let second = l.flights[1];
        assert!((second.start.x - 1.5).abs() < 1e-9 && (second.start.y - 1.89).abs() < 1e-9);
        assert!((second.dir.y + 1.0).abs() < 1e-12);
        // Austritt wieder auf Höhe des Antritts, eine Laufbreite daneben.
        let out = l.end(&second);
        assert!((out.x - 1.5).abs() < 1e-9 && out.y.abs() < 1e-9);
        assert_eq!(l.treads(&second).len(), 7);
        assert!((l.treads(&second)[6].top - (1.375 + 7.0 * 2.75 / 16.0)).abs() < 1e-12);
    }

    #[test]
    fn walking_line_runs_over_the_landing_in_an_arc() {
        let l = u_stair().layout(2.75);
        let line = l.walking_line();
        // Antritt, Podestkante, Halbkreis mit 11 Zwischenpunkten, Podestkante, Austritt.
        assert_eq!(line.len(), 15, "{line:?}");
        // Scheitel des Bogens in der Podestmitte.
        let top = line.iter().map(|p| p.y).fold(f64::MIN, f64::max);
        assert!((top - 2.39).abs() < 1e-9, "{top}");
        assert!(line.iter().all(|p| p.x >= 0.5 - 1e-9 && p.x <= 1.5 + 1e-9));
    }

    #[test]
    fn walking_line_turns_left_around_the_inner_corner() {
        let mut s = u_stair();
        s.flights[0].landing = Landing::Left;
        let line = s.layout(2.75).walking_line();
        // Viertelkreis mit Radius 0,5 um die innere Ecke (0,0 | 1,89).
        for p in &line[2..line.len() - 2] {
            assert!((p.distance(Point2::new(0.0, 1.89)) - 0.5).abs() < 1e-9, "{p:?}");
        }
    }

    #[test]
    fn quarter_landing_turns_left() {
        let mut s = u_stair();
        s.flights[0].landing = Landing::Left;
        let l = s.layout(2.75);
        let second = l.flights[1];
        assert!((second.dir.x + 1.0).abs() < 1e-12);
        // Mitte der linken Podestseite.
        assert!((second.start.x - 0.0).abs() < 1e-9 && (second.start.y - 2.39).abs() < 1e-9);
    }
}
