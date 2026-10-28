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

/// Ein Typ für die App.
fn type_info(p: &lot_model::Project, id: ElementId) -> Option<TypeInfo> {
    use lot_edit::types;
    let kind = types::kind_of(p, id)?;
    let layers = match p.elements.get(&id)? {
        lot_model::Element::WallType(t) => t.layers.as_slice(),
        lot_model::Element::SlabType(t) => t.layers.as_slice(),
        _ => &[],
    };
    let layers = layers
        .iter()
        .map(|l| LayerInfo {
            material: l.material.clone(),
            thickness: l.thickness,
            function: FUNCTIONS.iter().position(|(f, _)| *f == l.function).unwrap_or(0) as u32,
        })
        .collect();
    let uses = types::uses(p, id).len();
    let (one, many) = kind.element_names();
    let properties = kind
        .properties()
        .into_iter()
        .map(|prop| {
            let value = types::type_value(p, id, prop).unwrap_or_default();
            PropertyInfo {
                key: PROPERTIES.iter().position(|q| *q == prop).unwrap_or(0) as u32,
                title: prop.title().into(),
                value: value.clone(),
                from_type: true,
                locked: types::locked(p, id, prop),
                type_value: value,
                property: property_name(prop),
                is_size: matches!(prop, lot_model::Property::Width | lot_model::Property::Height),
            }
        })
        .collect();
    Some(TypeInfo {
        element: id.0,
        kind: match kind {
            types::TypeKind::Wall => 0,
            types::TypeKind::Slab => 1,
            types::TypeKind::Opening => 2,
        },
        kind_title: kind.title().into(),
        name: types::name(p, id)?,
        layers,
        uses: uses as u32,
        uses_text: format!("{uses} {}", if uses == 1 { one } else { many }),
        properties,
    })
}

/// Füllungen einer Öffnung zur Auswahl (ADR 0047), in der Reihenfolge von `OpeningInfo::fill`.
#[uniffi::export]
pub fn opening_fills() -> Vec<ChoiceItem> {
    lot_edit::openings::FILLS
        .iter()
        .enumerate()
        .map(|(i, t)| ChoiceItem { title: (*t).into(), command: lot_edit::Command::SetOpeningFill(i as u32).id() })
        .collect()
}

/// Eine Öffnung mit beiden Brüstungshöhen für den Inspektor (ADR 0047).
#[derive(Debug, Clone, uniffi::Record)]
pub struct OpeningInfo {
    pub element: u64,
    /// Index in [`opening_fills`].
    pub fill: u32,
    /// Fertige Brüstung (BRH) über OKFF, führend.
    pub sill: f64,
    /// Rohbaubrüstung über Oberkante Rohdecke; fehlt ohne Decke vor der Öffnung.
    pub rough_sill: Option<f64>,
    /// Fußbodenaufbau vor der Öffnung, von der Rohdecke bis OKFF.
    pub floor_build_up: Option<f64>,
    /// Unterer Anschluss aus dem Typ.
    pub sill_connection: f64,
    /// Typ mit Kennzeichen, etwa „F1 · Fenster“; leer ohne Typ.
    pub type_name: String,
    /// Tiefe einer Nische; fehlt bei Öffnungen durch die ganze Wand.
    pub recess_depth: Option<f64>,
    /// Die Nische liegt auf der linken Wandseite (in Zeichenrichtung).
    pub recess_left: bool,
    /// Wie viele Öffnungen den Typ nutzen.
    pub type_uses: u32,
}

/// Schichtfunktionen in der Reihenfolge von `LayerInfo::function`.
const FUNCTIONS: [(lot_model::LayerFunction, &str); 5] = [
    (lot_model::LayerFunction::Structure, "Tragschicht"),
    (lot_model::LayerFunction::Insulation, "Dämmung"),
    (lot_model::LayerFunction::AirGap, "Luftschicht"),
    (lot_model::LayerFunction::Service, "Installationsebene"),
    (lot_model::LayerFunction::Finish, "Putz oder Bekleidung"),
];

/// Die Schichtfunktionen zur Auswahl.
#[uniffi::export]
pub fn layer_functions() -> Vec<String> {
    FUNCTIONS.iter().map(|(_, t)| (*t).into()).collect()
}

/// Die Materialien des Bürostandards: Titel ist der Name, `command` der Schlüssel.
#[uniffi::export]
pub fn materials() -> Vec<ChoiceItem> {
    let mut list: Vec<ChoiceItem> = lot_plan::standard::OfficeStandard::default_standard()
        .materials
        .iter()
        .map(|(key, m)| ChoiceItem { title: m.name.clone(), command: key.clone() })
        .collect();
    list.sort_by(|a, b| a.title.cmp(&b.title));
    list
}

/// Eine Schicht eines Wandaufbaus.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct LayerInfo {
    /// Schlüssel des Materials im Bürostandard.
    pub material: String,
    pub thickness: f64,
    /// Index in [`layer_functions`].
    pub function: u32,
}

/// Werte, die ein Typ vorgeben kann (ADR 0048), in der Reihenfolge von `PropertyInfo::key`.
const PROPERTIES: [lot_model::Property; 6] = [
    lot_model::Property::Mark,
    lot_model::Property::IfcClass,
    lot_model::Property::FireRating,
    lot_model::Property::AcousticRating,
    lot_model::Property::Width,
    lot_model::Property::Height,
];

/// Ein Wert am Typ oder am Bauteil, mit Herkunft und den Befehlen dazu.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct PropertyInfo {
    /// Index in der Liste der Eigenschaften; für Befehle `property` verwenden.
    pub key: u32,
    pub title: String,
    pub value: String,
    /// Am Bauteil: Der Wert kommt vom Typ. Am Typ immer `true`.
    pub from_type: bool,
    /// Der Typ legt den Wert fest; Bauteile dürfen nicht abweichen.
    pub locked: bool,
    /// Wert des Typs, für „vom Typ“ und den Vergleich.
    pub type_value: String,
    /// RON-Name der Eigenschaft für Befehle, etwa „FireRating“.
    pub property: String,
    /// Ein Maß in Metern statt eines Texts.
    pub is_size: bool,
}

/// Ein Typ: Wandaufbau, Deckenaufbau, Tür- oder Fenstertyp (ADR 0048).
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct TypeInfo {
    pub element: u64,
    /// 0 Wandaufbau, 1 Deckenaufbau, 2 Tür- oder Fenstertyp.
    pub kind: u32,
    pub kind_title: String,
    pub name: String,
    /// Schichten von außen nach innen, bei Decken von oben nach unten; leer bei Tür- und Fenstertypen.
    pub layers: Vec<LayerInfo>,
    /// Wie viele Bauteile den Typ nutzen, und als Wort, etwa „14 Wände“.
    pub uses: u32,
    pub uses_text: String,
    pub properties: Vec<PropertyInfo>,
}

/// Name einer Eigenschaft in Befehlen (RON), etwa „FireRating“.
fn property_name(p: lot_model::Property) -> String {
    format!("{p:?}")
}

/// Eine Wand für den Inspektor./// Eine Wand für den Inspektor.
#[derive(Debug, Clone, uniffi::Record)]
pub struct WallInfo {
    pub element: u64,
    /// Index in [`wall_alignments`].
    pub alignment: u32,
    /// Index in [`wall_tops`].
    pub top: u32,
    pub wall_type: u64,
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
    /// Fertige Brüstung und Rohbaubrüstung je Öffnung beim letzten Blick,
    /// für den Hinweis, wenn sich die Rohbaubrüstung von selbst ändert (ADR 0047).
    sills: Mutex<std::collections::BTreeMap<ElementId, (f64, f64)>>,
}

impl LotDocument {
    pub(crate) fn lock(&self) -> MutexGuard<'_, Document> {
        // Ein Panic in einem anderen Aufruf soll die App nicht dauerhaft blockieren.
        self.inner.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn with(doc: Document, attachments: lot_io::project_file::Attachments) -> Arc<Self> {
        let sills = Mutex::new(doc.project().rough_sills());
        Arc::new(Self { inner: Mutex::new(doc), attachments: Mutex::new(attachments), sills })
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

    /// Die Öffnungen unter den Elementen `ids`.
    pub fn openings(&self, ids: Vec<u64>) -> Vec<OpeningInfo> {
        let doc = self.lock();
        let p = doc.project();
        ids.iter()
            .filter_map(|id| {
                let Some(lot_model::Element::Opening(o)) = p.elements.get(&ElementId(*id)) else { return None };
                let ty = o.opening_type.and_then(|t| p.opening_type(t));
                Some(OpeningInfo {
                    element: *id,
                    fill: lot_edit::openings::fill_index(o) as u32,
                    recess_depth: o.recess.map(|r| r.depth),
                    recess_left: o.recess.is_none_or(|r| r.side == lot_model::WallSide::Left),
                    sill: o.sill_height,
                    rough_sill: p.rough_sill(o),
                    floor_build_up: p.floor_build_up(o),
                    sill_connection: p.sill_connection(o),
                    type_name: ty
                        .map(|t| match &t.props.mark {
                            Some(m) => format!("{m} · {}", t.name),
                            None => t.name.clone(),
                        })
                        .unwrap_or_default(),
                    type_uses: o
                        .opening_type
                        .map_or(0, |t| p.openings().filter(|(_, x)| x.opening_type == Some(t)).count())
                        as u32,
                })
            })
            .collect()
    }

    /// Setzt die Rohbaubrüstung über Rohdecke; gespeichert wird die fertige Brüstung (ADR 0047).
    pub fn set_rough_sill(&self, element: u64, value: f64) -> Result<(), LotError> {
        let mut doc = self.lock();
        let Some(lot_model::Element::Opening(o)) = doc.project().elements.get(&ElementId(element)).cloned() else {
            return Err(LotError::Failed("Das ist keine Öffnung".into()));
        };
        let sill = lot_edit::openings::sill_for_rough(doc.project(), &o, value)
            .ok_or_else(|| LotError::Failed("Vor der Öffnung liegt keine Decke, die Rohbaubrüstung fehlt".into()))?;
        doc.set_param(ElementId(element), lot_model::Param::OpeningSillHeight, sill).map_err(LotError::from_display)
    }

    /// Hinweise auf Rohbaubrüstungen, die sich seit dem letzten Aufruf von
    /// selbst geändert haben, weil der Fußbodenaufbau anders wurde.
    pub fn rough_sill_notices(&self) -> Vec<String> {
        let doc = self.lock();
        let now = doc.project().rough_sills();
        let mut sills = self.sills.lock().unwrap_or_else(|e| e.into_inner());
        let notices = lot_model::rough_sill_changes(&sills, &now)
            .into_iter()
            .map(|(id, before, after)| {
                format!(
                    "{}: Rohbaubrüstung {} statt {}, weil sich der Fußbodenaufbau geändert hat. Die BRH bleibt.",
                    lot_edit::select::label(doc.project(), id),
                    lot_edit::format_m(after),
                    lot_edit::format_m(before)
                )
            })
            .collect();
        *sills = now;
        notices
    }

    /// Ein Typ mit Schichten, Nutzung und Werten.
    pub fn type_info(&self, element: u64) -> Option<TypeInfo> {
        let doc = self.lock();
        type_info(doc.project(), ElementId(element))
    }

    /// Alle Typen des Projekts für die Typen-Übersicht.
    pub fn types(&self) -> Vec<TypeInfo> {
        let doc = self.lock();
        let p = doc.project();
        lot_edit::types::all(p).into_iter().filter_map(|(id, _)| type_info(p, id)).collect()
    }

    /// Werte eines Bauteils, die sein Typ vorgibt, mit Herkunft (ADR 0048).
    pub fn element_properties(&self, element: u64) -> Vec<PropertyInfo> {
        let doc = self.lock();
        let p = doc.project();
        let id = ElementId(element);
        let Some(t) = lot_edit::types::type_of(p, id) else { return Vec::new() };
        let Some(kind) = lot_edit::types::kind_of(p, t) else { return Vec::new() };
        kind.properties()
            .into_iter()
            .map(|prop| {
                let (value, origin) = lot_edit::types::value(p, id, prop);
                PropertyInfo {
                    key: PROPERTIES.iter().position(|q| *q == prop).unwrap_or(0) as u32,
                    title: prop.title().into(),
                    value: value.unwrap_or_default(),
                    from_type: origin != lot_edit::types::Origin::Element,
                    locked: lot_edit::types::locked(p, t, prop),
                    type_value: lot_edit::types::type_value(p, t, prop).unwrap_or_default(),
                    property: property_name(prop),
                    is_size: matches!(prop, lot_model::Property::Width | lot_model::Property::Height),
                }
            })
            .collect()
    }

    /// Ändert Namen und Schichten eines Wand- oder Deckenaufbaus in einem
    /// Schritt. Die Bezugslinien der Wände bleiben stehen (ADR 0045).
    pub fn set_type_layers(&self, element: u64, name: String, layers: Vec<LayerInfo>) -> Result<(), LotError> {
        let mut doc = self.lock();
        let layers: Vec<lot_model::Layer> = layers
            .into_iter()
            .map(|l| lot_model::Layer {
                material: l.material,
                thickness: l.thickness,
                function: FUNCTIONS.get(l.function as usize).map_or(lot_model::LayerFunction::Finish, |f| f.0),
            })
            .collect();
        let new = match doc.project().elements.get(&ElementId(element)) {
            Some(lot_model::Element::WallType(t)) => {
                lot_model::Element::WallType(lot_model::WallType { name: name.trim().into(), layers, ..t.clone() })
            }
            Some(lot_model::Element::SlabType(t)) => {
                lot_model::Element::SlabType(lot_model::SlabType { name: name.trim().into(), layers, ..t.clone() })
            }
            _ => return Err(LotError::Failed("Das ist kein Wand- oder Deckenaufbau".into())),
        };
        doc.replace(ElementId(element), new).map_err(LotError::from_display)
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
                Some(WallInfo {
                    element: *id,
                    alignment,
                    top: w.top.mode() as u32,
                    top_height,
                    wall_type: w.wall_type.0,
                })
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
    fn wall_type_layers_change_in_one_step() {
        let doc = LotDocument::new_sample();
        let wall = doc.walls(vec![3]).pop().unwrap();
        let mut info = doc.type_info(wall.wall_type).unwrap();
        assert_eq!(info.uses, 1);
        let before = doc.lock().project().wall(ElementId(3)).unwrap().start;
        info.layers[0].thickness = 0.02;
        info.layers.insert(0, LayerInfo { material: "putz".into(), thickness: 0.01, function: 4 });
        doc.set_type_layers(info.element, "Neu".into(), info.layers.clone()).unwrap();
        let after = doc.type_info(info.element).unwrap();
        assert_eq!((after.name.as_str(), after.layers.len()), ("Neu", info.layers.len()));
        // Die Bezugslinie bleibt stehen.
        assert_eq!(doc.lock().project().wall(ElementId(3)).unwrap().start, before);
        let LotError::Failed(message) = doc
            .set_type_layers(
                info.element,
                "Neu".into(),
                vec![LayerInfo { material: "putz".into(), thickness: 0.0, function: 4 }],
            )
            .unwrap_err();
        assert!(message.contains("1 mm"), "{message}");
        assert!(doc.undo().unwrap());
        assert_eq!(doc.type_info(info.element).unwrap().layers.len(), info.layers.len() - 1);
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
