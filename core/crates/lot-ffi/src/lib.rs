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

    pub fn undo(&self) -> Result<bool, LotError> {
        self.lock().undo().map_err(LotError::from_display)
    }

    pub fn redo(&self) -> Result<bool, LotError> {
        self.lock().redo().map_err(LotError::from_display)
    }

    /// Bezeichnung eines Elements für die Anzeige, etwa „Wand · Außenwand WDVS 36“.
    pub fn element_label(&self, element: u64) -> String {
        let doc = self.lock();
        let p = doc.project();
        match p.elements.get(&ElementId(element)) {
            Some(lot_model::Element::Wall(w)) => {
                format!("Wand · {}", p.wall_type(w.wall_type).map_or("?", |t| t.name.as_str()))
            }
            Some(lot_model::Element::Opening(o)) => match o.kind {
                lot_model::OpeningKind::Plain => "Öffnung".into(),
                lot_model::OpeningKind::Door(_) => "Tür".into(),
                lot_model::OpeningKind::Window => "Fenster".into(),
            },
            Some(lot_model::Element::Storey(s)) => format!("Geschoss {}", s.name),
            Some(lot_model::Element::Column(_)) => "Stütze".into(),
            Some(lot_model::Element::DimensionChain(_)) => "Maßkette".into(),
            Some(lot_model::Element::Slab(s)) => match s.kind {
                lot_model::SlabKind::Floor => "Decke".into(),
                lot_model::SlabKind::Base => "Bodenplatte".into(),
            },
            Some(lot_model::Element::SlabType(t)) => format!("Deckenaufbau {}", t.name),
            Some(lot_model::Element::SlabOpening(_)) => "Deckendurchbruch".into(),
            Some(lot_model::Element::Roof(_)) => "Dach".into(),
            Some(lot_model::Element::RoofWindow(_)) => "Dachfenster".into(),
            Some(lot_model::Element::Dormer(_)) => "Gaube".into(),
            Some(lot_model::Element::Stair(s)) => format!("Treppe · {} Steigungen", s.risers()),
            Some(lot_model::Element::Railing(_)) => "Geländer".into(),
            Some(lot_model::Element::SectionView(v)) => v.name.clone(),
            Some(lot_model::Element::Underlay(u)) => format!("Unterlage · {}", u.source),
            Some(lot_model::Element::Terrain(t)) => format!("Gelände · {} Höhenpunkte", t.points.len()),
            Some(lot_model::Element::ReferenceModel(r)) => format!("Referenzmodell · {}", r.source),
            Some(lot_model::Element::VariantSet(v)) => format!("Varianten {} · {}", v.name, v.active_name()),
            Some(lot_model::Element::Annotation(a)) => match &a.shape {
                lot_model::AnnotationShape::Text { text, .. } => format!("Text · {text}"),
                lot_model::AnnotationShape::Polyline(_) => "Linie".into(),
                lot_model::AnnotationShape::Polygon { .. } => "Fläche".into(),
                lot_model::AnnotationShape::Circle { .. } => "Kreis".into(),
            },
            Some(lot_model::Element::ProjectInfo(_)) => "Projektangaben".into(),
            Some(lot_model::Element::Plot(p)) => {
                if p.neighbour {
                    format!("Nachbargrundstück {}", p.name)
                } else {
                    format!("Grundstück {}", p.name)
                }
            }
            Some(lot_model::Element::Elevator(e)) => {
                format!("Aufzug · Fahrkorb {:.2} x {:.2} m", e.cabin_width, e.cabin_depth).replace('.', ",")
            }
            Some(lot_model::Element::PlanSheet(s)) => format!("Planblatt {} {}", s.number, s.title),
            Some(lot_model::Element::Space(r)) => format!("Raum {} {}", r.number, r.name).trim_end().to_string(),
            Some(lot_model::Element::GridLine(g)) => format!("Achse {}", g.label),
            Some(lot_model::Element::WallType(t)) => format!("Wandaufbau {}", t.name),
            None => String::new(),
        }
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
