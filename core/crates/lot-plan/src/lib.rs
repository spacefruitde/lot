//! Planobjekte: was auf einem Planblatt steht, unabhängig vom Ausgabeformat.
//!
//! Aus einer Ansicht entstehen Planobjekte (Linie, Fläche, Text). Dieselben
//! Planobjekte gehen auf den Bildschirm, ins PDF, ins SVG und später ins DXF
//! (PLAN.md, Abschnitt 4, Grundentscheidung 9). Koordinaten auf dem Blatt sind
//! Millimeter, Ursprung links unten, y nach oben.

pub mod annotations;
pub mod area_table;
pub mod clip;
pub mod dimensions;
pub mod elevators;
pub mod findings;
pub mod floor_plan;
pub mod layout;
pub mod m0;
pub mod presentation;
pub mod reference;
pub mod section;
pub mod sheets;
pub mod site_plan;
pub mod stairs;
pub mod standard;
pub mod table;

/// Punkt auf dem Blatt in Millimetern.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PaperPoint {
    pub x: f64,
    pub y: f64,
}

impl PaperPoint {
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

/// Linienbreiten nach DIN EN ISO 128-20:2002, Abschnitt 4.2 (Reihe mit Faktor √2),
/// wie sie auch DIN 1356-1:1995-02 für Bauzeichnungen verwendet. Werte in mm.
pub const PEN_WIDTHS_MM: [f64; 7] = [0.13, 0.18, 0.25, 0.35, 0.5, 0.7, 1.0];

/// Ein Stift: Linienbreite in Millimetern auf dem Papier, unabhängig vom Maßstab.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pen {
    pub width_mm: f64,
}

impl Pen {
    pub const fn mm(width_mm: f64) -> Self {
        Self { width_mm }
    }
}

/// Linienart. Strichlängen in Millimetern auf dem Papier.
#[derive(Debug, Clone, PartialEq)]
pub enum LineStyle {
    Solid,
    /// Abwechselnd Strich und Lücke, beginnend mit einem Strich.
    Dashed(Vec<f64>),
}

/// Farbe auf dem Plan, sRGB-Anteile von 0 bis 1. Fast alles ist schwarz oder
/// grau; Farben braucht es für Bestand, Abbruch und Neubau (2.05).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

impl Color {
    pub const BLACK: Color = Color { r: 0.0, g: 0.0, b: 0.0 };

    /// Grauwert von 0 (schwarz) bis 1 (weiß).
    pub const fn gray(v: f64) -> Color {
        Color { r: v, g: v, b: v }
    }

    pub fn rgb8(r: u8, g: u8, b: u8) -> Color {
        Color { r: r as f64 / 255.0, g: g as f64 / 255.0, b: b as f64 / 255.0 }
    }

    /// Anteile als Bytes 0 bis 255.
    pub fn bytes(self) -> [u8; 3] {
        [self.r, self.g, self.b].map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    pub points: Vec<PaperPoint>,
    pub closed: bool,
    pub pen: Pen,
    pub style: LineStyle,
    pub color: Color,
}

/// Füllung einer Fläche.
#[derive(Debug, Clone, PartialEq)]
pub enum Fill {
    Solid(Color),
    /// Parallele Schraffurlinien. Winkel in Grad gegen die x-Achse, Abstand in mm.
    Hatch {
        angle_deg: f64,
        spacing_mm: f64,
        pen: Pen,
    },
}

/// Fläche aus einem oder mehreren Ringen. Innere Ringe sind Löcher (Regel gerade/ungerade).
#[derive(Debug, Clone, PartialEq)]
pub struct Area {
    pub rings: Vec<Vec<PaperPoint>>,
    pub fill: Fill,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Anchor {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Text {
    pub position: PaperPoint,
    pub text: String,
    /// Schrifthöhe (Versalhöhe) in mm, wie bei Normschrift nach DIN EN ISO 3098.
    pub height_mm: f64,
    pub anchor: Anchor,
    /// Drehung um `position` in Grad, gegen den Uhrzeigersinn. Lesbar bleibt
    /// Text nach DIN 1356-1 von unten und von rechts, also zwischen −90° und 90°.
    pub rotation: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlanObject {
    Line(Line),
    Area(Area),
    Text(Text),
}

impl PlanObject {
    /// Alle Punkte, für Ausdehnung und Verschieben.
    fn points_mut(&mut self) -> Vec<&mut PaperPoint> {
        match self {
            PlanObject::Line(l) => l.points.iter_mut().collect(),
            PlanObject::Area(a) => a.rings.iter_mut().flatten().collect(),
            PlanObject::Text(t) => vec![&mut t.position],
        }
    }

    pub fn translate(&mut self, dx: f64, dy: f64) {
        for p in self.points_mut() {
            p.x += dx;
            p.y += dy;
        }
    }
}

/// Ausdehnung von Planobjekten auf dem Blatt (min, max); Texte zählen mit
/// ihrem Bezugspunkt und ihrer Höhe.
pub fn bounds(objects: &[PlanObject]) -> Option<(PaperPoint, PaperPoint)> {
    let mut lo = PaperPoint::new(f64::INFINITY, f64::INFINITY);
    let mut hi = PaperPoint::new(f64::NEG_INFINITY, f64::NEG_INFINITY);
    let mut add = |p: PaperPoint, r: f64| {
        lo = PaperPoint::new(lo.x.min(p.x - r), lo.y.min(p.y - r));
        hi = PaperPoint::new(hi.x.max(p.x + r), hi.y.max(p.y + r));
    };
    for o in objects {
        match o {
            PlanObject::Line(l) => l.points.iter().for_each(|p| add(*p, 0.0)),
            PlanObject::Area(a) => a.rings.iter().flatten().for_each(|p| add(*p, 0.0)),
            PlanObject::Text(t) => add(t.position, t.height_mm),
        }
    }
    lo.x.is_finite().then_some((lo, hi))
}

/// Ein Planblatt.
#[derive(Debug, Clone, PartialEq)]
pub struct Sheet {
    pub title: String,
    pub width_mm: f64,
    pub height_mm: f64,
    /// In Zeichenreihenfolge: Spätere Objekte liegen oben.
    pub objects: Vec<PlanObject>,
}

impl Sheet {
    /// A4 quer.
    pub fn a4_landscape(title: impl Into<String>) -> Self {
        Self { title: title.into(), width_mm: 297.0, height_mm: 210.0, objects: Vec::new() }
    }

    pub fn line(&mut self, points: Vec<PaperPoint>, pen: Pen) {
        self.objects.push(PlanObject::Line(Line {
            points,
            closed: false,
            pen,
            style: LineStyle::Solid,
            color: Color::BLACK,
        }));
    }

    pub fn polygon(&mut self, points: Vec<PaperPoint>, pen: Pen) {
        self.objects.push(PlanObject::Line(Line {
            points,
            closed: true,
            pen,
            style: LineStyle::Solid,
            color: Color::BLACK,
        }));
    }

    pub fn rect(&mut self, x: f64, y: f64, w: f64, h: f64, pen: Pen) {
        self.polygon(rect_points(x, y, w, h), pen);
    }

    pub fn text(&mut self, x: f64, y: f64, text: impl Into<String>, height_mm: f64, anchor: Anchor) {
        self.objects.push(PlanObject::Text(Text {
            position: PaperPoint::new(x, y),
            text: text.into(),
            height_mm,
            anchor,
            rotation: 0.0,
        }));
    }
}

pub fn rect_points(x: f64, y: f64, w: f64, h: f64) -> Vec<PaperPoint> {
    vec![PaperPoint::new(x, y), PaperPoint::new(x + w, y), PaperPoint::new(x + w, y + h), PaperPoint::new(x, y + h)]
}

/// Abbildung vom Modell (Meter) aufs Blatt (Millimeter) in einem Maßstab 1:n.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Viewport {
    /// Nenner des Maßstabs, also 100 für 1:100.
    pub scale: f64,
    /// Lage des Modellursprungs auf dem Blatt.
    pub origin: PaperPoint,
}

impl Viewport {
    pub fn to_paper(&self, x_m: f64, y_m: f64) -> PaperPoint {
        let f = 1000.0 / self.scale;
        PaperPoint::new(self.origin.x + x_m * f, self.origin.y + y_m * f)
    }
}

/// Schraffurlinien einer Fläche als Strecken auf dem Blatt.
///
/// Die Linien werden hier geometrisch erzeugt und nicht dem Ausgabeformat
/// überlassen. So sehen Bildschirm, PDF, SVG und DXF gleich aus, und jede
/// Schraffurlinie hat die exakte Stiftbreite. Die Linien liegen auf einem
/// festen Raster durch den Blattursprung, damit benachbarte Flächen mit
/// gleicher Schraffur nahtlos ineinander übergehen.
pub fn hatch_segments(rings: &[Vec<PaperPoint>], angle_deg: f64, spacing_mm: f64) -> Vec<[PaperPoint; 2]> {
    if spacing_mm <= 0.0 {
        return Vec::new();
    }
    let (sin, cos) = angle_deg.to_radians().sin_cos();
    // In ein gedrehtes System, in dem die Schraffurlinien waagerecht liegen.
    let rotate = |p: &PaperPoint| (p.x * cos + p.y * sin, -p.x * sin + p.y * cos);
    let back = |u: f64, v: f64| PaperPoint::new(u * cos - v * sin, u * sin + v * cos);

    let edges: Vec<((f64, f64), (f64, f64))> = rings
        .iter()
        .flat_map(|ring| {
            let pts: Vec<(f64, f64)> = ring.iter().map(rotate).collect();
            (0..pts.len()).map(move |i| (pts[i], pts[(i + 1) % pts.len()]))
        })
        .collect();
    let (v_min, v_max) = edges
        .iter()
        .flat_map(|(a, b)| [a.1, b.1])
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), v| (lo.min(v), hi.max(v)));
    if !v_min.is_finite() {
        return Vec::new();
    }

    let mut segments = Vec::new();
    let mut k = (v_min / spacing_mm).ceil();
    while k * spacing_mm <= v_max {
        let v = k * spacing_mm;
        let mut hits: Vec<f64> = edges
            .iter()
            .filter_map(|&((u0, v0), (u1, v1))| {
                // Halboffenes Intervall, damit Eckpunkte genau einmal zählen.
                if (v0 <= v && v < v1) || (v1 <= v && v < v0) {
                    Some(u0 + (v - v0) / (v1 - v0) * (u1 - u0))
                } else {
                    None
                }
            })
            .collect();
        hits.sort_by(f64::total_cmp);
        for pair in hits.as_chunks::<2>().0 {
            segments.push([back(pair[0], v), back(pair[1], v)]);
        }
        k += 1.0;
    }
    segments
}

/// Zahl im deutschen Format mit fester Anzahl Nachkommastellen, etwa „5,00“.
pub fn format_de(value: f64, decimals: usize) -> String {
    // Kaufmännisch runden: 24,345 wird 24,35, auch wenn die Zahl binär knapp darunter liegt.
    let f = 10f64.powi(decimals as i32);
    let rounded = value.signum() * ((value.abs() * f) * (1.0 + 1e-12)).round() / f;
    format!("{rounded:.decimals$}").replace('.', ",")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn five_metres_at_1_100_are_50_mm() {
        let vp = Viewport { scale: 100.0, origin: PaperPoint::new(10.0, 20.0) };
        let a = vp.to_paper(0.0, 0.0);
        let b = vp.to_paper(5.0, 0.0);
        assert_eq!(b.x - a.x, 50.0);
    }

    #[test]
    fn horizontal_hatch_in_square() {
        let ring = rect_points(0.0, 0.0, 10.0, 10.0);
        let segs = hatch_segments(&[ring], 0.0, 2.0);
        // Linien bei y = 0, 2, 4, 6, 8; y = 10 liegt auf der Oberkante und zählt nicht.
        assert_eq!(segs.len(), 5);
        for s in &segs {
            assert!((s[0].x - 0.0).abs() < 1e-9 && (s[1].x - 10.0).abs() < 1e-9);
        }
    }

    #[test]
    fn hatch_skips_holes() {
        let outer = rect_points(0.0, 0.0, 10.0, 10.0);
        let hole = rect_points(4.0, 0.0, 2.0, 10.0);
        let segs = hatch_segments(&[outer, hole], 0.0, 2.0);
        assert_eq!(segs.len(), 10);
    }

    #[test]
    fn diagonal_hatch_stays_inside() {
        let ring = rect_points(0.0, 0.0, 30.0, 20.0);
        for s in hatch_segments(&[ring], 45.0, 2.0) {
            for p in s {
                assert!(p.x > -1e-9 && p.x < 30.0 + 1e-9 && p.y > -1e-9 && p.y < 20.0 + 1e-9);
            }
        }
    }

    #[test]
    fn german_number_format() {
        assert_eq!(format_de(5.0, 2), "5,00");
        assert_eq!(format_de(0.25, 2), "0,25");
    }
}
