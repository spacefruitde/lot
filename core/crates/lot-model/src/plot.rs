//! Grundstück (5.01): Grenzen des Baugrundstücks und der Nachbarn, für den
//! Lageplan und die Abstandsflächen.

use serde::{Deserialize, Serialize};

use crate::Point2;

/// Grundstück (Flurstück) mit seinen Grenzen. Das Baugrundstück trägt die
/// Abstandsflächen; Nachbargrundstücke werden im Lageplan nur dargestellt.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Plot {
    /// Bezeichnung, etwa „Flurstück 123/4“.
    #[serde(default)]
    pub name: String,
    /// Grenzpunkte in Metern, Richtung egal.
    pub outline: Vec<Point2>,
    /// Grenzen zu öffentlichen Verkehrs-, Grün- oder Wasserflächen. Dort dürfen
    /// Abstandsflächen bis zur Mitte dieser Fläche liegen (MBO § 6 Abs. 2 Satz 2).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub public_edges: Vec<PublicEdge>,
    /// Nachbargrundstück, nur zur Darstellung.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub neighbour: bool,
}

/// Grenze zu einer öffentlichen Fläche: Kante `edge` des Umrisses (von Punkt
/// `edge` zu Punkt `edge + 1`) und die Breite der Fläche davor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PublicEdge {
    pub edge: usize,
    pub width: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<PublicKind>,
    /// Bezeichnung im Lageplan, etwa „Musterstraße“.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PublicKind {
    /// Öffentliche Verkehrsfläche
    Road,
    /// Öffentliche Grünfläche
    Green,
    /// Öffentliche Wasserfläche
    Water,
}

impl Plot {
    /// Fläche in m² (Gaußsche Trapezformel).
    pub fn area(&self) -> f64 {
        let n = self.outline.len();
        (0..n)
            .map(|i| {
                let (a, b) = (self.outline[i], self.outline[(i + 1) % n]);
                a.x * b.y - b.x * a.y
            })
            .sum::<f64>()
            .abs()
            / 2.0
    }

    /// Kante `i` als Strecke.
    pub fn edge(&self, i: usize) -> Option<(Point2, Point2)> {
        let n = self.outline.len();
        (i < n).then(|| (self.outline[i], self.outline[(i + 1) % n]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn area_of_rectangle() {
        let p = Plot {
            name: "Flurstück 1".into(),
            outline: vec![
                Point2::new(0.0, 0.0),
                Point2::new(30.0, 0.0),
                Point2::new(30.0, 20.0),
                Point2::new(0.0, 20.0),
            ],
            public_edges: vec![],
            neighbour: false,
        };
        assert!((p.area() - 600.0).abs() < 1e-9);
        assert_eq!(p.edge(3), Some((Point2::new(0.0, 20.0), Point2::new(0.0, 0.0))));
    }
}
