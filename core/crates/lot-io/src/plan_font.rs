//! Planschrift: Maße und Umrisse der Schrift, mit der alle Ausgaben Texte setzen.
//!
//! Alle Ausgaben nutzen dieselbe Schriftdatei, damit PDF, SVG und PNG
//! unabhängig vom System gleich aussehen. Die Schrift (osifont, LGPL-3 mit
//! Font-Ausnahme, siehe ADR 0011) wird zur Laufzeit als eigene Datei geladen
//! und nicht in das Programm einkompiliert.

use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use lot_plan::{Anchor, Text};

/// Dateiname der Planschrift, für die SVG-Ausgabe.
pub const FAMILY: &str = "osifont";
const FILE_NAME: &str = "osifont-lgpl3fe.ttf";

pub struct PlanFont {
    data: Arc<Vec<u8>>,
    units_per_em: f64,
    cap_height: f64,
}

/// Ort der Schriftdatei: die Umgebungsvariable `LOT_PLAN_FONT`, sonst
/// `core/assets/fonts/` im Repository. Die Mac-App legt die Datei später in
/// ihr Bundle und setzt den Pfad.
pub fn font_path() -> PathBuf {
    std::env::var_os("LOT_PLAN_FONT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts").join(FILE_NAME))
}

/// Die Planschrift. Wird beim ersten Gebrauch gelesen.
pub fn plan_font() -> &'static PlanFont {
    static FONT: OnceLock<PlanFont> = OnceLock::new();
    FONT.get_or_init(|| {
        let path = font_path();
        let data = std::fs::read(&path).unwrap_or_else(|e| panic!("Planschrift {} nicht lesbar: {e}", path.display()));
        PlanFont::new(data).unwrap_or_else(|| panic!("{} ist keine lesbare Schrift", path.display()))
    })
}

impl PlanFont {
    pub fn new(data: Vec<u8>) -> Option<Self> {
        let face = ttf_parser::Face::parse(&data, 0).ok()?;
        let units_per_em = face.units_per_em() as f64;
        // Versalhöhe aus der Schrift, sonst aus der Höhe des „H“.
        let cap_height = face
            .capital_height()
            .filter(|h| *h > 0)
            .map(f64::from)
            .or_else(|| {
                let h = face.glyph_index('H')?;
                face.glyph_bounding_box(h).map(|b| b.y_max as f64)
            })
            .unwrap_or(units_per_em * 0.7);
        Some(Self { data: Arc::new(data), units_per_em, cap_height })
    }

    pub fn data(&self) -> &Arc<Vec<u8>> {
        &self.data
    }

    fn face(&self) -> ttf_parser::Face<'_> {
        ttf_parser::Face::parse(&self.data, 0).expect("Schrift wurde schon einmal gelesen")
    }

    /// Schriftgröße (Geviert) in mm für eine Versalhöhe in mm.
    pub fn size_for_cap_height(&self, cap_height_mm: f64) -> f64 {
        cap_height_mm * self.units_per_em / self.cap_height
    }

    /// Zeichen eines Textes, die in der Schrift fehlen und deshalb nicht erscheinen würden.
    pub fn missing_chars(&self, text: &str) -> Vec<char> {
        let face = self.face();
        let mut missing: Vec<char> = text.chars().filter(|c| face.glyph_index(*c).is_none()).collect();
        missing.dedup();
        missing
    }

    /// Breite eines Textes in mm bei gegebener Versalhöhe, ohne Unterschneidung.
    pub fn width(&self, text: &str, cap_height_mm: f64) -> f64 {
        let face = self.face();
        let units: f64 =
            text.chars().map(|c| face.glyph_index(c).and_then(|g| face.glyph_hor_advance(g)).unwrap_or(0) as f64).sum();
        units / self.units_per_em * self.size_for_cap_height(cap_height_mm)
    }

    /// Linker Startpunkt der Grundlinie, abhängig von der Ausrichtung.
    pub fn start_x(&self, text: &Text) -> f64 {
        let width = || self.width(&text.text, text.height_mm);
        match text.anchor {
            Anchor::Left => text.position.x,
            Anchor::Center => text.position.x - width() / 2.0,
            Anchor::Right => text.position.x - width(),
        }
    }

    /// Umrisse aller Zeichen eines Textes, in mm auf dem Blatt (y nach oben).
    pub fn outline(&self, text: &Text, sink: &mut impl OutlineSink) {
        let face = self.face();
        let size = self.size_for_cap_height(text.height_mm);
        let scale = size / self.units_per_em;
        let mut x = self.start_x(text);
        let (sin, cos) = text.rotation.to_radians().sin_cos();
        let origin = (text.position.x, text.position.y);
        for c in text.text.chars() {
            let Some(glyph) = face.glyph_index(c) else { continue };
            let mut builder = Builder { sink, x, y: text.position.y, scale, origin, sin, cos };
            face.outline_glyph(glyph, &mut builder);
            x += face.glyph_hor_advance(glyph).unwrap_or(0) as f64 * scale;
        }
    }
}

/// Empfängt die Umrisse von Zeichen, Koordinaten in mm.
pub trait OutlineSink {
    fn move_to(&mut self, x: f64, y: f64);
    fn line_to(&mut self, x: f64, y: f64);
    fn quad_to(&mut self, x1: f64, y1: f64, x: f64, y: f64);
    fn cubic_to(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, x: f64, y: f64);
    fn close(&mut self);
}

struct Builder<'a, S: OutlineSink> {
    sink: &'a mut S,
    x: f64,
    y: f64,
    scale: f64,
    /// Drehpunkt und Drehung des ganzen Textes.
    origin: (f64, f64),
    sin: f64,
    cos: f64,
}

impl<S: OutlineSink> Builder<'_, S> {
    fn p(&self, x: f32, y: f32) -> (f64, f64) {
        let (px, py) = (self.x + x as f64 * self.scale, self.y + y as f64 * self.scale);
        if self.sin == 0.0 {
            return (px, py);
        }
        let (dx, dy) = (px - self.origin.0, py - self.origin.1);
        (self.origin.0 + dx * self.cos - dy * self.sin, self.origin.1 + dx * self.sin + dy * self.cos)
    }
}

impl<S: OutlineSink> ttf_parser::OutlineBuilder for Builder<'_, S> {
    fn move_to(&mut self, x: f32, y: f32) {
        let (x, y) = self.p(x, y);
        self.sink.move_to(x, y);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let (x, y) = self.p(x, y);
        self.sink.line_to(x, y);
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let (x1, y1) = self.p(x1, y1);
        let (x, y) = self.p(x, y);
        self.sink.quad_to(x1, y1, x, y);
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let (x1, y1) = self.p(x1, y1);
        let (x2, y2) = self.p(x2, y2);
        let (x, y) = self.p(x, y);
        self.sink.cubic_to(x1, y1, x2, y2, x, y);
    }

    fn close(&mut self) {
        self.sink.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cap_height_matches_request() {
        let font = plan_font();
        // Ein „H“ mit 5 mm Versalhöhe ist 5 mm hoch.
        let face = font.face();
        let h = face.glyph_index('H').unwrap();
        let bbox = face.glyph_bounding_box(h).unwrap();
        let size = font.size_for_cap_height(5.0);
        let height = bbox.y_max as f64 / font.units_per_em * size;
        assert!((height - 5.0).abs() < 0.1, "H ist {height} mm hoch");
    }

    #[test]
    fn m0_sheet_uses_only_available_chars() {
        let sheet = lot_plan::m0::sheet(&lot_model::Project::sample(), "test");
        for object in &sheet.objects {
            if let lot_plan::PlanObject::Text(t) = object {
                let missing = plan_font().missing_chars(&t.text);
                assert!(missing.is_empty(), "„{}“: Zeichen {missing:?} fehlen in der Planschrift", t.text);
            }
        }
    }

    #[test]
    fn umlauts_are_in_font() {
        let face = plan_font().face();
        for c in "ÄÖÜäöüß²°".chars() {
            assert!(face.glyph_index(c).is_some(), "Zeichen {c} fehlt in der Planschrift");
        }
    }
}
