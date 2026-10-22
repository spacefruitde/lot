//! PNG-Ausgabe eines Planblatts zum schnellen Ansehen, etwa auf dem iPhone.
//!
//! Gerastert wird aus denselben Planobjekten wie PDF und SVG. Linien behalten
//! ihre Breite in mm, werden aber nie dünner als ein Pixel.

use lot_plan::{Fill as PlanFill, Gray, Line, LineStyle, PaperPoint, PlanObject, Sheet, hatch_segments};
use tiny_skia::{FillRule, LineCap, LineJoin, Paint, PathBuilder, Pixmap, Stroke, StrokeDash, Transform};

use crate::IoError;
use crate::plan_font::{OutlineSink, plan_font};

/// Rastert ein Blatt mit `dpi` Pixeln je Zoll und gibt die PNG-Datei zurück.
pub fn to_png(sheet: &Sheet, dpi: f64) -> Result<Vec<u8>, IoError> {
    let px_per_mm = dpi / 25.4;
    let width = (sheet.width_mm * px_per_mm).round() as u32;
    let height = (sheet.height_mm * px_per_mm).round() as u32;
    let mut pixmap = Pixmap::new(width, height).ok_or_else(|| IoError::Png("Ungültige Bildgröße".into()))?;
    pixmap.fill(tiny_skia::Color::WHITE);
    let h = sheet.height_mm;
    let px = move |p: &PaperPoint| ((p.x * px_per_mm) as f32, ((h - p.y) * px_per_mm) as f32);

    for object in &sheet.objects {
        match object {
            PlanObject::Line(line) => stroke_line(&mut pixmap, line, px_per_mm, &px),
            PlanObject::Area(area) => match &area.fill {
                PlanFill::Solid(gray) => {
                    let mut builder = PathBuilder::new();
                    for ring in &area.rings {
                        add_points(&mut builder, ring, true, &px);
                    }
                    if let Some(path) = builder.finish() {
                        pixmap.fill_path(&path, &paint(*gray), FillRule::EvenOdd, Transform::identity(), None);
                    }
                }
                PlanFill::Hatch { angle_deg, spacing_mm, pen } => {
                    let mut builder = PathBuilder::new();
                    for [a, b] in hatch_segments(&area.rings, *angle_deg, *spacing_mm) {
                        add_points(&mut builder, &[a, b], false, &px);
                    }
                    if let Some(path) = builder.finish() {
                        let stroke = stroke(pen.width_mm, px_per_mm, None);
                        pixmap.stroke_path(&path, &paint(Gray::BLACK), &stroke, Transform::identity(), None);
                    }
                }
            },
            PlanObject::Text(text) => {
                let mut sink = SkiaSink { builder: PathBuilder::new(), px_per_mm, height: h };
                plan_font().outline(text, &mut sink);
                if let Some(path) = sink.builder.finish() {
                    pixmap.fill_path(&path, &paint(Gray::BLACK), FillRule::Winding, Transform::identity(), None);
                }
            }
        }
    }
    pixmap.encode_png().map_err(|e| IoError::Png(e.to_string()))
}

fn stroke_line(pixmap: &mut Pixmap, line: &Line, px_per_mm: f64, px: &impl Fn(&PaperPoint) -> (f32, f32)) {
    let mut builder = PathBuilder::new();
    add_points(&mut builder, &line.points, line.closed, px);
    let Some(path) = builder.finish() else { return };
    let dash = match &line.style {
        LineStyle::Solid => None,
        LineStyle::Dashed(pattern) => StrokeDash::new(pattern.iter().map(|v| (v * px_per_mm) as f32).collect(), 0.0),
    };
    let stroke = stroke(line.pen.width_mm, px_per_mm, dash);
    pixmap.stroke_path(&path, &paint(line.color), &stroke, Transform::identity(), None);
}

fn stroke(width_mm: f64, px_per_mm: f64, dash: Option<StrokeDash>) -> Stroke {
    Stroke {
        width: ((width_mm * px_per_mm) as f32).max(1.0),
        miter_limit: 10.0,
        line_cap: LineCap::Butt,
        line_join: LineJoin::Miter,
        dash,
    }
}

fn paint(gray: Gray) -> Paint<'static> {
    let v = (gray.0.clamp(0.0, 1.0) * 255.0).round() as u8;
    let mut paint = Paint::default();
    paint.set_color_rgba8(v, v, v, 255);
    paint.anti_alias = true;
    paint
}

fn add_points(builder: &mut PathBuilder, points: &[PaperPoint], closed: bool, px: &impl Fn(&PaperPoint) -> (f32, f32)) {
    for (i, p) in points.iter().enumerate() {
        let (x, y) = px(p);
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

struct SkiaSink {
    builder: PathBuilder,
    px_per_mm: f64,
    height: f64,
}

impl SkiaSink {
    fn p(&self, x: f64, y: f64) -> (f32, f32) {
        ((x * self.px_per_mm) as f32, ((self.height - y) * self.px_per_mm) as f32)
    }
}

impl OutlineSink for SkiaSink {
    fn move_to(&mut self, x: f64, y: f64) {
        let (x, y) = self.p(x, y);
        self.builder.move_to(x, y);
    }

    fn line_to(&mut self, x: f64, y: f64) {
        let (x, y) = self.p(x, y);
        self.builder.line_to(x, y);
    }

    fn quad_to(&mut self, x1: f64, y1: f64, x: f64, y: f64) {
        let (x1, y1) = self.p(x1, y1);
        let (x, y) = self.p(x, y);
        self.builder.quad_to(x1, y1, x, y);
    }

    fn cubic_to(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, x: f64, y: f64) {
        let (x1, y1) = self.p(x1, y1);
        let (x2, y2) = self.p(x2, y2);
        let (x, y) = self.p(x, y);
        self.builder.cubic_to(x1, y1, x2, y2, x, y);
    }

    fn close(&mut self) {
        self.builder.close();
    }
}
