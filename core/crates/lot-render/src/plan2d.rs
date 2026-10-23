//! Planansicht auf dem Bildschirm: Planobjekte werden mit lyon in Dreiecke
//! zerlegt (ADR 0007). Koordinaten sind Blattmillimeter wie im PDF; die
//! Kamera bildet Millimeter auf Pixel ab. So sieht der Plan auf dem
//! Bildschirm aus wie gedruckt, Linienbreiten wachsen mit dem Zoom.

use bytemuck::{Pod, Zeroable};
use lot_io::plan_font::{OutlineSink, plan_font};
use lot_plan::{Fill, Gray, Line, LineStyle, PaperPoint, PlanObject, hatch_segments};
use lyon_tessellation::path::Path;
use lyon_tessellation::path::builder::NoAttributes;
use lyon_tessellation::path::path::BuilderImpl;
use lyon_tessellation::{
    BuffersBuilder, FillOptions, FillRule, FillTessellator, FillVertex, LineCap, LineJoin, StrokeOptions,
    StrokeTessellator, StrokeVertex, VertexBuffers,
};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct PlanVertex {
    /// Blattmillimeter
    pub position: [f32; 2],
    /// Farbe, linear, nicht vormultipliziert
    pub color: [f32; 4],
}

#[derive(Debug, Default)]
pub struct PlanMesh {
    pub vertices: Vec<PlanVertex>,
    pub indices: Vec<u32>,
}

/// Kamera der Planansicht: Mitte in Blattmillimetern, Zoom in Pixeln je Millimeter.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlanCamera {
    pub center: [f64; 2],
    pub px_per_mm: f64,
}

impl Default for PlanCamera {
    fn default() -> Self {
        Self { center: [0.0, 0.0], px_per_mm: 4.0 }
    }
}

/// Was über dem Plan gezeichnet wird. Größen in Pixeln, Lagen in Blattmillimetern.
#[derive(Debug, Clone, PartialEq)]
pub enum OverlayItem {
    Polygon { points: Vec<[f64; 2]>, fill: [f32; 4], stroke: [f32; 4], width_px: f32 },
    Square { center: [f64; 2], size_px: f32, color: [f32; 4] },
    Ring { center: [f64; 2], radius_px: f32, width_px: f32, color: [f32; 4] },
    Text { at: [f64; 2], text: String, height_px: f32, color: [f32; 4] },
}

struct Tess {
    mesh: VertexBuffers<PlanVertex, u32>,
    fill: FillTessellator,
    stroke: StrokeTessellator,
}

impl Tess {
    fn new() -> Self {
        Self { mesh: VertexBuffers::new(), fill: FillTessellator::new(), stroke: StrokeTessellator::new() }
    }

    fn fill(&mut self, path: &Path, color: [f32; 4], rule: FillRule) {
        let options = FillOptions::default().with_fill_rule(rule).with_tolerance(0.01);
        let _ = self.fill.tessellate_path(
            path,
            &options,
            &mut BuffersBuilder::new(&mut self.mesh, |v: FillVertex| PlanVertex {
                position: v.position().to_array(),
                color,
            }),
        );
    }

    fn stroke(&mut self, path: &Path, width: f32, color: [f32; 4]) {
        let options = StrokeOptions::default()
            .with_line_width(width)
            .with_line_join(LineJoin::MiterClip)
            .with_line_cap(LineCap::Butt)
            .with_tolerance(0.01);
        let _ = self.stroke.tessellate_path(
            path,
            &options,
            &mut BuffersBuilder::new(&mut self.mesh, |v: StrokeVertex| PlanVertex {
                position: v.position().to_array(),
                color,
            }),
        );
    }

    fn finish(self) -> PlanMesh {
        PlanMesh { vertices: self.mesh.vertices, indices: self.mesh.indices }
    }
}

/// Zerlegt Planobjekte in Dreiecke. Linien werden nie dünner als `min_width_mm`
/// (ein Pixel beim aktuellen Zoom).
pub fn tessellate(objects: &[PlanObject], min_width_mm: f32) -> PlanMesh {
    let mut t = Tess::new();
    for object in objects {
        match object {
            PlanObject::Line(line) => line_into(&mut t, line, min_width_mm),
            PlanObject::Area(area) => match &area.fill {
                Fill::Solid(gray) => {
                    let mut b = Path::builder();
                    for ring in &area.rings {
                        add_ring(&mut b, ring.iter().map(|p| [p.x, p.y]), true);
                    }
                    t.fill(&b.build(), color(*gray), FillRule::EvenOdd);
                }
                Fill::Hatch { angle_deg, spacing_mm, pen } => {
                    let mut b = Path::builder();
                    for [a, c] in hatch_segments(&area.rings, *angle_deg, *spacing_mm) {
                        add_ring(&mut b, [[a.x, a.y], [c.x, c.y]], false);
                    }
                    t.stroke(&b.build(), (pen.width_mm as f32).max(min_width_mm), color(Gray::BLACK));
                }
            },
            PlanObject::Text(text) => {
                let mut sink = LyonSink(Path::builder(), false);
                plan_font().outline(text, &mut sink);
                sink.finish_open();
                t.fill(&sink.0.build(), color(Gray::BLACK), FillRule::NonZero);
            }
        }
    }
    t.finish()
}

/// Zerlegt die Hinweise über dem Plan. `px_per_mm` rechnet Pixel in Millimeter um.
pub fn tessellate_overlay(items: &[OverlayItem], px_per_mm: f64) -> PlanMesh {
    let mm = |px: f32| (px as f64 / px_per_mm) as f32;
    let mut t = Tess::new();
    for item in items {
        match item {
            OverlayItem::Polygon { points, fill, stroke, width_px } => {
                let mut b = Path::builder();
                add_ring(&mut b, points.iter().copied(), true);
                let path = b.build();
                if fill[3] > 0.0 {
                    t.fill(&path, *fill, FillRule::NonZero);
                }
                t.stroke(&path, mm(*width_px), *stroke);
            }
            OverlayItem::Square { center, size_px, color } => {
                let h = mm(*size_px) as f64 / 2.0;
                let mut b = Path::builder();
                add_ring(
                    &mut b,
                    [
                        [center[0] - h, center[1] - h],
                        [center[0] + h, center[1] - h],
                        [center[0] + h, center[1] + h],
                        [center[0] - h, center[1] + h],
                    ],
                    true,
                );
                t.fill(&b.build(), *color, FillRule::NonZero);
            }
            OverlayItem::Ring { center, radius_px, width_px, color } => {
                let r = mm(*radius_px) as f64;
                let mut b = Path::builder();
                add_ring(
                    &mut b,
                    (0..32).map(|i| {
                        let a = i as f64 / 32.0 * std::f64::consts::TAU;
                        [center[0] + r * a.cos(), center[1] + r * a.sin()]
                    }),
                    true,
                );
                t.stroke(&b.build(), mm(*width_px), *color);
            }
            OverlayItem::Text { at, text, height_px, color } => {
                let mut sink = LyonSink(Path::builder(), false);
                let text = lot_plan::Text {
                    position: PaperPoint::new(at[0], at[1]),
                    text: text.clone(),
                    height_mm: mm(*height_px) as f64,
                    anchor: lot_plan::Anchor::Center,
                };
                plan_font().outline(&text, &mut sink);
                sink.finish_open();
                t.fill(&sink.0.build(), *color, FillRule::NonZero);
            }
        }
    }
    t.finish()
}

fn line_into(t: &mut Tess, line: &Line, min_width_mm: f32) {
    let width = (line.pen.width_mm as f32).max(min_width_mm);
    let mut b = Path::builder();
    match &line.style {
        LineStyle::Solid => add_ring(&mut b, line.points.iter().map(|p| [p.x, p.y]), line.closed),
        LineStyle::Dashed(pattern) => {
            let mut points: Vec<PaperPoint> = line.points.clone();
            if line.closed
                && let Some(first) = points.first().copied()
            {
                points.push(first);
            }
            for dash in dashes(&points, pattern) {
                add_ring(&mut b, dash.iter().map(|p| [p.x, p.y]), false);
            }
        }
    }
    t.stroke(&b.build(), width, color(line.color));
}

/// Teilt einen Linienzug in Striche nach Muster (Strich, Lücke, Strich, ...).
fn dashes(points: &[PaperPoint], pattern: &[f64]) -> Vec<Vec<PaperPoint>> {
    let total: f64 = pattern.iter().sum();
    if pattern.is_empty() || total <= 0.0 {
        return vec![points.to_vec()];
    }
    let mut out = Vec::new();
    let mut current: Vec<PaperPoint> = Vec::new();
    let (mut index, mut left, mut on) = (0usize, pattern[0], true);
    for w in points.windows(2) {
        let (a, b) = (w[0], w[1]);
        let len = (b.x - a.x).hypot(b.y - a.y);
        let mut pos = 0.0;
        while pos < len {
            let step = left.min(len - pos);
            let p0 = PaperPoint::new(a.x + (b.x - a.x) * pos / len, a.y + (b.y - a.y) * pos / len);
            let p1 = PaperPoint::new(a.x + (b.x - a.x) * (pos + step) / len, a.y + (b.y - a.y) * (pos + step) / len);
            if on {
                if current.is_empty() {
                    current.push(p0);
                }
                current.push(p1);
            }
            pos += step;
            left -= step;
            if left <= 1e-9 {
                if on && !current.is_empty() {
                    out.push(std::mem::take(&mut current));
                }
                index = (index + 1) % pattern.len();
                left = pattern[index];
                on = index % 2 == 0;
            }
        }
    }
    if current.len() >= 2 {
        out.push(current);
    }
    out
}

fn add_ring(b: &mut NoAttributes<BuilderImpl>, points: impl IntoIterator<Item = [f64; 2]>, close: bool) {
    let mut it = points.into_iter();
    let Some(first) = it.next() else { return };
    b.begin(point(first));
    for p in it {
        b.line_to(point(p));
    }
    b.end(close);
}

fn point(p: [f64; 2]) -> lyon_tessellation::math::Point {
    lyon_tessellation::math::point(p[0] as f32, p[1] as f32)
}

fn color(gray: Gray) -> [f32; 4] {
    let v = crate::srgb_component_to_linear(gray.0.clamp(0.0, 1.0) as f32);
    [v, v, v, 1.0]
}

/// Umrisse von Zeichen als lyon-Pfad.
struct LyonSink(NoAttributes<BuilderImpl>, bool);

impl LyonSink {
    fn finish_open(&mut self) {
        if self.1 {
            self.0.end(true);
            self.1 = false;
        }
    }
}

impl OutlineSink for LyonSink {
    fn move_to(&mut self, x: f64, y: f64) {
        self.finish_open();
        self.0.begin(point([x, y]));
        self.1 = true;
    }
    fn line_to(&mut self, x: f64, y: f64) {
        self.0.line_to(point([x, y]));
    }
    fn quad_to(&mut self, x1: f64, y1: f64, x: f64, y: f64) {
        self.0.quadratic_bezier_to(point([x1, y1]), point([x, y]));
    }
    fn cubic_to(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, x: f64, y: f64) {
        self.0.cubic_bezier_to(point([x1, y1]), point([x2, y2]), point([x, y]));
    }
    fn close(&mut self) {
        if self.1 {
            self.0.end(true);
            self.1 = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lot_plan::{Pen, Sheet};

    #[test]
    fn dash_pattern_splits_line() {
        let points = vec![PaperPoint::new(0.0, 0.0), PaperPoint::new(10.0, 0.0)];
        let d = dashes(&points, &[2.0, 1.0]);
        assert_eq!(d.len(), 4);
        assert!((d[0][1].x - 2.0).abs() < 1e-9);
        assert!((d[1][0].x - 3.0).abs() < 1e-9);
    }

    #[test]
    fn sheet_tessellates() {
        let mut s = Sheet::a4_landscape("t");
        s.rect(10.0, 10.0, 50.0, 30.0, Pen::mm(0.5));
        s.text(10.0, 50.0, "Wand", 3.5, lot_plan::Anchor::Left);
        let mesh = tessellate(&s.objects, 0.1);
        assert!(mesh.vertices.len() > 8);
        assert_eq!(mesh.indices.len() % 3, 0);
    }
}
