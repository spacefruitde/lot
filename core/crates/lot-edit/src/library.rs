//! Büro-Bibliothek (ADR 0048): Typen einzeln ins Projekt holen, mit
//! Vorher-Nachher-Vergleich, wenn es dort schon einen Typ gleichen Namens
//! gibt, und Typen aus dem Projekt in die Bibliothek legen. Die Bibliothek
//! steht neben dem Bürostandard (ADR 0040); ohne eigene gilt die eingebaute.

use lot_model::{Element, ElementId, Layer, OpeningType, Operation, Project, Properties, Property, SlabType, WallType};
use serde::{Deserialize, Serialize};

use crate::types::{self, TypeKind};

/// Typen eines Büros.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Library {
    #[serde(default)]
    pub wall_types: Vec<WallType>,
    #[serde(default)]
    pub slab_types: Vec<SlabType>,
    #[serde(default)]
    pub opening_types: Vec<OpeningType>,
}

impl Library {
    /// Die eingebaute Bibliothek, ein Vorschlag zum Start.
    pub fn builtin() -> Library {
        Library::parse(include_str!("bibliothek.ron")).expect("eingebaute Bibliothek ist lesbar")
    }

    pub fn parse(text: &str) -> Result<Library, String> {
        ron::from_str(text).map_err(|e| format!("Die Bibliothek ist nicht lesbar: {e}"))
    }

    pub fn to_ron(&self) -> String {
        let config = ron::ser::PrettyConfig::new().depth_limit(3);
        let body = ron::ser::to_string_pretty(self, config).unwrap_or_default();
        format!("// Typen-Bibliothek des Büros (ADR 0048), geschrieben von Lot.\n{body}\n")
    }

    /// Alle Typen der Bibliothek als Elemente, in fester Reihenfolge.
    pub fn items(&self) -> Vec<(TypeKind, Element)> {
        let walls = self.wall_types.iter().map(|t| (TypeKind::Wall, Element::WallType(t.clone())));
        let slabs = self.slab_types.iter().map(|t| (TypeKind::Slab, Element::SlabType(t.clone())));
        let openings = self.opening_types.iter().map(|t| (TypeKind::Opening, Element::OpeningType(t.clone())));
        walls.chain(slabs).chain(openings).collect()
    }

    /// Die Bibliothek mit dem Typ `type_id` aus dem Projekt; ein Typ gleichen
    /// Namens und gleicher Art wird ersetzt.
    pub fn with_type(&self, project: &Project, type_id: ElementId) -> Option<Library> {
        let mut out = self.clone();
        match project.elements.get(&type_id)? {
            Element::WallType(t) => put(&mut out.wall_types, t.clone(), |x| &x.name),
            Element::SlabType(t) => put(&mut out.slab_types, t.clone(), |x| &x.name),
            Element::OpeningType(t) => put(&mut out.opening_types, t.clone(), |x| &x.name),
            _ => return None,
        }
        Some(out)
    }
}

fn put<T>(list: &mut Vec<T>, item: T, name: impl Fn(&T) -> &String) {
    match list.iter().position(|x| name(x) == name(&item)) {
        Some(i) => list[i] = item,
        None => list.push(item),
    }
}

/// Wie ein Typ der Bibliothek zum Projekt steht.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// Im Projekt gibt es keinen Typ dieses Namens.
    New,
    /// Ein Typ gleichen Namens ist genauso.
    Same,
    /// Ein Typ gleichen Namens weicht ab: Vorher-Nachher-Vergleich.
    Different,
}

/// Ein Typ der Bibliothek im Vergleich zum Projekt.
#[derive(Debug, Clone, PartialEq)]
pub struct Comparison {
    /// Index in [`Library::items`].
    pub index: usize,
    pub kind: TypeKind,
    pub name: String,
    pub status: Status,
    /// Der Typ gleichen Namens im Projekt.
    pub existing: Option<ElementId>,
    /// Was sich ändert, etwa „Schicht 2: Dämmung 16 cm → 18 cm“.
    pub differences: Vec<String>,
    /// Bauteile, die der Typ im Projekt schon nutzt.
    pub uses: usize,
}

/// Jeder Typ der Bibliothek im Vergleich zum Projekt. `material` gibt den
/// Namen eines Materials für die Anzeige.
pub fn compare(project: &Project, library: &Library, material: &dyn Fn(&str) -> String) -> Vec<Comparison> {
    library
        .items()
        .into_iter()
        .enumerate()
        .map(|(index, (kind, element))| {
            let name = element_name(&element);
            let existing = types::all(project)
                .into_iter()
                .find(|(id, k)| *k == kind && types::name(project, *id).as_deref() == Some(name.as_str()))
                .map(|(id, _)| id);
            let differences = existing
                .and_then(|id| project.elements.get(&id))
                .map(|old| differences(old, &element, material))
                .unwrap_or_default();
            let status = match (existing, differences.is_empty()) {
                (None, _) => Status::New,
                (Some(_), true) => Status::Same,
                (Some(_), false) => Status::Different,
            };
            let uses = existing.map_or(0, |id| types::uses(project, id).len());
            Comparison { index, kind, name, status, existing, differences, uses }
        })
        .collect()
}

fn element_name(e: &Element) -> String {
    match e {
        Element::WallType(t) => t.name.clone(),
        Element::SlabType(t) => t.name.clone(),
        Element::OpeningType(t) => t.name.clone(),
        _ => String::new(),
    }
}

fn cm(m: f64) -> String {
    let v = (m * 1000.0).round() / 10.0;
    format!("{v} cm").replace('.', ",")
}

/// Unterschiede von `old` (Projekt) zu `new` (Bibliothek) in Worten.
fn differences(old: &Element, new: &Element, material: &dyn Fn(&str) -> String) -> Vec<String> {
    let mut out = Vec::new();
    let layers = |e: &Element| -> Option<Vec<Layer>> {
        match e {
            Element::WallType(t) => Some(t.layers.clone()),
            Element::SlabType(t) => Some(t.layers.clone()),
            _ => None,
        }
    };
    if let (Some(a), Some(b)) = (layers(old), layers(new)) {
        let layer = |l: &Layer| format!("{} {}", material(&l.material), cm(l.thickness));
        if a.len() != b.len() {
            out.push(format!("{} statt {} Schichten", b.len(), a.len()));
        }
        for (i, (x, y)) in a.iter().zip(&b).enumerate() {
            if x != y {
                out.push(format!("Schicht {}: {} → {}", i + 1, layer(x), layer(y)));
            }
        }
        for (i, y) in b.iter().enumerate().skip(a.len()) {
            out.push(format!("Schicht {} neu: {}", i + 1, layer(y)));
        }
    }
    if let (Element::OpeningType(a), Element::OpeningType(b)) = (old, new) {
        for p in [Property::Width, Property::Height] {
            if a.size(p) != b.size(p) {
                let f = |v: Option<f64>| v.map_or("keine".into(), crate::format_m);
                out.push(format!("{}: {} → {}", p.title(), f(a.size(p)), f(b.size(p))));
            }
        }
        if a.sill_connection != b.sill_connection {
            out.push(format!(
                "Unterer Anschluss: {} → {}",
                crate::format_m(a.sill_connection),
                crate::format_m(b.sill_connection)
            ));
        }
    }
    let props = |e: &Element| -> Properties {
        match e {
            Element::WallType(t) => t.props.clone(),
            Element::SlabType(t) => t.props.clone(),
            Element::OpeningType(t) => t.props.clone(),
            _ => Properties::default(),
        }
    };
    let (pa, pb) = (props(old), props(new));
    for p in Property::TEXT {
        if pa.get(p) != pb.get(p) {
            let f = |v: Option<&String>| v.cloned().unwrap_or_else(|| "keins".into());
            out.push(format!("{}: {} → {}", p.title(), f(pa.get(p)), f(pb.get(p))));
        }
    }
    out
}

/// Wie ein Typ aus der Bibliothek ins Projekt kommt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImportMode {
    /// Als neuer Typ; bei gleichem Namen mit Zusatz „(Bibliothek)“.
    Add,
    /// Ersetzt den Typ gleichen Namens; alle Bauteile des Typs ändern sich mit.
    Replace,
}

/// Operationen, die den Typ `index` der Bibliothek ins Projekt holen, und die Kennung des Typs dort.
pub fn import_ops(
    project: &Project,
    library: &Library,
    index: usize,
    mode: ImportMode,
) -> Result<(ElementId, Vec<Operation>), String> {
    let (kind, mut element) = library.items().into_iter().nth(index).ok_or("Diesen Typ gibt es nicht")?;
    let name = element_name(&element);
    let existing = types::all(project)
        .into_iter()
        .find(|(id, k)| *k == kind && types::name(project, *id).as_deref() == Some(name.as_str()))
        .map(|(id, _)| id);
    match (mode, existing) {
        (ImportMode::Replace, Some(id)) => {
            let old = project.elements.get(&id).cloned().ok_or("Diesen Typ gibt es nicht")?;
            let mut ops = vec![Operation::Replace { id, old, new: element.clone() }];
            if let Element::OpeningType(t) = &element {
                for p in [Property::Width, Property::Height] {
                    if let Some(v) = t.size(p) {
                        ops.extend(types::sync_sizes(project, id, p, v));
                    }
                }
            }
            Ok((id, ops))
        }
        (ImportMode::Replace, None) | (ImportMode::Add, None) => {
            let id = project.next_id();
            Ok((id, vec![Operation::Insert { id, element }]))
        }
        (ImportMode::Add, Some(_)) => {
            let taken =
                |n: &str| types::all(project).iter().any(|(id, _)| types::name(project, *id).as_deref() == Some(n));
            let new_name = (1..)
                .map(|i| if i == 1 { format!("{name} (Bibliothek)") } else { format!("{name} (Bibliothek {i})") })
                .take(100)
                .find(|n| !taken(n))
                .ok_or("Kein freier Name")?;
            match &mut element {
                Element::WallType(t) => t.name = new_name,
                Element::SlabType(t) => t.name = new_name,
                Element::OpeningType(t) => t.name = new_name,
                _ => {}
            }
            let id = project.next_id();
            Ok((id, vec![Operation::Insert { id, element }]))
        }
    }
}
