//! Gefälle (ADR 0051): eine Eigenschaft einer Schicht der Decke, mit
//! Richtung, Prozent, Abläufen und Notüberläufen. Die Schicht ist am tiefsten
//! Punkt so dick wie im Aufbau und wird gegen die Fließrichtung dicker; was
//! über ihr liegt, steigt mit.

use serde::{Deserialize, Serialize};

use crate::Point2;

/// Wofür das Gefälle ist; bestimmt den Mindestwert der Prüfung.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum FallPurpose {
    /// Flachdach.
    #[default]
    Roof,
    /// Balkon, Loggia, Terrasse.
    Balcony,
    /// Bodengleiche Dusche.
    Shower,
}

impl FallPurpose {
    pub const ALL: [FallPurpose; 3] = [FallPurpose::Roof, FallPurpose::Balcony, FallPurpose::Shower];

    pub fn title(self) -> &'static str {
        match self {
            FallPurpose::Roof => "Flachdach",
            FallPurpose::Balcony => "Balkon, Loggia, Terrasse",
            FallPurpose::Shower => "Bodengleiche Dusche",
        }
    }
}

/// Gefälle einer Schicht.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Fall {
    #[serde(default)]
    pub purpose: FallPurpose,
    /// Schicht des Aufbaus, die das Gefälle bildet (Gefälledämmung, Gefälleestrich), von oben gezählt.
    pub layer: usize,
    /// Gefälle in Prozent.
    pub percent: f64,
    /// Richtung, in die das Wasser fließt, in Grad gegen den Uhrzeigersinn von +x.
    pub direction: f64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub drains: Vec<Point2>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub overflows: Vec<Point2>,
}

impl Fall {
    /// Fließrichtung als Einheitsvektor.
    pub fn flow(&self) -> (f64, f64) {
        let a = self.direction.to_radians();
        (a.cos(), a.sin())
    }

    /// Mehrdicke der Gefälleschicht an `p` über dem tiefsten Punkt des Umrisses, in Metern.
    pub fn extra_at(&self, outline: &[Point2], p: Point2) -> f64 {
        let (dx, dy) = self.flow();
        let low = outline.iter().map(|q| q.x * dx + q.y * dy).fold(f64::NEG_INFINITY, f64::max);
        (self.percent / 100.0 * (low - (p.x * dx + p.y * dy))).max(0.0)
    }

    /// Größte Mehrdicke im Umriss, am höchsten Punkt.
    pub fn max_extra(&self, outline: &[Point2]) -> f64 {
        outline.iter().map(|q| self.extra_at(outline, *q)).fold(0.0, f64::max)
    }
}
