//! PDF-Ausgabe eines Planblatts mit krilla.
//!
//! Linienbreiten werden exakt übertragen: 1 mm auf dem Blatt sind 72/25,4 pt.
//! Schraffuren sind einzelne Linien (siehe `lot_plan::hatch_segments`), Texte
//! stehen in der eingebetteten Planschrift als Teilmenge.

use std::sync::Arc;

use krilla::color::rgb;
use krilla::geom::{PathBuilder, Point};
use krilla::metadata::Metadata;
use krilla::num::NormalizedF32;
use krilla::page::PageSettings;
use krilla::paint::{Fill, FillRule, LineCap, LineJoin, Stroke, StrokeDash};
use krilla::surface::Surface;
use krilla::text::{Font, TextDirection};
use krilla::{Document, SerializeSettings};
use lot_plan::{Color, Fill as PlanFill, Line, LineStyle, PaperPoint, PlanObject, Sheet, hatch_segments};

use crate::IoError;
use crate::plan_font::plan_font;

/// Punkte (1/72 Zoll) je Millimeter.
pub const PT_PER_MM: f64 = 72.0 / 25.4;

pub fn to_pdf(sheet: &Sheet) -> Result<Vec<u8>, IoError> {
    render(std::slice::from_ref(sheet), SerializeSettings::default())
}

/// Mehrere Blätter als Seiten eines PDF, etwa der Wandknoten-Katalog.
pub fn to_pdf_pages(sheets: &[Sheet]) -> Result<Vec<u8>, IoError> {
    render(sheets, SerializeSettings::default())
}

fn render(sheets: &[Sheet], settings: SerializeSettings) -> Result<Vec<u8>, IoError> {
    let font_data: Arc<Vec<u8>> = plan_font().data().clone();
    let font = Font::new(font_data.into(), 0).ok_or_else(|| IoError::Pdf("Planschrift nicht lesbar".into()))?;
    let title = sheets.first().map_or(String::new(), |s| s.title.clone());

    let mut document = Document::new_with(settings);
    document.set_metadata(Metadata::new().title(title).creator(concat!("Lot ", env!("CARGO_PKG_VERSION")).to_string()));
    for sheet in sheets {
        draw_page(&mut document, sheet, &font)?;
    }
    document.finish().map_err(|e| IoError::Pdf(format!("{e:?}")))
}

fn draw_page(document: &mut Document, sheet: &Sheet, font: &Font) -> Result<(), IoError> {
    let (w, h) = (sheet.width_mm * PT_PER_MM, sheet.height_mm * PT_PER_MM);
    let settings =
        PageSettings::from_wh(w as f32, h as f32).ok_or_else(|| IoError::Pdf("Ungültige Blattgröße".into()))?;
    let mut page = document.start_page_with(settings);
    let mut surface = page.surface();
    let height = sheet.height_mm;
    // krilla zählt y von oben, das Blatt von unten.
    let pt = |p: &PaperPoint| ((p.x * PT_PER_MM) as f32, ((height - p.y) * PT_PER_MM) as f32);

    for object in &sheet.objects {
        match object {
            PlanObject::Line(line) => stroke_line(&mut surface, line, &pt),
            PlanObject::Area(area) => match &area.fill {
                PlanFill::Solid(gray) => {
                    let mut builder = PathBuilder::new();
                    for ring in &area.rings {
                        add_points(&mut builder, ring, true, &pt);
                    }
                    if let Some(path) = builder.finish() {
                        surface.set_stroke(None);
                        surface.set_fill(Some(Fill {
                            paint: rgb_paint(*gray),
                            opacity: NormalizedF32::ONE,
                            rule: FillRule::EvenOdd,
                        }));
                        surface.draw_path(&path);
                    }
                }
                PlanFill::Hatch { angle_deg, spacing_mm, pen } => {
                    let mut builder = PathBuilder::new();
                    for [a, b] in hatch_segments(&area.rings, *angle_deg, *spacing_mm) {
                        add_points(&mut builder, &[a, b], false, &pt);
                    }
                    if let Some(path) = builder.finish() {
                        surface.set_fill(None);
                        surface.set_stroke(Some(stroke(pen.width_mm, Color::BLACK, None)));
                        surface.draw_path(&path);
                    }
                }
            },
            PlanObject::Text(text) => {
                let font_size = plan_font().size_for_cap_height(text.height_mm) * PT_PER_MM;
                let start = PaperPoint::new(plan_font().start_x(text), text.position.y);
                let (x, y) = pt(&start);
                let rotated = text.rotation != 0.0;
                if rotated {
                    // PDF zählt hier y nach unten, eine Drehung gegen den Uhrzeigersinn ist negativ.
                    let (ox, oy) = pt(&text.position);
                    surface.push_transform(&krilla::geom::Transform::from_rotate_at(-text.rotation as f32, ox, oy));
                }
                surface.set_stroke(None);
                surface.set_fill(Some(Fill::default()));
                surface.draw_text(
                    Point::from_xy(x, y),
                    font.clone(),
                    font_size as f32,
                    &text.text,
                    false,
                    TextDirection::Auto,
                );
                if rotated {
                    surface.pop();
                }
            }
        }
    }
    surface.finish();
    page.finish();
    Ok(())
}

fn stroke_line(surface: &mut Surface<'_>, line: &Line, pt: &impl Fn(&PaperPoint) -> (f32, f32)) {
    let mut builder = PathBuilder::new();
    add_points(&mut builder, &line.points, line.closed, pt);
    let Some(path) = builder.finish() else { return };
    let dash = match &line.style {
        LineStyle::Solid => None,
        LineStyle::Dashed(pattern) => {
            Some(StrokeDash { array: pattern.iter().map(|v| (v * PT_PER_MM) as f32).collect(), offset: 0.0 })
        }
    };
    surface.set_fill(None);
    surface.set_stroke(Some(stroke(line.pen.width_mm, line.color, dash)));
    surface.draw_path(&path);
}

fn stroke(width_mm: f64, color: Color, dash: Option<StrokeDash>) -> Stroke {
    Stroke {
        paint: rgb_paint(color),
        width: (width_mm * PT_PER_MM) as f32,
        miter_limit: 10.0,
        line_cap: LineCap::Butt,
        line_join: LineJoin::Miter,
        opacity: NormalizedF32::ONE,
        dash,
    }
}

fn add_points(builder: &mut PathBuilder, points: &[PaperPoint], closed: bool, pt: &impl Fn(&PaperPoint) -> (f32, f32)) {
    for (i, p) in points.iter().enumerate() {
        let (x, y) = pt(p);
        if i == 0 {
            builder.move_to(x, y);
        } else {
            builder.line_to(x, y);
        }
    }
    if closed && points.len() > 2 {
        builder.close();
    }
}

fn rgb_paint(color: Color) -> krilla::paint::Paint {
    let [r, g, b] = color.bytes();
    rgb::Color::new(r, g, b).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use lot_plan::Pen;

    #[test]
    fn pdf_has_exact_line_width() {
        let mut sheet = Sheet::a4_landscape("Test");
        sheet.line(vec![PaperPoint::new(10.0, 10.0), PaperPoint::new(60.0, 10.0)], Pen::mm(0.5));
        sheet.text(10.0, 20.0, "Wohnen 24,31 m²", 3.5, lot_plan::Anchor::Left);
        let settings = SerializeSettings { compress_content_streams: false, ..Default::default() };
        let pdf = render(std::slice::from_ref(&sheet), settings).unwrap();
        let text = String::from_utf8_lossy(&pdf);
        // A4 quer: 297 x 210 mm = 841,89 x 595,28 pt.
        assert!(text.contains("/MediaBox[0 0 841.8898 595.2756]"), "Blattgröße falsch");
        // 0,5 mm = 1,41732 pt.
        assert!(text.contains("1.4173229 w"), "Linienbreite falsch: {text}");
        // Die Schrift ist eingebettet.
        assert!(text.contains("/FontFile2"), "Schrift nicht eingebettet");
    }
}
