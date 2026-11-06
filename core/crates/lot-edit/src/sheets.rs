//! Planblätter in der App (ADR 0049): anlegen, Ansichten platzieren,
//! verschieben, entfernen, Maßstab und Heimat. Eine Ansicht darf auf mehreren
//! Blättern liegen; Schnittmarken verweisen auf ihre Heimat.

use lot_model::{AnnotationView, Element, ElementId, Operation, PlacedView, PlanSheet, Point2, Project, ViewContent};

use crate::navigation::ViewRef;

/// Blattformate quer nach DIN EN ISO 216.
pub const FORMATS: [&str; 5] = ["A4", "A3", "A2", "A1", "A0"];

fn sheet(project: &Project, id: ElementId) -> Option<&PlanSheet> {
    match project.elements.get(&id) {
        Some(Element::PlanSheet(s)) => Some(s),
        _ => None,
    }
}

fn replace(project: &Project, id: ElementId, new: PlanSheet) -> Vec<Operation> {
    match project.elements.get(&id) {
        Some(old) if *old != Element::PlanSheet(new.clone()) => {
            vec![Operation::Replace { id, old: old.clone(), new: Element::PlanSheet(new) }]
        }
        _ => Vec::new(),
    }
}

/// Nächste freie Plannummer: nach der höchsten rein numerischen eins weiter, sonst „101“.
pub fn next_number(project: &Project) -> String {
    let sheets = project.plan_sheets();
    let highest = sheets.iter().filter_map(|(_, s)| s.number.parse::<u32>().ok()).max();
    let mut n = highest.map_or(101, |h| h + 1);
    while sheets.iter().any(|(_, s)| s.number == n.to_string()) {
        n += 1;
    }
    n.to_string()
}

/// Neues leeres Blatt A3 quer.
pub fn add_sheet_ops(project: &Project) -> (ElementId, Vec<Operation>) {
    let id = project.next_id();
    let sheet = PlanSheet {
        number: next_number(project),
        title: "Neues Blatt".into(),
        format: "A3".into(),
        date: String::new(),
        index: String::new(),
        views: Vec::new(),
    };
    (id, vec![Operation::Insert { id, element: Element::PlanSheet(sheet) }])
}

/// Maßstab, mit dem eine Ansicht auf ein Blatt kommt: Lageplan 1:500, sonst 1:100.
pub fn default_scale(view: ViewRef) -> f64 {
    match view {
        ViewRef::SitePlan => 500.0,
        _ => 100.0,
    }
}

/// Legt eine Ansicht mit ihrer Mitte an `center` (Blatt-mm) aufs Blatt.
pub fn place_ops(
    project: &Project,
    sheet_id: ElementId,
    view: ViewRef,
    center: Point2,
) -> Result<Vec<Operation>, String> {
    let Some(old) = sheet(project, sheet_id) else { return Err("Dieses Planblatt gibt es nicht".into()) };
    let Some(content) = view.content() else {
        return Err(format!("{} lässt sich nicht aufs Blatt legen", view.title(project)));
    };
    if !view.exists(project) {
        return Err("Diese Ansicht gibt es nicht".into());
    }
    let mut new = old.clone();
    new.views.push(PlacedView { content, scale: default_scale(view), center });
    Ok(replace(project, sheet_id, new))
}

pub fn move_ops(project: &Project, sheet_id: ElementId, index: usize, center: Point2) -> Vec<Operation> {
    let Some(old) = sheet(project, sheet_id) else { return Vec::new() };
    let mut new = old.clone();
    match new.views.get_mut(index) {
        Some(v) => v.center = center,
        None => return Vec::new(),
    }
    replace(project, sheet_id, new)
}

pub fn remove_ops(project: &Project, sheet_id: ElementId, index: usize) -> Vec<Operation> {
    let Some(old) = sheet(project, sheet_id) else { return Vec::new() };
    if index >= old.views.len() {
        return Vec::new();
    }
    let mut new = old.clone();
    new.views.remove(index);
    let mut ops = replace(project, sheet_id, new);
    // War dies die gewählte Heimat eines Schnitts und liegt er dort nicht mehr, verfällt die Wahl.
    if let ViewContent::Section { view } = old.views[index].content
        && let Some(Element::SectionView(v)) = project.elements.get(&view)
        && v.home == Some(sheet_id)
        && old.views.iter().filter(|p| p.content == old.views[index].content).count() == 1
    {
        let mut nv = v.clone();
        nv.home = None;
        ops.push(Operation::Replace { id: view, old: Element::SectionView(v.clone()), new: Element::SectionView(nv) });
    }
    ops
}

pub fn scale_ops(project: &Project, sheet_id: ElementId, index: usize, scale: f64) -> Vec<Operation> {
    let Some(old) = sheet(project, sheet_id) else { return Vec::new() };
    let mut new = old.clone();
    match new.views.get_mut(index) {
        Some(v) if scale > 0.0 => v.scale = scale,
        _ => return Vec::new(),
    }
    replace(project, sheet_id, new)
}

/// Plannummer, Planinhalt und Format.
pub fn info_ops(project: &Project, sheet_id: ElementId, number: &str, title: &str, format: &str) -> Vec<Operation> {
    let Some(old) = sheet(project, sheet_id) else { return Vec::new() };
    let mut new = old.clone();
    new.number = number.trim().to_string();
    new.title = title.trim().to_string();
    new.format = format.to_string();
    replace(project, sheet_id, new)
}

/// Löscht ein Blatt mit seinen 2D-Elementen; Schnitte mit diesem Blatt als Heimat verlieren die Wahl.
pub fn delete_sheet_ops(project: &Project, sheet_id: ElementId) -> Vec<Operation> {
    let Some(old) = project.elements.get(&sheet_id) else { return Vec::new() };
    let mut ops = annotation_deletes(project, AnnotationView::Sheet { sheet: sheet_id });
    for (id, v) in project.section_views() {
        if v.home == Some(sheet_id) {
            let mut nv = v.clone();
            nv.home = None;
            ops.push(Operation::Replace { id, old: Element::SectionView(v.clone()), new: Element::SectionView(nv) });
        }
    }
    ops.push(Operation::Delete { id: sheet_id, element: old.clone() });
    ops
}

fn annotation_deletes(project: &Project, view: AnnotationView) -> Vec<Operation> {
    project
        .elements
        .iter()
        .filter(|(_, e)| matches!(e, Element::Annotation(a) if a.view == view))
        .map(|(id, e)| Operation::Delete { id: *id, element: e.clone() })
        .collect()
}

/// Vor dem Löschen eines Schnitts: seine Platzierungen und 2D-Elemente.
pub fn forget_section_ops(project: &Project, view: ElementId) -> Vec<Operation> {
    let mut ops = annotation_deletes(project, AnnotationView::Section { view });
    let content = ViewContent::Section { view };
    for (id, s) in project.plan_sheets() {
        if s.views.iter().any(|p| p.content == content) {
            let mut new = s.clone();
            new.views.retain(|p| p.content != content);
            ops.extend(replace(project, id, new));
        }
    }
    ops
}

/// Wählt die Platzierung auf `sheet_id` als Heimat eines Schnitts.
pub fn set_home_ops(project: &Project, view: ElementId, sheet_id: ElementId) -> Result<Vec<Operation>, String> {
    let Some(Element::SectionView(v)) = project.elements.get(&view) else {
        return Err("Heimat gibt es nur für Schnitte und Ansichten".into());
    };
    let content = ViewContent::Section { view };
    if !sheet(project, sheet_id).is_some_and(|s| s.views.iter().any(|p| p.content == content)) {
        return Err(format!("{} liegt nicht auf diesem Blatt", v.name));
    }
    if v.home == Some(sheet_id) {
        return Ok(Vec::new());
    }
    let mut nv = v.clone();
    nv.home = Some(sheet_id);
    Ok(vec![Operation::Replace { id: view, old: Element::SectionView(v.clone()), new: Element::SectionView(nv) }])
}
