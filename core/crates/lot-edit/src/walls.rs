//! Wandlogik (ADR 0045): Die Bezugslinie ist eine Lesart. Ein Wechsel ihrer
//! Lage und das Umdrehen lassen die Wand liegen; Ecken mit anderen Wänden
//! bleiben verbunden, Öffnungen bleiben an ihrer Stelle.

use std::collections::BTreeMap;

use lot_model::{DimRef, Element, ElementId, JoinRule, OpeningKind, Operation, Point2, Project, Wall, WallAlignment};

use crate::geometry::{JOIN_TOLERANCE, intersect};

/// Wände, deren Richtungen weniger als 5° trennt, gelten als parallel.
const PARALLEL_SIN: f64 = 0.0872;

/// Eine Wand mit neuer Bezugslinie: `new` ist die Wand vor dem Verschieben
/// (beim Umdrehen schon umgedreht), `shift` der Versatz der Bezugslinie nach
/// links, damit die Wand liegen bleibt.
struct Change {
    id: ElementId,
    old: Wall,
    new: Wall,
    shift: f64,
    flipped: bool,
}

/// Neue Lage der Bezugslinie für die Wände `ids`; die Wände bleiben liegen.
pub fn set_alignment_ops(project: &Project, ids: &[ElementId], alignment: WallAlignment) -> Vec<Operation> {
    let changes = ids
        .iter()
        .filter_map(|id| {
            let w = project.wall(*id)?;
            let t = project.wall_type(w.wall_type)?;
            let mut new = w.clone();
            new.alignment = alignment;
            // Die linke Fläche liegt `depth` links der Bezugslinie und bleibt, wo sie ist.
            let shift = w.alignment.depth(t) - alignment.depth(t);
            Some(Change { id: *id, old: w.clone(), new, shift, flipped: false })
        })
        .filter(|c| c.old != c.new)
        .collect();
    relined_ops(project, changes)
}

/// Dreht die Wände `ids` um: Innen und außen tauschen, die Wand bleibt liegen,
/// die Bezugslinie behält ihre Lesart von der anderen Seite (Außenfläche wird
/// Innenfläche). Türen schlagen weiter zur selben Seite auf.
pub fn flip_ops(project: &Project, ids: &[ElementId]) -> Vec<Operation> {
    let changes = ids
        .iter()
        .filter_map(|id| {
            let w = project.wall(*id)?;
            let t = project.wall_type(w.wall_type)?;
            let mut new = w.clone();
            std::mem::swap(&mut new.start, &mut new.end);
            // Der Bogen bleibt, wo er ist: links und rechts tauschen auch hier.
            new.sagitta = -w.sagitta;
            new.alignment = w.alignment.mirrored();
            new.joins = [w.joins[1], w.joins[0]];
            // Die alte Bezugslinie liegt nun `dicke - alte Tiefe` rechts der neuen Außenfläche.
            let shift = t.thickness() - w.alignment.depth(t) - new.alignment.depth(t);
            Some(Change { id: *id, old: w.clone(), new, shift, flipped: true })
        })
        .collect();
    relined_ops(project, changes)
}

/// Operationen für Wände mit verschobener Bezugslinie. Ein Ende, das an einer
/// Ecke mit einer anderen Wand liegt, wandert auf den Schnitt der neuen
/// Bezugslinie mit der Bezugslinie der anderen Wand; deren Ende zieht entlang
/// ihrer Linie mit. So bleiben Ecken verbunden und die Wände, wo sie sind.
fn relined_ops(project: &Project, changes: Vec<Change>) -> Vec<Operation> {
    let changed: Vec<ElementId> = changes.iter().map(|c| c.id).collect();
    // Neue Bezugslinien, zunächst nur quer verschoben.
    let shifted: BTreeMap<ElementId, Wall> = changes.iter().map(|c| (c.id, shifted(&c.new, c.shift))).collect();
    let line = |id: ElementId, w: &Wall| -> (Point2, (f64, f64)) {
        let w = shifted.get(&id).unwrap_or(w);
        (w.start, w.direction())
    };
    // Neue Stände aller betroffenen Wände; Bezug für Öffnungen ist jeweils `base`.
    let mut result: BTreeMap<ElementId, (Wall, Wall)> = BTreeMap::new();
    for c in &changes {
        result.insert(c.id, (c.new.clone(), shifted[&c.id].clone()));
    }
    for c in changes.iter().filter(|c| !c.new.is_curved()) {
        let dir = c.new.direction();
        let moved = shifted[&c.id].clone();
        for (at_start, old_point, on_line) in [(true, c.new.start, moved.start), (false, c.new.end, moved.end)] {
            let mut target = on_line;
            let mut found = false;
            let mut followers: Vec<(ElementId, bool)> = Vec::new();
            for (pid, pw) in project.walls_on(c.new.storey) {
                if pid == c.id || pw.is_curved() {
                    continue;
                }
                let end_here = [pw.start, pw.end].map(|q| q.distance(old_point) <= JOIN_TOLERANCE);
                let (s, t) = pw.locate(old_point);
                let crossing = t.abs() <= JOIN_TOLERANCE && s > -JOIN_TOLERANCE && s < pw.length() + JOIN_TOLERANCE;
                if !(end_here[0] || end_here[1] || crossing) {
                    continue;
                }
                let (q, e) = line(pid, pw);
                if (dir.0 * e.1 - dir.1 * e.0).abs() < PARALLEL_SIN {
                    continue;
                }
                if !found && let Some(x) = intersect(on_line, dir, q, e) {
                    target = x;
                    found = true;
                }
                if !changed.contains(&pid) {
                    for (k, here) in end_here.iter().enumerate() {
                        if *here {
                            followers.push((pid, k == 0));
                        }
                    }
                }
            }
            let entry = result.get_mut(&c.id).expect("geänderte Wand");
            if at_start {
                entry.1.start = target;
            } else {
                entry.1.end = target;
            }
            for (pid, start) in followers {
                let Some(pw) = project.wall(pid) else { continue };
                // Nur mitziehen, wenn der neue Punkt auf ihrer Linie liegt; sonst würde sie sich drehen.
                if pw.locate(target).1.abs() > JOIN_TOLERANCE {
                    continue;
                }
                let e = result.entry(pid).or_insert_with(|| (pw.clone(), pw.clone()));
                if start {
                    e.1.start = target;
                } else {
                    e.1.end = target;
                }
            }
        }
    }

    let mut ops = Vec::new();
    for (id, (base, n)) in &result {
        let Some(old) = project.wall(*id) else { continue };
        let flipped = changes.iter().any(|c| c.id == *id && c.flipped);
        ops.push(Operation::Replace { id: *id, old: Element::Wall(old.clone()), new: Element::Wall(n.clone()) });
        // Öffnungen bleiben an ihrem Ort: beim Umdrehen zählt der Abstand vom
        // anderen Ende, und er folgt einem Anfang, der entlang der Wand wandert.
        let along = if n.is_curved() { 0.0 } else { base.locate(n.start).0 };
        let scale = if n.is_curved() { n.length() / base.length().max(1e-12) } else { 1.0 };
        for (oid, o) in project.openings_in(*id) {
            let mut m = o.clone();
            if flipped {
                m.offset = old.length() - o.offset - o.width;
                if let OpeningKind::Door(d) = &mut m.kind {
                    d.opens_to = d.opens_to.other();
                }
            }
            m.offset = m.offset * scale - along;
            if m != *o {
                ops.push(Operation::Replace { id: oid, old: Element::Opening(o.clone()), new: Element::Opening(m) });
            }
        }
    }
    // Maßketten an umgedrehten Wänden: Anfang und Ende, links und rechts tauschen.
    let flipped: Vec<ElementId> = changes.iter().filter(|c| c.flipped).map(|c| c.id).collect();
    if !flipped.is_empty() {
        let in_flipped = |opening: ElementId| match project.elements.get(&opening) {
            Some(Element::Opening(o)) => flipped.contains(&o.host),
            _ => false,
        };
        for (did, e) in &project.elements {
            let Element::DimensionChain(d) = e else { continue };
            let mut n = d.clone();
            for r in &mut n.refs {
                *r = match *r {
                    DimRef::WallEnd { wall, end, side } if flipped.contains(&wall) => {
                        DimRef::WallEnd { wall, end: end.other(), side: side.other() }
                    }
                    DimRef::WallFace { wall, side } if flipped.contains(&wall) => {
                        DimRef::WallFace { wall, side: side.other() }
                    }
                    DimRef::Jamb { opening, end, side } if in_flipped(opening) => {
                        DimRef::Jamb { opening, end: end.other(), side: side.other() }
                    }
                    r => r,
                };
            }
            if n != *d {
                ops.push(Operation::Replace {
                    id: *did,
                    old: Element::DimensionChain(d.clone()),
                    new: Element::DimensionChain(n),
                });
            }
        }
    }
    ops
}

/// Die Wand mit um `shift` nach links verschobener Bezugslinie; ein Bogen behält seinen Mittelpunkt.
fn shifted(w: &Wall, shift: f64) -> Wall {
    let mut n = w.clone();
    if shift.abs() < 1e-12 {
        return n;
    }
    let length = w.length();
    n.start = w.point_at(0.0, shift);
    n.end = w.point_at(length, shift);
    if w.is_curved() {
        let mid = w.point_at(length / 2.0, shift);
        let probe = Wall { sagitta: 0.0, ..n.clone() };
        n.sagitta = probe.locate(mid).1;
    }
    n
}

/// Setzt die Ausnahme am Anschluss eines Wandendes.
pub fn set_join_ops(project: &Project, wall: ElementId, end: lot_model::WallEnd, rule: JoinRule) -> Vec<Operation> {
    let Some(w) = project.wall(wall) else { return Vec::new() };
    let mut n = w.clone();
    n.joins[end.index()] = rule;
    if n == *w {
        return Vec::new();
    }
    vec![Operation::Replace { id: wall, old: Element::Wall(w.clone()), new: Element::Wall(n) }]
}
