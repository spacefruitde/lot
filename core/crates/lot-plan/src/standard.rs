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
    /// Das Büro, für den Plankopf.
    #[serde(default)]
    pub office: Office,
    /// Eigene Layernamen für DXF und DWG: Name in Lot, Name im Büro.
    #[serde(default)]
    pub layers: BTreeMap<String, String>,
    /// Planblätter.
    #[serde(default)]
    pub sheets: SheetSettings,
    /// Präsentationspläne (3.11).
    #[serde(default)]
    pub presentation: PresentationStyle,
}

/// Angaben zum Büro für den Plankopf.
#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct Office {
    #[serde(default)]
    pub name: String,
    /// Anschrift in einer Zeile.
    #[serde(default)]
    pub address: String,
    /// Telefon, E-Mail oder Kammer-Nummer.
    #[serde(default)]
    pub contact: String,
}

/// Einstellungen für Planblätter.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct SheetSettings {
    /// Zeile mit Unterschriftsfeldern für Bauherr und Entwurfsverfasser im Plankopf
    /// (Bauvorlagen tragen beide Unterschriften).
    #[serde(default = "yes")]
    pub signatures: bool,
}

impl Default for SheetSettings {
    fn default() -> Self {
        SheetSettings { signatures: true }
    }
}

fn yes() -> bool {
    true
}

/// Präsentationspläne (3.11): Wände als dunkle Fläche, Räume farbig nach
/// Nutzung, keine Maße.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct PresentationStyle {
    /// Füllung geschnittener Wände, sRGB.
    pub walls: [u8; 3],
    /// Füllung der Räume je Nutzung nach DIN 277 („NUF 1“ bis „VF“); fehlt sie, bleibt der Raum weiß.
    pub spaces: BTreeMap<String, [u8; 3]>,
    /// Grundstück und Gelände im Lageplan.
    pub garden: [u8; 3],
}

impl Default for PresentationStyle {
    fn default() -> Self {
        let spaces = [
            ("NUF 1", [246, 234, 214]),
            ("NUF 4", [236, 236, 236]),
            ("NUF 7", [222, 236, 244]),
            ("TF", [236, 236, 236]),
            ("VF", [242, 240, 236]),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect();
        PresentationStyle { walls: [60, 60, 60], spaces, garden: [214, 230, 196] }
    }
}

/// Bürostandard eines Büros: nur, was vom eingebauten abweicht. Materialien
/// und Layernamen ergänzen oder ersetzen einzelne Einträge, alles andere
/// ersetzt den Abschnitt ganz.
#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OfficeOverrides {
    #[serde(default)]
    pub materials: BTreeMap<String, Material>,
    #[serde(default)]
    pub scales: Option<Vec<ScaleStyle>>,
    #[serde(default)]
    pub phases: Option<PhaseColors>,
    #[serde(default)]
    pub office: Option<Office>,
    #[serde(default)]
    pub layers: BTreeMap<String, String>,
    #[serde(default)]
    pub sheets: Option<SheetSettings>,
    #[serde(default)]
    pub presentation: Option<PresentationStyle>,
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

/// Der Bürostandard, den die App beim Start einsetzt (siehe [`OfficeStandard::install`]).
static INSTALLED: OnceLock<OfficeStandard> = OnceLock::new();

impl OfficeStandard {
    /// Der eingebaute Bürostandard, oder der beim Start eingesetzte.
    pub fn default_standard() -> &'static OfficeStandard {
        INSTALLED.get().unwrap_or_else(Self::builtin)
    }

    /// Der eingebaute Bürostandard, unverändert.
    pub fn builtin() -> &'static OfficeStandard {
        static STANDARD: OnceLock<OfficeStandard> = OnceLock::new();
        STANDARD.get_or_init(|| {
            ron::from_str(include_str!("buerostandard.ron")).expect("eingebauter Bürostandard ist lesbar")
        })
    }

    /// Setzt den Bürostandard für die ganze Sitzung, einmal beim Start. `false`,
    /// wenn schon einer gesetzt oder benutzt war.
    pub fn install(standard: OfficeStandard) -> bool {
        INSTALLED.set(standard).is_ok()
    }

    /// Der eingebaute Bürostandard mit den Abweichungen eines Büros (RON-Text).
    pub fn with_overrides(text: &str) -> Result<OfficeStandard, String> {
        let o: OfficeOverrides = ron::from_str(text).map_err(|e| format!("Bürostandard nicht lesbar: {e}"))?;
        let mut s = Self::builtin().clone();
        s.materials.extend(o.materials);
        s.layers.extend(o.layers);
        if let Some(v) = o.scales {
            if v.is_empty() {
                return Err("Bürostandard: mindestens ein Maßstabsbereich".into());
            }
            s.scales = v;
        }
        if let Some(v) = o.phases {
            s.phases = v;
        }
        if let Some(v) = o.office {
            s.office = v;
        }
        if let Some(v) = o.sheets {
            s.sheets = v;
        }
        if let Some(v) = o.presentation {
            s.presentation = v;
        }
        Ok(s)
    }

    /// Layername im Büro für einen Layer von Lot.
    pub fn layer_name<'a>(&'a self, lot: &'a str) -> &'a str {
        self.layers.get(lot).map_or(lot, |s| s.as_str())
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
    fn overrides_replace_only_what_they_name() {
        let text = r#"(
            office: Some((name: "Büro Muster", address: "Hauptstraße 1, 12345 Musterstadt")),
            layers: { "A_Bemassung": "A-ANNO-DIMS" },
            materials: { "lehm": (name: "Lehm", color: (200, 170, 120), cut: Lines(angle: 45.0, spacing: 2.0)) },
            sheets: Some((signatures: false)),
        )"#;
        let s = OfficeStandard::with_overrides(text).unwrap();
        assert_eq!(s.office.name, "Büro Muster");
        assert_eq!(s.layer_name("A_Bemassung"), "A-ANNO-DIMS");
        assert_eq!(s.layer_name("A_Treppen"), "A_Treppen");
        assert!(s.materials.contains_key("lehm") && s.materials.contains_key("kalksandstein"));
        assert_eq!(s.scales, OfficeStandard::builtin().scales);
        assert!(!s.sheets.signatures);
        assert!(OfficeStandard::with_overrides("(scales: Some([]))").is_err());
        assert!(OfficeStandard::with_overrides("(gibtsnicht: 1)").is_err());
    }

    #[test]
    fn unknown_material_is_empty() {
        assert_eq!(OfficeStandard::default_standard().material("gibtsnicht").cut, CutPattern::Empty);
    }
}
