//! Treppen nach DIN 18065:2020-08 „Gebäudetreppen; Begriffe, Messregeln,
//! Hauptmaße“ (1.07).
//!
//! Geprüft werden Steigung und Auftritt gegen die Grenzwerte der Tabelle 1,
//! das Schrittmaß 2s + a, die Laufbreite und die lichte Treppendurchgangshöhe.
//! Die Laufbreite wird als Abstand der Laufseiten genommen; Handläufe, die
//! hineinragen, zählen noch nicht (Prüfpaket M3).

use lot_geom::Area;
use lot_model::{BuildingClass, Element, ElementId, Point2, Project, Stair};

/// Ergebnis der Prüfung einer Treppe. Maße in m.
#[derive(Debug, Clone, PartialEq)]
pub struct StairCheck {
    pub stair: ElementId,
    pub risers: u32,
    /// Steigung s
    pub rise: f64,
    /// Auftritt a
    pub tread: f64,
    /// Schrittmaß 2s + a
    pub step: f64,
    /// Kleinste lichte Durchgangshöhe über den Stufenvorderkanten und Podesten;
    /// `None`, wenn nichts darüber liegt.
    pub headroom: Option<f64>,
    /// Abweichungen, leer wenn die Treppe den Grenzwerten entspricht.
    pub problems: Vec<String>,
}

/// Grenzwerte einer Zeile der Tabelle 1, in m.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Limits {
    pub width: f64,
    pub rise: (f64, f64),
    pub tread: (f64, f64),
}

/// DIN 18065:2020-08, Tabelle 1: nutzbare Laufbreite, Steigung und Auftritt
/// je Gebäudeart und Treppenart. Steigung ab 14 cm und Auftritt bis 37 cm gelten
/// für alle Zeilen.
pub fn limits(building: BuildingClass, necessary: bool) -> Limits {
    match (building, necessary) {
        (BuildingClass::SmallResidential, true) => Limits { width: 0.80, rise: (0.14, 0.20), tread: (0.23, 0.37) },
        (BuildingClass::General, true) => Limits { width: 1.00, rise: (0.14, 0.19), tread: (0.26, 0.37) },
        (_, false) => Limits { width: 0.50, rise: (0.14, 0.21), tread: (0.21, 0.37) },
    }
}

/// Schrittmaßregel nach DIN 18065:2020-08: 59 cm ≤ 2s + a ≤ 65 cm.
pub const STEP_RANGE: (f64, f64) = (0.59, 0.65);
/// Lichte Treppendurchgangshöhe nach DIN 18065:2020-08: mindestens 2,00 m.
const MIN_HEADROOM: f64 = 2.00;

/// Prüft alle Treppen des Projekts.
pub fn check_stairs(project: &Project) -> Vec<StairCheck> {
    project
        .elements
        .iter()
        .filter_map(|(id, e)| match e {
            Element::Stair(s) => check_stair(project, *id, s),
            _ => None,
        })
        .collect()
}

pub fn check_stair(project: &Project, id: ElementId, stair: &Stair) -> Option<StairCheck> {
    let layout = project.stair_layout(stair)?;
    let z0 = project.storey(stair.storey)?.elevation;
    let limits = limits(project.info().building, stair.necessary);
    let (s, a) = (layout.rise, layout.tread);
    let step = 2.0 * s + a;
    let mut problems = Vec::new();
    let cm = |v: f64| lot_format(v * 100.0, 1);
    if s < limits.rise.0 - 1e-9 || s > limits.rise.1 + 1e-9 {
        problems.push(format!(
            "Steigung {} cm außerhalb {} bis {} cm (DIN 18065:2020-08, Tabelle 1)",
            cm(s),
            cm(limits.rise.0),
            cm(limits.rise.1)
        ));
    }
    if a < limits.tread.0 - 1e-9 || a > limits.tread.1 + 1e-9 {
        problems.push(format!(
            "Auftritt {} cm außerhalb {} bis {} cm (DIN 18065:2020-08, Tabelle 1)",
            cm(a),
            cm(limits.tread.0),
            cm(limits.tread.1)
        ));
    }
    if step < STEP_RANGE.0 - 1e-9 || step > STEP_RANGE.1 + 1e-9 {
        problems.push(format!("Schrittmaß 2s + a = {} cm außerhalb 59 bis 65 cm (DIN 18065:2020-08)", cm(step)));
    }
    // Nutzbare Laufbreite; bei Spindeltreppen ab der inneren Begrenzung (6.2.4).
    if layout.width < limits.width - 1e-9 {
        problems.push(format!(
            "Laufbreite {} cm unter {} cm (DIN 18065:2020-08, Tabelle 1)",
            cm(layout.width),
            cm(limits.width)
        ));
    }
    // Lichte Durchgangshöhe: über jeder Stufenvorderkante an beiden Seiten und
    // in der Mitte, auf Podesten an den Ecken und in der Mitte.
    let mut samples: Vec<(Point2, f64)> = Vec::new();
    // Wendel- und Spindelstufen (ADR 0051): an jeder Vorderkante innen, Mitte, außen.
    for step in &layout.steps {
        let (i, o) = step.front;
        let (i, o) = (add_toward(i, o, 0.01), add_toward(o, i, 0.01));
        for p in [i, Point2::new((i.x + o.x) / 2.0, (i.y + o.y) / 2.0), o] {
            samples.push((p, z0 + step.top));
        }
    }
    // Mindestauftritt an der inneren Begrenzung (DIN 18065:2020-08, Tabelle 1):
    // Wendelstufen Nr. 6.2.1, Spindeltreppen Nr. 6.2.4 (in Wohngebäuden ohne Anforderung).
    let min_inner = match (stair.spiral.is_some(), project.info().building) {
        (false, BuildingClass::General) => Some((0.10, "6.2.1")),
        (false, BuildingClass::SmallResidential) => Some((0.05, "6.2.1")),
        (true, BuildingClass::General) => Some((0.10, "6.2.4")),
        (true, BuildingClass::SmallResidential) => None,
    };
    if let Some((min, nr)) = min_inner
        && let Some(narrowest) = layout.steps.iter().map(|s| s.inner).reduce(f64::min)
        && narrowest < min - 1e-6
    {
        problems.push(format!(
            "Auftritt an der inneren Begrenzung {} cm unter {} cm (DIN 18065:2020-08, Tabelle 1, Nr. {nr})",
            cm(narrowest),
            cm(min)
        ));
    }
    let h = layout.width / 2.0 - 0.01;
    for f in &layout.flights {
        let left = Point2::new(-f.dir.y, f.dir.x);
        for k in 1..=f.risers {
            let c = add(f.start, f.dir, (k - 1) as f64 * a);
            for across in [-h, 0.0, h] {
                samples.push((add(c, left, across), z0 + f.base + k as f64 * s));
            }
        }
    }
    for (outline, top) in &layout.landings {
        let n = outline.len() as f64;
        let center =
            Point2::new(outline.iter().map(|p| p.x).sum::<f64>() / n, outline.iter().map(|p| p.y).sum::<f64>() / n);
        samples.push((center, z0 + top));
        for p in outline {
            samples.push((Point2::new(p.x + (center.x - p.x) * 0.02, p.y + (center.y - p.y) * 0.02), z0 + top));
        }
    }
    let obstacles = Obstacles::new(project);
    let headroom =
        samples.iter().filter_map(|&(p, z)| obstacles.lowest_above(p, z + 0.01).map(|o| o - z)).reduce(f64::min);
    if let Some(clear) = headroom
        && clear < MIN_HEADROOM - 1e-9
    {
        problems.push(format!("Lichte Durchgangshöhe {} m unter 2,00 m (DIN 18065:2020-08)", lot_format(clear, 2)));
    }
    Some(StairCheck { stair: id, risers: stair.risers(), rise: s, tread: a, step, headroom, problems })
}

/// Was über einer Treppe liegen kann: Decken (ohne Durchbrüche), Läufe und
/// Podeste aller Treppen, Dächer.
struct Obstacles<'a> {
    project: &'a Project,
    /// Decken: Grundfläche und Unterkante über ±0,00.
    slabs: Vec<(Area, f64)>,
}

impl<'a> Obstacles<'a> {
    fn new(project: &'a Project) -> Self {
        let slabs = project
            .elements
            .iter()
            .filter_map(|(id, e)| match e {
                Element::Slab(slab) => {
                    let top = project.storey(slab.storey)?.elevation + slab.top_offset;
                    let thickness: f64 = project.slab_type(slab.slab_type)?.layers.iter().map(|l| l.thickness).sum();
                    Some((lot_geom::slabs::slab_area(project, *id, slab), top - thickness))
                }
                _ => None,
            })
            .collect();
        Obstacles { project, slabs }
    }

    /// Unterseite des niedrigsten Hindernisses über `z` an der Stelle `p`.
    fn lowest_above(&self, p: Point2, z: f64) -> Option<f64> {
        let mut lowest: Option<f64> = None;
        let mut take = |v: f64| {
            if v > z {
                lowest = Some(lowest.map_or(v, |l: f64| l.min(v)));
            }
        };
        for (area, bottom) in &self.slabs {
            if area.contains([p.x, p.y]) {
                take(*bottom);
            }
        }
        for e in self.project.elements.values() {
            match e {
                Element::Stair(other) => {
                    let (Some(layout), Some(storey)) =
                        (self.project.stair_layout(other), self.project.storey(other.storey))
                    else {
                        continue;
                    };
                    let z0 = storey.elevation;
                    let cos = layout.tread / layout.tread.hypot(layout.rise);
                    let vertical = other.thickness / cos;
                    // Der Lauf unter dem Prüfpunkt liegt selbst darunter und fällt heraus.
                    for f in &layout.flights {
                        let left = Point2::new(-f.dir.y, f.dir.x);
                        let (dx, dy) = (p.x - f.start.x, p.y - f.start.y);
                        let u = dx * f.dir.x + dy * f.dir.y;
                        let across = dx * left.x + dy * left.y;
                        let inside =
                            u > 1e-6 && u < layout.flight_length(f) - 1e-6 && across.abs() < layout.width / 2.0;
                        if inside {
                            let under = z0 + f.base + layout.rise - vertical + layout.rise / layout.tread * u;
                            take(under);
                        }
                    }
                    for (outline, top) in &layout.landings {
                        let ring: Vec<[f64; 2]> = outline.iter().map(|q| [q.x, q.y]).collect();
                        if Area::from_ring(&ring).contains([p.x, p.y]) {
                            take(z0 + top - other.thickness);
                        }
                    }
                }
                Element::Roof(roof) => {
                    let Some(storey) = self.project.storey(roof.storey) else { continue };
                    let outline: Vec<[f64; 2]> = roof.outer_outline().iter().map(|q| [q.x, q.y]).collect();
                    if Area::from_ring(&outline).contains([p.x, p.y]) {
                        take(storey.elevation + roof.underside_at(p, self.project.roof_thickness(roof)));
                    }
                }
                _ => {}
            }
        }
        lowest
    }
}

fn add(p: Point2, d: Point2, t: f64) -> Point2 {
    Point2::new(p.x + d.x * t, p.y + d.y * t)
}

/// Zahl mit Komma, `decimals` Nachkommastellen.
fn lot_format(value: f64, decimals: usize) -> String {
    format!("{value:.decimals$}").replace('.', ",")
}

/// Punkt `a`, um `t` in Richtung `b` gerückt.
fn add_toward(a: Point2, b: Point2, t: f64) -> Point2 {
    let len = a.distance(b).max(1e-12);
    Point2::new(a.x + (b.x - a.x) / len * t, a.y + (b.y - a.y) / len * t)
}

#[cfg(test)]
mod tests {
    use lot_model::{Landing, ProjectInfo, StairFlight};

    use super::*;

    fn project(risers: u32, tread: f64) -> Project {
        let mut p = Project::sample();
        let storey = p.storeys()[0].0;
        if let Some(Element::Storey(s)) = p.elements.get_mut(&storey) {
            s.height = 2.75;
        }
        p.elements.insert(
            ElementId(800),
            Element::ProjectInfo(ProjectInfo { building: BuildingClass::SmallResidential, ..Default::default() }),
        );
        p.elements.insert(
            ElementId(900),
            Element::Stair(Stair {
                phase: Default::default(),
                storey,
                start: Point2::new(0.5, 0.0),
                direction: Point2::new(0.0, 1.0),
                width: 1.0,
                tread,
                flights: vec![
                    StairFlight { risers: risers / 2, landing: Landing::BackRight },
                    StairFlight { risers: risers - risers / 2, landing: Landing::Straight },
                ],
                thickness: 0.18,
                necessary: true,
                spiral: None,
                eye: 0.0,
            }),
        );
        p
    }

    #[test]
    fn good_stair_passes() {
        let checks = check_stairs(&project(16, 0.27));
        assert_eq!(checks.len(), 1);
        let c = &checks[0];
        assert!((c.rise - 0.171875).abs() < 1e-9);
        assert!((c.step - 0.61375).abs() < 1e-9);
        assert!(c.problems.is_empty(), "{:?}", c.problems);
    }

    #[test]
    fn steep_stair_fails_rise_and_step() {
        // 13 Steigungen: 21,2 cm, 2s + a = 66,3 cm.
        let c = &check_stairs(&project(13, 0.24))[0];
        assert_eq!(c.problems.len(), 2, "{:?}", c.problems);
        assert!(c.problems[0].starts_with("Steigung 21,2 cm"));
        assert!(c.problems[1].contains("66,3 cm"));
    }

    #[test]
    fn stricter_limits_for_other_buildings() {
        let mut p = project(16, 0.27);
        p.elements.remove(&ElementId(800));
        let c = check_stair(&p, ElementId(900), p.stair(ElementId(900)).unwrap()).unwrap();
        // Laufbreite 1,00 m reicht, Steigung 17,2 cm und Auftritt 27 cm auch.
        assert!(c.problems.is_empty(), "{:?}", c.problems);
        let mut p = project(16, 0.25);
        p.elements.remove(&ElementId(800));
        let c = check_stair(&p, ElementId(900), p.stair(ElementId(900)).unwrap()).unwrap();
        assert!(c.problems.iter().any(|m| m.starts_with("Auftritt 25,0 cm")), "{:?}", c.problems);
    }

    #[test]
    fn low_slab_above_limits_headroom() {
        // Eine Decke über dem ganzen Lauf: Kopfhöhe reicht nicht.
        let mut p = project(16, 0.27);
        let storey = p.storeys()[0].0;
        let s = p.storey(storey).unwrap().clone();
        let above = ElementId(801);
        p.elements.insert(
            above,
            Element::Storey(lot_model::Storey { name: "OG".into(), elevation: s.elevation + 2.75, height: 2.75 }),
        );
        p.elements.insert(
            ElementId(802),
            Element::SlabType(lot_model::SlabType {
                locked: Default::default(),
                props: Default::default(),
                name: "Decke".into(),
                layers: vec![lot_model::Layer {
                    material: "stahlbeton".into(),
                    thickness: 0.25,
                    function: lot_model::LayerFunction::Structure,
                }],
            }),
        );
        p.elements.insert(
            ElementId(803),
            Element::Slab(lot_model::Slab {
                props: Default::default(),
                phase: Default::default(),
                storey: above,
                slab_type: ElementId(802),
                outline: vec![
                    Point2::new(-1.0, -1.0),
                    Point2::new(4.0, -1.0),
                    Point2::new(4.0, 4.0),
                    Point2::new(-1.0, 4.0),
                ],
                kind: lot_model::SlabKind::Floor,
                follows_walls: false,
                boundary_walls: Vec::new(),
                fall: None,
                top_offset: 0.0,
            }),
        );
        let c = check_stair(&p, ElementId(900), p.stair(ElementId(900)).unwrap()).unwrap();
        let clear = c.headroom.unwrap();
        // Höchste Stufe unter der Deckenunterkante 2,50 ist die 14.: 2,41 m.
        assert!((clear - (2.5 - 14.0 * 2.75 / 16.0)).abs() < 1e-9, "{clear}");
        assert!(c.problems.iter().any(|m| m.starts_with("Lichte Durchgangshöhe")), "{:?}", c.problems);
    }
}
