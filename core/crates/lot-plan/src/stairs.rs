//! Treppen und Geländer im Grundriss (Vorschlag nach DIN 1356-1:1995-02).
//!
//! Im Geschoss, in dem die Treppe beginnt, ist sie bis zur Schnittebene
//! sichtbar; eine schräge Linie quer über den Lauf zeigt, wo die Schnittebene
//! ihn trifft. Was darüber liegt, ist gestrichelt. Im Geschoss darüber liegt
//! die ganze Treppe unter der Schnittebene und ist durch das Treppenloch zu
//! sehen, soweit sie keine Treppe dieses Geschosses unterhalb der
//! Schnittebene verdeckt. Die Lauflinie beginnt mit einem Kreis an der ersten
//! Stufe und endet mit einem Pfeil am Austritt. Am Antritt stehen Zahl der
//! Steigungen und Steigungsverhältnis, etwa „16 Stg. 17,2/27,0“.

use lot_model::{ElementId, FlightLayout, Point2, Project, RailingPath, Stair, StairLayout};

use crate::clip::inside;
use crate::standard::ScaleStyle;
use crate::{Anchor, Area as PlanArea, Color, Fill, Line, LineStyle, PaperPoint, Pen, PlanObject, Text, Viewport};

/// Strich und Lücke für Treppenteile über der Schnittebene, in mm.
const ABOVE_DASH: [f64; 2] = [1.5, 1.0];

/// Wie ein Strich erscheint.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Look {
    Solid,
    Dashed,
}

/// Zeichnet Treppen und Geländer im Grundriss von `storey`.
pub fn draw_stairs(
    objects: &mut Vec<PlanObject>,
    project: &Project,
    storey: ElementId,
    cut_height: f64,
    style: &ScaleStyle,
    viewport: Viewport,
) {
    let storeys = project.storeys();
    let below = storeys.iter().position(|(id, _)| *id == storey).and_then(|i| i.checked_sub(1)).map(|i| storeys[i].0);
    // Was die eigenen Treppen unterhalb der Schnittebene verdecken.
    let mut cover: Vec<Vec<PaperPoint>> = Vec::new();
    for (_, stair) in project.stairs_on(storey) {
        let Some(layout) = project.stair_layout(stair) else { continue };
        for f in &layout.flights {
            for t in layout.treads(f).iter().filter(|t| t.top <= cut_height) {
                cover.push(t.outline.iter().map(|p| viewport.to_paper(p.x, p.y)).collect());
            }
        }
        for (outline, top) in &layout.landings {
            if *top <= cut_height {
                cover.push(outline.iter().map(|p| viewport.to_paper(p.x, p.y)).collect());
            }
        }
    }
    for (id, stair) in project.stairs_on(storey) {
        draw_stair(objects, project, id, stair, cut_height, Mode::Own, &[], style, viewport);
    }
    if let Some(below) = below {
        for (id, stair) in project.stairs_on(below) {
            draw_stair(objects, project, id, stair, cut_height, Mode::FromBelow, &cover, style, viewport);
        }
    }
    // Freie Geländer stehen auf OKFF und werden geschnitten.
    for (_, railing) in project.railings_on(storey) {
        if let RailingPath::Points(points) = &railing.path {
            objects.push(line(
                points.iter().map(|p| viewport.to_paper(p.x, p.y)).collect(),
                style.pens.view,
                Look::Solid,
            ));
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Mode {
    /// Grundriss des Geschosses, in dem die Treppe beginnt.
    Own,
    /// Grundriss des Geschosses darüber.
    FromBelow,
}

#[allow(clippy::too_many_arguments)]
fn draw_stair(
    objects: &mut Vec<PlanObject>,
    project: &Project,
    id: ElementId,
    stair: &Stair,
    cut_height: f64,
    mode: Mode,
    cover: &[Vec<PaperPoint>],
    style: &ScaleStyle,
    viewport: Viewport,
) {
    let Some(layout) = project.stair_layout(stair) else { return };
    let cut = Cut::new(&layout, cut_height);
    // Alle Kanten als kurze Strecken mit Höhe, damit Schnittlinie und
    // Verdeckung je Stück entschieden werden können.
    let mut edges: Vec<(Point2, Point2, Part)> = Vec::new();
    for (i, f) in layout.flights.iter().enumerate() {
        for t in layout.treads(f) {
            let [fr, fl, bl, br] = t.outline;
            edges.extend([(fr, fl, Part::Flight(i)), (fl, bl, Part::Flight(i)), (br, fr, Part::Flight(i))]);
        }
        // Letzte Steigung: Vorderkante von Podest oder Austritt.
        let end = layout.end(f);
        let left = Point2::new(-f.dir.y, f.dir.x);
        let h = layout.width / 2.0;
        edges.push((offset(end, left, -h), offset(end, left, h), Part::Flight(i)));
    }
    for (j, (outline, _)) in layout.landings.iter().enumerate() {
        for i in 0..outline.len() {
            edges.push((outline[i], outline[(i + 1) % outline.len()], Part::Landing(j)));
        }
    }
    // Geländer an den Läufen dieser Treppe.
    for (_, r) in project.railings_on(stair.storey) {
        if let RailingPath::Flight { stair: s, flight, side } = r.path
            && s == id
            && let Some(f) = layout.flights.get(flight)
        {
            let [(a, _), (b, _)] = layout.flight_side(f, side, lot_geom::stairs::RAILING_INSET);
            let n = f.risers.max(2) as usize - 1;
            for k in 0..n {
                let (t0, t1) = (k as f64 / n as f64, (k + 1) as f64 / n as f64);
                edges.push((lerp(a, b, t0), lerp(a, b, t1), Part::Flight(flight)));
            }
        }
    }
    let pen = style.pens.view;
    let view = View { cut: &cut, mode, cover, viewport };
    for (a, b, part) in edges {
        for (p, q, look) in view.pieces(a, b, part) {
            let width = if look == Look::Dashed { style.pens.hidden } else { pen };
            objects.push(line(vec![p, q], width, look));
        }
    }
    // Schnittlinie quer über den Lauf, nur im eigenen Geschoss.
    if mode == Mode::Own
        && let Some((a, b)) = cut.line()
    {
        objects.push(line(vec![viewport.to_paper(a.x, a.y), viewport.to_paper(b.x, b.y)], pen, Look::Solid));
    }
    draw_walking_line(objects, &layout, &view, style);
    if mode == Mode::Own {
        label(objects, &layout, stair, style, viewport);
    }
}

/// Was von einer Treppe in einem Grundriss zu sehen ist.
struct View<'a> {
    cut: &'a Cut<'a>,
    mode: Mode,
    /// Im Geschoss darüber: Flächen, die eigene Treppen verdecken, in mm.
    cover: &'a [Vec<PaperPoint>],
    viewport: Viewport,
}

impl View<'_> {
    /// Sichtbare Stücke der Strecke a-b auf dem Blatt, mit Strichart.
    fn pieces(&self, a: Point2, b: Point2, part: Part) -> Vec<(PaperPoint, PaperPoint, Look)> {
        match self.mode {
            Mode::Own => self
                .cut
                .split(a, b, part)
                .into_iter()
                .map(|(p, q, below)| {
                    let look = if below { Look::Solid } else { Look::Dashed };
                    (self.viewport.to_paper(p.x, p.y), self.viewport.to_paper(q.x, q.y), look)
                })
                .collect(),
            Mode::FromBelow => {
                // In kurzen Stücken, damit teilweise verdeckte Kanten richtig enden.
                let n = (a.distance(b) / 0.05).ceil().max(1.0) as usize;
                let mut out: Vec<(PaperPoint, PaperPoint, Look)> = Vec::new();
                for i in 0..n {
                    let (p, q) = (lerp(a, b, i as f64 / n as f64), lerp(a, b, (i + 1) as f64 / n as f64));
                    let (pp, qq) = (self.viewport.to_paper(p.x, p.y), self.viewport.to_paper(q.x, q.y));
                    if self.hidden(PaperPoint::new((pp.x + qq.x) / 2.0, (pp.y + qq.y) / 2.0)) {
                        continue;
                    }
                    match out.last_mut() {
                        Some(last) if last.1 == pp => last.1 = qq,
                        _ => out.push((pp, qq, Look::Solid)),
                    }
                }
                out
            }
        }
    }

    fn hidden(&self, p: PaperPoint) -> bool {
        self.mode == Mode::FromBelow && inside(p, self.cover)
    }
}

/// Zu welchem Teil der Treppe eine Kante gehört.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Part {
    Flight(usize),
    Landing(usize),
}

/// Wo die Schnittebene die Treppe trifft: Lauf, Lage entlang des Laufs und
/// die schräge Linie quer darüber.
struct Cut<'a> {
    layout: &'a StairLayout,
    /// Höhe der Schnittebene über OKFF.
    height: f64,
    /// Lauf mit der Schnittlinie und Abstand der Linie vom Antritt, in der Laufmitte.
    at: Option<(usize, f64)>,
}

impl<'a> Cut<'a> {
    fn new(layout: &'a StairLayout, height: f64) -> Self {
        let at = layout.flights.iter().enumerate().find_map(|(i, f)| {
            let top = f.base + f.risers as f64 * layout.rise;
            // Höhe der Vorderkantenlinie: eine Steigung über dem Antritt, dann
            // gleichmäßig bis zum Podest.
            let first = f.base + layout.rise;
            (height >= first && height < top).then(|| {
                let u = (height - first) / layout.rise * layout.tread;
                (i, u.clamp(0.0, layout.flight_length(f)))
            })
        });
        Cut { layout, height, at }
    }

    /// Halbe Länge der Schnittlinie in Laufrichtung, für die Schräge.
    fn skew(&self) -> f64 {
        0.25 * self.layout.width
    }

    /// Endpunkte der Schnittlinie.
    fn line(&self) -> Option<(Point2, Point2)> {
        let (i, u) = self.at?;
        let f = &self.layout.flights[i];
        let left = Point2::new(-f.dir.y, f.dir.x);
        let h = self.layout.width / 2.0;
        let c = offset(f.start, f.dir, u);
        Some((offset(offset(c, f.dir, -self.skew()), left, -h), offset(offset(c, f.dir, self.skew()), left, h)))
    }

    /// Ob ein Punkt eines Teils der Treppe unterhalb der Schnittebene liegt.
    fn below(&self, p: Point2, part: Part) -> bool {
        match part {
            Part::Landing(j) => self.layout.landings.get(j).is_none_or(|(_, top)| *top <= self.height),
            Part::Flight(i) => {
                let f = &self.layout.flights[i];
                match self.at {
                    Some((j, ub)) if j == i => {
                        let left = Point2::new(-f.dir.y, f.dir.x);
                        let (dx, dy) = (p.x - f.start.x, p.y - f.start.y);
                        let u = dx * f.dir.x + dy * f.dir.y;
                        let s = dx * left.x + dy * left.y;
                        u < ub + self.skew() * s / (self.layout.width / 2.0) - 1e-6
                    }
                    _ => self.flight_below(f),
                }
            }
        }
    }

    /// Teil der Treppe, auf dem ein Punkt der Lauflinie liegt.
    fn part_of(&self, p: Point2) -> Part {
        for (j, (outline, _)) in self.layout.landings.iter().enumerate() {
            let ring: Vec<PaperPoint> = outline.iter().map(|q| PaperPoint::new(q.x, q.y)).collect();
            if inside(PaperPoint::new(p.x, p.y), std::slice::from_ref(&ring)) && !on_ring(p, outline) {
                return Part::Landing(j);
            }
        }
        let h = self.layout.width / 2.0;
        let i = self
            .layout
            .flights
            .iter()
            .position(|f| {
                let left = Point2::new(-f.dir.y, f.dir.x);
                let (dx, dy) = (p.x - f.start.x, p.y - f.start.y);
                let u = dx * f.dir.x + dy * f.dir.y;
                let s = dx * left.x + dy * left.y;
                u >= -1e-6 && u <= self.layout.flight_length(f) + 1e-6 && s.abs() <= h + 1e-6
            })
            .unwrap_or(0);
        Part::Flight(i)
    }

    fn flight_below(&self, f: &FlightLayout) -> bool {
        f.base + f.risers as f64 * self.layout.rise <= self.height
    }

    /// Strecke a-b an der Schnittlinie geteilt; je Stück, ob es unterhalb liegt.
    fn split(&self, a: Point2, b: Point2, part: Part) -> Vec<(Point2, Point2, bool)> {
        let mut ts = vec![0.0, 1.0];
        if let Some((p, q)) = self.line() {
            // Schnittpunkt mit der Geraden durch die Schnittlinie.
            let (rx, ry) = (b.x - a.x, b.y - a.y);
            let (sx, sy) = (q.x - p.x, q.y - p.y);
            let den = rx * sy - ry * sx;
            if den.abs() > 1e-12 {
                let t = ((p.x - a.x) * sy - (p.y - a.y) * sx) / den;
                if t > 1e-6 && t < 1.0 - 1e-6 {
                    ts.insert(1, t);
                }
            }
        }
        ts.windows(2)
            .map(|w| {
                let (p, q) = (lerp(a, b, w[0]), lerp(a, b, w[1]));
                (p, q, self.below(lerp(a, b, (w[0] + w[1]) / 2.0), part))
            })
            .collect()
    }
}

/// Lauflinie: Kreis am Antritt, Pfeil am Austritt. Im eigenen Geschoss oberhalb
/// der Schnittlinie gestrichelt, im Geschoss darüber, soweit nicht verdeckt.
fn draw_walking_line(objects: &mut Vec<PlanObject>, layout: &StairLayout, view: &View, style: &ScaleStyle) {
    let viewport = view.viewport;
    let points = layout.walking_line();
    let pen = style.pens.annotation;
    for w in points.windows(2) {
        let part = view.cut.part_of(lerp(w[0], w[1], 0.5));
        for (p, q, look) in view.pieces(w[0], w[1], part) {
            objects.push(line(vec![p, q], pen, look));
        }
    }
    let (Some(first), Some(last)) = (points.first(), points.last()) else { return };
    let c = viewport.to_paper(first.x, first.y);
    if !view.hidden(c) {
        // Kreis, 1,5 mm Durchmesser auf dem Blatt.
        let ring: Vec<PaperPoint> = (0..16)
            .map(|i| {
                let a = i as f64 / 16.0 * std::f64::consts::TAU;
                PaperPoint::new(c.x + 0.75 * a.cos(), c.y + 0.75 * a.sin())
            })
            .collect();
        objects.push(PlanObject::Line(Line {
            points: ring,
            closed: true,
            pen: Pen::mm(pen),
            style: LineStyle::Solid,
            color: Color::BLACK,
        }));
    }
    // Pfeil am Austritt, gefüllt, 2,5 mm lang.
    if let Some(prev) = points.iter().rev().nth(1)
        && !view.hidden(viewport.to_paper(last.x, last.y))
    {
        let (tip, from) = (viewport.to_paper(last.x, last.y), viewport.to_paper(prev.x, prev.y));
        let len = (tip.x - from.x).hypot(tip.y - from.y);
        if len > 1e-9 {
            let d = ((tip.x - from.x) / len, (tip.y - from.y) / len);
            let back = PaperPoint::new(tip.x - d.0 * 2.5, tip.y - d.1 * 2.5);
            let (s0, s1) = (
                PaperPoint::new(back.x - d.1 * 0.8, back.y + d.0 * 0.8),
                PaperPoint::new(back.x + d.1 * 0.8, back.y - d.0 * 0.8),
            );
            objects
                .push(PlanObject::Area(PlanArea { rings: vec![vec![tip, s0, s1]], fill: Fill::Solid(Color::BLACK) }));
        }
    }
}

/// Steigungen und Steigungsverhältnis vor dem Antritt.
fn label(objects: &mut Vec<PlanObject>, layout: &StairLayout, stair: &Stair, style: &ScaleStyle, viewport: Viewport) {
    let Some(f) = layout.flights.first() else { return };
    let text = format!(
        "{} Stg. {}/{}",
        stair.risers(),
        crate::format_de(layout.rise * 100.0, 1),
        crate::format_de(layout.tread * 100.0, 1)
    );
    let start = viewport.to_paper(f.start.x, f.start.y);
    let size = style.space_text;
    // Vor der ersten Stufe, in Laufrichtung gedreht und von unten oder rechts lesbar.
    let mut angle = f.dir.y.atan2(f.dir.x).to_degrees() + 90.0;
    while angle > 90.0 {
        angle -= 180.0;
    }
    while angle < -90.0 {
        angle += 180.0;
    }
    let gap = 1.5 + size / 2.0;
    let at = PaperPoint::new(start.x - f.dir.x * gap, start.y - f.dir.y * gap);
    // Text mittig auf `at`; die Schriftlinie liegt eine halbe Schrifthöhe tiefer.
    let rad = angle.to_radians();
    let at = PaperPoint::new(at.x + rad.sin() * size / 2.0, at.y - rad.cos() * size / 2.0);
    objects.push(PlanObject::Text(Text {
        position: at,
        text,
        height_mm: size,
        anchor: Anchor::Center,
        rotation: angle,
    }));
}

fn line(points: Vec<PaperPoint>, width: f64, look: Look) -> PlanObject {
    PlanObject::Line(Line {
        points,
        closed: false,
        pen: Pen::mm(width),
        style: match look {
            Look::Solid => LineStyle::Solid,
            Look::Dashed => LineStyle::Dashed(ABOVE_DASH.to_vec()),
        },
        color: Color::BLACK,
    })
}

fn offset(p: Point2, d: Point2, t: f64) -> Point2 {
    Point2::new(p.x + d.x * t, p.y + d.y * t)
}

fn lerp(a: Point2, b: Point2, t: f64) -> Point2 {
    Point2::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t)
}

/// Liegt p auf dem Rand des Polygons?
fn on_ring(p: Point2, ring: &[Point2]) -> bool {
    (0..ring.len()).any(|i| {
        let (a, b) = (ring[i], ring[(i + 1) % ring.len()]);
        let len = a.distance(b);
        if len < 1e-12 {
            return false;
        }
        let t = ((p.x - a.x) * (b.x - a.x) + (p.y - a.y) * (b.y - a.y)) / (len * len);
        let q = lerp(a, b, t.clamp(0.0, 1.0));
        q.distance(p) < 1e-6
    })
}

#[cfg(test)]
mod tests {
    use lot_model::{Element, Landing, Railing, StairFlight, WallSide};

    use super::*;
    use crate::standard::OfficeStandard;

    fn project() -> (Project, ElementId, ElementId) {
        let mut p = Project::sample();
        let storeys = p.storeys();
        let (lower, upper) = (storeys[0].0, storeys.get(1).map(|s| s.0));
        let upper = upper.unwrap_or_else(|| {
            let id = ElementId(800);
            p.elements
                .insert(id, Element::Storey(lot_model::Storey { name: "OG".into(), elevation: 2.75, height: 2.75 }));
            id
        });
        p.elements.insert(
            ElementId(900),
            Element::Stair(Stair {
                phase: Default::default(),
                storey: lower,
                start: Point2::new(0.5, 0.0),
                direction: Point2::new(0.0, 1.0),
                width: 1.0,
                tread: 0.27,
                flights: vec![
                    StairFlight { risers: 8, landing: Landing::BackRight },
                    StairFlight { risers: 8, landing: Landing::Straight },
                ],
                thickness: 0.18,
                necessary: true,
            }),
        );
        p.elements.insert(
            ElementId(901),
            Element::Railing(Railing {
                phase: Default::default(),
                around: None,
                storey: lower,
                path: RailingPath::Flight { stair: ElementId(900), flight: 0, side: WallSide::Left },
                height: 0.9,
            }),
        );
        (p, lower, upper)
    }

    fn draw(p: &Project, storey: ElementId) -> Vec<PlanObject> {
        let mut objects = Vec::new();
        let style = OfficeStandard::default_standard().style_for(100.0);
        let viewport = Viewport { scale: 100.0, origin: PaperPoint::new(0.0, 0.0) };
        draw_stairs(&mut objects, p, storey, 1.0, style, viewport);
        objects
    }

    fn count(objects: &[PlanObject], dashed: bool) -> usize {
        objects
            .iter()
            .filter(|o| matches!(o, PlanObject::Line(l) if matches!(l.style, LineStyle::Dashed(_)) == dashed))
            .count()
    }

    #[test]
    fn lower_storey_shows_cut_and_dashed_upper_part() {
        let (p, lower, _) = project();
        let objects = draw(&p, lower);
        assert!(count(&objects, false) > 10 && count(&objects, true) > 10);
        let text: Vec<&str> = objects
            .iter()
            .filter_map(|o| match o {
                PlanObject::Text(t) => Some(t.text.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(text, ["16 Stg. 17,2/27,0"]);
    }

    #[test]
    fn cut_line_sits_where_the_nosings_reach_the_cut_height() {
        let (p, _, _) = project();
        let stair = p.stair(ElementId(900)).unwrap();
        let layout = p.stair_layout(stair).unwrap();
        let cut = Cut::new(&layout, 1.0);
        let (flight, u) = cut.at.unwrap();
        // Vorderkanten: 17,19 cm je Stufe ab einer Steigung über dem Boden.
        assert_eq!(flight, 0);
        assert!((u - (1.0 - 2.75 / 16.0) / (2.75 / 16.0) * 0.27).abs() < 1e-9, "{u}");
        // Die erste Stufe liegt unter, das Podest über der Schnittebene.
        assert!(cut.below(Point2::new(0.5, 0.1), Part::Flight(0)));
        assert!(!cut.below(Point2::new(1.0, 2.4), Part::Landing(0)));
        assert_eq!(cut.part_of(Point2::new(1.0, 2.39)), Part::Landing(0));
        assert_eq!(cut.part_of(Point2::new(1.5, 1.0)), Part::Flight(1));
    }

    #[test]
    fn upper_storey_shows_the_stair_below_solid_without_cut_line() {
        let (p, lower, upper) = project();
        let objects = draw(&p, upper);
        assert_eq!(count(&objects, true), 0);
        assert!(objects.iter().all(|o| !matches!(o, PlanObject::Text(_))));
        // Alle Kanten des unteren Grundrisses ohne die Schnittlinie.
        let below = draw(&p, lower);
        assert!(count(&objects, false) > 10 && count(&objects, false) < count(&below, false) + count(&below, true));
    }
}
