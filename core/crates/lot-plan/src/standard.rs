//! Bürostandard (2.08): Darstellungsregeln als Daten, nicht als Code
//! (PLAN.md, Abschnitt 4, Grundentscheidung 10).
//!
//! Die Voreinstellung steht in `buerostandard.ron` neben dieser Datei. Sie
//! ist ein Vorschlag nach DIN 1356-1:1995-02 und wird von Ben geprüft.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct OfficeStandard {
    pub materials: BTreeMap<String, Material>,
    pub scales: Vec<ScaleStyle>,
    /// Farben für Bestand, Abbruch und Neubau (2.05).
    #[serde(default)]
    pub phases: PhaseColors,
}

/// Füllfarben geschnittener Bauteile im Umbauplan, sRGB. Die Bauvorlagenverordnungen
/// der Länder schreiben Bestand grau, Abbruch gelb und Neubau rot vor; die Töne
/// selbst legt das Büro fest.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct PhaseColors {
    pub existing: [u8; 3],
    pub demolished: [u8; 3],
    pub new: [u8; 3],
}

impl Default for PhaseColors {
    fn default() -> Self {
        PhaseColors { existing: [170, 170, 170], demolished: [255, 221, 0], new: [226, 35, 26] }
    }
}

impl PhaseColors {
    pub fn of(&self, phase: lot_model::Phase) -> crate::Color {
        let [r, g, b] = match phase {
            lot_model::Phase::Existing => self.existing,
            lot_model::Phase::Demolished => self.demolished,
            lot_model::Phase::New => self.new,
        };
        crate::Color::rgb8(r, g, b)
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Material {
    pub name: String,
    /// Farbe in der 3D-Ansicht, sRGB.
    pub color: [u8; 3],
    /// Darstellung geschnittener Flächen bei voller Detailstufe.
    pub cut: CutPattern,
}

/// Muster einer geschnittenen Fläche.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub enum CutPattern {
    Empty,
    /// Vollfläche, Grauwert 0 (schwarz) bis 1 (weiß).
    Solid {
        gray: f64,
    },
    /// Parallele Linien.
    Lines {
        angle: f64,
        spacing: f64,
    },
    /// Zwei Scharen paralleler Linien, um 90° versetzt.
    Cross {
        angle: f64,
        spacing: f64,
    },
    /// Dämmstoff: Schlangenlinie (weich) oder Zickzack (hart) entlang der Wand.
    Insulation {
        soft: bool,
    },
}

/// Detailgrad eines Maßstabsbereichs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum Detail {
    /// Wände als gefüllte Fläche, keine Schichten.
    Coarse,
    /// Tragschicht mit Muster, weitere Schichten nur als Linie.
    Medium,
    /// Alle Schichten mit Materialmuster.
    Fine,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ScaleStyle {
    /// Gilt bis zu dieser Maßstabszahl (einschließlich).
    pub up_to: f64,
    pub detail: Detail,
    pub pens: Pens,
    /// Schrifthöhe für Beschriftungen, in mm.
    pub text: f64,
    /// Schrifthöhe des Raumnamens im Raumstempel, in mm.
    pub space_name: f64,
    /// Schrifthöhe von Raumnummer und Fläche im Raumstempel, in mm.
    pub space_text: f64,
    /// Schrifthöhe der Maßzahlen, in mm.
    pub dimension_text: f64,
    /// Durchmesser der Achskreise, in mm.
    pub grid_bubble: f64,
}

/// Linienbreiten in mm.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub struct Pens {
    /// Umriss geschnittener Bauteile
    pub cut: f64,
    /// Grenzen zwischen Schichten im Schnitt
    pub layer: f64,
    /// Schraffurlinien
    pub hatch: f64,
    /// sichtbare Kanten unterhalb der Schnittebene
    pub view: f64,
    /// verdeckte Kanten
    pub hidden: f64,
    /// Achsen
    pub grid: f64,
    /// Beschriftung, Maßlinien
    pub annotation: f64,
}

impl OfficeStandard {
    /// Der eingebaute Bürostandard.
    pub fn default_standard() -> &'static OfficeStandard {
        static STANDARD: OnceLock<OfficeStandard> = OnceLock::new();
        STANDARD.get_or_init(|| {
            ron::from_str(include_str!("buerostandard.ron")).expect("eingebauter Bürostandard ist lesbar")
        })
    }

    /// Darstellung für einen Maßstab 1:`scale`.
    pub fn style_for(&self, scale: f64) -> &ScaleStyle {
        self.scales.iter().find(|s| scale <= s.up_to).unwrap_or_else(|| self.scales.last().expect("Maßstäbe definiert"))
    }

    /// Material zu einem Schlüssel; unbekannte Materialien erscheinen leer und grau.
    pub fn material(&self, key: &str) -> Material {
        self.materials.get(key).cloned().unwrap_or_else(|| Material {
            name: key.to_string(),
            color: [200, 200, 200],
            cut: CutPattern::Empty,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_standard_loads() {
        let s = OfficeStandard::default_standard();
        assert!(s.materials.contains_key("kalksandstein"));
        assert_eq!(s.style_for(50.0).detail, Detail::Fine);
        assert_eq!(s.style_for(100.0).detail, Detail::Medium);
        assert_eq!(s.style_for(200.0).detail, Detail::Coarse);
        assert_eq!(s.style_for(20.0).detail, Detail::Fine);
    }

    #[test]
    fn unknown_material_is_empty() {
        assert_eq!(OfficeStandard::default_standard().material("gibtsnicht").cut, CutPattern::Empty);
    }
}
