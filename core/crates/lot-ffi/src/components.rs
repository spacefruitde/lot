//! Decken, Dächer und Treppen im Inspektor (ADR 0051).

use lot_model::{Element, ElementId, FallPurpose, RoofEdge, SlabKind};

use crate::LotDocument;

/// Eine gewählte Decke: folgt sie den Wänden, und ihr Gefälle.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct SlabInfo {
    pub element: u64,
    /// „Decke“, „Bodenplatte“ oder „Flachdach“.
    pub kind: String,
    pub follows: bool,
    /// Soll folgen, kann aber nicht: eine Wand fehlt oder kam dazu.
    pub broken: bool,
    pub has_fall: bool,
    /// Index in [`fall_purposes`].
    pub purpose: u32,
    pub percent: f64,
    pub direction: f64,
    /// Schicht des Gefälles, von oben, und die Namen der Schichten.
    pub layer: u32,
    pub layers: Vec<String>,
    pub drains: u32,
    pub overflows: u32,
}

/// Ein gewähltes Dach mit seinen Kanten.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct RoofInfo {
    pub element: u64,
    pub edges: Vec<RoofEdgeInfo>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct RoofEdgeInfo {
    pub index: u32,
    /// Traufe, sonst Giebel.
    pub eave: bool,
    /// Neigung in Grad, bei Giebeln 0.
    pub slope: f64,
    pub overhang: f64,
    /// Länge der Kante in m.
    pub length: f64,
}

/// Eine gewählte Treppe mit der Prüfung nach DIN 18065.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct StairInfo {
    pub element: u64,
    /// Etwa „16 Stg. 17,2/28,5“.
    pub summary: String,
    pub problems: Vec<String>,
}

#[uniffi::export]
impl LotDocument {
    pub fn slabs(&self, ids: Vec<u64>) -> Vec<SlabInfo> {
        let doc = self.lock();
        let project = doc.project();
        let standard = lot_plan::standard::OfficeStandard::default_standard();
        ids.into_iter()
            .filter_map(|id| match project.elements.get(&ElementId(id)) {
                Some(Element::Slab(s)) => {
                    let layers = project
                        .slab_type(s.slab_type)
                        .map(|t| {
                            t.layers
                                .iter()
                                .map(|l| {
                                    format!(
                                        "{} {}",
                                        standard.material(&l.material).name,
                                        lot_edit::format_m(l.thickness)
                                    )
                                })
                                .collect()
                        })
                        .unwrap_or_default();
                    let fall = s.fall.as_deref();
                    Some(SlabInfo {
                        element: id,
                        kind: match s.kind {
                            SlabKind::Floor => "Decke",
                            SlabKind::Base => "Bodenplatte",
                            SlabKind::Roof => "Flachdach",
                        }
                        .into(),
                        follows: s.follows_walls,
                        broken: lot_edit::follow::broken(project, s),
                        has_fall: fall.is_some(),
                        purpose: fall.and_then(|f| FallPurpose::ALL.iter().position(|p| *p == f.purpose)).unwrap_or(0)
                            as u32,
                        percent: fall.map_or(2.0, |f| f.percent),
                        direction: fall.map_or(0.0, |f| f.direction),
                        layer: fall.map_or(0, |f| f.layer as u32),
                        layers,
                        drains: fall.map_or(0, |f| f.drains.len() as u32),
                        overflows: fall.map_or(0, |f| f.overflows.len() as u32),
                    })
                }
                _ => None,
            })
            .collect()
    }

    pub fn roofs(&self, ids: Vec<u64>) -> Vec<RoofInfo> {
        let doc = self.lock();
        let project = doc.project();
        ids.into_iter()
            .filter_map(|id| match project.elements.get(&ElementId(id)) {
                Some(Element::Roof(r)) => {
                    let n = r.outline.len();
                    Some(RoofInfo {
                        element: id,
                        edges: r
                            .edges
                            .iter()
                            .enumerate()
                            .map(|(i, e)| {
                                let length = r.outline[i].distance(r.outline[(i + 1) % n]);
                                match *e {
                                    RoofEdge::Eave { slope, overhang } => {
                                        RoofEdgeInfo { index: i as u32, eave: true, slope, overhang, length }
                                    }
                                    RoofEdge::Verge { overhang } => {
                                        RoofEdgeInfo { index: i as u32, eave: false, slope: 0.0, overhang, length }
                                    }
                                }
                            })
                            .collect(),
                    })
                }
                _ => None,
            })
            .collect()
    }

    pub fn stairs(&self, ids: Vec<u64>) -> Vec<StairInfo> {
        let doc = self.lock();
        let project = doc.project();
        ids.into_iter()
            .filter_map(|id| {
                let stair = project.stair(ElementId(id))?;
                let check = lot_rules::stairs::check_stair(project, ElementId(id), stair)?;
                let cm = |v: f64| format!("{:.1}", v * 100.0).replace('.', ",");
                Some(StairInfo {
                    element: id,
                    summary: format!(
                        "{} Stg. {}/{} · Schrittmaß {} cm",
                        check.risers,
                        cm(check.rise),
                        cm(check.tread),
                        cm(check.step)
                    ),
                    problems: check.problems,
                })
            })
            .collect()
    }
}

/// Verwendungen eines Gefälles: Flachdach, Balkon, Dusche.
#[uniffi::export]
pub fn fall_purposes() -> Vec<String> {
    FallPurpose::ALL.iter().map(|p| p.title().to_string()).collect()
}

/// RON der Verwendung nach Index, für `SetFallPurpose`.
#[uniffi::export]
pub fn fall_purpose_field(index: Option<u32>) -> String {
    match index.and_then(|i| FallPurpose::ALL.get(i as usize)) {
        Some(p) => format!("SetFallPurpose(Some({p:?}))"),
        None => "SetFallPurpose(None)".into(),
    }
}
