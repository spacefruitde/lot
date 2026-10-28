//! Öffnungen (ADR 0047): Die Öffnung mit Rohbaumaß ist das Bauteil, Tür oder
//! Fenster füllen sie, ohne dass sich die Öffnung ändert. Die fertige
//! Brüstung (BRH über OKFF) ist führend, die Rohbaubrüstung ergibt sich.

use lot_model::{Door, Element, ElementId, Handing, Opening, OpeningKind, OpeningType, Operation, Project, WallSide};

/// Füllungen zur Auswahl, in der Reihenfolge von [`fill_index`].
pub const FILLS: [&str; 3] = ["Durchgang", "Tür", "Fenster"];

pub fn fill_index(kind: &OpeningKind) -> usize {
    match kind {
        OpeningKind::Plain => 0,
        OpeningKind::Door(_) => 1,
        OpeningKind::Window => 2,
    }
}

/// Neue Füllung für die Öffnungen `ids`; Lage und Rohbaumaß bleiben.
pub fn set_fill_ops(project: &Project, ids: &[ElementId], fill: usize) -> Vec<Operation> {
    ids.iter()
        .filter_map(|id| {
            let Some(Element::Opening(o)) = project.elements.get(id) else { return None };
            if fill_index(&o.kind) == fill {
                return None;
            }
            let mut n = o.clone();
            n.kind = match fill {
                0 => OpeningKind::Plain,
                1 => OpeningKind::Door(Door { opens_to: WallSide::Left, hinge: Handing::DinLeft }),
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
    let mut next = project.next_id().0;
    for fill in 0..FILLS.len() {
        let untyped: Vec<&(ElementId, &Opening)> =
            openings.iter().filter(|(_, o)| o.opening_type.is_none() && fill_index(&o.kind) == fill).collect();
        if untyped.is_empty() {
            continue;
        }
        let letter = ["D", "T", "F"][fill];
        let used = |n: usize| project.opening_types().any(|(_, t)| t.mark == format!("{letter}{n}"));
        let number = (1..).find(|n| !used(*n)).unwrap_or(1);
        let id = ElementId(next);
        next += 1;
        let ty = OpeningType { name: FILLS[fill].into(), mark: format!("{letter}{number}"), sill_connection: value };
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
