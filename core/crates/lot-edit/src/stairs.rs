//! Treppenwerkzeug (ADR 0051): Antritt klicken, Richtung zeigen; Lot rechnet
//! Steigungen und Auftritt aus der Geschosshöhe nach DIN 18065, zeigt die
//! Prüfung als Ampel, schneidet das Loch in die Decke darüber, so groß wie
//! die Durchgangshöhe es verlangt, und setzt Geländer an die freien Seiten.
//! Loch und Umwehrung bleiben mit der Treppe verbunden (siehe [`crate::follow`]).

use lot_geom::Area;
use lot_model::{
    BuildingClass, Element, ElementId, Landing, Operation, Point2, Project, Railing, RailingPath, SlabOpening, Stair,
    StairFlight, WallSide,
};
use serde::{Deserialize, Serialize};

/// Form einer neuen Treppe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum StairForm {
    /// Ein gerader Lauf.
    #[default]
    Straight,
    /// Zweiläufig mit Halbpodest.
    HalfLanding,
    /// L-Form mit Viertelpodest.
    QuarterLanding,
    /// Viertelgewendelt: L-Form mit Wendelstufen statt Podest.
    QuarterWinder,
    /// Halbgewendelt: zurück mit Wendelstufen um ein Treppenauge.
    HalfWinder,
    /// Spindeltreppe: erster Klick Mittelpunkt, zweiter Antritt und Radius.
    Spiral,
}

impl StairForm {
    pub const ALL: [StairForm; 6] = [
        StairForm::Straight,
        StairForm::HalfLanding,
        StairForm::QuarterLanding,
        StairForm::QuarterWinder,
        StairForm::HalfWinder,
        StairForm::Spiral,
    ];

    pub fn title(self) -> &'static str {
        match self {
            StairForm::Straight => "gerade",
            StairForm::HalfLanding => "zweiläufig mit Halbpodest",
            StairForm::QuarterLanding => "L-Form mit Viertelpodest",
            StairForm::QuarterWinder => "viertelgewendelt",
            StairForm::HalfWinder => "halbgewendelt",
            StairForm::Spiral => "Spindeltreppe",
        }
    }
}

/// Lichte Treppendurchgangshöhe nach DIN 18065:2020-08.
pub const HEADROOM: f64 = 2.00;
/// Höhe eines Geländers über der Stufenvorderkante und an der Umwehrung,
/// Vorschlag nach MBO § 38 (Umwehrungen 0,90 m bis 12 m Absturzhöhe).
pub const RAILING_HEIGHT: f64 = 0.90;
/// Treppenauge halbgewendelter Treppen, Vorschlag.
pub const EYE: f64 = 0.20;
/// Spindelradius neuer Spindeltreppen, Vorschlag.
pub const SPINDLE: f64 = 0.10;

/// Gewünschte Steigung für den Vorschlag: Wohngebäude bequem, sonst flacher.
fn target_rise(building: BuildingClass) -> f64 {
    match building {
        BuildingClass::SmallResidential => 0.175,
        BuildingClass::General => 0.17,
    }
}

/// Vorschlag nach DIN 18065:2020-08: Zahl der Steigungen, sodass die Steigung
/// nahe der gewünschten liegt und in Tabelle 1 passt, und der Auftritt aus der
/// Schrittmaßregel 2s + a = 63 cm, auf 5 mm gerundet und in den Grenzen.
pub fn propose(height: f64, building: BuildingClass, necessary: bool) -> (u32, f64) {
    let limits = lot_rules::stairs::limits(building, necessary);
    let mut risers = (height / target_rise(building)).round().max(1.0) as u32;
    while height / risers as f64 > limits.rise.1 + 1e-9 {
        risers += 1;
    }
    let rise = height / risers as f64;
    let tread = ((0.63 - 2.0 * rise) / 0.005).round() * 0.005;
    (risers, tread.clamp(limits.tread.0, limits.tread.1))
}

/// Eine neue Treppe am Antritt `start` in Richtung `toward`; bei Halb- und
/// Viertelpodest liegt der zweite Lauf links (`left`) oder rechts.
pub fn new_stair(
    project: &Project,
    storey: ElementId,
    start: Point2,
    toward: Point2,
    form: StairForm,
    left: bool,
) -> Option<Stair> {
    let s = project.storey(storey)?;
    let building = project.info().building;
    let (risers, tread) = propose(s.height, building, true);
    let width = match building {
        BuildingClass::SmallResidential => 1.00,
        BuildingClass::General => 1.20,
    };
    let len = start.distance(toward);
    if len < 1e-6 {
        return None;
    }
    if form == StairForm::Spiral {
        // Spindeltreppe: `start` ist der Mittelpunkt, `toward` der Antritt außen; Steigung
        // steiler als sonst, wie bei Spindeltreppen üblich.
        let radius = ((len / 0.05).round() * 0.05).max(0.6);
        let limits = lot_rules::stairs::limits(building, true);
        let risers = (s.height / limits.rise.1.min(0.19)).ceil().max(2.0) as u32;
        let rise = s.height / risers as f64;
        let tread = (((0.63 - 2.0 * rise) / 0.005).round() * 0.005).clamp(limits.tread.0, limits.tread.1);
        return Some(Stair {
            phase: Default::default(),
            storey,
            start: toward,
            direction: Point2::new(toward.x - start.x, toward.y - start.y),
            width: radius - SPINDLE,
            tread,
            flights: vec![StairFlight { risers, landing: Landing::Straight }],
            thickness: 0.18,
            necessary: true,
            spiral: Some(lot_model::Spiral { center: start, radius, spindle: SPINDLE, clockwise: !left }),
            eye: 0.0,
        });
    }
    let direction = Point2::new((toward.x - start.x) / len, (toward.y - start.y) / len);
    let first = risers.div_ceil(2);
    let flights = match form {
        StairForm::Straight => vec![StairFlight { risers, landing: Landing::Straight }],
        StairForm::HalfLanding => vec![
            StairFlight { risers: first, landing: if left { Landing::BackLeft } else { Landing::BackRight } },
            StairFlight { risers: risers - first, landing: Landing::Straight },
        ],
        StairForm::QuarterLanding => vec![
            StairFlight { risers: first, landing: if left { Landing::Left } else { Landing::Right } },
            StairFlight { risers: risers - first, landing: Landing::Straight },
        ],
        StairForm::QuarterWinder => vec![
            StairFlight { risers: first, landing: if left { Landing::WinderLeft } else { Landing::WinderRight } },
            StairFlight { risers: risers - first, landing: Landing::Straight },
        ],
        StairForm::HalfWinder => vec![
            StairFlight {
                risers: first,
                landing: if left { Landing::WinderBackLeft } else { Landing::WinderBackRight },
            },
            StairFlight { risers: risers - first, landing: Landing::Straight },
        ],
        StairForm::Spiral => Vec::new(),
    };
    Some(Stair {
        phase: Default::default(),
        storey,
        start,
        direction,
        width,
        tread,
        flights,
        thickness: 0.18,
        necessary: true,
        spiral: None,
        eye: if form == StairForm::HalfWinder { EYE } else { 0.0 },
    })
}

/// Ampel nach DIN 18065 für eine Treppe, die gleich gesetzt wird, mit dem
/// Loch, das sie bekommt: Kurztext „16 Stg. 17,2/27,0“ und die Abweichungen.
pub fn verdict(project: &Project, stair: &Stair) -> Option<(String, Vec<String>)> {
    let (id, ops) = create_ops(project, stair.clone());
    let mut scratch = project.clone();
    for op in ops {
        if let Operation::Insert { id, element } = op {
            scratch.elements.insert(id, element);
        }
    }
    let check = lot_rules::stairs::check_stair(&scratch, id, stair)?;
    let cm = |v: f64| format!("{:.1}", v * 100.0).replace('.', ",");
    Some((format!("{} Stg. {}/{}", check.risers, cm(check.rise), cm(check.tread)), check.problems))
}

/// Die Decke über der Treppe: im Geschoss darüber, über dem Antritt.
fn slab_above(project: &Project, stair: &Stair) -> Option<(ElementId, f64)> {
    let above = crate::navigation::storey_step(project, stair.storey, true)?;
    let layout = project.stair_layout(stair)?;
    let probe = exit_point(&layout)?;
    project.slabs_on(above).find(|(_, s)| contains(&s.outline, probe)).and_then(|(id, s)| {
        let levels = lot_geom::slabs::floor_levels(project, s)?;
        Some((id, levels.bottom))
    })
}

fn contains(ring: &[Point2], p: Point2) -> bool {
    Area::from_ring(&ring.iter().map(|q| [q.x, q.y]).collect::<Vec<_>>()).contains([p.x, p.y])
}

/// Umriss des Treppenlochs: alle Stufen und Podeste, über denen die Decke
/// näher als die Durchgangshöhe käme, bis zum Austritt.
pub fn hole_outline(project: &Project, stair: &Stair) -> Option<Vec<Point2>> {
    let (_, bottom) = slab_above(project, stair)?;
    let z0 = project.storey(stair.storey)?.elevation;
    let layout = project.stair_layout(stair)?;
    let mut parts: Vec<Area> = Vec::new();
    let ring = |points: &[Point2]| Area::from_ring(&points.iter().map(|q| [q.x, q.y]).collect::<Vec<_>>());
    for f in &layout.flights {
        // Vorderkante jeder Steigung: von dort bis zum nächsten Auftritt muss es frei sein.
        let left = Point2::new(-f.dir.y, f.dir.x);
        let h = layout.width / 2.0;
        for k in 1..=f.risers {
            let top = z0 + f.base + k as f64 * layout.rise;
            if top + HEADROOM <= bottom + 1e-9 {
                continue;
            }
            let front = Point2::new(
                f.start.x + f.dir.x * (k - 1) as f64 * layout.tread,
                f.start.y + f.dir.y * (k - 1) as f64 * layout.tread,
            );
            let depth = if k == f.risers { 0.01 } else { layout.tread };
            let back = Point2::new(front.x + f.dir.x * depth, front.y + f.dir.y * depth);
            let at = |p: Point2, t: f64| Point2::new(p.x + left.x * t, p.y + left.y * t);
            parts.push(ring(&[at(front, -h), at(back, -h), at(back, h), at(front, h)]));
        }
    }
    for (outline, top) in &layout.landings {
        if z0 + top + HEADROOM > bottom + 1e-9 {
            parts.push(ring(outline));
        }
    }
    // Wendel- und Spindelstufen: jede, über der weniger als die Durchgangshöhe frei wäre.
    for step in &layout.steps {
        if z0 + step.top + HEADROOM > bottom + 1e-9 {
            parts.push(ring(&step.outline));
        }
    }
    // 1 cm größer, damit die Vorderkante der ersten Stufe darunter eindeutig im Loch liegt.
    let union = Area::union_all(parts.iter()).offset(0.01);
    let outer = union
        .rings()
        .into_iter()
        .filter(|r| lot_geom::signed_area(r) > 0.0)
        .max_by(|a, b| Area::from_ring(a).area().total_cmp(&Area::from_ring(b).area()))?;
    Some(outer.into_iter().map(|p| Point2::new(p[0], p[1])).collect())
}

/// Umwehrung um das Loch: alle Kanten außer der am Austritt, als offener Linienzug.
pub fn guard_points(project: &Project, stair: &Stair, hole: &[Point2]) -> Option<Vec<Point2>> {
    let layout = project.stair_layout(stair)?;
    let exit = exit_point(&layout)?;
    let n = hole.len();
    if n < 3 {
        return None;
    }
    let distance = |i: usize| {
        let (a, b) = (hole[i], hole[(i + 1) % n]);
        segment_distance(a, b, exit)
    };
    let skip = (0..n).min_by(|a, b| distance(*a).total_cmp(&distance(*b)))?;
    // Ab dem Ende der Austrittskante einmal herum bis zu ihrem Anfang.
    Some((1..=n).map(|k| hole[(skip + k) % n]).collect())
}

fn segment_distance(a: Point2, b: Point2, p: Point2) -> f64 {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let len2 = (dx * dx + dy * dy).max(1e-18);
    let t = (((p.x - a.x) * dx + (p.y - a.y) * dy) / len2).clamp(0.0, 1.0);
    p.distance(Point2::new(a.x + dx * t, a.y + dy * t))
}

/// Seiten der Läufe, an denen keine Wand steht: dort kommt ein Geländer hin.
pub fn free_sides(project: &Project, stair: &Stair) -> Vec<(usize, WallSide)> {
    let Some(layout) = project.stair_layout(stair) else { return Vec::new() };
    let z = project.storey(stair.storey).map_or(0.0, |s| s.elevation) + 1.0;
    let walls = lot_geom::solid_regions(project, stair.storey, z);
    let in_wall = |p: Point2| walls.iter().any(|r| project.wall(r.element).is_some() && r.area.contains([p.x, p.y]));
    let mut out = Vec::new();
    for (i, f) in layout.flights.iter().enumerate() {
        for side in [WallSide::Left, WallSide::Right] {
            let [(a, _), (b, _)] = layout.flight_side(f, side, -0.05);
            let samples = [a, Point2::new((a.x + b.x) / 2.0, (a.y + b.y) / 2.0), b];
            if !samples.iter().any(|p| in_wall(*p)) {
                out.push((i, side));
            }
        }
    }
    out
}

/// Operationen für eine neue Treppe mit Loch, Umwehrung und Geländern, und die Kennung der Treppe.
pub fn create_ops(project: &Project, stair: Stair) -> (ElementId, Vec<Operation>) {
    let id = project.next_id();
    let mut next = id.0 + 1;
    let mut fresh = || {
        let i = ElementId(next);
        next += 1;
        i
    };
    let mut scratch = project.clone();
    scratch.elements.insert(id, Element::Stair(stair.clone()));
    let mut ops = vec![Operation::Insert { id, element: Element::Stair(stair.clone()) }];
    for (flight, side) in free_sides(&scratch, &stair) {
        let railing = Railing {
            storey: stair.storey,
            path: RailingPath::Flight { stair: id, flight, side },
            height: RAILING_HEIGHT,
            phase: Default::default(),
            around: None,
        };
        ops.push(Operation::Insert { id: fresh(), element: Element::Railing(railing) });
    }
    if let (Some((host, _)), Some(outline)) = (slab_above(&scratch, &stair), hole_outline(&scratch, &stair)) {
        let opening_id = fresh();
        let guard = guard_points(&scratch, &stair, &outline);
        let host_storey = match scratch.elements.get(&host) {
            Some(Element::Slab(s)) => s.storey,
            _ => stair.storey,
        };
        ops.push(Operation::Insert {
            id: opening_id,
            element: Element::SlabOpening(SlabOpening { host, outline, phase: Default::default(), stair: Some(id) }),
        });
        if let Some(points) = guard {
            let railing = Railing {
                storey: host_storey,
                path: RailingPath::Points(points),
                height: RAILING_HEIGHT,
                phase: Default::default(),
                around: Some(opening_id),
            };
            ops.push(Operation::Insert { id: fresh(), element: Element::Railing(railing) });
        }
    }
    (id, ops)
}

/// Nachführen (siehe [`crate::follow`]): Treppenlöcher folgen ihrer Treppe,
/// Umwehrungen ihrem Loch; ohne Treppe verschwinden Loch und Umwehrung.
pub fn follow_ops(project: &Project) -> Vec<Operation> {
    let mut ops = Vec::new();
    let mut holes: std::collections::BTreeMap<ElementId, Option<Vec<Point2>>> = Default::default();
    for (id, e) in &project.elements {
        let Element::SlabOpening(o) = e else { continue };
        let Some(stair_id) = o.stair else { continue };
        let Some(stair) = project.stair(stair_id) else {
            ops.push(Operation::Delete { id: *id, element: e.clone() });
            holes.insert(*id, None);
            continue;
        };
        match hole_outline(project, stair) {
            Some(outline) => {
                if !same(&outline, &o.outline) {
                    let mut n = o.clone();
                    n.outline = outline.clone();
                    ops.push(Operation::Replace { id: *id, old: e.clone(), new: Element::SlabOpening(n) });
                }
                holes.insert(*id, Some(outline));
            }
            None => {
                holes.insert(*id, Some(o.outline.clone()));
            }
        }
    }
    for (id, e) in &project.elements {
        let Element::Railing(r) = e else { continue };
        let Some(hole) = r.around else { continue };
        let outline = match holes.get(&hole) {
            Some(Some(outline)) => outline.clone(),
            Some(None) => {
                ops.push(Operation::Delete { id: *id, element: e.clone() });
                continue;
            }
            None => match project.elements.get(&hole) {
                Some(Element::SlabOpening(o)) => o.outline.clone(),
                _ => {
                    ops.push(Operation::Delete { id: *id, element: e.clone() });
                    continue;
                }
            },
        };
        let stair = match project.elements.get(&hole) {
            Some(Element::SlabOpening(o)) => o.stair.and_then(|s| project.stair(s)),
            _ => None,
        };
        let Some(points) = stair.and_then(|s| guard_points(project, s, &outline)) else { continue };
        if let RailingPath::Points(old) = &r.path
            && same(old, &points)
        {
            continue;
        }
        let mut n = r.clone();
        n.path = RailingPath::Points(points);
        ops.push(Operation::Replace { id: *id, old: e.clone(), new: Element::Railing(n) });
    }
    ops
}

fn same(a: &[Point2], b: &[Point2]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(p, q)| p.distance(*q) < 1e-3)
}

/// Wo die Treppe oben ankommt: Mitte der Vorderkante des Austritts.
fn exit_point(layout: &lot_model::StairLayout) -> Option<Point2> {
    match layout.flights.last() {
        Some(f) => Some(layout.end(f)),
        None => layout.walking.last().copied(),
    }
}
