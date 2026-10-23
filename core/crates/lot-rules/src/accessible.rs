//! Barrierefreiheit (5.03) nach DIN 18040-2, Werte als Daten in
//! `normen/din18040-2.ron`. Ob Wohnungen barrierefrei sein müssen, sagt das
//! Landesrecht (MBO § 50 Abs. 1: Gebäude mit mehr als zwei Wohnungen).
//!
//! Geprüft wird, was das Modell kennt: Erreichbarkeit der Geschosse über
//! Eingang und Aufzug, Fahrkorb und Fläche vor dem Aufzug, lichte Breiten der
//! Türen, Breite der Flure, Bewegungsflächen in den Räumen, Handläufe an
//! Treppen und der Aufschlag von Badtüren. Schwellen, Rampen, Bedienelemente
//! und Ausstattung kennt das Modell nicht.

use std::sync::OnceLock;

use lot_geom::Area;
use lot_model::{Element, ElementId, OpeningKind, Point2, Project, SpaceUsage, WallSide};
use serde::Deserialize;

use crate::landesrecht::StateRules;
use crate::report::{Finding, ReportSection, Status, de};
use crate::rooms::StoreyRooms;

/// Mindestmaße nach DIN 18040-2.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct AccessibleNorm {
    pub source: String,
    pub door_deduction: f64,
    pub window_deduction: f64,
    pub door_common: f64,
    pub door_common_source: String,
    pub door_dwelling: f64,
    pub door_dwelling_source: String,
    pub corridor: f64,
    pub corridor_source: String,
    pub turning: f64,
    pub turning_source: String,
    pub cabin_width: f64,
    pub cabin_depth: f64,
    pub elevator_door: f64,
    pub elevator_front: f64,
    pub elevator_source: String,
    pub handrails_source: String,
    pub bathroom_door_source: String,
}

impl AccessibleNorm {
    pub fn get() -> &'static AccessibleNorm {
        static NORM: OnceLock<AccessibleNorm> = OnceLock::new();
        NORM.get_or_init(|| ron::from_str(include_str!("../normen/din18040-2.ron")).expect("DIN 18040-2 lesbar"))
    }

    fn cite(&self, section: &str) -> String {
        format!("{}, {}", self.source, section)
    }

    /// Lichte Durchgangsbreite einer Tür aus dem Rohbaumaß.
    pub fn clear_door(&self, width: f64) -> f64 {
        width - self.door_deduction
    }
}

/// Ob ein Raum ein Sanitärraum ist: Nutzung „sonstige“ und Name mit Bad, WC oder Dusche.
pub fn is_bathroom(space: &lot_model::Space) -> bool {
    let name = space.name.to_lowercase();
    space.usage == SpaceUsage::Other && ["bad", "wc", "dusch"].iter().any(|k| name.contains(k))
}

/// Ob ein Quadrat mit Kantenlänge `size` in die Fläche passt, in einer der
/// Richtungen ihrer Kanten.
pub fn square_fits(area: &Area, size: f64) -> bool {
    let mut angles: Vec<f64> = vec![0.0];
    for ring in area.rings() {
        for i in 0..ring.len() {
            let (a, b) = (ring[i], ring[(i + 1) % ring.len()]);
            let angle = (b[1] - a[1]).atan2(b[0] - a[0]).to_degrees().rem_euclid(90.0);
            if !angles.iter().any(|x| (x - angle).abs() < 0.5) {
                angles.push(angle);
            }
        }
    }
    angles.iter().any(|a| !area.rotated(-a).offset(-size / 2.0 + 1e-4).is_empty())
}

/// Teile der Fläche, die schmaler als `width` sind (Öffnen: erst schrumpfen, dann wachsen).
pub fn narrow_parts(area: &Area, width: f64) -> Area {
    let opened = area.offset(-width / 2.0 + 1e-4).offset(width / 2.0 - 1e-4);
    area.difference(&opened)
}

fn centroid(area: &Area) -> Point2 {
    let (lo, hi) = area.bounds().unwrap_or(([0.0, 0.0], [0.0, 0.0]));
    let c = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0];
    if area.contains(c) {
        return Point2::new(c[0], c[1]);
    }
    lot_geom::spaces::interior_point(area).map_or(Point2::new(c[0], c[1]), |p| Point2::new(p[0], p[1]))
}

pub fn check_accessibility(project: &Project, rules: &StateRules) -> ReportSection {
    let norm = AccessibleNorm::get();
    let basis = format!("{}, {}", rules.accessible.source, norm.source);
    let mut f = Vec::new();
    let dwellings: Vec<String> = crate::rooms::units(project)
        .into_iter()
        .filter(|u| {
            project
                .elements
                .values()
                .any(|e| matches!(e, Element::Space(s) if s.unit == *u && s.usage == SpaceUsage::Living))
        })
        .collect();
    if dwellings.len() < rules.accessible.min_dwellings {
        f.push(Finding::new(
            Status::Note,
            format!(
                "{} Wohnungen: Barrierefreiheit erst ab {} Wohnungen verlangt",
                dwellings.len(),
                rules.accessible.min_dwellings
            ),
            rules.accessible.source.clone(),
        ));
        return ReportSection { title: "Barrierefreiheit".into(), basis, findings: f };
    }

    let storeys = project.storeys();
    let all: Vec<StoreyRooms> = storeys.iter().map(|(id, _)| StoreyRooms::new(project, *id)).collect();
    let elevators: Vec<&lot_model::Elevator> = project
        .elements
        .values()
        .filter_map(|e| match e {
            Element::Elevator(x) => Some(x),
            _ => None,
        })
        .collect();
    let name = |id: ElementId| project.storey(id).map_or("?".to_string(), |s| s.name.clone());

    // Erreichbarkeit: Eingangsgeschoss mit einer Tür ins Freie, dazu jede Haltestelle eines
    // Aufzugs, der im Eingangsgeschoss hält.
    let entrance: Vec<ElementId> = all
        .iter()
        .filter(|s| {
            s.passages.iter().any(|p| p.to_outside() && matches!(p.opening.kind, OpeningKind::Door(_)) && p.sill < 0.05)
        })
        .map(|s| s.storey)
        .collect();
    let mut reachable: Vec<ElementId> = entrance.iter().take(1).copied().collect();
    for e in &elevators {
        if entrance.iter().any(|s| e.stops_at(*s)) {
            for s in &e.storeys {
                if !reachable.contains(s) {
                    reachable.push(*s);
                }
            }
        }
    }
    let with_dwellings: Vec<ElementId> =
        all.iter().filter(|s| s.rooms.iter().any(|r| dwellings.contains(&r.space.unit))).map(|s| s.storey).collect();
    let reached: Vec<String> = with_dwellings.iter().filter(|s| reachable.contains(s)).map(|s| name(*s)).collect();
    let missing: Vec<String> = with_dwellings.iter().filter(|s| !reachable.contains(s)).map(|s| name(*s)).collect();
    f.push(Finding::new(
        if reached.is_empty() { Status::Fail } else { Status::Ok },
        if missing.is_empty() {
            format!("Wohnungen aller Geschosse stufenlos erreichbar: {}", reached.join(", "))
        } else {
            format!(
                "Stufenlos erreichbar: {}; nicht erreichbar: {}",
                if reached.is_empty() { "keins".to_string() } else { reached.join(", ") },
                missing.join(", ")
            )
        },
        rules.accessible.source.clone(),
    ));
    f.push(Finding::new(
        Status::Note,
        "Schwellen am Hauseingang, Rampen, Bedienelemente und Ausstattung sind nicht im Modell",
        norm.source.clone(),
    ));

    // Aufzug.
    for e in &elevators {
        let cabin_ok = e.cabin_width + 1e-9 >= norm.cabin_width && e.cabin_depth + 1e-9 >= norm.cabin_depth;
        let door_ok = e.door_width + 1e-9 >= norm.elevator_door;
        let mut problems = Vec::new();
        let (d, r) = e.frame();
        let c = e.door_center();
        for s in &all {
            if !e.stops_at(s.storey) {
                continue;
            }
            let h = norm.elevator_front / 2.0;
            let at = |a: f64, b: f64| [c.x + d[0] * a + r[0] * b, c.y + d[1] * a + r[1] * b];
            // Fläche vor der Schachttür, ab der Wandfläche: dem ersten Punkt vor dem
            // Fahrkorb, der in einem anderen Raum als dem Schacht liegt.
            let shaft = s.rooms.iter().position(|x| x.area.contains([e.position.x, e.position.y]));
            let start = (0..100)
                .map(|k| k as f64 * 0.01)
                .find(|t| {
                    let q = at(*t, 0.0);
                    s.rooms.iter().enumerate().any(|(i, x)| Some(i) != shaft && x.area.contains(q))
                })
                .unwrap_or(0.25);
            let square = Area::from_ring(&[
                at(start, -h),
                at(start + norm.elevator_front, -h),
                at(start + norm.elevator_front, h),
                at(start, h),
            ]);
            let probe = at(start + 0.3, 0.0);
            let room = s.rooms.iter().find(|x| x.area.contains(probe));
            let free = room.is_some_and(|x| square.difference(&x.area).area() < 1e-3);
            if !free {
                problems.push(format!(
                    "vor der Tür im {} keine freie Fläche {} x {} m",
                    name(s.storey),
                    de(norm.elevator_front, 2),
                    de(norm.elevator_front, 2)
                ));
            }
        }
        let status = if cabin_ok && door_ok && problems.is_empty() { Status::Ok } else { Status::Fail };
        let mut text = format!(
            "Aufzug: Fahrkorb {} x {} m (mindestens {} x {} m), Tür {} m (mindestens {} m), Fläche vor der Tür {} x {} m an {} Haltestellen",
            de(e.cabin_width, 2),
            de(e.cabin_depth, 2),
            de(norm.cabin_width, 2),
            de(norm.cabin_depth, 2),
            de(e.door_width, 2),
            de(norm.elevator_door, 2),
            de(norm.elevator_front, 2),
            de(norm.elevator_front, 2),
            e.storeys.len()
        );
        if !problems.is_empty() {
            text = format!("{text}; {}", problems.join("; "));
        }
        let first = e.storeys.first().copied();
        let mut finding = Finding::new(status, text, norm.cite(&norm.elevator_source));
        if let Some(st) = first {
            finding = finding.located(st, e.position);
        }
        f.push(finding);
    }
    if elevators.is_empty() && with_dwellings.len() > 1 {
        f.push(Finding::new(
            Status::Note,
            "Kein Aufzug: nur die Wohnungen im Eingangsgeschoss sind stufenlos erreichbar",
            rules.accessible.source.clone(),
        ));
    }

    // Türen im Gebäude: Hauseingang, Treppenraum, Wohnungseingänge.
    let mut narrow_common = Vec::new();
    let mut count_common = 0;
    for s in &all {
        let elevator_room = |i: usize| {
            elevators.iter().any(|e| e.stops_at(s.storey) && s.rooms[i].area.contains([e.position.x, e.position.y]))
        };
        for p in s.passages.iter().filter(|p| p.is_door_or_passage()) {
            let unit = |side: Option<usize>| side.map(|i| s.rooms[i].space.unit.clone());
            let (l, r) = (unit(p.left), unit(p.right));
            let common = matches!(l.as_deref(), Some("")) || matches!(r.as_deref(), Some("")) || p.to_outside();
            let inside_unit = l.is_some() && l == r && !l.as_deref().unwrap_or("").is_empty();
            if !common || inside_unit {
                continue;
            }
            if p.left.is_some_and(elevator_room) || p.right.is_some_and(elevator_room) {
                continue;
            }
            count_common += 1;
            let clear = norm.clear_door(p.opening.width);
            if clear + 1e-9 < norm.door_common {
                narrow_common.push(p);
                f.push(
                    Finding::new(
                        Status::Fail,
                        format!(
                            "Tür im {}: lichte Breite {} m, mindestens {} m",
                            name(s.storey),
                            de(clear, 2),
                            de(norm.door_common, 2)
                        ),
                        norm.cite(&norm.door_common_source),
                    )
                    .located(s.storey, p.center),
                );
            }
        }
    }
    if narrow_common.is_empty() && count_common > 0 {
        f.push(Finding::new(
            Status::Ok,
            format!(
                "{count_common} Türen im Gebäude (Hauseingang, Wohnungseingänge) mit lichter Breite ab {} m (Rohbaumaß minus {} m)",
                de(norm.door_common, 2),
                de(norm.door_deduction, 2)
            ),
            norm.cite(&norm.door_common_source),
        ));
    }

    // Flure im Gebäude.
    for s in &all {
        for room in s.rooms.iter().filter(|r| r.space.unit.is_empty() && r.space.usage == SpaceUsage::Circulation) {
            if elevators.iter().any(|e| room.area.contains([e.position.x, e.position.y])) {
                continue;
            }
            let narrow = narrow_parts(&room.area, norm.corridor);
            if narrow.area() > 0.05 {
                f.push(
                    Finding::new(
                        Status::Fail,
                        format!("{} im {}: schmaler als {} m", room.label(), name(s.storey), de(norm.corridor, 2)),
                        norm.cite(&norm.corridor_source),
                    )
                    .located(s.storey, centroid(&narrow)),
                );
            }
        }
    }

    // Handläufe an beiden Seiten notwendiger Treppen.
    for (id, stair) in project.elements.iter().filter_map(|(id, e)| match e {
        Element::Stair(s) if s.necessary => Some((*id, s)),
        _ => None,
    }) {
        let mut sides: Vec<(usize, WallSide)> = Vec::new();
        for r in project.railings_on(stair.storey) {
            if let lot_model::RailingPath::Flight { stair: sid, flight, side } = r.1.path
                && sid == id
            {
                sides.push((flight, side));
            }
        }
        let mut missing = Vec::new();
        for (i, flight) in stair.flights.iter().enumerate() {
            for side in [WallSide::Left, WallSide::Right] {
                let has = sides.contains(&(i, side));
                // Bei Halbpodesten liegen die Innenseiten zweier Läufe aneinander; ein Handlauf genügt.
                let inner = |j: usize, landing: lot_model::Landing| match landing {
                    lot_model::Landing::BackLeft => Some((j, WallSide::Left)),
                    lot_model::Landing::BackRight => Some((j, WallSide::Right)),
                    _ => None,
                };
                let shared = (i > 0
                    && inner(i - 1, stair.flights[i - 1].landing) == Some((i - 1, side))
                    && sides.contains(&(i - 1, side)))
                    || (i + 1 < stair.flights.len()
                        && inner(i, flight.landing) == Some((i, side))
                        && sides.contains(&(i + 1, side)));
                if !has && !shared {
                    missing.push(format!("Lauf {} {}", i + 1, if side == WallSide::Left { "links" } else { "rechts" }));
                }
            }
        }
        let storey = name(stair.storey);
        f.push(
            Finding::new(
                if missing.is_empty() { Status::Ok } else { Status::Fail },
                if missing.is_empty() {
                    format!("Treppe {storey}: Handläufe an beiden Seiten")
                } else {
                    format!("Treppe {storey}: Handlauf fehlt ({})", missing.join(", "))
                },
                norm.cite(&norm.handrails_source),
            )
            .located(stair.storey, stair.start),
        );
    }

    // Wohnungen: Türen, Flure, Bewegungsflächen, Badtüren.
    for unit in &dwellings {
        let mut problems: Vec<(String, ElementId, Point2, String)> = Vec::new();
        let mut doors = 0;
        let mut rooms = 0;
        for s in &all {
            let mine: Vec<usize> = (0..s.rooms.len()).filter(|i| s.rooms[*i].space.unit == *unit).collect();
            for p in s.passages.iter().filter(|p| p.is_door_or_passage()) {
                let inside = p.left.is_some_and(|i| mine.contains(&i)) && p.right.is_some_and(|i| mine.contains(&i));
                // MBO § 50 Abs. 1 verlangt Wohn- und Schlafräume, Bad, Toilette und Küche;
                // Abstell- und Technikräume nicht.
                let storage = [p.left, p.right]
                    .into_iter()
                    .flatten()
                    .any(|i| matches!(s.rooms[i].space.usage, SpaceUsage::Storage | SpaceUsage::Technical));
                if !inside || storage {
                    continue;
                }
                doors += 1;
                let clear = norm.clear_door(p.opening.width);
                if clear + 1e-9 < norm.door_dwelling {
                    problems.push((
                        format!("Tür lichte Breite {} m (mindestens {} m)", de(clear, 2), de(norm.door_dwelling, 2)),
                        s.storey,
                        p.center,
                        norm.cite(&norm.door_dwelling_source),
                    ));
                }
                if let OpeningKind::Door(_) = p.opening.kind
                    && let Some(room) = p.swing_room()
                    && is_bathroom(s.rooms[room].space)
                {
                    problems.push((
                        format!("Tür von {} schlägt nach innen auf", s.rooms[room].label()),
                        s.storey,
                        p.center,
                        norm.cite(&norm.bathroom_door_source),
                    ));
                }
            }
            for &i in &mine {
                let room = &s.rooms[i];
                let needs =
                    matches!(room.space.usage, SpaceUsage::Living | SpaceUsage::Circulation) || is_bathroom(room.space);
                if !needs {
                    continue;
                }
                rooms += 1;
                if !square_fits(&room.area, norm.turning) {
                    problems.push((
                        format!(
                            "{}: keine Bewegungsfläche {} x {} m",
                            room.label(),
                            de(norm.turning, 2),
                            de(norm.turning, 2)
                        ),
                        s.storey,
                        room.space.position,
                        norm.cite(&norm.turning_source),
                    ));
                }
                if room.space.usage == SpaceUsage::Circulation {
                    let narrow = narrow_parts(&room.area, norm.corridor);
                    if narrow.area() > 0.05 {
                        problems.push((
                            format!("{}: schmaler als {} m", room.label(), de(norm.corridor, 2)),
                            s.storey,
                            centroid(&narrow),
                            norm.cite(&norm.corridor_source),
                        ));
                    }
                }
            }
        }
        if problems.is_empty() {
            f.push(Finding::new(
                Status::Ok,
                format!(
                    "Wohnung {unit}: {doors} Türen ab {} m licht, {rooms} Räume mit Bewegungsfläche {} x {} m, Flure ab {} m, Badtür schlägt nach außen auf",
                    de(norm.door_dwelling, 2),
                    de(norm.turning, 2),
                    de(norm.turning, 2),
                    de(norm.corridor, 2)
                ),
                norm.source.clone(),
            ));
        }
        for (text, storey, at, source) in problems {
            f.push(Finding::new(Status::Fail, format!("Wohnung {unit}: {text}"), source).located(storey, at));
        }
    }
    ReportSection { title: "Barrierefreiheit".into(), basis, findings: f }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn norm_loads() {
        let n = AccessibleNorm::get();
        assert!((n.clear_door(1.01) - 0.90).abs() < 1e-9);
        assert_eq!(n.turning, 1.2);
    }

    #[test]
    fn squares_and_narrow_parts() {
        // L-förmiger Flur: 1,30 m breit, ein Arm nur 0,90 m.
        let l = Area::from_ring(&[[0.0, 0.0], [5.0, 0.0], [5.0, 1.3], [0.9, 1.3], [0.9, 4.0], [0.0, 4.0]]);
        assert!(square_fits(&l, 1.2));
        assert!(!square_fits(&l, 1.4));
        let narrow = narrow_parts(&l, 1.2);
        assert!(narrow.area() > 2.0, "{}", narrow.area());
        // Gedrehtes Quadrat 1,25 m passt nur gedreht.
        let turned = Area::from_ring(&[[0.0, 0.0], [1.25, 0.0], [1.25, 1.25], [0.0, 1.25]]).rotated(30.0);
        assert!(square_fits(&turned, 1.2));
    }
}
