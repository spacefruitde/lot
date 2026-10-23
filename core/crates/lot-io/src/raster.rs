//! Bilder lesen, etwa Unterlagen als PNG. PDF-Seiten rendert die App mit den
//! Mitteln des Systems (auf dem Mac PDFKit, ADR 0016) und gibt die Bildpunkte weiter.

use crate::IoError;

/// Bild mit 8 Bit je Kanal, RGBA, Zeilen von oben nach unten.
#[derive(Debug, Clone, PartialEq)]
pub struct Raster {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

/// Liest ein PNG.
pub fn decode_png(bytes: &[u8]) -> Result<Raster, IoError> {
    let pixmap = tiny_skia::Pixmap::decode_png(bytes).map_err(|e| IoError::Png(e.to_string()))?;
    let rgba = pixmap
        .pixels()
        .iter()
        .flat_map(|p| {
            let c = p.demultiply();
            [c.red(), c.green(), c.blue(), c.alpha()]
        })
        .collect();
    Ok(Raster { width: pixmap.width(), height: pixmap.height(), rgba })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn png_roundtrip_keeps_size_and_colour() {
        let mut pixmap = tiny_skia::Pixmap::new(3, 2).unwrap();
        pixmap.fill(tiny_skia::Color::from_rgba8(200, 100, 50, 255));
        let png = pixmap.encode_png().unwrap();
        let r = decode_png(&png).unwrap();
        assert_eq!((r.width, r.height, r.rgba.len()), (3, 2, 24));
        assert_eq!(&r.rgba[..4], &[200, 100, 50, 255]);
        assert!(decode_png(b"kein Bild").is_err());
    }
}
