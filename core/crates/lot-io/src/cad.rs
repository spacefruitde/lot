//! DXF und DWG (6.03, 6.04) über acadrust (ADR 0012).
//!
//! Ausgabe: Planobjekte aus derselben Planpipeline wie PDF und Bildschirm,
//! je Layer, als Polylinien, Schraffuren und Texte. Grundrisse und Schnitte
//! stehen im Modellbereich in Metern (1 Einheit = 1 m), Planblätter in
//! Millimetern. Linienbreiten gehen als Linienstärke mit, Farben als
//! RGB-Farbe, gestrichelte Linien mit eigenen Linientypen.
//!
//! Eingabe: Linien, Polylinien, Kreise, Bögen und Texte einer DXF- oder
//! DWG-Datei als Vektoren, etwa als Unterlage zum Nachzeichnen.

use std::io::Cursor;

use acadrust::entities::{
    BoundaryEdge, BoundaryPath, BoundaryPathFlags, EntityType, Hatch, HatchPattern, HatchPatternLine,
    HatchPatternType, LwPolyline, PolylineEdge, Text, TextHorizontalAlignment,
};
use acadrust::tables::{Layer, LineType, LineTypeElement};
use acadrust::types::{Color as CadColor, DxfVersion, LineWeight, Vector2, Vector3};
pub use acadrust::CadDocument;
use acadrust::{DwgReader, DwgWriter, DxfReader, DxfWriter};
use lot_plan::{Anchor, Color, Fill, LineStyle, PaperPoint, PlanObject};

use crate::IoError;

/// Einheit der Zeichnung.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CadUnits {
    Meters,
    Millimeters,
}

impl CadUnits {
    /// INSUNITS nach DXF-Referenz: 4 = mm, 6 = m.
    fn insunits(self) -> i16 {
        match self {
            CadUnits::Meters => 6,
            CadUnits::Millimeters => 4,
        }
    }
}

/// Fasst Planobjekte je Layer zu einem CAD-Dokument zusammen. `factor` sind
/// Zeichnungseinheiten je Blattmillimeter: bei Grundrissen und Schnitten in
/// Metern die Maßstabszahl durch 1000, bei Planblättern in mm 1.
pub fn to_cad(layers: &[(&str, Vec<PlanObject>)], factor: f64, units: CadUnits) -> Result<CadDocument, IoError> {
    let mut doc = CadDocument::with_version(DxfVersion::AC1027);
    doc.header.insertion_units = units.insunits();
    doc.header.measurement = 1;
    let mut dashes: Vec<Vec<f64>> = Vec::new();
    for (name, objects) in layers {
        let mut layer = Layer::new(*name);
        layer.handle = doc.allocate_handle();
        layer.color = CadColor::Index(7);
        doc.layers.add(layer).map_err(IoError::Cad)?;
        for o in objects {
            for e in entities(o, name, factor, &mut doc, &mut dashes)? {
                doc.add_entity(e).map_err(|e| IoError::Cad(e.to_string()))?;
            }
        }
    }
    Ok(doc)
}

/// Als DXF (ASCII).
pub fn dxf_bytes(doc: &CadDocument) -> Result<Vec<u8>, IoError> {
    DxfWriter::new(doc).write_to_vec().map_err(|e| IoError::Cad(e.to_string()))
}

/// Als DWG.
pub fn dwg_bytes(doc: &CadDocument) -> Result<Vec<u8>, IoError> {
    DwgWriter::write_to_vec(doc).map_err(|e| IoError::Cad(e.to_string()))
}

fn point(p: &PaperPoint, factor: f64) -> Vector2 {
    Vector2::new(p.x * factor, p.y * factor)
}

/// Nächste übliche Linienstärke in 1/100 mm.
fn line_weight(mm: f64) -> LineWeight {
    const WEIGHTS: [i16; 24] =
        [0, 5, 9, 13, 15, 18, 20, 25, 30, 35, 40, 50, 53, 60, 70, 80, 90, 100, 106, 120, 140, 158, 200, 211];
    let target = (mm * 100.0).round() as i16;
    let best = WEIGHTS.iter().copied().min_by_key(|w| (w - target).abs()).unwrap_or(25);
    LineWeight::Value(best)
}

/// Schwarz bleibt Farbe 7 (schwarz auf weißem, weiß auf dunklem Grund), alles andere RGB.
fn color(c: Color) -> CadColor {
    let [r, g, b] = c.bytes();
    if [r, g, b] == [0, 0, 0] { CadColor::Index(7) } else { CadColor::Rgb { r, g, b } }
}

/// Linientyp für ein Strichmuster in Blattmillimetern, einmal je Muster angelegt.
fn line_type(pattern: &[f64], factor: f64, doc: &mut CadDocument, known: &mut Vec<Vec<f64>>) -> Result<String, IoError> {
    let index = match known.iter().position(|k| k == pattern) {
        Some(i) => i,
        None => {
            let name = format!("LOT_STRICH_{}", known.len() + 1);
            let mut lt = LineType::new(&name);
            lt.handle = doc.allocate_handle();
            lt.description = "Lot, gestrichelt".into();
            for (i, len) in pattern.iter().enumerate() {
                let l = len * factor;
                lt.add_element(if i % 2 == 0 { LineTypeElement::dash(l) } else { LineTypeElement::space(l) });
            }
            lt.pattern_length = pattern.iter().map(|l| l.abs() * factor).sum();
            doc.line_types.add(lt).map_err(IoError::Cad)?;
            known.push(pattern.to_vec());
            known.len() - 1
        }
    };
    Ok(format!("LOT_STRICH_{}", index + 1))
}

fn entities(
    o: &PlanObject,
    layer: &str,
    factor: f64,
    doc: &mut CadDocument,
    dashes: &mut Vec<Vec<f64>>,
) -> Result<Vec<EntityType>, IoError> {
    Ok(match o {
        PlanObject::Line(l) => {
            if l.points.len() < 2 {
                return Ok(Vec::new());
            }
            let mut p = LwPolyline::from_points(l.points.iter().map(|q| point(q, factor)).collect());
            if l.closed {
                p.close();
            }
            p.common.layer = layer.into();
            p.common.line_weight = line_weight(l.pen.width_mm);
            p.common.color = color(l.color);
            if let LineStyle::Dashed(pattern) = &l.style {
                p.common.linetype = line_type(pattern, factor, doc, dashes)?;
            }
            vec![EntityType::LwPolyline(p)]
        }
        PlanObject::Area(a) => {
            let rings: Vec<&Vec<PaperPoint>> = a.rings.iter().filter(|r| r.len() >= 3).collect();
            if rings.is_empty() {
                return Ok(Vec::new());
            }
            let mut hatch = match &a.fill {
                Fill::Solid(c) => {
                    let mut h = Hatch::solid();
                    h.common.color = color(*c);
                    h
                }
                Fill::Hatch { angle_deg, spacing_mm, pen } => {
                    // Benutzerdefinierte Schraffur: eine Linienschar mit Winkel und Abstand.
                    let angle = angle_deg.to_radians();
                    let spacing = spacing_mm * factor;
                    let mut pattern = HatchPattern::new("_USER");
                    pattern.add_line(HatchPatternLine {
                        angle,
                        base_point: Vector2::new(0.0, 0.0),
                        offset: Vector2::new(-angle.sin() * spacing, angle.cos() * spacing),
                        dash_lengths: Vec::new(),
                    });
                    let mut h = Hatch::with_pattern(pattern);
                    h.pattern_type = HatchPatternType::UserDefined;
                    h.pattern_angle = angle;
                    h.pattern_scale = spacing;
                    h.common.line_weight = line_weight(pen.width_mm);
                    h.common.color = CadColor::Index(7);
                    h
                }
            };
            for ring in rings {
                let mut flags = BoundaryPathFlags::new();
                flags.set_external(true);
                flags.set_polyline(true);
                let mut path = BoundaryPath::with_flags(flags);
                path.add_edge(BoundaryEdge::Polyline(PolylineEdge::new(
                    ring.iter().map(|q| point(q, factor)).collect(),
                    true,
                )));
                hatch.add_path(path);
            }
            hatch.common.layer = layer.into();
            vec![EntityType::Hatch(Box::new(hatch))]
        }
        PlanObject::Text(t) => {
            let at = point(&t.position, factor);
            let mut text = Text::with_value(t.text.clone(), Vector3::new(at.x, at.y, 0.0))
                .with_height(t.height_mm * factor)
                .with_rotation(t.rotation.to_radians());
            match t.anchor {
                Anchor::Left => {}
                Anchor::Center => {
                    text.horizontal_alignment = TextHorizontalAlignment::Center;
                    text.alignment_point = Some(Vector3::new(at.x, at.y, 0.0));
                }
                Anchor::Right => {
                    text.horizontal_alignment = TextHorizontalAlignment::Right;
                    text.alignment_point = Some(Vector3::new(at.x, at.y, 0.0));
                }
            }
            text.common.layer = layer.into();
            text.common.color = CadColor::Index(7);
            vec![EntityType::Text(text)]
        }
    })
}

/// Vektoren aus einer DXF- oder DWG-Datei, in Zeichnungseinheiten.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CadVectors {
    /// Linienzüge (Linien, Polylinien, Kreise und Bögen als Polygone).
    pub lines: Vec<Vec<[f64; 2]>>,
    /// Texte: Lage, Inhalt, Höhe.
    pub texts: Vec<([f64; 2], String, f64)>,
    /// INSUNITS der Datei (0 unbekannt, 4 mm, 5 cm, 6 m).
    pub units: i16,
}

impl CadVectors {
    /// Meter je Zeichnungseinheit nach INSUNITS; ohne Angabe nach der Größe
    /// geschätzt (über 1000 Einheiten breit: Millimeter, sonst Meter).
    pub fn metres_per_unit(&self) -> f64 {
        match self.units {
            4 => 0.001,
            5 => 0.01,
            6 => 1.0,
            1 => 0.0254,
            2 => 0.3048,
            _ => {
                let (lo, hi) = self.bounds().unwrap_or(([0.0, 0.0], [1.0, 1.0]));
                if (hi[0] - lo[0]).max(hi[1] - lo[1]) > 1000.0 { 0.001 } else { 1.0 }
            }
        }
    }

    pub fn bounds(&self) -> Option<([f64; 2], [f64; 2])> {
        let mut points = self.lines.iter().flatten().chain(self.texts.iter().map(|t| &t.0));
        let first = *points.next()?;
        Some(points.fold((first, first), |(lo, hi), p| {
            ([lo[0].min(p[0]), lo[1].min(p[1])], [hi[0].max(p[0]), hi[1].max(p[1])])
        }))
    }
}

/// Liest eine DXF- oder DWG-Datei (am Anfang erkannt: DWG beginnt mit „AC10“).
pub fn read_vectors(bytes: &[u8]) -> Result<CadVectors, IoError> {
    let doc = if bytes.starts_with(b"AC10") {
        DwgReader::from_stream(Cursor::new(bytes.to_vec())).read().map_err(|e| IoError::Cad(e.to_string()))?
    } else {
        DxfReader::from_reader(Cursor::new(bytes.to_vec()))
            .and_then(|r| r.read())
            .map_err(|e| IoError::Cad(e.to_string()))?
    };
    let mut out = CadVectors { units: doc.header.insertion_units, ..Default::default() };
    for e in doc.entities() {
        match e {
            EntityType::Line(l) => out.lines.push(vec![[l.start.x, l.start.y], [l.end.x, l.end.y]]),
            EntityType::LwPolyline(p) => {
                let mut pts: Vec<[f64; 2]> = p.vertices.iter().map(|v| [v.location.x, v.location.y]).collect();
                if p.is_closed
                    && let Some(first) = pts.first().copied()
                {
                    pts.push(first);
                }
                if pts.len() >= 2 {
                    out.lines.push(pts);
                }
            }
            EntityType::Circle(c) => out.lines.push(arc_points([c.center.x, c.center.y], c.radius, 0.0, 360.0)),
            EntityType::Arc(a) => {
                let (s, mut e) = (a.start_angle.to_degrees(), a.end_angle.to_degrees());
                if e < s {
                    e += 360.0;
                }
                out.lines.push(arc_points([a.center.x, a.center.y], a.radius, s, e));
            }
            EntityType::Text(t) => {
                out.texts.push(([t.insertion_point.x, t.insertion_point.y], t.value.clone(), t.height))
            }
            _ => {}
        }
    }
    Ok(out)
}

/// Bogen als Linienzug, Winkel in Grad.
fn arc_points(center: [f64; 2], r: f64, from: f64, to: f64) -> Vec<[f64; 2]> {
    let n = (((to - from).abs() / 10.0).ceil() as usize).max(2);
    (0..=n)
        .map(|i| {
            let a = (from + (to - from) * i as f64 / n as f64).to_radians();
            [center[0] + r * a.cos(), center[1] + r * a.sin()]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use lot_plan::{Area, Line, Pen, Text as PlanText};

    use super::*;

    fn objects() -> Vec<PlanObject> {
        vec![
            PlanObject::Line(Line {
                points: vec![PaperPoint::new(0.0, 0.0), PaperPoint::new(50.0, 0.0), PaperPoint::new(50.0, 30.0)],
                closed: false,
                pen: Pen::mm(0.5),
                style: LineStyle::Solid,
                color: Color::BLACK,
            }),
            PlanObject::Line(Line {
                points: vec![PaperPoint::new(0.0, 10.0), PaperPoint::new(40.0, 10.0)],
                closed: false,
                pen: Pen::mm(0.18),
                style: LineStyle::Dashed(vec![2.0, 1.0]),
                color: Color::BLACK,
            }),
            PlanObject::Area(Area {
                rings: vec![vec![PaperPoint::new(0.0, 0.0), PaperPoint::new(10.0, 0.0), PaperPoint::new(10.0, 5.0)]],
                fill: Fill::Solid(Color::rgb8(226, 35, 26)),
            }),
            PlanObject::Area(Area {
                rings: vec![vec![PaperPoint::new(20.0, 0.0), PaperPoint::new(30.0, 0.0), PaperPoint::new(30.0, 5.0)]],
                fill: Fill::Hatch { angle_deg: 45.0, spacing_mm: 1.0, pen: Pen::mm(0.13) },
            }),
            PlanObject::Text(PlanText {
                position: PaperPoint::new(5.0, 20.0),
                text: "Wohnen".into(),
                height_mm: 2.5,
                anchor: Anchor::Center,
                rotation: 0.0,
            }),
        ]
    }

    #[test]
    fn plan_objects_become_layered_dxf_in_metres_and_read_back() {
        // 1:100: 50 mm auf dem Blatt sind 5 m.
        let doc = to_cad(&[("A_Waende", objects())], 0.1, CadUnits::Meters).unwrap();
        assert_eq!(doc.header.insertion_units, 6);
        assert!(doc.layers.iter().any(|l| l.name == "A_Waende"));
        let dashed = doc.line_types.iter().find(|l| l.name == "LOT_STRICH_1").unwrap();
        assert!((dashed.pattern_length - 0.3).abs() < 1e-12);
        let bytes = dxf_bytes(&doc).unwrap();
        let text = String::from_utf8_lossy(&bytes);
        assert!(text.contains("A_Waende") && text.contains("Wohnen"));
        let v = read_vectors(&bytes).unwrap();
        assert_eq!(v.units, 6);
        assert_eq!(v.lines.len(), 2);
        let first = &v.lines[0];
        assert!((first[1][0] - 5.0).abs() < 1e-9 && (first[2][1] - 3.0).abs() < 1e-9, "{first:?}");
        assert_eq!(v.texts.len(), 1);
        assert!((v.texts[0].2 - 0.25).abs() < 1e-9);
        assert!((v.metres_per_unit() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn dwg_roundtrip_keeps_the_lines() {
        let doc = to_cad(&[("A_Waende", objects())], 0.1, CadUnits::Meters).unwrap();
        let bytes = dwg_bytes(&doc).unwrap();
        assert!(bytes.starts_with(b"AC10"));
        let v = read_vectors(&bytes).unwrap();
        assert_eq!(v.lines.len(), 2, "{v:?}");
    }

    #[test]
    fn line_weights_snap_to_standard_values() {
        assert_eq!(line_weight(0.5), LineWeight::Value(50));
        assert_eq!(line_weight(0.13), LineWeight::Value(13));
        assert_eq!(line_weight(0.7), LineWeight::Value(70));
    }
}
