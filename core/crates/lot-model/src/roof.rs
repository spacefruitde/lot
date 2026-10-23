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
}

/// Art einer Umrisskante.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RoofEdge {
    /// Traufe: Neigung in Grad, Dachüberstand waagerecht gemessen.
    Eave { slope: f64, overhang: f64 },
    /// Giebel (Ortgang): senkrechter Abschluss, Überstand waagerecht.
    Gable { overhang: f64 },
}

impl RoofEdge {
    pub fn overhang(&self) -> f64 {
        match *self {
            RoofEdge::Eave { overhang, .. } | RoofEdge::Gable { overhang } => overhang,
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
                RoofEdge::Gable { .. } => None,
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Satteldach über 10 × 8 m, Traufen an den Längsseiten (y = 0 und y = 8), 45°.
    fn gable_roof() -> Roof {
        Roof {
            storey: ElementId(1),
            roof_type: ElementId(2),
            outline: vec![Point2::new(0.0, 0.0), Point2::new(10.0, 0.0), Point2::new(10.0, 8.0), Point2::new(0.0, 8.0)],
            edges: vec![
                RoofEdge::Eave { slope: 45.0, overhang: 0.5 },
                RoofEdge::Gable { overhang: 0.3 },
                RoofEdge::Eave { slope: 45.0, overhang: 0.5 },
                RoofEdge::Gable { overhang: 0.3 },
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
        r.edges = vec![RoofEdge::Gable { overhang: 0.0 }; 4];
        assert_eq!(r.top_at(Point2::new(5.0, 4.0)), 1.0);
        assert!(r.is_convex());
    }
}
