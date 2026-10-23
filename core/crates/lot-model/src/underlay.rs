//! Unterlagen (6.05): alte Bestandspläne als PDF oder Bild unter einem
//! Grundriss, maßstäblich ausgerichtet zum Nachzeichnen.
//!
//! Gespeichert wird nur, welche Datei es ist und wie sie liegt. Die Datei
//! selbst liegt in der Projektdatei (bei Testprojekten im Projektordner).

use serde::{Deserialize, Serialize};

use crate::{ElementId, Point2};

/// Unterlage unter dem Grundriss eines Geschosses.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Underlay {
    pub storey: ElementId,
    /// Dateiname, etwa „bestand-eg.pdf“.
    pub source: String,
    /// Seite bei PDF-Dateien, ab 0.
    #[serde(default)]
    pub page: u32,
    /// Wo die linke untere Ecke der Seite oder des Bildes im Grundriss liegt, in m.
    pub origin: Point2,
    /// Meter im Modell je Einheit der Vorlage: je PDF-Punkt (1/72 Zoll) oder je Bildpunkt.
    pub scale: f64,
    /// Größe der Seite oder des Bildes in Einheiten der Vorlage.
    pub width: f64,
    pub height: f64,
    /// Drehung um `origin` gegen den Uhrzeigersinn, in Grad.
    #[serde(default)]
    pub rotation: f64,
    /// Deckkraft von 0 bis 1; blass genug, dass der Plan darüber lesbar bleibt.
    #[serde(default = "default_opacity")]
    pub opacity: f64,
}

fn default_opacity() -> f64 {
    0.45
}

/// Meter je PDF-Punkt bei einem Plan im Maßstab 1:`scale`.
pub fn pdf_point_scale(scale: f64) -> f64 {
    0.0254 / 72.0 * scale
}

impl Underlay {
    /// Punkt im Grundriss zu Vorlagenkoordinaten (u nach rechts, v nach oben, ab der linken unteren Ecke).
    pub fn to_model(&self, u: f64, v: f64) -> Point2 {
        let (s, c) = self.rotation.to_radians().sin_cos();
        let (x, y) = (u * self.scale, v * self.scale);
        Point2::new(self.origin.x + x * c - y * s, self.origin.y + x * s + y * c)
    }

    /// Umkehrung von [`Underlay::to_model`].
    pub fn to_source(&self, p: Point2) -> (f64, f64) {
        let (s, c) = self.rotation.to_radians().sin_cos();
        let (dx, dy) = (p.x - self.origin.x, p.y - self.origin.y);
        ((dx * c + dy * s) / self.scale, (-dx * s + dy * c) / self.scale)
    }

    /// Ecken im Grundriss: unten links, unten rechts, oben rechts, oben links.
    pub fn corners(&self) -> [Point2; 4] {
        let (w, h) = (self.width, self.height);
        [self.to_model(0.0, 0.0), self.to_model(w, 0.0), self.to_model(w, h), self.to_model(0.0, h)]
    }

    /// Kalibriert über zwei Punkte, wie sie jetzt im Grundriss liegen, und ihren
    /// wahren Abstand in m. `a` bleibt, wo es ist.
    pub fn calibrated(&self, a: Point2, b: Point2, distance: f64) -> Underlay {
        let measured = a.distance(b);
        if measured < 1e-9 || distance <= 0.0 {
            return self.clone();
        }
        let f = distance / measured;
        let mut out = self.clone();
        out.scale = self.scale * f;
        // Um a strecken: origin' = a + (origin - a) · f.
        out.origin = Point2::new(a.x + (self.origin.x - a.x) * f, a.y + (self.origin.y - a.y) * f);
        out
    }

    /// Dreht um `a`, sodass die Strecke a-b waagerecht oder senkrecht liegt,
    /// je nachdem, was näher ist.
    pub fn aligned(&self, a: Point2, b: Point2) -> Underlay {
        if a.distance(b) < 1e-9 {
            return self.clone();
        }
        let angle = (b.y - a.y).atan2(b.x - a.x).to_degrees();
        let target = (angle / 90.0).round() * 90.0;
        self.rotated(a, target - angle)
    }

    /// Dreht um den Punkt `center` um `degrees` gegen den Uhrzeigersinn.
    pub fn rotated(&self, center: Point2, degrees: f64) -> Underlay {
        let (s, c) = degrees.to_radians().sin_cos();
        let (dx, dy) = (self.origin.x - center.x, self.origin.y - center.y);
        let mut out = self.clone();
        out.origin = Point2::new(center.x + dx * c - dy * s, center.y + dx * s + dy * c);
        out.rotation = self.rotation + degrees;
        out
    }
}

/// Referenzmodell (6.02): ein IFC-Modell eines Fachplaners, im Grundriss, im
/// Schnitt und im 3D zu sehen, aber nicht zu bearbeiten. Die Datei liegt wie
/// Unterlagen im Projekt.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReferenceModel {
    /// Dateiname, etwa „tragwerk.ifc“.
    pub source: String,
    /// Wohin der Ursprung der IFC-Datei im Lot-Modell kommt, in m.
    #[serde(default = "zero")]
    pub origin: Point2,
    /// Höhe des IFC-Ursprungs über ±0,00, in m.
    #[serde(default)]
    pub elevation: f64,
    /// Drehung um den Ursprung gegen den Uhrzeigersinn, in Grad.
    #[serde(default)]
    pub rotation: f64,
}

fn zero() -> Point2 {
    Point2::new(0.0, 0.0)
}

impl ReferenceModel {
    /// Punkt der IFC-Datei (m) im Lot-Modell.
    pub fn to_model(&self, p: [f64; 3]) -> [f64; 3] {
        let (s, c) = self.rotation.to_radians().sin_cos();
        [self.origin.x + p[0] * c - p[1] * s, self.origin.y + p[0] * s + p[1] * c, self.elevation + p[2]]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn underlay() -> Underlay {
        Underlay {
            storey: ElementId(1),
            source: "bestand.pdf".into(),
            page: 0,
            origin: Point2::new(1.0, 2.0),
            scale: pdf_point_scale(100.0),
            width: 842.0,
            height: 595.0,
            rotation: 0.0,
            opacity: 0.45,
        }
    }

    #[test]
    fn pdf_at_1_to_100_maps_points_to_metres() {
        let u = underlay();
        // 72 Punkte sind 1 Zoll, in 1:100 also 2,54 m.
        let p = u.to_model(72.0, 0.0);
        assert!((p.x - 1.0 - 2.54).abs() < 1e-12 && (p.y - 2.0).abs() < 1e-12);
        let (s, t) = u.to_source(p);
        assert!((s - 72.0).abs() < 1e-9 && t.abs() < 1e-9);
    }

    #[test]
    fn calibration_keeps_the_first_point_and_fixes_the_distance() {
        let mut u = underlay();
        u.rotation = 10.0;
        let (a, b) = (u.to_model(100.0, 50.0), u.to_model(300.0, 50.0));
        let c = u.calibrated(a, b, 10.0);
        let (a2, b2) = (c.to_model(100.0, 50.0), c.to_model(300.0, 50.0));
        assert!(a2.distance(a) < 1e-9);
        assert!((a2.distance(b2) - 10.0).abs() < 1e-9);
    }

    #[test]
    fn alignment_turns_a_slightly_skewed_line_level() {
        let mut u = underlay();
        u.rotation = 0.7;
        let (a, b) = (u.to_model(100.0, 50.0), u.to_model(300.0, 50.0));
        let r = u.aligned(a, b);
        let (a2, b2) = (r.to_model(100.0, 50.0), r.to_model(300.0, 50.0));
        assert!(a2.distance(a) < 1e-9);
        assert!((a2.y - b2.y).abs() < 1e-9, "{a2:?} {b2:?}");
        assert!(r.rotation.abs() < 1e-9);
    }
}
