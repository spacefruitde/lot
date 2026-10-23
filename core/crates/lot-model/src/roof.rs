//! Dächer (1.04): Umriss mit Traufen und Giebeln, Dachflächen als Ebenen.

use serde::{Deserialize, Serialize};

use crate::{ElementId, Point2};

/// Dach (IfcRoof). Der Umriss liegt an der Außenkante der Wände; dort hat die
/// Dachhaut die Traufhöhe. Jede Umrisskante ist Traufe (mit Neigung) oder
/// Giebel. Die Dachflächen steigen von den Traufen nach innen an; wo sie sich
/// treffen, entstehen First und Grate.
///
/// Die Dachfläche ist die untere Hülle der Traufebenen. Das ist für konvexe
/// Umrisse exakt (Sattel-, Walm-, Pult- und Flachdach); verwinkelte Grundrisse
/// brauchen ein gewichtetes Straight Skeleton und folgen später.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Roof {
    pub storey: ElementId,
    /// Dachaufbau, Schichten von oben nach unten (wie bei Decken).
    pub roof_type: ElementId,
    pub outline: Vec<Point2>,
    /// Je Umrisskante, Kante i läuft von Punkt i zu Punkt i + 1.
    pub edges: Vec<RoofEdge>,
    /// Höhe der Dachhaut über OKFF am Umriss (Traufhöhe).
    pub eave_height: f64,
    /// Bestand, Abbruch oder Neubau (2.05).
    #[serde(default, skip_serializing_if = "crate::Phase::is_new")]
    pub phase: crate::Phase,
}

/// Art einer Umrisskante.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RoofEdge {
    /// Traufe: Neigung in Grad, Dachüberstand waagerecht gemessen.
    Eave { slope: f64, overhang: f64 },
    /// Giebel (Ortgang): senkrechter Abschluss, Überstand waagerecht.
    Verge { overhang: f64 },
}

impl RoofEdge {
    pub fn overhang(&self) -> f64 {
        match *self {
            RoofEdge::Eave { overhang, .. } | RoofEdge::Verge { overhang } => overhang,
        }
    }
}

/// Ebene einer Dachfläche über einer Traufkante.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EavePlane {
    /// Ein Punkt auf der Traufkante.
    pub origin: Point2,
    /// Einheitsvektor von der Kante nach innen.
    pub inward: (f64, f64),
    pub tan: f64,
    pub cos: f64,
}

impl EavePlane {
    /// Waagerechter Abstand von der Traufkante nach innen.
    pub fn distance(&self, p: Point2) -> f64 {
        (p.x - self.origin.x) * self.inward.0 + (p.y - self.origin.y) * self.inward.1
    }
}

impl Roof {
    /// Ob der Umriss gegen den Uhrzeigersinn läuft.
    fn counter_clockwise(&self) -> bool {
        let n = self.outline.len();
        let area: f64 = (0..n)
            .map(|i| {
                let (a, b) = (self.outline[i], self.outline[(i + 1) % n]);
                a.x * b.y - b.x * a.y
            })
            .sum();
        area > 0.0
    }

    /// Einheitsvektor von Kante `i` nach innen.
    pub fn inward(&self, i: usize) -> (f64, f64) {
        let n = self.outline.len();
        let (a, b) = (self.outline[i], self.outline[(i + 1) % n]);
        let len = a.distance(b).max(1e-12);
        let (dx, dy) = ((b.x - a.x) / len, (b.y - a.y) / len);
        if self.counter_clockwise() { (-dy, dx) } else { (dy, -dx) }
    }

    /// Ebenen aller Traufen.
    pub fn eave_planes(&self) -> Vec<EavePlane> {
        self.edges
            .iter()
            .enumerate()
            .filter_map(|(i, e)| match *e {
                RoofEdge::Eave { slope, .. } => {
                    let r = slope.to_radians();
                    Some(EavePlane { origin: self.outline[i], inward: self.inward(i), tan: r.tan(), cos: r.cos() })
                }
                RoofEdge::Verge { .. } => None,
            })
            .collect()
    }

    /// Höhe der Dachhaut über OKFF an der Stelle `p` (Grundriss).
    pub fn top_at(&self, p: Point2) -> f64 {
        self.underside_at(p, 0.0)
    }

    /// Höhe der Dachunterseite über OKFF an der Stelle `p`, bei Dachdicke `thickness`
    /// (senkrecht zur Dachfläche gemessen).
    pub fn underside_at(&self, p: Point2, thickness: f64) -> f64 {
        let planes = self.eave_planes();
        if planes.is_empty() {
            return self.eave_height - thickness;
        }
        planes
            .iter()
            .map(|e| self.eave_height + e.tan * e.distance(p) - thickness / e.cos)
            .fold(f64::INFINITY, f64::min)
    }

    /// Umriss einschließlich Dachüberstand: jede Kante um ihren Überstand nach außen versetzt.
    pub fn outer_outline(&self) -> Vec<Point2> {
        let n = self.outline.len();
        let line = |i: usize| {
            let (ix, iy) = self.inward(i);
            let o = self.edges.get(i).map_or(0.0, RoofEdge::overhang);
            let a = self.outline[i];
            let b = self.outline[(i + 1) % n];
            (Point2::new(a.x - ix * o, a.y - iy * o), (b.x - a.x, b.y - a.y))
        };
        (0..n)
            .map(|i| {
                let (p, d) = line((i + n - 1) % n);
                let (q, e) = line(i);
                let denom = d.0 * e.1 - d.1 * e.0;
                if denom.abs() < 1e-12 {
                    return q;
                }
                let t = ((q.x - p.x) * e.1 - (q.y - p.y) * e.0) / denom;
                Point2::new(p.x + d.0 * t, p.y + d.1 * t)
            })
            .collect()
    }

    /// Ob der Umriss konvex ist (nur dann ist die Dachform exakt).
    pub fn is_convex(&self) -> bool {
        let n = self.outline.len();
        let mut sign = 0.0;
        for i in 0..n {
            let (a, b, c) = (self.outline[i], self.outline[(i + 1) % n], self.outline[(i + 2) % n]);
            let cross = (b.x - a.x) * (c.y - b.y) - (b.y - a.y) * (c.x - b.x);
            if cross.abs() < 1e-12 {
                continue;
            }
            if sign == 0.0 {
                sign = cross.signum();
            } else if cross.signum() != sign {
                return false;
            }
        }
        true
    }
}

/// Eine Dachfläche, gesehen von ihrer Traufe: Punkt (a, d) liegt `a` entlang
/// der Traufkante ab ihrem Anfang und `d` waagerecht nach innen.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RoofFace {
    pub origin: Point2,
    /// Einheitsvektor entlang der Traufkante.
    pub along: (f64, f64),
    /// Einheitsvektor nach innen.
    pub inward: (f64, f64),
    pub tan: f64,
    pub cos: f64,
}

impl RoofFace {
    pub fn point(&self, a: f64, d: f64) -> Point2 {
        Point2::new(
            self.origin.x + self.along.0 * a + self.inward.0 * d,
            self.origin.y + self.along.1 * a + self.inward.1 * d,
        )
    }

    /// Rechteck a0..a1 × d0..d1 im Grundriss, gegen den Uhrzeigersinn.
    pub fn rect(&self, (a0, a1): (f64, f64), (d0, d1): (f64, f64)) -> Vec<Point2> {
        let pts = vec![self.point(a0, d0), self.point(a1, d0), self.point(a1, d1), self.point(a0, d1)];
        let area: f64 = (0..4).map(|i| pts[i].x * pts[(i + 1) % 4].y - pts[(i + 1) % 4].x * pts[i].y).sum();
        if area < 0.0 { pts.into_iter().rev().collect() } else { pts }
    }
}

impl Roof {
    /// Dachfläche der Traufe `edge`; `None` bei Giebeln.
    pub fn face(&self, edge: usize) -> Option<RoofFace> {
        let RoofEdge::Eave { slope, .. } = *self.edges.get(edge)? else { return None };
        let n = self.outline.len();
        let (a, b) = (self.outline[edge], self.outline[(edge + 1) % n]);
        let len = a.distance(b).max(1e-12);
        let r = slope.to_radians();
        Some(RoofFace {
            origin: a,
            along: ((b.x - a.x) / len, (b.y - a.y) / len),
            inward: self.inward(edge),
            tan: r.tan(),
            cos: r.cos(),
        })
    }
}

/// Rechteck in einer Dachfläche: Fläche, (a0, a1) entlang der Traufe, (d0, d1) nach innen.
pub type FaceRect = (RoofFace, (f64, f64), (f64, f64));

/// Dachfenster (1.05) in der Dachfläche einer Traufe, Maße in m.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoofWindow {
    pub roof: ElementId,
    /// Traufkante, zu deren Dachfläche das Fenster gehört.
    pub edge: usize,
    /// Mitte des Fensters entlang der Traufe, ab dem Anfang der Kante.
    pub offset: f64,
    /// Außenmaß des Rahmens, quer und in Richtung der Neigung.
    pub width: f64,
    pub height: f64,
    /// Höhe der Unterkante auf der Dachhaut über OKFF des Dachgeschosses.
    pub sill_height: f64,
}

impl RoofWindow {
    /// Lage im Grundriss: Dachfläche, (a0, a1) entlang der Traufe, (d0, d1) nach innen.
    pub fn extent(&self, roof: &Roof) -> Option<FaceRect> {
        let face = roof.face(self.edge)?;
        if face.tan < 1e-9 {
            return None;
        }
        let d0 = (self.sill_height - roof.eave_height) / face.tan;
        let d1 = d0 + self.height * face.cos;
        Some((face, (self.offset - self.width / 2.0, self.offset + self.width / 2.0), (d0, d1)))
    }
}

/// Gaube (1.05), vorerst als Schleppgaube: Vorderwand und zwei Wangen, darüber
/// ein Pultdach mit geringerer Neigung als das Hauptdach, das in die
/// Dachfläche der Traufe `edge` einschneidet. Maße in m.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Dormer {
    pub roof: ElementId,
    pub edge: usize,
    /// Mitte der Gaube entlang der Traufe, ab dem Anfang der Kante.
    pub offset: f64,
    /// Außenbreite über die Wangen.
    pub width: f64,
    /// Abstand der Außenseite der Vorderwand vom Dachumriss nach innen.
    pub setback: f64,
    /// Höhe der Dachhaut der Gaube über OKFF, an der Außenseite der Vorderwand.
    pub eave_height: f64,
    /// Neigung des Gaubendachs in Grad.
    pub slope: f64,
    /// Überstand des Gaubendachs vorn und seitlich, waagerecht.
    #[serde(default = "default_dormer_overhang")]
    pub overhang: f64,
    /// Aufbau von Vorderwand und Wangen, Schichten von außen nach innen.
    pub wall_type: ElementId,
    #[serde(default)]
    pub window: Option<DormerWindow>,
}

fn default_dormer_overhang() -> f64 {
    0.3
}

/// Fenster in der Vorderwand einer Gaube, mittig.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DormerWindow {
    pub width: f64,
    pub height: f64,
    /// Brüstungshöhe über OKFF.
    pub sill_height: f64,
}

impl Dormer {
    /// Neigung des Gaubendachs als (tan, cos).
    pub fn pitch(&self) -> (f64, f64) {
        let r = self.slope.to_radians();
        (r.tan(), r.cos())
    }

    /// Wo die Dachhaut der Gaube die des Hauptdachs trifft, als Abstand vom
    /// Umriss nach innen. `None`, wenn die Gaube nicht aus dem Dach ragt oder
    /// ihr Dach nicht flacher ist.
    pub fn ridge_distance(&self, roof: &Roof) -> Option<f64> {
        let face = roof.face(self.edge)?;
        let (tan, _) = self.pitch();
        let front = roof.eave_height + face.tan * self.setback;
        if self.eave_height <= front + 1e-6 || tan >= face.tan - 1e-9 {
            return None;
        }
        Some(self.setback + (self.eave_height - front) / (face.tan - tan))
    }

    /// Höhe der Dachhaut der Gaube über OKFF im Abstand `d` vom Umriss.
    pub fn top_at(&self, d: f64) -> f64 {
        self.eave_height + self.pitch().0 * (d - self.setback)
    }

    /// Außenkanten (a0, a1) entlang der Traufe.
    pub fn span(&self) -> (f64, f64) {
        (self.offset - self.width / 2.0, self.offset + self.width / 2.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Satteldach über 10 × 8 m, Traufen an den Längsseiten (y = 0 und y = 8), 45°.
    fn gable_roof() -> Roof {
        Roof {
            phase: Default::default(),
            storey: ElementId(1),
            roof_type: ElementId(2),
            outline: vec![Point2::new(0.0, 0.0), Point2::new(10.0, 0.0), Point2::new(10.0, 8.0), Point2::new(0.0, 8.0)],
            edges: vec![
                RoofEdge::Eave { slope: 45.0, overhang: 0.5 },
                RoofEdge::Verge { overhang: 0.3 },
                RoofEdge::Eave { slope: 45.0, overhang: 0.5 },
                RoofEdge::Verge { overhang: 0.3 },
            ],
            eave_height: 1.0,
        }
    }

    #[test]
    fn ridge_lies_in_the_middle() {
        let r = gable_roof();
        assert!((r.top_at(Point2::new(5.0, 4.0)) - 5.0).abs() < 1e-9, "First 1 + 4 · tan 45°");
        assert!((r.top_at(Point2::new(2.0, 1.0)) - 2.0).abs() < 1e-9);
        // Dicke 0,3 m senkrecht zur Fläche ist bei 45° senkrecht 0,3 · √2.
        assert!((r.underside_at(Point2::new(5.0, 4.0), 0.3) - (5.0 - 0.3 * 2f64.sqrt())).abs() < 1e-9);
    }

    #[test]
    fn overhang_moves_eaves_and_gables_out() {
        let outer = gable_roof().outer_outline();
        assert!((outer[0].x + 0.3).abs() < 1e-9 && (outer[0].y + 0.5).abs() < 1e-9, "{outer:?}");
        assert!((outer[2].x - 10.3).abs() < 1e-9 && (outer[2].y - 8.5).abs() < 1e-9);
        // Läuft der Umriss andersherum, ändert sich nichts.
        let mut r = gable_roof();
        r.outline.reverse();
        r.edges = vec![r.edges[2], r.edges[1], r.edges[0], r.edges[3]];
        assert!((r.top_at(Point2::new(5.0, 4.0)) - 5.0).abs() < 1e-9);
    }

    #[test]
    fn flat_roof_without_eaves_stays_level() {
        let mut r = gable_roof();
        r.edges = vec![RoofEdge::Verge { overhang: 0.0 }; 4];
        assert_eq!(r.top_at(Point2::new(5.0, 4.0)), 1.0);
        assert!(r.is_convex());
    }
}
