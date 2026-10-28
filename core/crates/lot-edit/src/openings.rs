//! Öffnungen (ADR 0047): Die Öffnung mit Rohbaumaß ist das Bauteil, Tür oder
//! Fenster füllen sie, ohne dass sich die Öffnung ändert. Die fertige
//! Brüstung (BRH über OKFF) ist führend, die Rohbaubrüstung ergibt sich.

use lot_model::{
    Door, Element, ElementId, Handing, Opening, OpeningKind, OpeningType, Operation, Project, Recess, WallSide,
};

/// Füllungen zur Auswahl, in der Reihenfolge von [`fill_index`].
pub const FILLS: [&str; 4] = ["Durchgang", "Nische", "Tür", "Fenster"];

pub fn fill_index(opening: &Opening) -> usize {
    match opening.kind {
        OpeningKind::Plain if opening.recess.is_some() => 1,
        OpeningKind::Plain => 0,
        OpeningKind::Door(_) => 2,
        OpeningKind::Window => 3,
    }
}

/// Neue Füllung für die Öffnungen `ids`; Lage und Rohbaumaß bleiben.
pub fn set_fill_ops(project: &Project, ids: &[ElementId], fill: usize) -> Vec<Operation> {
    ids.iter()
        .filter_map(|id| {
            let Some(Element::Opening(o)) = project.elements.get(id) else { return None };
            if fill_index(o) == fill {
                return None;
            }
            let mut n = o.clone();
            n.recess = None;
            n.kind = match fill {
                0 => OpeningKind::Plain,
                1 => {
                    // Nische 10 cm tief, höchstens bis zur Hälfte der Wand (Vorschlag).
                    let thickness = project
                        .wall(o.host)
                        .and_then(|w| project.wall_type(w.wall_type))
                        .map_or(0.2, |t| t.thickness());
                    n.recess = Some(Recess { side: WallSide::Left, depth: (thickness / 2.0).min(0.1) });
                    OpeningKind::Plain
                }
                2 => OpeningKind::Door(Door { opens_to: WallSide::Left, hinge: Handing::DinLeft }),
                _ => OpeningKind::Window,
            };
            Some(Operation::Replace { id: *id, old: Element::Opening(o.clone()), new: Element::Opening(n) })
        })
        .collect()
}

/// Setzt den unteren Anschluss am Typ der Öffnungen `ids`; er wirkt auf alle
/// Öffnungen dieses Typs (ADR 0048). Öffnungen ohne Typ bekommen je Füllung
/// einen neuen Typ, etwa „Fenster“ mit dem Kennzeichen F1.
pub fn set_sill_connection_ops(project: &Project, ids: &[ElementId], value: f64) -> Vec<Operation> {
    let openings: Vec<(ElementId, &Opening)> = ids
        .iter()
        .filter_map(|id| match project.elements.get(id) {
            Some(Element::Opening(o)) => Some((*id, o)),
            _ => None,
        })
        .collect();
    let mut ops = Vec::new();
    let mut types: Vec<ElementId> = openings.iter().filter_map(|(_, o)| o.opening_type).collect();
    types.sort();
    types.dedup();
    for t in types {
        let Some(old) = project.opening_type(t) else { continue };
        let mut new = old.clone();
        new.sill_connection = value;
        if new != *old {
            ops.push(Operation::Replace {
                id: t,
                old: Element::OpeningType(old.clone()),
                new: Element::OpeningType(new),
            });
        }
    }
    ops.extend(new_types_ops(project, &openings, value));
    ops
}

/// Legt für die Öffnungen `ids` ohne Typ je Füllung einen Typ an und weist ihn
/// zu, etwa „Fenster 1,26 × 1,385“ mit Kennzeichen F1. Haben alle Öffnungen
/// einer Füllung dasselbe Rohbaumaß, gibt der Typ es vor.
pub fn create_types_ops(project: &Project, ids: &[ElementId]) -> Vec<Operation> {
    let openings: Vec<(ElementId, &Opening)> = ids
        .iter()
        .filter_map(|id| match project.elements.get(id) {
            Some(Element::Opening(o)) => Some((*id, o)),
            _ => None,
        })
        .collect();
    new_types_ops(project, &openings, 0.0)
}

fn new_types_ops(project: &Project, openings: &[(ElementId, &Opening)], sill_connection: f64) -> Vec<Operation> {
    let mut ops = Vec::new();
    let mut next = project.next_id().0;
    let mut marks: Vec<String> = project.opening_types().filter_map(|(_, t)| t.props.mark.clone()).collect();
    for fill in 0..FILLS.len() {
        let untyped: Vec<&(ElementId, &Opening)> =
            openings.iter().filter(|(_, o)| o.opening_type.is_none() && fill_index(o) == fill).collect();
        let Some((_, first)) = untyped.first() else { continue };
        let letter = ["D", "N", "T", "F"][fill];
        let number = (1..).find(|n| !marks.contains(&format!("{letter}{n}"))).unwrap_or(1);
        marks.push(format!("{letter}{number}"));
        let same = untyped.iter().all(|(_, o)| o.width == first.width && o.height == first.height);
        let size = |v: f64| format!("{}", (v * 1000.0).round() / 1000.0).replace('.', ",");
        let name = if same {
            format!("{} {} × {}", FILLS[fill], size(first.width), size(first.height))
        } else {
            FILLS[fill].to_string()
        };
        let id = ElementId(next);
        next += 1;
        let ty = OpeningType {
            name,
            sill_connection,
            width: same.then_some(first.width),
            height: same.then_some(first.height),
            props: lot_model::Properties { mark: Some(format!("{letter}{number}")), ..Default::default() },
            locked: Vec::new(),
        };
        ops.push(Operation::Insert { id, element: Element::OpeningType(ty) });
        for (oid, o) in untyped {
            let mut n = (*o).clone();
            n.opening_type = Some(id);
            ops.push(Operation::Replace { id: *oid, old: Element::Opening((*o).clone()), new: Element::Opening(n) });
        }
    }
    ops
}

/// Fertige Brüstung, die zu einer getippten Rohbaubrüstung über Rohdecke gehört.
pub fn sill_for_rough(project: &Project, opening: &Opening, rough: f64) -> Option<f64> {
    Some(rough - project.floor_build_up(opening)? + project.sill_connection(opening))
}

/// Lage einer neuen Öffnung beim Setzen (ADR 0047): wo sie sitzt, und die
/// Anschläge davor und dahinter für die Maße und den getippten Abstand.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placement {
    pub offset: f64,
    /// Anschlag vor und hinter der Öffnung, als Abstand vom Wandanfang.
    pub before: Option<f64>,
    pub after: Option<f64>,
    /// Mittig zwischen den beiden Anschlägen gefangen.
    pub centered: bool,
}

/// Lage einer Öffnung der Breite `width` auf der Wand `host`, wenn der Zeiger
/// `along` vom Wandanfang steht: mittig zum Zeiger auf 1 cm, nahe der Mitte
/// zwischen zwei Anschlägen genau mittig. `None`, wenn die Wand zu kurz ist.
pub fn place(project: &Project, host: ElementId, along: f64, width: f64, tolerance: f64) -> Option<Placement> {
    let length = project.wall(host)?.length();
    if length < width + 0.02 {
        return None;
    }
    let stops = crate::tempdims::stops(project, host, None);
    let before = stops.iter().copied().filter(|s| *s <= along).max_by(f64::total_cmp);
    let after = stops.iter().copied().filter(|s| *s >= along).min_by(f64::total_cmp);
    let mut offset = ((along - width / 2.0) / 0.01).round() * 0.01;
    let mut centered = false;
    if let (Some(a), Some(b)) = (before, after)
        && b - a >= width
        && (along - (a + b) / 2.0).abs() <= tolerance
    {
        offset = (a + b) / 2.0 - width / 2.0;
        centered = true;
    }
    Some(Placement { offset: offset.clamp(0.0, length - width), before, after, centered })
}

impl Placement {
    /// Lage bei getipptem Abstand zum Anschlag davor, oder mit `from_after` zum Anschlag dahinter.
    pub fn at_distance(&self, width: f64, distance: f64, from_after: bool) -> Option<f64> {
        if from_after { Some(self.after? - distance - width) } else { Some(self.before? + distance) }
    }
}

/// Tiefe und Seite der Nischen unter den Öffnungen `ids`.
pub fn set_recess_ops(project: &Project, ids: &[ElementId], side: WallSide, depth: f64) -> Vec<Operation> {
    ids.iter()
        .filter_map(|id| {
            let Some(Element::Opening(o)) = project.elements.get(id) else { return None };
            o.recess?;
            let mut n = o.clone();
            n.recess = Some(Recess { side, depth });
            (n != *o).then(|| Operation::Replace {
                id: *id,
                old: Element::Opening(o.clone()),
                new: Element::Opening(n),
            })
        })
        .collect()
}
