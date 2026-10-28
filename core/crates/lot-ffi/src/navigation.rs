//! Navigation für die App (ADR 0049): Seitenleiste „Projekt“, Suche,
//! Geschossleiste, Schnitte und Planblätter im Inspektor.

use lot_edit::navigation::{self, Target, ViewRef};
use lot_model::{Element, ElementId, SectionKind};

use crate::LotDocument;

/// Eine Gruppe der Seitenleiste, etwa „Schnitte und Ansichten“.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct NavGroup {
    pub title: String,
    pub entries: Vec<NavEntry>,
}

/// Eine Zeile der Seitenleiste; `view` ist die Kennung der Ansicht, etwa `Section(12)`.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct NavEntry {
    /// Eindeutig in der Seitenleiste, auch wenn eine Ansicht auf mehreren Blättern liegt.
    pub key: String,
    pub view: String,
    pub title: String,
    pub detail: String,
    /// Bei Ansichten auf einem Blatt: Blatt und Stelle; sonst 0 und -1.
    pub sheet: u64,
    pub index: i32,
    pub home: bool,
    /// Lässt sich aufs Blatt ziehen (Grundriss, Schnitt, Ansicht, Lageplan).
    pub placeable: bool,
    pub children: Vec<NavEntry>,
}

/// Ein Treffer für ⌘K: `target` ist die Kennung des Sprungziels.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct NavHit {
    pub target: String,
    pub title: String,
    pub detail: String,
}

/// Ein Geschoss in der Geschossleiste, von oben nach unten.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct StoreyRow {
    pub element: u64,
    pub name: String,
    /// OKFF über ±0,00 in Metern.
    pub elevation: f64,
    pub height: f64,
}

/// Ein gewählter Schnitt oder eine Ansicht für den Inspektor.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct SectionInfo {
    pub element: u64,
    pub name: String,
    pub mark: String,
    pub depth: f64,
    pub elevation: bool,
    /// „auf Plan 201, 301“ oder leer.
    pub sheets: String,
}

/// Plannummer, Planinhalt und Format eines Blatts.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct SheetInfo {
    pub element: u64,
    pub number: String,
    pub title: String,
    pub format: String,
}

fn entry(e: navigation::Entry, parent: &str) -> NavEntry {
    let (sheet, index) = e.placement.map_or((0, -1), |(s, i)| (s.0, i as i32));
    let key = if index >= 0 { format!("{parent}/{sheet}/{index}") } else { format!("{parent}/{}", e.view.id()) };
    NavEntry {
        view: e.view.id(),
        placeable: e.view.content().is_some(),
        title: e.title,
        detail: e.detail,
        sheet,
        index,
        home: e.home,
        children: e.children.into_iter().map(|c| entry(c, &key)).collect(),
        key,
    }
}

#[uniffi::export]
impl LotDocument {
    /// Die Seitenleiste „Projekt“.
    pub fn project_tree(&self) -> Vec<NavGroup> {
        let doc = self.lock();
        navigation::tree(doc.project())
            .into_iter()
            .map(|g| {
                let title = g.title.clone();
                NavGroup { entries: g.entries.into_iter().map(|e| entry(e, &title)).collect(), title }
            })
            .collect()
    }

    /// Ansichten und Räume für ⌘K, nach allen Wörtern der Suche.
    pub fn search(&self, query: String) -> Vec<NavHit> {
        let doc = self.lock();
        navigation::search(doc.project(), &query)
            .into_iter()
            .map(|h| NavHit { target: h.target.id(), title: h.title, detail: h.detail })
            .collect()
    }

    /// Die Geschosse von oben nach unten.
    pub fn storey_rows(&self) -> Vec<StoreyRow> {
        let doc = self.lock();
        doc.project()
            .storeys()
            .into_iter()
            .rev()
            .map(|(id, s)| StoreyRow { element: id.0, name: s.name.clone(), elevation: s.elevation, height: s.height })
            .collect()
    }

    /// Schnitte und Ansichten unter den Kennungen.
    pub fn sections(&self, ids: Vec<u64>) -> Vec<SectionInfo> {
        let doc = self.lock();
        let project = doc.project();
        ids.into_iter()
            .filter_map(|id| match project.elements.get(&ElementId(id)) {
                Some(Element::SectionView(v)) => Some(SectionInfo {
                    element: id,
                    name: v.name.clone(),
                    mark: v.mark.clone(),
                    depth: v.depth,
                    elevation: v.kind == SectionKind::Elevation,
                    sheets: navigation::tree(project)
                        .into_iter()
                        .flat_map(|g| g.entries)
                        .find(|e| e.view == ViewRef::Section(id))
                        .map(|e| e.detail)
                        .unwrap_or_default(),
                }),
                _ => None,
            })
            .collect()
    }

    pub fn sheet_info(&self, sheet: u64) -> Option<SheetInfo> {
        let doc = self.lock();
        match doc.project().elements.get(&ElementId(sheet)) {
            Some(Element::PlanSheet(s)) => Some(SheetInfo {
                element: sheet,
                number: s.number.clone(),
                title: s.title.clone(),
                format: s.format.clone(),
            }),
            _ => None,
        }
    }

    /// Titel einer Ansicht oder eines Sprungziels, etwa für das Fenster.
    pub fn view_title(&self, view: String) -> String {
        let doc = self.lock();
        match (ViewRef::parse(&view), Target::parse(&view)) {
            (Some(v), _) | (_, Some(Target::View(v))) => v.title(doc.project()),
            _ => String::new(),
        }
    }
}

/// Blattformate für die Auswahl im Inspektor.
#[uniffi::export]
pub fn sheet_formats() -> Vec<String> {
    lot_edit::sheets::FORMATS.iter().map(|f| f.to_string()).collect()
}
