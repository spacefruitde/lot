//! Schnittstelle des Kerns für die Swift-App, erzeugt mit UniFFI.
//!
//! Die Swift-Seite sieht zwei Objekte:
//!
//! - [`LotDocument`]: ein geöffnetes Projekt mit Parametern, Undo/Redo, Speichern und Laden
//! - [`Viewport`]: Planansicht und 3D-Ansicht, die über wgpu in einen NSView
//!   der App zeichnen, mit Werkzeugen und Befehlen (`viewport.rs`)
//!
//! Die Swift-Anbindung entsteht mit `app-mac/scripts/build-core.sh`.

mod viewport;

use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};

use lot_model::{Document, ElementId, Param, Project};
pub use viewport::Viewport;

uniffi::setup_scaffolding!();

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum LotError {
    #[error("{0}")]
    Failed(String),
}

impl LotError {
    fn from_display(e: impl std::fmt::Display) -> Self {
        LotError::Failed(e.to_string())
    }
}

/// Parameter, die die App ändern kann. Spiegelt `lot_model::Param`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, uniffi::Enum)]
pub enum ParamKind {
    WallLength,
    WallSagitta,
    WallHeight,
    WallTopOffset,
    BaseOffset,
    OpeningOffset,
    OpeningWidth,
    OpeningHeight,
    OpeningSillHeight,
    StoreyElevation,
    StoreyHeight,
    ColumnWidth,
    ColumnDepth,
    RoofEaveHeight,
    RoofSlope,
}

impl From<ParamKind> for Param {
    fn from(k: ParamKind) -> Self {
        match k {
            ParamKind::WallLength => Param::WallLength,
            ParamKind::WallSagitta => Param::WallSagitta,
            ParamKind::WallHeight => Param::WallHeight,
            ParamKind::WallTopOffset => Param::WallTopOffset,
            ParamKind::BaseOffset => Param::BaseOffset,
            ParamKind::OpeningOffset => Param::OpeningOffset,
            ParamKind::OpeningWidth => Param::OpeningWidth,
            ParamKind::OpeningHeight => Param::OpeningHeight,
            ParamKind::OpeningSillHeight => Param::OpeningSillHeight,
            ParamKind::StoreyElevation => Param::StoreyElevation,
            ParamKind::StoreyHeight => Param::StoreyHeight,
            ParamKind::ColumnWidth => Param::ColumnWidth,
            ParamKind::ColumnDepth => Param::ColumnDepth,
            ParamKind::RoofEaveHeight => Param::RoofEaveHeight,
            ParamKind::RoofSlope => Param::RoofSlope,
        }
    }
}

const ALL_PARAMS: [ParamKind; 15] = [
    ParamKind::WallLength,
    ParamKind::WallSagitta,
    ParamKind::WallHeight,
    ParamKind::WallTopOffset,
    ParamKind::BaseOffset,
    ParamKind::OpeningOffset,
    ParamKind::OpeningWidth,
    ParamKind::OpeningHeight,
    ParamKind::OpeningSillHeight,
    ParamKind::StoreyElevation,
    ParamKind::StoreyHeight,
    ParamKind::ColumnWidth,
    ParamKind::ColumnDepth,
    ParamKind::RoofEaveHeight,
    ParamKind::RoofSlope,
];

/// Raum mit Stempeltext und Flächen, für den Inspektor.
#[derive(Debug, Clone, uniffi::Record)]
pub struct SpaceInfo {
    pub element: u64,
    pub number: String,
    pub name: String,
    /// Index in [`space_usages`].
    pub usage: u32,
    /// Zählt zur Wohnfläche nach WoFlV.
    pub living_space: bool,
    /// Netto-Raumfläche nach DIN 277 in m², fehlt bei offenen Räumen.
    pub net_area: Option<f64>,
    /// Wohnfläche nach WoFlV in m².
    pub living_area: Option<f64>,
}

/// Tür mit Anschlag und Aufschlag, für den Inspektor.
#[derive(Debug, Clone, uniffi::Record)]
pub struct DoorInfo {
    pub element: u64,
    /// DIN links nach DIN 107, sonst DIN rechts.
    pub din_left: bool,
    /// Schlägt zur linken Seite der Wand auf (in Zeichenrichtung).
    pub opens_left: bool,
}

/// Nutzungsgruppe nach DIN 277 zur Auswahl.
/// Teil einer Auswahl gleicher Art und gleichen Typs, etwa 12 Wände KS 17,5.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct SelectionGroup {
    pub label: String,
    pub ids: Vec<u64>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct SpaceUsageItem {
    pub code: String,
    pub title: String,
}

/// Die Nutzungsgruppen nach DIN 277:2021-08, in der Reihenfolge der Norm.
#[uniffi::export]
pub fn space_usages() -> Vec<SpaceUsageItem> {
    lot_model::SpaceUsage::ALL
        .iter()
        .map(|u| SpaceUsageItem { code: u.code().into(), title: u.title().into() })
        .collect()
}

/// Eine Lage der Bezugslinie zur Auswahl, mit dem Befehl, der sie setzt.
#[derive(Debug, Clone, uniffi::Record)]
pub struct ChoiceItem {
    pub title: String,
    pub command: String,
}

/// Die sechs Lagen der Bezugslinie (ADR 0045), in der Reihenfolge von `WallInfo::alignment`.
#[uniffi::export]
pub fn wall_alignments() -> Vec<ChoiceItem> {
    lot_model::WallAlignment::ALL
        .iter()
        .map(|a| ChoiceItem { title: a.title().into(), command: lot_edit::Command::SetAlignment(*a).id() })
        .collect()
}

/// Die Arten der Oberkante einer Wand, in der Reihenfolge von `WallInfo::top`.
#[uniffi::export]
pub fn wall_tops() -> Vec<ChoiceItem> {
    lot_model::Top::TITLES
        .iter()
        .enumerate()
        .map(|(i, t)| ChoiceItem { title: (*t).into(), command: lot_edit::Command::SetWallTop(i as u32).id() })
        .collect()
}

/// Eine Wand für den Inspektor.
#[derive(Debug, Clone, uniffi::Record)]
pub struct WallInfo {
    pub element: u64,
    /// Index in [`wall_alignments`].
    pub alignment: u32,
    /// Index in [`wall_tops`].
    pub top: u32,
    /// Oberkante über OKFF des Geschosses, wie sie sich ergibt.
    pub top_height: f64,
}

/// Flächen eines Geschosses in m².
#[derive(Debug, Clone, uniffi::Record)]
pub struct StoreyAreaInfo {
    pub name: String,
    pub net: f64,
    pub gross: f64,
    pub living: f64,
}

/// Ein Parameterwert eines Elements, in Metern.
#[derive(Debug, Clone, uniffi::Record)]
pub struct ParamValue {
    pub element: u64,
    pub kind: ParamKind,
    pub value: f64,
}

/// Ein Befund des Prüfberichts für die App.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct FindingInfo {
    /// Abschnitt, etwa „Abstandsflächen“.
    pub section: String,
    /// 0 erfüllt, 1 Hinweis, 2 nicht erfüllt.
    pub status: u32,
    pub text: String,
    pub source: String,
    /// Geschoss und Ort im Grundriss, falls der Befund einen hat.
    pub storey: Option<u64>,
    pub x: f64,
    pub y: f64,
}

/// Setzt den Bürostandard für die Sitzung: der eingebaute mit den
/// Abweichungen aus `text` (Inhalt von `buerostandard.ron`). Einmal beim Start,
/// bevor ein Plan gezeichnet wird.
#[uniffi::export]
pub fn install_office_standard(text: String) -> Result<(), LotError> {
    let standard = lot_plan::standard::OfficeStandard::with_overrides(&text).map_err(LotError::Failed)?;
    if lot_plan::standard::OfficeStandard::install(standard) {
        Ok(())
    } else {
        Err(LotError::Failed("Der Bürostandard ist schon gesetzt".into()))
    }
}

/// Der Prüfbericht eines Projekts als Liste für die App.
pub(crate) fn findings_of(report: &lot_rules::CheckReport) -> Vec<FindingInfo> {
    report
        .sections
        .iter()
        .flat_map(|s| {
            s.findings.iter().map(move |f| FindingInfo {
                section: s.title.clone(),
                status: match f.status {
                    lot_rules::Status::Ok => 0,
                    lot_rules::Status::Note => 1,
                    lot_rules::Status::Fail => 2,
                },
                text: f.text.clone(),
                source: f.source.clone(),
                storey: f.storey.map(|id| id.0),
                x: f.at.map_or(0.0, |p| p.x),
                y: f.at.map_or(0.0, |p| p.y),
            })
        })
        .collect()
}

#[derive(uniffi::Object)]
pub struct LotDocument {
    inner: Mutex<Document>,
    /// Dateien im Projekt, etwa Unterlagen (6.05).
    attachments: Mutex<lot_io::project_file::Attachments>,
}

impl LotDocument {
    pub(crate) fn lock(&self) -> MutexGuard<'_, Document> {
        // Ein Panic in einem anderen Aufruf soll die App nicht dauerhaft blockieren.
        self.inner.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn with(doc: Document, attachments: lot_io::project_file::Attachments) -> Arc<Self> {
        Arc::new(Self { inner: Mutex::new(doc), attachments: Mutex::new(attachments) })
    }

    pub(crate) fn attachment_bytes(&self, name: &str) -> Option<Vec<u8>> {
        self.attachments.lock().unwrap_or_else(|e| e.into_inner()).get(name).cloned()
    }
}

#[uniffi::export]
impl LotDocument {
    /// Beispielprojekt aus M0: eine Wand mit Fenster.
    #[uniffi::constructor]
    pub fn new_sample() -> Arc<Self> {
        Self::with(Document::new(Project::sample()), Default::default())
    }

    /// Neues, leeres Projekt mit Erdgeschoss und üblichen Wandaufbauten.
    #[uniffi::constructor]
    pub fn new_empty() -> Arc<Self> {
        Self::with(Document::new(Project::template("Neues Projekt")), Default::default())
    }

    /// Testprojekt aus einem Ordner mit `projekt.ron` (zum Ausprobieren, etwa das EFH).
    #[uniffi::constructor]
    pub fn open_testproject(path: String) -> Result<Arc<Self>, LotError> {
        let path = Path::new(&path);
        let project = lot_io::testproject::load(path).map_err(LotError::from_display)?;
        // Unterlagen liegen bei Testprojekten als Dateien im Projektordner.
        let dir = if path.is_dir() { path } else { path.parent().unwrap_or(Path::new(".")) };
        let attachments = lot_io::testproject::attachments(&project, dir);
        Ok(Self::with(Document::new(project), attachments))
    }

    #[uniffi::constructor]
    pub fn open(path: String) -> Result<Arc<Self>, LotError> {
        let doc = lot_io::project_file::load(Path::new(&path)).map_err(LotError::from_display)?;
        let attachments = lot_io::project_file::load_attachments(Path::new(&path)).map_err(LotError::from_display)?;
        Ok(Self::with(doc, attachments))
    }

    pub fn save(&self, path: String) -> Result<(), LotError> {
        let attachments = self.attachments.lock().unwrap_or_else(|e| e.into_inner()).clone();
        lot_io::project_file::save_with(&self.lock(), &attachments, Path::new(&path)).map_err(LotError::from_display)
    }

    /// Legt eine Datei im Projekt ab, etwa eine Unterlage. Gleicher Name ersetzt.
    pub fn add_attachment(&self, name: String, data: Vec<u8>) {
        self.attachments.lock().unwrap_or_else(|e| e.into_inner()).insert(name, data);
    }

    /// Inhalt einer Datei im Projekt.
    pub fn attachment(&self, name: String) -> Option<Vec<u8>> {
        self.attachment_bytes(&name)
    }

    pub fn name(&self) -> String {
        self.lock().project().name.clone()
    }

    /// Alle änderbaren Parameter der Elemente `ids`, oder aller Elemente, wenn `ids` leer ist.
    pub fn parameters(&self, ids: Vec<u64>) -> Vec<ParamValue> {
        let doc = self.lock();
        let mut out = Vec::new();
        for (id, element) in &doc.project().elements {
            if !ids.is_empty() && !ids.contains(&id.0) {
                continue;
            }
            for kind in ALL_PARAMS {
                if let Some(value) = element.param(kind.into()) {
                    out.push(ParamValue { element: id.0, kind, value });
                }
            }
        }
        out
    }

    pub fn set_param(&self, element: u64, kind: ParamKind, value: f64) -> Result<(), LotError> {
        self.lock().set_param(ElementId(element), kind.into(), value).map_err(LotError::from_display)
    }

    /// Einen Parameter an mehreren Elementen setzen, als ein Undo-Schritt;
    /// `values` gehört der Reihe nach zu `elements`.
    pub fn set_params(&self, elements: Vec<u64>, kind: ParamKind, values: Vec<f64>) -> Result<(), LotError> {
        let changes: Vec<(ElementId, lot_model::Param, f64)> =
            elements.into_iter().zip(values).map(|(e, v)| (ElementId(e), kind.into(), v)).collect();
        self.lock().set_params("Eigenschaft ändern", &changes).map_err(LotError::from_display)
    }

    pub fn undo(&self) -> Result<bool, LotError> {
        self.lock().undo().map_err(LotError::from_display)
    }

    pub fn redo(&self) -> Result<bool, LotError> {
        self.lock().redo().map_err(LotError::from_display)
    }

    /// Bezeichnung eines Elements für die Anzeige, etwa „Wand · Außenwand WDVS 36 · 4,20 m“.
    pub fn element_label(&self, element: u64) -> String {
        lot_edit::select::label(self.lock().project(), ElementId(element))
    }

    /// Die Elemente nach Art und Typ gruppiert, für den Inspektor (ADR 0043).
    pub fn selection_groups(&self, ids: Vec<u64>) -> Vec<SelectionGroup> {
        let ids: Vec<ElementId> = ids.into_iter().map(ElementId).collect();
        lot_edit::select::groups(self.lock().project(), &ids)
            .into_iter()
            .map(|(label, ids)| SelectionGroup { label, ids: ids.into_iter().map(|i| i.0).collect() })
            .collect()
    }

    /// Raumdaten der gewählten Elemente, die Räume sind.
    pub fn spaces(&self, ids: Vec<u64>) -> Vec<SpaceInfo> {
        let doc = self.lock();
        let project = doc.project();
        let wanted: Vec<_> = ids
            .iter()
            .filter_map(|id| match project.elements.get(&ElementId(*id)) {
                Some(lot_model::Element::Space(r)) => Some((*id, r)),
                _ => None,
            })
            .collect();
        if wanted.is_empty() {
            return Vec::new();
        }
        let report = lot_rules::area_report(&project.variant_view());
        wanted
            .into_iter()
            .map(|(id, r)| {
                let area = report.spaces().find(|a| a.id == ElementId(id));
                SpaceInfo {
                    element: id,
                    number: r.number.clone(),
                    name: r.name.clone(),
                    usage: lot_model::SpaceUsage::ALL.iter().position(|u| *u == r.usage).unwrap_or(0) as u32,
                    living_space: r.living_space,
                    net_area: area.and_then(|a| a.net),
                    living_area: area.and_then(|a| a.living),
                }
            })
            .collect()
    }

    /// Die Wände unter den Elementen `ids`.
    pub fn walls(&self, ids: Vec<u64>) -> Vec<WallInfo> {
        let doc = self.lock();
        ids.iter()
            .filter_map(|id| {
                let w = doc.project().wall(ElementId(*id))?;
                let alignment = lot_model::WallAlignment::ALL.iter().position(|a| *a == w.alignment)? as u32;
                let p = doc.project();
                let top_height = p.wall_extent(w)?.1 - p.storey(w.storey)?.elevation;
                Some(WallInfo { element: *id, alignment, top: w.top.mode() as u32, top_height })
            })
            .collect()
    }

    /// Die Türen unter den Elementen `ids`.
    pub fn doors(&self, ids: Vec<u64>) -> Vec<DoorInfo> {
        let doc = self.lock();
        ids.iter()
            .filter_map(|id| match doc.project().elements.get(&ElementId(*id)) {
                Some(lot_model::Element::Opening(lot_model::Opening {
                    kind: lot_model::OpeningKind::Door(d), ..
                })) => Some(DoorInfo {
                    element: *id,
                    din_left: d.hinge == lot_model::Handing::DinLeft,
                    opens_left: d.opens_to == lot_model::WallSide::Left,
                }),
                _ => None,
            })
            .collect()
    }

    /// Ändert Nummer, Name, Nutzung und Wohnflächen-Merkmal eines Raums, als eine Operation.
    pub fn set_space(
        &self,
        element: u64,
        number: String,
        name: String,
        usage: u32,
        living_space: bool,
    ) -> Result<(), LotError> {
        let mut doc = self.lock();
        let id = ElementId(element);
        let Some(lot_model::Element::Space(old)) = doc.project().elements.get(&id).cloned() else {
            return Err(LotError::Failed("Das ist kein Raum".into()));
        };
        let usage = *lot_model::SpaceUsage::ALL
            .get(usage as usize)
            .ok_or_else(|| LotError::Failed("Unbekannte Nutzung".into()))?;
        let new =
            lot_model::Space { number: number.trim().into(), name: name.trim().into(), usage, living_space, ..old };
        doc.replace(id, lot_model::Element::Space(new)).map_err(LotError::from_display)
    }

    /// Bauzustand eines Elements: 0 Bestand, 1 Abbruch, 2 Neubau; `None`, wenn es keinen hat.
    pub fn element_phase(&self, element: u64) -> Option<u32> {
        let doc = self.lock();
        let mut e = doc.project().elements.get(&ElementId(element))?.clone();
        // Nur Elemente, die einen Bauzustand haben.
        if !e.set_phase(lot_model::Phase::New) {
            return None;
        }
        Some(match doc.project().phase_of(ElementId(element)) {
            lot_model::Phase::Existing => 0,
            lot_model::Phase::Demolished => 1,
            lot_model::Phase::New => 2,
        })
    }

    /// Inhalt eines 2D-Texts; `None`, wenn das Element kein Text ist.
    pub fn annotation_text(&self, element: u64) -> Option<String> {
        match self.lock().project().elements.get(&ElementId(element)) {
            Some(lot_model::Element::Annotation(lot_model::Annotation {
                shape: lot_model::AnnotationShape::Text { text, .. },
                ..
            })) => Some(text.clone()),
            _ => None,
        }
    }

    /// Ändert Inhalt und Schrifthöhe (mm auf dem Papier) eines 2D-Texts.
    pub fn set_annotation_text(&self, element: u64, text: String, height: f64) -> Result<(), LotError> {
        let mut doc = self.lock();
        let id = ElementId(element);
        let Some(lot_model::Element::Annotation(old)) = doc.project().elements.get(&id).cloned() else {
            return Err(LotError::Failed("Das ist kein Text".into()));
        };
        let lot_model::AnnotationShape::Text { position, rotation, .. } = old.shape else {
            return Err(LotError::Failed("Das ist kein Text".into()));
        };
        let shape = lot_model::AnnotationShape::Text { position, text: text.trim().into(), height, rotation };
        doc.replace(id, lot_model::Element::Annotation(lot_model::Annotation { shape, ..old }))
            .map_err(LotError::from_display)
    }

    /// Schrifthöhe eines 2D-Texts in mm; `None`, wenn das Element kein Text ist.
    pub fn annotation_text_height(&self, element: u64) -> Option<f64> {
        match self.lock().project().elements.get(&ElementId(element)) {
            Some(lot_model::Element::Annotation(lot_model::Annotation {
                shape: lot_model::AnnotationShape::Text { height, .. },
                ..
            })) => Some(*height),
            _ => None,
        }
    }

    /// Flächen je Geschoss (NRF, BGF, Wohnfläche).
    pub fn storey_areas(&self) -> Vec<StoreyAreaInfo> {
        let doc = self.lock();
        lot_rules::area_report(&doc.project().variant_view())
            .storeys
            .iter()
            .map(|s| StoreyAreaInfo {
                name: s.name.clone(),
                net: s.net,
                gross: s.gross,
                living: s.spaces.iter().filter_map(|r| r.living).sum(),
            })
            .collect()
    }

    /// Prüfbericht (5.01 bis 5.05): Gebäudeklasse, Abstandsflächen, Maß der
    /// baulichen Nutzung, Barrierefreiheit, Rettungswege, Treppen.
    pub fn check_report(&self) -> Vec<FindingInfo> {
        let project = self.lock().project().variant_view();
        match lot_geom::build(&project) {
            Ok(geometry) => findings_of(&lot_rules::check_project(&project, &geometry)),
            Err(e) => vec![FindingInfo {
                section: "Prüfung".into(),
                status: 2,
                text: format!("Geometrie nicht berechenbar: {e}"),
                source: String::new(),
                storey: None,
                x: 0.0,
                y: 0.0,
            }],
        }
    }

    pub fn can_undo(&self) -> bool {
        self.lock().can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.lock().can_redo()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn value(doc: &LotDocument, kind: ParamKind) -> f64 {
        doc.parameters(Vec::new()).into_iter().find(|p| p.kind == kind).unwrap().value
    }

    #[test]
    fn sample_has_all_parameters() {
        let doc = LotDocument::new_sample();
        // Geschoss: Höhe und Lage; Wand: Länge, Stich, Höhe, Versatz; Öffnung: vier Maße.
        assert_eq!(doc.parameters(Vec::new()).len(), 2 + 4 + 4);
        assert_eq!(doc.parameters(vec![4]).len(), 4);
        assert_eq!(value(&doc, ParamKind::OpeningWidth), 1.5);
    }

    #[test]
    fn change_undo_redo() {
        let doc = LotDocument::new_sample();
        doc.set_param(4, ParamKind::OpeningWidth, 2.0).unwrap();
        assert!(doc.can_undo());
        assert!(doc.undo().unwrap());
        assert_eq!(value(&doc, ParamKind::OpeningWidth), 1.5);
        assert!(doc.redo().unwrap());
        assert_eq!(value(&doc, ParamKind::OpeningWidth), 2.0);
    }

    #[test]
    fn invalid_value_gives_german_message() {
        let doc = LotDocument::new_sample();
        let LotError::Failed(message) = doc.set_param(4, ParamKind::OpeningWidth, 21.5).unwrap_err();
        assert!(message.contains("ragt seitlich aus der Wand"), "{message}");
        assert_eq!(value(&doc, ParamKind::OpeningWidth), 1.5);
    }

    #[test]
    fn save_and_open() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("wand.lot").to_string_lossy().into_owned();
        let doc = LotDocument::new_sample();
        doc.set_param(3, ParamKind::WallHeight, 2.75).unwrap();
        doc.save(path.clone()).unwrap();
        let opened = LotDocument::open(path).unwrap();
        assert_eq!(value(&opened, ParamKind::WallHeight), 2.75);
        assert_eq!(opened.name(), "M0 Wand mit Fenster");
    }

    #[test]
    fn check_report_lists_findings() {
        let doc = LotDocument::new_sample();
        let findings = doc.check_report();
        assert!(findings.iter().any(|f| f.section == "Gebäudeklasse" && f.status == 0), "{findings:?}");
    }

    #[test]
    fn opening_missing_file_fails_cleanly() {
        assert!(LotDocument::open("/gibt/es/nicht.lot".into()).is_err());
    }

    #[test]
    fn space_can_be_renamed_and_undone() {
        let doc = LotDocument::new_empty();
        let storey = doc.lock().project().storeys()[0].0;
        let id = doc
            .lock()
            .insert(lot_model::Element::Space(lot_model::Space {
                phase: Default::default(),
                storey,
                position: lot_model::Point2::new(1.0, 1.0),
                number: "0.01".into(),
                name: "Raum".into(),
                usage: lot_model::SpaceUsage::Living,
                living_space: true,
                unit: String::new(),
            }))
            .unwrap();
        doc.set_space(id.0, " 0.07 ".into(), "Küche".into(), 8, false).unwrap();
        let info = &doc.spaces(vec![id.0])[0];
        assert_eq!(
            (info.number.as_str(), info.name.as_str(), info.usage, info.living_space),
            ("0.07", "Küche", 8, false)
        );
        assert!(info.net_area.is_none(), "ohne Wände ist der Raum offen");
        assert_eq!(space_usages()[8].code, "VF");
        doc.undo().unwrap();
        assert_eq!(doc.spaces(vec![id.0])[0].name, "Raum");
        assert!(doc.set_space(id.0, "1".into(), " ".into(), 0, true).is_err(), "Name darf nicht leer sein");
    }
}
