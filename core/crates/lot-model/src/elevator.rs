//! Aufzug (5.02, 5.03): Fahrkorb im Schacht. Den Schacht bilden Wände und
//! Deckendurchbrüche; der Aufzug selbst trägt die Maße des Fahrkorbs, die der
//! Bauantrag und die Barrierefreiheit brauchen.

use serde::{Deserialize, Serialize};

use crate::{ElementId, Point2};

/// Personenaufzug (IfcTransportElement).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Elevator {
    /// Geschosse mit Haltestelle, von unten nach oben.
    pub storeys: Vec<ElementId>,
    /// Mitte des Fahrkorbs.
    pub position: Point2,
    /// Richtung vom Fahrkorb zur Tür, die Länge zählt nicht.
    pub door_direction: Point2,
    /// Fahrkorb innen, Breite (parallel zur Tür) und Tiefe in m.
    pub cabin_width: f64,
    pub cabin_depth: f64,
    /// Lichte Breite der Fahrkorb- und Schachttür in m.
    pub door_width: f64,
    /// Bestand, Abbruch oder Neubau (2.05).
    #[serde(default, skip_serializing_if = "crate::Phase::is_new")]
    pub phase: crate::Phase,
}

impl Elevator {
    /// Einheitsvektoren zur Tür und nach rechts davon (Blick zur Tür).
    pub fn frame(&self) -> ([f64; 2], [f64; 2]) {
        let len = self.door_direction.x.hypot(self.door_direction.y).max(1e-12);
        let d = [self.door_direction.x / len, self.door_direction.y / len];
        (d, [d[1], -d[0]])
    }

    /// Ecken des Fahrkorbs, gegen den Uhrzeigersinn.
    pub fn cabin(&self) -> [Point2; 4] {
        let (d, r) = self.frame();
        let (w, t) = (self.cabin_width / 2.0, self.cabin_depth / 2.0);
        let p =
            |a: f64, b: f64| Point2::new(self.position.x + d[0] * a + r[0] * b, self.position.y + d[1] * a + r[1] * b);
        [p(-t, w), p(t, w), p(t, -w), p(-t, -w)]
    }

    /// Mitte der Tür an der Vorderkante des Fahrkorbs.
    pub fn door_center(&self) -> Point2 {
        let (d, _) = self.frame();
        let t = self.cabin_depth / 2.0;
        Point2::new(self.position.x + d[0] * t, self.position.y + d[1] * t)
    }

    /// Nutzbare Grundfläche des Fahrkorbs in m².
    pub fn cabin_area(&self) -> f64 {
        self.cabin_width * self.cabin_depth
    }

    pub fn stops_at(&self, storey: ElementId) -> bool {
        self.storeys.contains(&storey)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cabin_corners_and_door() {
        let e = Elevator {
            storeys: vec![ElementId(1)],
            position: Point2::new(0.0, 0.0),
            door_direction: Point2::new(0.0, 2.0),
            cabin_width: 1.1,
            cabin_depth: 1.4,
            door_width: 0.9,
            phase: Default::default(),
        };
        let c = e.cabin();
        assert!(c[1].distance(Point2::new(0.55, 0.7)) < 1e-12, "{c:?}");
        assert!(c[3].distance(Point2::new(-0.55, -0.7)) < 1e-12, "{c:?}");
        assert!(e.door_center().distance(Point2::new(0.0, 0.7)) < 1e-12);
        assert!((e.cabin_area() - 1.54).abs() < 1e-12);
    }
}
