//! Varianten (2.04): Bauteile, die es in mehreren Fassungen gibt, etwa eine
//! Küche offen oder abgetrennt. Bauteile, die in keiner Variante stehen,
//! gehören zu allen.

use serde::{Deserialize, Serialize};

use crate::ElementId;

/// Variantengruppe: eine Frage mit mehreren Antworten.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VariantSet {
    pub name: String,
    pub variants: Vec<Variant>,
    /// Welche Variante Pläne und 3D zeigen, ab 0.
    #[serde(default)]
    pub active: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Variant {
    pub name: String,
    /// Bauteile, die nur in dieser Variante vorkommen.
    #[serde(default)]
    pub elements: Vec<ElementId>,
}

impl VariantSet {
    /// Bauteile der Varianten, die gerade nicht gezeigt werden.
    pub fn hidden(&self) -> impl Iterator<Item = ElementId> + '_ {
        self.variants
            .iter()
            .enumerate()
            .filter(move |(i, _)| *i != self.active)
            .flat_map(|(_, v)| v.elements.iter().copied())
    }

    pub fn active_name(&self) -> &str {
        self.variants.get(self.active).map_or("", |v| v.name.as_str())
    }
}
