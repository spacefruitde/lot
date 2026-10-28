//! Typen (ADR 0048): Wandaufbau, Deckenaufbau, Tür- und Fenstertyp. Der Typ
//! gibt Werte für alle seine Bauteile vor; ein Bauteil übernimmt sie oder
//! weicht ab, wenn der Typ es zulässt. Typwerte ändert man direkt, sie wirken
//! auf alle Bauteile des Typs.

use lot_model::{Element, ElementId, Operation, Project, Properties, Property};

/// Art eines Typs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeKind {
    Wall,
    Slab,
    Opening,
}

impl TypeKind {
    pub fn title(self) -> &'static str {
        match self {
            TypeKind::Wall => "Wandaufbau",
            TypeKind::Slab => "Deckenaufbau",
            TypeKind::Opening => "Tür- oder Fenstertyp",
        }
    }

    /// Was ein Bauteil dieses Typs ist, für „wirkt auf 14 Wände“: (eins, mehrere).
    pub fn element_names(self) -> (&'static str, &'static str) {
        match self {
            TypeKind::Wall => ("Wand", "Wände"),
            TypeKind::Slab => ("Decke oder Dach", "Decken und Dächer"),
            TypeKind::Opening => ("Öffnung", "Öffnungen"),
        }
    }

    /// Werte, die dieser Typ vorgibt.
    pub fn properties(self) -> Vec<Property> {
        let mut out = Property::TEXT.to_vec();
        if self == TypeKind::Opening {
            out.extend([Property::Width, Property::Height]);
        }
        out
    }
}

pub fn kind_of(project: &Project, id: ElementId) -> Option<TypeKind> {
    match project.elements.get(&id)? {
        Element::WallType(_) => Some(TypeKind::Wall),
        Element::SlabType(_) => Some(TypeKind::Slab),
        Element::OpeningType(_) => Some(TypeKind::Opening),
        _ => None,
    }
}

/// Alle Typen, nach Art und Name.
pub fn all(project: &Project) -> Vec<(ElementId, TypeKind)> {
    let mut out: Vec<(ElementId, TypeKind)> =
        project.elements.keys().filter_map(|id| Some((*id, kind_of(project, *id)?))).collect();
    out.sort_by_key(|(id, k)| (*k as u8, name(project, *id).unwrap_or_default().to_lowercase()));
    out
}

pub fn name(project: &Project, id: ElementId) -> Option<String> {
    Some(match project.elements.get(&id)? {
        Element::WallType(t) => t.name.clone(),
        Element::SlabType(t) => t.name.clone(),
        Element::OpeningType(t) => t.name.clone(),
        _ => return None,
    })
}

/// Der Typ eines Bauteils.
pub fn type_of(project: &Project, element: ElementId) -> Option<ElementId> {
    match project.elements.get(&element)? {
        Element::Wall(w) => Some(w.wall_type),
        Element::Slab(s) => Some(s.slab_type),
        Element::Opening(o) => o.opening_type,
        Element::Roof(r) => Some(r.roof_type),
        _ => None,
    }
}

/// Die Bauteile, die einen Typ nutzen.
pub fn uses(project: &Project, type_id: ElementId) -> Vec<ElementId> {
    project.elements.keys().copied().filter(|id| type_of(project, *id) == Some(type_id)).collect()
}

fn type_props(e: &Element) -> Option<(&Properties, &[Property])> {
    match e {
        Element::WallType(t) => Some((&t.props, &t.locked)),
        Element::SlabType(t) => Some((&t.props, &t.locked)),
        Element::OpeningType(t) => Some((&t.props, &t.locked)),
        _ => None,
    }
}

fn type_props_mut(e: &mut Element) -> Option<(&mut Properties, &mut Vec<Property>)> {
    match e {
        Element::WallType(t) => Some((&mut t.props, &mut t.locked)),
        Element::SlabType(t) => Some((&mut t.props, &mut t.locked)),
        Element::OpeningType(t) => Some((&mut t.props, &mut t.locked)),
        _ => None,
    }
}

fn element_props_mut(e: &mut Element) -> Option<&mut Properties> {
    match e {
        Element::Wall(w) => Some(&mut w.props),
        Element::Slab(s) => Some(&mut s.props),
        Element::Opening(o) => Some(&mut o.props),
        _ => None,
    }
}

fn element_props(e: &Element) -> Option<&Properties> {
    match e {
        Element::Wall(w) => Some(&w.props),
        Element::Slab(s) => Some(&s.props),
        Element::Opening(o) => Some(&o.props),
        _ => None,
    }
}

/// Ob der Typ einen Wert festlegt, sodass Bauteile nicht abweichen dürfen.
pub fn locked(project: &Project, type_id: ElementId, p: Property) -> bool {
    project.elements.get(&type_id).and_then(type_props).is_some_and(|(_, l)| l.contains(&p))
}

/// Wert einer Eigenschaft am Typ, als Text; Maße in Metern mit Komma.
pub fn type_value(project: &Project, type_id: ElementId, p: Property) -> Option<String> {
    match (project.elements.get(&type_id)?, p) {
        (Element::OpeningType(t), Property::Width | Property::Height) => t.size(p).map(crate::format_m),
        (e, _) => type_props(e)?.0.get(p).cloned(),
    }
}

/// Woher der Wert eines Bauteils kommt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    /// Vom Typ übernommen.
    Type,
    /// Am Bauteil abweichend.
    Element,
    /// Weder Typ noch Bauteil haben einen Wert.
    None,
}

/// Wert einer Eigenschaft an einem Bauteil und woher er kommt.
pub fn value(project: &Project, element: ElementId, p: Property) -> (Option<String>, Origin) {
    let Some(e) = project.elements.get(&element) else { return (None, Origin::None) };
    let ty = type_of(project, element);
    if let (Element::Opening(o), Property::Width | Property::Height) = (e, p) {
        let v = if p == Property::Width { o.width } else { o.height };
        let from_type = ty.and_then(|t| project.opening_type(t)).and_then(|t| t.size(p)).is_some();
        let origin = if from_type && !o.own.contains(&p) { Origin::Type } else { Origin::Element };
        return (Some(crate::format_m(v)), origin);
    }
    if let Some(own) = element_props(e).and_then(|props| props.get(p)) {
        return (Some(own.clone()), Origin::Element);
    }
    match ty.and_then(|t| type_value(project, t, p)) {
        Some(v) => (Some(v), Origin::Type),
        None => (None, Origin::None),
    }
}

fn replace(project: &Project, id: ElementId, change: impl FnOnce(&mut Element)) -> Option<Operation> {
    let old = project.elements.get(&id)?;
    let mut new = old.clone();
    change(&mut new);
    (new != *old).then(|| Operation::Replace { id, old: old.clone(), new })
}

/// Öffnungen des Typs, die ein Maß vom Typ übernehmen, bekommen den neuen Wert.
fn sync_sizes(project: &Project, type_id: ElementId, p: Property, size: f64) -> Vec<Operation> {
    project
        .openings()
        .filter(|(_, o)| o.opening_type == Some(type_id) && !o.own.contains(&p))
        .filter_map(|(id, _)| {
            replace(project, id, |e| {
                if let Element::Opening(o) = e {
                    if p == Property::Width {
                        o.width = size;
                    } else {
                        o.height = size;
                    }
                }
            })
        })
        .collect()
}

/// Setzt einen Textwert am Typ; er wirkt auf alle Bauteile, die nicht abweichen.
pub fn set_type_property_ops(
    project: &Project,
    type_id: ElementId,
    p: Property,
    value: Option<String>,
) -> Vec<Operation> {
    replace(project, type_id, |e| {
        if let Some((props, _)) = type_props_mut(e) {
            props.set(p, value);
        }
    })
    .into_iter()
    .collect()
}

/// Setzt ein Rohbaumaß am Tür- oder Fenstertyp; Öffnungen, die es übernehmen, folgen.
pub fn set_type_size_ops(project: &Project, type_id: ElementId, p: Property, size: Option<f64>) -> Vec<Operation> {
    let mut ops: Vec<Operation> = replace(project, type_id, |e| {
        if let Element::OpeningType(t) = e {
            match p {
                Property::Width => t.width = size,
                Property::Height => t.height = size,
                _ => {}
            }
        }
    })
    .into_iter()
    .collect();
    if let Some(v) = size {
        ops.extend(sync_sizes(project, type_id, p, v));
    }
    ops
}

/// Legt einen Wert im Typ fest oder gibt ihn frei. Beim Festlegen übernehmen
/// alle Bauteile wieder den Wert des Typs.
pub fn set_locked_ops(project: &Project, type_id: ElementId, p: Property, lock: bool) -> Vec<Operation> {
    let mut ops: Vec<Operation> = replace(project, type_id, |e| {
        if let Some((_, locked)) = type_props_mut(e) {
            locked.retain(|q| *q != p);
            if lock {
                locked.push(p);
                locked.sort();
            }
        }
    })
    .into_iter()
    .collect();
    if lock {
        ops.extend(reset_ops(project, &uses(project, type_id), p));
    }
    ops
}

/// Setzt einen Textwert an den Bauteilen `ids`. Gleich dem Typwert heißt:
/// Das Bauteil übernimmt ihn wieder vom Typ.
pub fn set_element_property_ops(
    project: &Project,
    ids: &[ElementId],
    p: Property,
    value: Option<String>,
) -> Result<Vec<Operation>, String> {
    let mut ops = Vec::new();
    for id in ids {
        let ty = type_of(project, *id);
        if let Some(t) = ty
            && locked(project, t, p)
        {
            return Err(format!(
                "{} ist im Typ {} festgelegt; im Typ ändern oder einen anderen Typ wählen",
                p.title(),
                name(project, t).unwrap_or_default()
            ));
        }
        let from_type = ty.and_then(|t| type_value(project, t, p));
        let own = value.clone().filter(|v| Some(v.trim()) != from_type.as_deref());
        ops.extend(replace(project, *id, |e| {
            if let Some(props) = element_props_mut(e) {
                props.set(p, own);
            }
        }));
    }
    Ok(ops)
}

/// Zurücksetzen: Die Bauteile `ids` übernehmen den Wert wieder vom Typ.
pub fn reset_ops(project: &Project, ids: &[ElementId], p: Property) -> Vec<Operation> {
    ids.iter()
        .filter_map(|id| {
            let size = type_of(project, *id).and_then(|t| project.opening_type(t)).and_then(|t| t.size(p));
            replace(project, *id, |e| match e {
                Element::Opening(o) if matches!(p, Property::Width | Property::Height) => {
                    o.own.retain(|q| *q != p);
                    if let Some(v) = size {
                        if p == Property::Width {
                            o.width = v;
                        } else {
                            o.height = v;
                        }
                    }
                }
                e => {
                    if let Some(props) = element_props_mut(e) {
                        props.set(p, None);
                    }
                }
            })
        })
        .collect()
}

/// In den Typ übernehmen: Der Wert dieses Bauteils wird der Wert des Typs;
/// er gilt dann für alle Bauteile des Typs, die nicht abweichen.
pub fn apply_to_type_ops(project: &Project, element: ElementId, p: Property) -> Result<Vec<Operation>, String> {
    let t = type_of(project, element).ok_or("Dieses Bauteil hat keinen Typ")?;
    let mut ops = Vec::new();
    match (project.elements.get(&element), p) {
        (Some(Element::Opening(o)), Property::Width | Property::Height) => {
            let v = if p == Property::Width { o.width } else { o.height };
            ops.extend(replace(project, t, |e| {
                if let Element::OpeningType(ty) = e {
                    if p == Property::Width {
                        ty.width = Some(v);
                    } else {
                        ty.height = Some(v);
                    }
                }
            }));
            ops.extend(sync_sizes(project, t, p, v));
            // Das Maß gilt jetzt, weil der Typ es vorgibt.
            ops.extend(replace(project, element, |e| {
                if let Element::Opening(o) = e {
                    o.own.retain(|q| *q != p);
                }
            }));
        }
        _ => {
            let (v, _) = value(project, element, p);
            ops.extend(set_type_property_ops(project, t, p, v));
            ops.extend(reset_ops(project, &[element], p));
        }
    }
    Ok(ops)
}

/// Kopie eines Typs mit neuem Namen und ohne Kennzeichen; gibt auch die neue Kennung zurück.
pub fn duplicate_ops(project: &Project, type_id: ElementId) -> Option<(ElementId, Vec<Operation>)> {
    let mut e = project.elements.get(&type_id)?.clone();
    let base = name(project, type_id)?;
    let taken = |n: &str| all(project).iter().any(|(id, _)| name(project, *id).as_deref() == Some(n));
    let new_name = (1..).map(|i| if i == 1 { format!("{base} Kopie") } else { format!("{base} Kopie {i}") });
    let new_name = new_name.take(100).find(|n| !taken(n))?;
    match &mut e {
        Element::WallType(t) => t.name = new_name,
        Element::SlabType(t) => t.name = new_name,
        Element::OpeningType(t) => t.name = new_name,
        _ => return None,
    }
    if let Some((props, _)) = type_props_mut(&mut e) {
        props.mark = None;
    }
    let id = project.next_id();
    Some((id, vec![Operation::Insert { id, element: e }]))
}

pub fn rename_ops(project: &Project, type_id: ElementId, new_name: &str) -> Vec<Operation> {
    let n = new_name.trim().to_string();
    if n.is_empty() {
        return Vec::new();
    }
    replace(project, type_id, |e| match e {
        Element::WallType(t) => t.name = n,
        Element::SlabType(t) => t.name = n,
        Element::OpeningType(t) => t.name = n,
        _ => {}
    })
    .into_iter()
    .collect()
}

/// Gibt den Bauteilen `ids` den Typ `new` (Typ tauschen). Öffnungen übernehmen
/// dessen Rohbaumaß, wo er eines vorgibt; Abweichungen bei Eigenschaften bleiben.
pub fn assign_ops(project: &Project, ids: &[ElementId], new: Option<ElementId>) -> Result<Vec<Operation>, String> {
    let kind = new.and_then(|t| kind_of(project, t));
    let opening_type = new.and_then(|t| project.opening_type(t));
    let mut ops = Vec::new();
    for id in ids {
        let Some(e) = project.elements.get(id) else { continue };
        let fits = matches!(
            (e, kind),
            (Element::Wall(_), Some(TypeKind::Wall))
                | (Element::Slab(_) | Element::Roof(_), Some(TypeKind::Slab))
                | (Element::Opening(_), Some(TypeKind::Opening) | None)
        );
        if !fits {
            return Err("Dieser Typ passt nicht zu dem Bauteil".into());
        }
        ops.extend(replace(project, *id, |e| match e {
            Element::Wall(w) => w.wall_type = new.unwrap_or(w.wall_type),
            Element::Slab(s) => s.slab_type = new.unwrap_or(s.slab_type),
            Element::Roof(r) => r.roof_type = new.unwrap_or(r.roof_type),
            Element::Opening(o) => {
                o.opening_type = new;
                o.own.clear();
                if let Some(t) = opening_type {
                    o.width = t.width.unwrap_or(o.width);
                    o.height = t.height.unwrap_or(o.height);
                }
            }
            _ => {}
        }));
    }
    Ok(ops)
}

/// Löscht einen Typ; Bauteile, die ihn nutzen, bekommen den Ersatztyp. Ohne
/// Ersatz geht es nur, wenn niemand den Typ nutzt, oder bei Tür- und
/// Fenstertypen (die Öffnungen haben dann keinen Typ mehr).
pub fn delete_ops(
    project: &Project,
    type_id: ElementId,
    replacement: Option<ElementId>,
) -> Result<Vec<Operation>, String> {
    let kind = kind_of(project, type_id).ok_or("Das ist kein Typ")?;
    let users = uses(project, type_id);
    if replacement == Some(type_id) {
        return Err("Der Ersatz muss ein anderer Typ sein".into());
    }
    if replacement.is_some_and(|r| kind_of(project, r) != Some(kind)) {
        return Err("Der Ersatz muss ein Typ derselben Art sein".into());
    }
    if replacement.is_none() && !users.is_empty() && kind != TypeKind::Opening {
        let (one, many) = kind.element_names();
        return Err(format!(
            "Der Typ wird von {} {} genutzt; erst einen Ersatz wählen",
            users.len(),
            if users.len() == 1 { one } else { many }
        ));
    }
    let mut ops = assign_ops(project, &users, replacement)?;
    let element = project.elements.get(&type_id).cloned().ok_or("Das ist kein Typ")?;
    ops.push(Operation::Delete { id: type_id, element });
    Ok(ops)
}
