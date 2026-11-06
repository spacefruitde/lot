//! Fangen (ADR 0046): Der Punkt unter dem Zeiger gewinnt, Art und Name stehen
//! am Zeiger, Tab blättert durch die Kandidaten. Gefangen werden Wandenden,
//! Mitten, Schnittpunkte, Ecken der Wandflächen, Lotfußpunkte, Achsen,
//! Deckenecken, Bezugspunkte mit ihren Spurlinien und Unterlagen; dazu
//! Winkel in Schritten zur Zeichenrichtung.

use lot_model::{ElementId, Point2, Project};

use crate::geometry::{JOIN_TOLERANCE, faces, intersect, local};

/// Woran der Zeiger eingerastet ist.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapKind {
    /// Ende einer Wand (Bezugslinie)
    Endpoint,
    /// Mitte einer geraden Wand
    Midpoint,
    /// Schnittpunkt der Bezugslinien zweier Wände
    Intersection,
    /// Ecke der Wandflächen, auch an Anschlüssen
    FaceCorner,
    /// Fußpunkt des Lots vom Anfangspunkt auf eine Wand oder Wandfläche
    Perpendicular,
    /// Schnittpunkt zweier Achsen
    GridIntersection,
    /// Ecke einer Decke
    Corner,
    /// Ein gesetzter Bezugspunkt (Q)
    Reference,
    /// Schnitt zweier Spurlinien
    TrackingIntersection,
    /// Punkt auf der Bezugslinie einer Wand
    OnWall,
    /// Punkt auf einer Wandfläche
    Face,
    /// Punkt auf einer Achse
    Grid,
    /// Punkt auf einer Spurlinie eines Bezugspunkts
    Tracking,
    /// Richtung in 15°-Schritten vom Anfangspunkt
    Angle,
    /// Frei, auf 1 cm gerundet
    Free,
    /// Endpunkt einer Linie in einer Unterlage aus DXF oder DWG
    Underlay,
}

impl SnapKind {
    /// Name am Zeiger.
    pub fn label(self) -> &'static str {
        match self {
            SnapKind::Endpoint => "Ende Wand",
            SnapKind::Midpoint => "Mitte",
            SnapKind::Intersection => "Schnittpunkt",
            SnapKind::FaceCorner => "Ecke Wandfläche",
            SnapKind::Perpendicular => "lotrecht",
            SnapKind::GridIntersection => "Achsschnittpunkt",
            SnapKind::Corner => "Ecke Decke",
            SnapKind::Reference => "Bezugspunkt",
            SnapKind::TrackingIntersection => "Spurlinien kreuzen",
            SnapKind::OnWall => "auf Wand",
            SnapKind::Face => "Wandfläche",
            SnapKind::Grid => "auf Achse",
            SnapKind::Tracking => "Spurlinie",
            SnapKind::Angle => "Winkel",
            SnapKind::Free => "",
            SnapKind::Underlay => "Unterlage",
        }
    }

    /// Ob es ein Punkt ist (gewinnt vor Linien).
    fn is_point(self) -> bool {
        !matches!(
            self,
            SnapKind::OnWall | SnapKind::Face | SnapKind::Grid | SnapKind::Tracking | SnapKind::Angle | SnapKind::Free
        )
    }

    /// Rang bei gleichem Abstand.
    fn rank(self) -> u8 {
        self as u8
    }
}

/// Fangkürzel für einen Klick (wie Revit): nur diese Art fangen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SnapOnly {
    /// SE: Endpunkte
    Endpoints,
    /// SM: Mitten
    Midpoints,
    /// SI: Schnittpunkte
    Intersections,
    /// SP: lotrecht
    Perpendicular,
    /// SN: nächster Punkt auf einer Wand, Fläche oder Achse
    Nearest,
    /// SO: Fangen aus
    Off,
}

impl SnapOnly {
    fn allows(self, kind: SnapKind) -> bool {
        match self {
            SnapOnly::Endpoints => matches!(kind, SnapKind::Endpoint | SnapKind::FaceCorner | SnapKind::Corner),
            SnapOnly::Midpoints => kind == SnapKind::Midpoint,
            SnapOnly::Intersections => {
                matches!(kind, SnapKind::Intersection | SnapKind::GridIntersection | SnapKind::TrackingIntersection)
            }
            SnapOnly::Perpendicular => kind == SnapKind::Perpendicular,
            SnapOnly::Nearest => matches!(kind, SnapKind::OnWall | SnapKind::Face | SnapKind::Grid),
            SnapOnly::Off => false,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            SnapOnly::Endpoints => "nur Endpunkte",
            SnapOnly::Midpoints => "nur Mitten",
            SnapOnly::Intersections => "nur Schnittpunkte",
            SnapOnly::Perpendicular => "nur lotrecht",
            SnapOnly::Nearest => "nur auf Linien",
            SnapOnly::Off => "Fangen aus",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Snap {
    pub point: Point2,
    pub kind: SnapKind,
}

/// Alles, was neben dem Modell das Fangen bestimmt.
#[derive(Debug, Clone, Copy, Default)]
pub struct Context<'a> {
    /// Anfangspunkt beim Zeichnen.
    pub from: Option<Point2>,
    /// Nur rechte Winkel zur Zeichenrichtung (⇧).
    pub orthogonal: bool,
    /// Wände, die nicht fangen, etwa die gerade verschobenen.
    pub exclude: &'a [ElementId],
    /// Bezugspunkte (Q), der zuletzt gesetzte zuletzt.
    pub references: &'a [Point2],
    /// Zeichenrichtung in Grad (0 = waagerecht).
    pub direction: f64,
    /// Fangkürzel für diesen Klick.
    pub only: Option<SnapOnly>,
    /// Endpunkte in Unterlagen aus DXF oder DWG.
    pub underlay_points: &'a [Point2],
}

/// Raster für freie Punkte und Längen, in Metern.
pub const ROUND: f64 = 0.01;
/// Winkelschritt beim Zeichnen, in Grad.
const ANGLE_STEP: f64 = 15.0;
/// Bis zu dieser Abweichung rastet der Winkel ein, in Grad.
const ANGLE_CATCH: f64 = 3.0;
/// Länge der Spurlinien, in Metern.
pub const TRACK_REACH: f64 = 60.0;

/// Fangpunkt zum Zeiger `p`. `from` ist der Anfangspunkt beim Zeichnen;
/// mit `orthogonal` rastet die Richtung nur in 90°-Schritten ein.
/// `exclude` nimmt Wände aus, etwa die gerade verschobenen.
pub fn snap(
    project: &Project,
    storey: ElementId,
    p: Point2,
    tolerance: f64,
    from: Option<Point2>,
    orthogonal: bool,
    exclude: &[ElementId],
) -> Snap {
    let ctx = Context { from, orthogonal, exclude, ..Default::default() };
    candidates(project, storey, p, tolerance, &ctx)[0]
}

/// Alle Fangpunkte zum Zeiger `p`, der beste zuerst: erst Punkte nach
/// Abstand, dann Linien nach Abstand, dann der Winkel, zuletzt der freie
/// Punkt. Die Liste ist nie leer.
pub fn candidates(project: &Project, storey: ElementId, p: Point2, tolerance: f64, ctx: &Context) -> Vec<Snap> {
    let mut found = find(project, storey, p, tolerance, ctx);
    if let Some(only) = ctx.only {
        found.retain(|(s, _)| only.allows(s.kind));
    }
    found.sort_by(|a, b| {
        (!a.0.kind.is_point(), a.0.kind == SnapKind::Angle)
            .cmp(&(!b.0.kind.is_point(), b.0.kind == SnapKind::Angle))
            .then(a.1.total_cmp(&b.1))
            .then(a.0.kind.rank().cmp(&b.0.kind.rank()))
    });
    // Rechenrauschen aus Winkelfunktionen entfernen (4,0 statt 4,000000000000001).
    let clean = |v: f64| (v * 1e9).round() / 1e9;
    let mut out: Vec<Snap> = Vec::new();
    for (s, _) in found {
        let s = Snap { point: Point2::new(clean(s.point.x), clean(s.point.y)), kind: s.kind };
        if !out.iter().any(|o| o.point.distance(s.point) < 1e-6) {
            out.push(s);
        }
    }
    let free = Point2::new((p.x / ROUND).round() * ROUND, (p.y / ROUND).round() * ROUND);
    if ctx.only != Some(SnapOnly::Off) || out.is_empty() {
        out.push(Snap { point: free, kind: SnapKind::Free });
    }
    out
}

/// Richtung in Grad als Einheitsvektor.
fn dir(deg: f64) -> (f64, f64) {
    let (s, c) = deg.to_radians().sin_cos();
    (c, s)
}

fn find(project: &Project, storey: ElementId, p: Point2, tolerance: f64, ctx: &Context) -> Vec<(Snap, f64)> {
    let walls: Vec<_> = project.walls_on(storey).filter(|(id, _)| !ctx.exclude.contains(id)).collect();
    let mut out: Vec<(Snap, f64)> = Vec::new();
    let point = |q: Point2, kind: SnapKind, out: &mut Vec<(Snap, f64)>| {
        let d = q.distance(p);
        if d <= tolerance {
            out.push((Snap { point: q, kind }, d));
        }
    };

    // Punkte an Wänden: Enden, Mitten, Ecken der Flächen.
    for (_, w) in &walls {
        point(w.start, SnapKind::Endpoint, &mut out);
        point(w.end, SnapKind::Endpoint, &mut out);
        if !w.is_curved() {
            point(w.point_at(w.length() / 2.0, 0.0), SnapKind::Midpoint, &mut out);
            let (l, r) = faces(project, w);
            for t in [l, r] {
                point(w.point_at(0.0, t), SnapKind::FaceCorner, &mut out);
                point(w.point_at(w.length(), t), SnapKind::FaceCorner, &mut out);
            }
        }
    }
    // Schnittpunkte von Bezugslinien und, an Anschlüssen, von Wandflächen.
    for (i, (_, a)) in walls.iter().enumerate() {
        if a.is_curved() {
            continue;
        }
        for (_, b) in &walls[i + 1..] {
            if b.is_curved() {
                continue;
            }
            let (da, db) = (a.direction(), b.direction());
            if let Some(q) = intersect(a.start, da, b.start, db) {
                let (sa, sb) = (a.locate(q).0, b.locate(q).0);
                let inside = |s: f64, len: f64| s > JOIN_TOLERANCE && s < len - JOIN_TOLERANCE;
                if (inside(sa, a.length()) || inside(sb, b.length()))
                    && (-1e-9..=a.length() + 1e-9).contains(&sa)
                    && (-1e-9..=b.length() + 1e-9).contains(&sb)
                {
                    point(q, SnapKind::Intersection, &mut out);
                }
            }
            let joined =
                [a.start, a.end].iter().any(|e| [b.start, b.end].iter().any(|f| e.distance(*f) <= JOIN_TOLERANCE));
            if joined {
                let (la, ra) = faces(project, a);
                let (lb, rb) = faces(project, b);
                for ta in [la, ra] {
                    for tb in [lb, rb] {
                        if let Some(q) = intersect(a.point_at(0.0, ta), da, b.point_at(0.0, tb), db) {
                            point(q, SnapKind::FaceCorner, &mut out);
                        }
                    }
                }
            }
        }
    }
    // Lotfußpunkte vom Anfangspunkt.
    if let Some(start) = ctx.from {
        for (_, w) in walls.iter().filter(|(_, w)| !w.is_curved()) {
            let (l, r) = faces(project, w);
            for t in [0.0, l, r] {
                let a = w.point_at(0.0, t);
                let d = w.direction();
                let s = (start.x - a.x) * d.0 + (start.y - a.y) * d.1;
                if s > 1e-6 && s < w.length() - 1e-6 {
                    point(Point2::new(a.x + d.0 * s, a.y + d.1 * s), SnapKind::Perpendicular, &mut out);
                }
            }
        }
    }
    // Achsschnittpunkte.
    let grids: Vec<_> = project.grid_lines().map(|(_, g)| g).collect();
    for (i, a) in grids.iter().enumerate() {
        for b in &grids[i + 1..] {
            if let Some(q) = intersect(a.start, unit(a.start, a.end), b.start, unit(b.start, b.end)) {
                point(q, SnapKind::GridIntersection, &mut out);
            }
        }
    }
    // Ecken von Decken.
    for (_, s) in project.slabs_on(storey) {
        for q in &s.outline {
            point(*q, SnapKind::Corner, &mut out);
        }
    }
    // Bezugspunkte und Schnitte ihrer Spurlinien.
    let tracks: Vec<(Point2, (f64, f64))> =
        ctx.references.iter().flat_map(|r| [(*r, dir(ctx.direction)), (*r, dir(ctx.direction + 90.0))]).collect();
    for r in ctx.references {
        point(*r, SnapKind::Reference, &mut out);
    }
    for (i, (a, da)) in tracks.iter().enumerate() {
        for (b, db) in &tracks[i + 1..] {
            if a.distance(*b) > 1e-9
                && let Some(q) = intersect(*a, *da, *b, *db)
            {
                point(q, SnapKind::TrackingIntersection, &mut out);
            }
        }
    }
    for q in ctx.underlay_points {
        point(*q, SnapKind::Underlay, &mut out);
    }

    // Mit Anfangspunkt: Winkel in Schritten zur Zeichenrichtung, Länge auf 1 cm.
    let angled = ctx.from.and_then(|start| {
        let (dx, dy) = (p.x - start.x, p.y - start.y);
        let len = dx.hypot(dy);
        if len < 1e-6 {
            return None;
        }
        let angle = dy.atan2(dx).to_degrees() - ctx.direction;
        let step = if ctx.orthogonal { 90.0 } else { ANGLE_STEP };
        let snapped = (angle / step).round() * step;
        let off = (angle - snapped + 180.0).rem_euclid(360.0) - 180.0;
        if !ctx.orthogonal && off.abs() > ANGLE_CATCH {
            return None;
        }
        let len = (len / ROUND).round() * ROUND;
        let (c, s) = dir(snapped + ctx.direction);
        Some(Point2::new(start.x + c * len, start.y + s * len))
    });

    // Auf Linien; mit Winkel: Schnitt der Richtung mit der Linie.
    for (_, w) in walls.iter().filter(|(_, w)| w.is_curved()) {
        let ray = ctx.from.zip(angled).map(|(start, q)| (start, unit(start, q)));
        if let Some(q) = on_arc(w, p, ray) {
            let d = q.distance(p);
            if d <= tolerance {
                out.push((Snap { point: q, kind: SnapKind::OnWall }, d));
            }
        }
    }
    let mut lines: Vec<(Point2, Point2, SnapKind)> = Vec::new();
    for (_, w) in walls.iter().filter(|(_, w)| !w.is_curved()) {
        lines.push((w.start, w.end, SnapKind::OnWall));
        let (l, r) = faces(project, w);
        for t in [l, r] {
            lines.push((w.point_at(0.0, t), w.point_at(w.length(), t), SnapKind::Face));
        }
    }
    lines.extend(grids.iter().map(|g| (g.start, g.end, SnapKind::Grid)));
    for (a, (dx, dy)) in &tracks {
        let reach = TRACK_REACH;
        lines.push((
            Point2::new(a.x - dx * reach, a.y - dy * reach),
            Point2::new(a.x + dx * reach, a.y + dy * reach),
            SnapKind::Tracking,
        ));
    }
    for (a, b, kind) in lines {
        let d = unit(a, b);
        let len = a.distance(b);
        let candidate = match (ctx.from, angled) {
            (Some(start), Some(q)) => intersect(start, unit(start, q), a, d),
            _ => {
                let mut s = ((p.x - a.x) * d.0 + (p.y - a.y) * d.1).clamp(0.0, len);
                // Auf Spurlinien zählt der Abstand vom Bezugspunkt in ganzen Zentimetern.
                if kind == SnapKind::Tracking {
                    s = len / 2.0 + ((s - len / 2.0) / ROUND).round() * ROUND;
                }
                Some(Point2::new(a.x + d.0 * s, a.y + d.1 * s))
            }
        };
        let Some(q) = candidate else { continue };
        let s = (q.x - a.x) * d.0 + (q.y - a.y) * d.1;
        if s < -1e-9 || s > len + 1e-9 {
            continue;
        }
        let dist = q.distance(p);
        if dist <= tolerance {
            out.push((Snap { point: q, kind }, dist));
        }
    }
    if let Some(q) = angled {
        out.push((Snap { point: q, kind: SnapKind::Angle }, q.distance(p)));
    }
    out
}

/// Punkt auf dem Bogen einer gebogenen Wand: der nächste zu `p`, oder mit
/// `ray` (Anfang, Richtung) der Schnitt dieses Strahls mit dem Bogen.
fn on_arc(w: &lot_model::Wall, p: Point2, ray: Option<(Point2, (f64, f64))>) -> Option<Point2> {
    let arc = w.arc()?;
    let on = |q: Point2| {
        let (s, _) = w.locate(q);
        (-1e-9..=w.length() + 1e-9).contains(&s)
    };
    match ray {
        None => {
            let (s, _) = w.locate(p);
            Some(w.point_at(s.clamp(0.0, w.length()), 0.0))
        }
        Some((start, d)) => {
            // |start + u d - c|² = r²
            let (ox, oy) = (start.x - arc.center.x, start.y - arc.center.y);
            let b = d.0 * ox + d.1 * oy;
            let disc = b * b - (ox * ox + oy * oy - arc.radius * arc.radius);
            if disc < 0.0 {
                return None;
            }
            [-b - disc.sqrt(), -b + disc.sqrt()]
                .into_iter()
                .map(|u| Point2::new(start.x + d.0 * u, start.y + d.1 * u))
                .filter(|q| on(*q))
                .min_by(|a, b| a.distance(p).total_cmp(&b.distance(p)))
        }
    }
}

fn unit(a: Point2, b: Point2) -> (f64, f64) {
    let len = a.distance(b).max(1e-12);
    ((b.x - a.x) / len, (b.y - a.y) / len)
}

/// Ob der Punkt auf der Bezugslinie der Wand liegt (für Tests).
pub fn on_reference(project: &Project, wall: ElementId, p: Point2) -> bool {
    project.wall(wall).is_some_and(|w| {
        let (s, t) = local(w, p);
        t.abs() < 1e-6 && s >= -1e-6 && s <= w.length() + 1e-6
    })
}
