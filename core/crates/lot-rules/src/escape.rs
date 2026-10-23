//! Rettungswege (5.05) nach Landesrecht, als Voreinstellung MBO §§ 33 bis 35.
//!
//! - Erster Rettungsweg: Jede Nutzungseinheit erreicht in jedem Geschoss
//!   einen notwendigen Treppenraum oder das Freie (§ 33 Abs. 1), von jeder
//!   Stelle eines Aufenthaltsraums in höchstens 35 m (§ 35 Abs. 2). Gemessen
//!   wird die Luftlinie innerhalb der Räume, von Tür zu Tür.
//! - Zweiter Rettungsweg über Rettungsgeräte der Feuerwehr: ein Fenster im
//!   Aufenthaltsraum, im Lichten mindestens 0,90 x 1,20 m, Unterkante höchstens
//!   1,20 m über dem Fußboden (§ 37 Abs. 5). Liegt die Brüstung mehr als 8 m
//!   über dem Gelände, braucht die Feuerwehr ein Hubrettungsfahrzeug (§ 33 Abs. 3).
//! - Notwendiger Treppenraum: Ausgang unmittelbar ins Freie (§ 35 Abs. 3), in
//!   jedem oberirdischen Geschoss ein Fenster zum Lüften oder oben eine Öffnung
//!   zur Rauchableitung (§ 35 Abs. 8).

use std::collections::BTreeMap;

use lot_model::{ElementId, OpeningKind, Point2, Project};

use crate::accessible::AccessibleNorm;
use crate::building::{BuildingClassResult, ground_at, is_habitable};
use crate::landesrecht::StateRules;
use crate::report::{Finding, ReportSection, Status, de};
use crate::rooms::StoreyRooms;

pub fn check_escape(project: &Project, rules: &StateRules, class: &BuildingClassResult) -> ReportSection {
    let r = &rules.escape;
    let norm = AccessibleNorm::get();
    let basis = rules.name.clone();
    let mut f = Vec::new();
    let name = |id: ElementId| project.storey(id).map_or("?".to_string(), |s| s.name.clone());

    for storey in &class.above_ground {
        let s = StoreyRooms::new(project, *storey);
        let elevation = project.storey(*storey).map_or(0.0, |x| x.elevation);
        let stair_rooms = s.stair_rooms(project);

        // Nutzungseinheiten in diesem Geschoss; ohne Angabe gilt das Geschoss als eine.
        let mut units: BTreeMap<String, Vec<usize>> = BTreeMap::new();
        for (i, room) in s.rooms.iter().enumerate() {
            if stair_rooms.contains(&i) {
                continue;
            }
            if room.space.unit.is_empty() && class.units.iter().any(|(u, _)| !u.is_empty()) {
                continue;
            }
            units.entry(room.space.unit.clone()).or_default().push(i);
        }

        for (unit, mine) in &units {
            let label = if unit.is_empty() { name(*storey) } else { format!("{unit} ({})", name(*storey)) };
            // Ausgänge: Türen aus der Einheit in einen Treppenraum oder ins Freie.
            let walkable: Vec<usize> = s
                .passages
                .iter()
                .enumerate()
                .filter(|(_, p)| p.is_door_or_passage() && p.sill < 0.05)
                .map(|(i, _)| i)
                .collect();
            let exits: Vec<usize> = walkable
                .iter()
                .copied()
                .filter(|&i| {
                    let p = &s.passages[i];
                    let in_unit = |x: Option<usize>| x.is_some_and(|k| mine.contains(&k));
                    let to_stair = |x: Option<usize>| x.is_some_and(|k| stair_rooms.contains(&k));
                    (in_unit(p.left) && (to_stair(p.right) || p.right_outside))
                        || (in_unit(p.right) && (to_stair(p.left) || p.left_outside))
                })
                .collect();
            // Kürzeste Wege von Tür zu Tür innerhalb der Einheit (Dijkstra, Luftlinie im Raum).
            let mut dist: BTreeMap<usize, f64> = exits.iter().map(|&i| (i, 0.0)).collect();
            let mut done: Vec<usize> = Vec::new();
            loop {
                let next = dist
                    .iter()
                    .filter(|(i, _)| !done.contains(i))
                    .min_by(|a, b| a.1.total_cmp(b.1))
                    .map(|(i, d)| (*i, *d));
                let Some((i, d)) = next else { break };
                done.push(i);
                let p = &s.passages[i];
                for room in [p.left, p.right].into_iter().flatten().filter(|k| mine.contains(k)) {
                    for &j in &walkable {
                        let q = &s.passages[j];
                        if q.left != Some(room) && q.right != Some(room) {
                            continue;
                        }
                        let nd = d + p.center.distance(q.center);
                        if dist.get(&j).is_none_or(|old| nd < *old) {
                            dist.insert(j, nd);
                        }
                    }
                }
            }
            // Längster Weg: von jeder Ecke eines Aufenthaltsraums zur nächsten erreichbaren Tür.
            let mut longest: Option<(f64, Point2)> = None;
            let mut unreachable = Vec::new();
            for &k in mine {
                let room = &s.rooms[k];
                if !is_habitable(room.space.usage) {
                    continue;
                }
                let doors: Vec<(Point2, f64)> = s
                    .passages
                    .iter()
                    .enumerate()
                    .filter(|(j, p)| (p.left == Some(k) || p.right == Some(k)) && dist.contains_key(j))
                    .map(|(j, p)| (p.center, dist[&j]))
                    .collect();
                if doors.is_empty() {
                    unreachable.push(room.label());
                    continue;
                }
                for ring in room.area.rings() {
                    for v in ring {
                        let v = Point2::new(v[0], v[1]);
                        let best = doors.iter().map(|(c, d)| v.distance(*c) + d).fold(f64::INFINITY, f64::min);
                        if longest.is_none_or(|(l, _)| best > l) {
                            longest = Some((best, v));
                        }
                    }
                }
            }
            let source = r.distance_source.clone();
            if exits.is_empty() {
                f.push(
                    Finding::new(
                        Status::Fail,
                        format!("{label}: kein Ausgang in einen notwendigen Treppenraum oder ins Freie"),
                        source,
                    )
                    .located(*storey, s.rooms[mine[0]].space.position),
                );
            } else if !unreachable.is_empty() {
                f.push(
                    Finding::new(
                        Status::Fail,
                        format!("{label}: kein Weg aus {} zum Ausgang", unreachable.join(", ")),
                        source,
                    )
                    .located(*storey, s.rooms[mine[0]].space.position),
                );
            } else if let Some((length, at)) = longest {
                let status = if length <= r.max_distance + 1e-9 { Status::Ok } else { Status::Fail };
                f.push(
                    Finding::new(
                        status,
                        format!(
                            "{label}: erster Rettungsweg über den Treppenraum, längster Weg {} m (höchstens {} m)",
                            de(length, 1),
                            de(r.max_distance, 0)
                        ),
                        source,
                    )
                    .located(*storey, at),
                );
            }

            // Zweiter Rettungsweg: Fenster zum Anleitern.
            let mut best: Option<(f64, Point2)> = None;
            for p in s.passages.iter().filter(|p| p.to_outside() && matches!(p.opening.kind, OpeningKind::Window)) {
                let room = [p.left, p.right].into_iter().flatten().find(|k| mine.contains(k));
                let Some(room) = room else { continue };
                if !is_habitable(s.rooms[room].space.usage) {
                    continue;
                }
                let (w, h) = (p.opening.width - norm.window_deduction, p.opening.height - norm.window_deduction);
                if w + 1e-9 < r.window_width || h + 1e-9 < r.window_height || p.sill > r.window_sill + 1e-9 {
                    continue;
                }
                let outside = Point2::new(
                    p.center.x + p.normal[0] * if p.left_outside { 1.0 } else { -1.0 },
                    p.center.y + p.normal[1] * if p.left_outside { 1.0 } else { -1.0 },
                );
                // Oberkante der Brüstung über dem Gelände.
                let ladder = elevation + p.sill - ground_at(project, outside);
                if best.is_none_or(|(b, _)| ladder < b) {
                    best = Some((ladder, p.center));
                }
            }
            let source = format!("{}, {}", r.window_source, r.ladder_source);
            match best {
                None => f.push(
                    Finding::new(
                        Status::Fail,
                        format!(
                            "{label}: kein Fenster für den zweiten Rettungsweg (im Lichten {} x {} m, Brüstung höchstens {} m)",
                            de(r.window_width, 2),
                            de(r.window_height, 2),
                            de(r.window_sill, 2)
                        ),
                        source,
                    )
                    .located(*storey, s.rooms[mine[0]].space.position),
                ),
                Some((ladder, at)) if ladder > r.ladder_height + 1e-9 => f.push(
                    Finding::new(
                        Status::Note,
                        format!(
                            "{label}: zweiter Rettungsweg über Fenster, Brüstung {} m über Gelände: nur mit Hubrettungsfahrzeug der Feuerwehr (über {} m)",
                            de(ladder, 2),
                            de(r.ladder_height, 0)
                        ),
                        source,
                    )
                    .located(*storey, at),
                ),
                Some((ladder, at)) => f.push(
                    Finding::new(
                        Status::Ok,
                        format!("{label}: zweiter Rettungsweg über Fenster, Brüstung {} m über Gelände (tragbare Leiter)", de(ladder, 2)),
                        source,
                    )
                    .located(*storey, at),
                ),
            }
        }

        // Treppenraum: Lüftung in jedem oberirdischen Geschoss.
        for &k in &stair_rooms {
            let window = s
                .passages_of(k)
                .filter(|p| p.to_outside() && matches!(p.opening.kind, OpeningKind::Window))
                .map(|p| p.opening.width * p.opening.height)
                .fold(0.0, f64::max);
            let room = &s.rooms[k];
            let status = if window + 1e-9 >= r.stair_window_area { Status::Ok } else { Status::Fail };
            f.push(
                Finding::new(
                    status,
                    if status == Status::Ok {
                        format!(
                            "{} im {}: Fenster ins Freie {} m² (Rohbaumaß, zu öffnen mindestens {} m²)",
                            room.label(),
                            name(*storey),
                            de(window, 2),
                            de(r.stair_window_area, 2)
                        )
                    } else {
                        format!(
                            "{} im {}: kein Fenster ins Freie mit {} m²; Rauchableitung oben ist nicht im Modell",
                            room.label(),
                            name(*storey),
                            de(r.stair_window_area, 2)
                        )
                    },
                    r.stair_source.clone(),
                )
                .located(*storey, room.space.position),
            );
        }
    }

    // Ausgang des Treppenraums ins Freie, im untersten Geschoss mit Treppenraum.
    let mut exit_found = None;
    let mut stair_seen = false;
    for (storey, _) in project.storeys() {
        let s = StoreyRooms::new(project, storey);
        let stairs = s.stair_rooms(project);
        if stairs.is_empty() {
            continue;
        }
        stair_seen = true;
        if let Some(p) = stairs.iter().flat_map(|&k| s.passages_of(k)).find(|p| {
            p.to_outside() && matches!(p.opening.kind, OpeningKind::Door(_) | OpeningKind::Plain) && p.sill < 0.05
        }) {
            exit_found = Some((storey, p.center));
            break;
        }
    }
    if stair_seen {
        match exit_found {
            Some((storey, at)) => f.push(
                Finding::new(
                    Status::Ok,
                    format!("Treppenraum mit Ausgang unmittelbar ins Freie im {}", name(storey)),
                    "§ 35 Abs. 3 MBO",
                )
                .located(storey, at),
            ),
            None => f.push(Finding::new(Status::Fail, "Treppenraum ohne Ausgang ins Freie", "§ 35 Abs. 3 MBO")),
        }
    }
    f.push(Finding::new(
        Status::Note,
        format!(
            "Lichte Fenstermaße: Rohbaumaß minus {} m angenommen; Wege als Luftlinie im Raum, ohne Möbel",
            de(norm.window_deduction, 2)
        ),
        "",
    ));
    ReportSection { title: "Rettungswege".into(), basis, findings: f }
}
