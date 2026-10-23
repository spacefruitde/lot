//! Abstandsflächen (5.01) nach Landesrecht, als Voreinstellung MBO § 6.
//!
//! Jede Außenwand jedes Geschosses wirft eine Abstandsfläche senkrecht vor
//! sich. Ihre Tiefe T ist an jeder Stelle der Wand `factor · H`, mindestens
//! `minimum` (MBO § 6 Abs. 5 Satz 1). H ist die Wandhöhe vom Gelände bis zum
//! oberen Abschluss der Wand oder bis zur Dachhaut (§ 6 Abs. 4 Satz 2); steile
//! Dächer zählen zu einem Drittel oder ganz dazu (§ 6 Abs. 4 Satz 3 und 4).
//! Die Ecken bleiben frei, wie es die MBO vorsieht.
//!
//! H wird aus der 3D-Geometrie gemessen: über jedem Punkt der Wand die
//! höchste Kante des Gebäudes, gleich hinter der Außenfläche. Dachüberstände
//! und Gesimse vor der Wand zählen so nicht mit (§ 6 Abs. 6 Nr. 1).
//!
//! Die Abstandsflächen müssen auf dem Baugrundstück liegen, auf öffentlichen
//! Verkehrs-, Grün- und Wasserflächen bis zu deren Mitte (§ 6 Abs. 2).

use lot_geom::{Area, ModelGeometry};
use lot_model::{ElementId, Plot, Point2, Project, RoofEdge};

use crate::building::{BuildingClassResult, TopProbe, compass, ground_at, storey_outline};
use crate::landesrecht::StateRules;

/// Abstand der Messpunkte entlang der Wand, in m.
const STEP: f64 = 0.25;
/// Abstand hinter der Außenfläche, in dem die Oberkante gemessen wird, in m.
const INSET: f64 = 0.01;

/// Ein Messpunkt entlang einer Außenwand.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SetbackSample {
    /// Abstand vom Wandanfang in m.
    pub along: f64,
    /// Gelände vor der Wand über ±0,00.
    pub ground: f64,
    /// Oberer Abschluss der Wand oder Schnitt mit der Dachhaut über ±0,00.
    pub top: f64,
    /// Wandhöhe H einschließlich Dachanteil, in m.
    pub height: f64,
    /// Tiefe T, in m.
    pub depth: f64,
}

/// Abstandsfläche einer Außenwand (Glossar: `SetbackArea`).
#[derive(Debug, Clone, PartialEq)]
pub struct SetbackArea {
    pub storey: ElementId,
    /// Außenfläche der Wand, gesehen von innen nach außen von links nach rechts.
    pub start: Point2,
    pub end: Point2,
    /// Einheitsvektor nach außen.
    pub outward: [f64; 2],
    /// Himmelsrichtung, in die die Wand schaut, etwa „Süd“.
    pub facing: &'static str,
    pub samples: Vec<SetbackSample>,
    /// Zur Wandhöhe gezählter Dachanteil in m (0 bei flachen Dächern).
    pub roof_share: f64,
    pub area: Area,
    /// Kleinster Abstand von der Wand bis zur Grenze der zulässigen Fläche
    /// (Grundstücksgrenze oder Mitte der öffentlichen Fläche), in m.
    pub available: Option<f64>,
    /// Teil der Abstandsfläche außerhalb der zulässigen Fläche.
    pub outside: Area,
}

impl SetbackArea {
    pub fn length(&self) -> f64 {
        self.start.distance(self.end)
    }

    /// Kleinste und größte Wandhöhe.
    pub fn height(&self) -> (f64, f64) {
        range(self.samples.iter().map(|s| s.height))
    }

    /// Kleinste und größte Tiefe.
    pub fn depth(&self) -> (f64, f64) {
        range(self.samples.iter().map(|s| s.depth))
    }

    pub fn ok(&self) -> bool {
        self.outside.area() < 1e-4
    }
}

fn range(values: impl Iterator<Item = f64>) -> (f64, f64) {
    values.fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), v| (lo.min(v), hi.max(v)))
}

/// Alle Abstandsflächen eines Projekts.
#[derive(Debug, Clone, PartialEq)]
pub struct SetbackReport {
    pub rules: &'static StateRules,
    /// Das Land im Projekt hat keine Daten, es gilt die Musterbauordnung.
    pub fallback: bool,
    pub areas: Vec<SetbackArea>,
    /// Wo Abstandsflächen liegen dürfen; `None` ohne Baugrundstück.
    pub allowed: Option<Area>,
    /// Alle Abstandsflächen vereinigt.
    pub union: Area,
    /// Teile außerhalb der zulässigen Fläche (im Lageplan rot).
    pub violations: Area,
    /// Ob die feste Tiefe für kleine Wohngebäude gilt.
    pub small_building: bool,
    pub notes: Vec<String>,
}

impl SetbackReport {
    pub fn ok(&self) -> bool {
        self.violations.area() < 1e-4
    }
}

/// Das Baugrundstück, falls es eins gibt.
pub fn building_plot(project: &Project) -> Option<&Plot> {
    project.elements.values().find_map(|e| match e {
        lot_model::Element::Plot(p) if !p.neighbour => Some(p),
        _ => None,
    })
}

/// Fläche, auf der Abstandsflächen liegen dürfen: das Grundstück und, falls das
/// Land es erlaubt, die öffentlichen Flächen davor bis zu ihrer Mitte.
pub fn allowed_area(plot: &Plot, to_center: bool) -> Area {
    let ring: Vec<[f64; 2]> = plot.outline.iter().map(|p| [p.x, p.y]).collect();
    let mut area = Area::from_ring(&ring);
    if !to_center {
        return area;
    }
    let ccw = lot_geom::signed_area(&ring) > 0.0;
    for e in &plot.public_edges {
        let Some((a, b)) = plot.edge(e.edge) else { continue };
        let len = a.distance(b);
        if len < 1e-9 || e.width <= 0.0 {
            continue;
        }
        let dir = [(b.x - a.x) / len, (b.y - a.y) / len];
        // Links der Kante liegt bei einem Umriss gegen den Uhrzeigersinn das Grundstück.
        let across = if ccw { (0.0, -e.width / 2.0) } else { (0.0, e.width / 2.0) };
        area = area.union(&lot_geom::poly::strip([a.x, a.y], dir, (0.0, len), across));
    }
    area
}

/// Höhe eines Dachs über der Traufe, in m: höchster Punkt der Dachhaut minus Traufhöhe.
fn roof_rise(roof: &lot_model::Roof) -> f64 {
    let (mut lo, mut hi) = ([f64::INFINITY; 2], [f64::NEG_INFINITY; 2]);
    for p in &roof.outline {
        lo = [lo[0].min(p.x), lo[1].min(p.y)];
        hi = [hi[0].max(p.x), hi[1].max(p.y)];
    }
    let ring: Vec<[f64; 2]> = roof.outline.iter().map(|p| [p.x, p.y]).collect();
    let inside = Area::from_ring(&ring);
    let mut top = roof.eave_height;
    let n = 40;
    for i in 0..=n {
        for j in 0..=n {
            let p = [lo[0] + (hi[0] - lo[0]) * i as f64 / n as f64, lo[1] + (hi[1] - lo[1]) * j as f64 / n as f64];
            if inside.contains(p) {
                top = top.max(roof.top_at(Point2::new(p[0], p[1])));
            }
        }
    }
    top - roof.eave_height
}

/// Dachanteil an der Wandhöhe für eine Wand auf der Linie a-b, nach außen `n`.
fn roof_share(project: &Project, storey: ElementId, a: Point2, b: Point2, n: [f64; 2], rules: &StateRules) -> f64 {
    let s = &rules.setbacks;
    let mut share: f64 = 0.0;
    for (_, roof) in project.roofs_on(storey) {
        for (i, edge) in roof.edges.iter().enumerate() {
            let RoofEdge::Eave { slope, .. } = *edge else { continue };
            let factor = if slope > s.roof_full_above {
                1.0
            } else if slope > s.roof_third_above {
                1.0 / 3.0
            } else {
                continue;
            };
            let (p, q) = (roof.outline[i], roof.outline[(i + 1) % roof.outline.len()]);
            let inward = roof.inward(i);
            // Dieselbe Linie, dieselbe Seite.
            let on_line = |x: Point2| ((x.x - p.x) * inward.0 + (x.y - p.y) * inward.1).abs() < 0.05;
            let same_side = n[0] * inward.0 + n[1] * inward.1 < -0.99;
            if same_side && on_line(a) && on_line(b) && on_line(q) {
                share = share.max(factor * roof_rise(roof));
            }
        }
    }
    share
}

/// Rechnet alle Abstandsflächen.
pub fn setbacks(project: &Project, geometry: &ModelGeometry, class: &BuildingClassResult) -> SetbackReport {
    let (rules, fallback) = StateRules::for_state(&project.info().state);
    let s = &rules.setbacks;
    let mut notes = Vec::new();
    if fallback {
        notes.push(format!("Für das Land „{}“ fehlen Daten; es gilt die {}", project.info().state, rules.name));
    }
    if project.terrain().is_none() {
        notes.push("Kein Gelände im Modell: Wandhöhen ab ±0,00".to_string());
    }
    let residential = project.info().building == lot_model::BuildingClass::SmallResidential
        || project
            .elements
            .values()
            .filter_map(|e| match e {
                lot_model::Element::Space(r) => Some(r.usage),
                _ => None,
            })
            .all(|u| !crate::building::is_habitable(u) || u == lot_model::SpaceUsage::Living);
    let small_building = s
        .small_buildings
        .is_some_and(|sb| residential && class.class <= sb.max_class && class.above_ground.len() <= sb.max_storeys);
    let depth_of = |h: f64| match s.small_buildings {
        Some(sb) if small_building => sb.depth,
        _ => (s.factor * h).max(s.minimum),
    };

    let probe = TopProbe::new(project, geometry);
    let mut candidates: Vec<SetbackArea> = Vec::new();
    let mut storeys = project.storeys();
    storeys.sort_by(|a, b| b.1.elevation.total_cmp(&a.1.elevation));
    for (storey, _) in &storeys {
        let outline = storey_outline(project, *storey);
        for ring in outline.rings() {
            let ccw = lot_geom::signed_area(&ring) > 0.0;
            if !ccw {
                continue;
            }
            for i in 0..ring.len() {
                let (a, b) = (ring[i], ring[(i + 1) % ring.len()]);
                let len = (b[0] - a[0]).hypot(b[1] - a[1]);
                if len < 0.05 {
                    continue;
                }
                let d = [(b[0] - a[0]) / len, (b[1] - a[1]) / len];
                // Gegen den Uhrzeigersinn liegt das Gebäude links, außen ist rechts.
                let n = [d[1], -d[0]];
                let (pa, pb) = (Point2::new(a[0], a[1]), Point2::new(b[0], b[1]));
                let share = roof_share(project, *storey, pa, pb, n, rules);
                let count = (len / STEP).ceil().max(1.0) as usize;
                let samples: Vec<SetbackSample> = (0..=count)
                    .filter_map(|k| {
                        let along = len * k as f64 / count as f64;
                        // An den Ecken knapp innerhalb messen, damit die Nachbarwand nicht zählt.
                        let t = along.clamp(INSET, len - INSET);
                        let at = |off: f64| [a[0] + d[0] * t - n[0] * off, a[1] + d[1] * t - n[1] * off];
                        let (p1, p2) = (at(INSET), at(2.0 * INSET));
                        let z1 = probe.top(p1[0], p1[1])?;
                        let z2 = probe.top(p2[0], p2[1]).unwrap_or(z1);
                        // Auf die Außenfläche hochgerechnet, bei schrägen Dachflächen genau.
                        let top = if (z1 - z2).abs() < 0.05 { 2.0 * z1 - z2 } else { z1 };
                        let outside = Point2::new(a[0] + d[0] * t + n[0] * 0.05, a[1] + d[1] * t + n[1] * 0.05);
                        let ground = ground_at(project, outside);
                        let height = (top - ground).max(0.0) + share;
                        Some(SetbackSample { along, ground, top, height, depth: depth_of(height) })
                    })
                    .collect();
                if samples.len() < 2 {
                    continue;
                }
                let mut ring_out: Vec<[f64; 2]> = vec![a, b];
                for smp in samples.iter().rev() {
                    let base = [a[0] + d[0] * smp.along, a[1] + d[1] * smp.along];
                    ring_out.push([base[0] + n[0] * smp.depth, base[1] + n[1] * smp.depth]);
                }
                candidates.push(SetbackArea {
                    storey: *storey,
                    start: pa,
                    end: pb,
                    outward: n,
                    facing: compass(project, n),
                    samples,
                    roof_share: share,
                    area: Area::from_ring(&ring_out),
                    available: None,
                    outside: Area::empty(),
                });
            }
        }
    }

    // Wände gleicher Flucht aus tieferen Geschossen fallen weg, wenn eine höhere
    // Wand sie mit mindestens gleicher Höhe überdeckt.
    let mut areas: Vec<SetbackArea> = Vec::new();
    for c in candidates {
        let covered = areas.iter().any(|k| covers(k, &c));
        if !covered {
            areas.push(c);
        }
    }

    let plot = building_plot(project);
    if plot.is_none() {
        notes.push("Kein Baugrundstück im Modell: Abstandsflächen ohne Prüfung gegen die Grenzen".to_string());
    }
    let allowed = plot.map(|p| allowed_area(p, s.public_to_center));
    if let Some(allowed) = &allowed {
        for a in &mut areas {
            a.outside = a.area.difference(allowed);
            a.available = a
                .samples
                .iter()
                .filter_map(|smp| {
                    let base = [
                        a.start.x + (a.end.x - a.start.x) * smp.along / a.length(),
                        a.start.y + (a.end.y - a.start.y) * smp.along / a.length(),
                    ];
                    exit_distance(allowed, base, a.outward)
                })
                .reduce(f64::min);
        }
    }
    let union = Area::union_all(areas.iter().map(|a| &a.area));
    let violations = Area::union_all(areas.iter().map(|a| &a.outside));
    notes.push(
        "Nicht geprüft: Überdeckung eigener Abstandsflächen (§ 6 Abs. 3), Vorbauten und Dachaufbauten (§ 6 Abs. 6)"
            .to_string(),
    );
    SetbackReport { rules, fallback, areas, allowed, union, violations, small_building, notes }
}

/// Ob `k` die Wand `c` überdeckt: gleiche Flucht, gleiche Richtung, `c` liegt
/// ganz in `k` und ist nirgends höher.
fn covers(k: &SetbackArea, c: &SetbackArea) -> bool {
    let same_dir = k.outward[0] * c.outward[0] + k.outward[1] * c.outward[1] > 0.9999;
    let off = |p: Point2| (p.x - k.start.x) * k.outward[0] + (p.y - k.start.y) * k.outward[1];
    if !same_dir || off(c.start).abs() > 0.005 || off(c.end).abs() > 0.005 {
        return false;
    }
    let d = [(k.end.x - k.start.x) / k.length(), (k.end.y - k.start.y) / k.length()];
    let along = |p: Point2| (p.x - k.start.x) * d[0] + (p.y - k.start.y) * d[1];
    let (s0, s1) = (along(c.start), along(c.end));
    if s0.min(s1) < -0.005 || s0.max(s1) > k.length() + 0.005 {
        return false;
    }
    let depth_at = |s: f64| {
        let samples = &k.samples;
        let i = samples.partition_point(|x| x.along < s).clamp(1, samples.len() - 1);
        let (p, q) = (samples[i - 1], samples[i]);
        let t = if q.along > p.along { ((s - p.along) / (q.along - p.along)).clamp(0.0, 1.0) } else { 0.0 };
        p.depth + (q.depth - p.depth) * t
    };
    c.samples.iter().all(|smp| {
        let s = s0 + (s1 - s0) * smp.along / c.length();
        smp.depth <= depth_at(s) + 0.01
    })
}

/// Abstand von `p` in Richtung `dir` bis zum Rand der Fläche, falls `p` darin liegt.
fn exit_distance(area: &Area, p: [f64; 2], dir: [f64; 2]) -> Option<f64> {
    let mut best: Option<f64> = None;
    for ring in area.rings() {
        for i in 0..ring.len() {
            let (a, b) = (ring[i], ring[(i + 1) % ring.len()]);
            let e = [b[0] - a[0], b[1] - a[1]];
            let den = dir[0] * e[1] - dir[1] * e[0];
            if den.abs() < 1e-12 {
                continue;
            }
            let w = [a[0] - p[0], a[1] - p[1]];
            let t = (w[0] * e[1] - w[1] * e[0]) / den;
            let u = (w[0] * dir[1] - w[1] * dir[0]) / den;
            if t > 1e-6 && (-1e-9..=1.0 + 1e-9).contains(&u) {
                best = Some(best.map_or(t, |m| m.min(t)));
            }
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use lot_model::{Element, Plot, PublicEdge, Terrain, TerrainPoint};

    use super::*;
    use crate::building::building_class;

    /// Würfel aus dem Beispielprojekt mit ebenem Gelände und Grundstück drumherum.
    fn project(gap: f64) -> Project {
        let mut p = Project::sample();
        let (lo, hi) = lot_geom::plan_extent(&p).unwrap();
        p.elements.insert(
            ElementId(9001),
            Element::Plot(Plot {
                name: "Flurstück 1".into(),
                outline: vec![
                    Point2::new(lo[0] - gap, lo[1] - 10.0),
                    Point2::new(hi[0] + 10.0, lo[1] - 10.0),
                    Point2::new(hi[0] + 10.0, hi[1] + 10.0),
                    Point2::new(lo[0] - gap, hi[1] + 10.0),
                ],
                public_edges: vec![],
                neighbour: false,
            }),
        );
        let t = |x, y| TerrainPoint { x, y, z: -0.5 };
        p.elements.insert(
            ElementId(9002),
            Element::Terrain(Terrain { points: vec![t(-50.0, -50.0), t(50.0, -50.0), t(50.0, 50.0), t(-50.0, 50.0)] }),
        );
        p
    }

    #[test]
    fn sample_walls_need_minimum_depth() {
        let p = project(10.0);
        let g = lot_geom::build(&p).unwrap();
        let (rules, _) = StateRules::for_state("");
        let class = building_class(&p, rules);
        let r = setbacks(&p, &g, &class);
        assert!(!r.areas.is_empty());
        for a in &r.areas {
            let (lo, hi) = a.height();
            assert!(lo > 0.5 && hi < 7.5, "{lo} {hi}");
            assert!(a.depth().0 >= 3.0 - 1e-9);
        }
        assert!(r.ok(), "{:?}", r.violations.area());
    }

    #[test]
    fn close_boundary_is_a_violation() {
        let p = project(2.0);
        let g = lot_geom::build(&p).unwrap();
        let (rules, _) = StateRules::for_state("");
        let r = setbacks(&p, &g, &building_class(&p, rules));
        assert!(!r.ok());
        let west: Vec<_> = r.areas.iter().filter(|a| a.facing == "West").collect();
        assert!(!west.is_empty());
        assert!(west.iter().all(|a| !a.ok() && a.available.unwrap() < 2.0 + 1e-6));
    }

    #[test]
    fn public_edge_counts_to_its_center() {
        let plot = Plot {
            name: String::new(),
            outline: vec![
                Point2::new(0.0, 0.0),
                Point2::new(10.0, 0.0),
                Point2::new(10.0, 10.0),
                Point2::new(0.0, 10.0),
            ],
            public_edges: vec![PublicEdge { edge: 0, width: 8.0, kind: None, name: String::new() }],
            neighbour: false,
        };
        let a = allowed_area(&plot, true);
        assert!((a.area() - 140.0).abs() < 1e-6);
        assert!(a.contains([5.0, -3.9]));
        assert!((allowed_area(&plot, false).area() - 100.0).abs() < 1e-6);
    }
}
