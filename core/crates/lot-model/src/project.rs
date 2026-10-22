use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    Column, Element, ElementId, Layer, LayerFunction, Opening, Point2, Storey, Top, Wall, WallAlignment, WallType,
};

/// Ein Projekt: alle Elemente mit ihren Parametern.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub elements: BTreeMap<ElementId, Element>,
}

impl Project {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), elements: BTreeMap::new() }
    }

    /// Nächste freie Kennung. Wird aus dem Bestand abgeleitet und nicht gespeichert.
    pub fn next_id(&self) -> ElementId {
        ElementId(self.elements.keys().last().map_or(1, |id| id.0 + 1))
    }

    pub fn storey(&self, id: ElementId) -> Option<&Storey> {
        match self.elements.get(&id) {
            Some(Element::Storey(s)) => Some(s),
            _ => None,
        }
    }

    pub fn wall_type(&self, id: ElementId) -> Option<&WallType> {
        match self.elements.get(&id) {
            Some(Element::WallType(t)) => Some(t),
            _ => None,
        }
    }

    pub fn wall(&self, id: ElementId) -> Option<&Wall> {
        match self.elements.get(&id) {
            Some(Element::Wall(w)) => Some(w),
            _ => None,
        }
    }

    /// Alle Geschosse, von unten nach oben.
    pub fn storeys(&self) -> Vec<(ElementId, &Storey)> {
        let mut storeys: Vec<_> = self
            .elements
            .iter()
            .filter_map(|(id, e)| match e {
                Element::Storey(s) => Some((*id, s)),
                _ => None,
            })
            .collect();
        storeys.sort_by(|a, b| a.1.elevation.total_cmp(&b.1.elevation).then(a.0.cmp(&b.0)));
        storeys
    }

    pub fn walls(&self) -> impl Iterator<Item = (ElementId, &Wall)> {
        self.elements.iter().filter_map(|(id, e)| match e {
            Element::Wall(w) => Some((*id, w)),
            _ => None,
        })
    }

    pub fn walls_on(&self, storey: ElementId) -> impl Iterator<Item = (ElementId, &Wall)> {
        self.walls().filter(move |(_, w)| w.storey == storey)
    }

    pub fn columns_on(&self, storey: ElementId) -> impl Iterator<Item = (ElementId, &Column)> {
        self.elements.iter().filter_map(move |(id, e)| match e {
            Element::Column(c) if c.storey == storey => Some((*id, c)),
            _ => None,
        })
    }

    pub fn grid_lines(&self) -> impl Iterator<Item = (ElementId, &crate::GridLine)> {
        self.elements.iter().filter_map(|(id, e)| match e {
            Element::GridLine(g) => Some((*id, g)),
            _ => None,
        })
    }

    /// Alle Öffnungen, die in der Wand `host` sitzen.
    pub fn openings_in(&self, host: ElementId) -> impl Iterator<Item = (ElementId, &Opening)> {
        self.elements.iter().filter_map(move |(id, e)| match e {
            Element::Opening(o) if o.host == host => Some((*id, o)),
            _ => None,
        })
    }

    /// Unter- und Oberkante eines Bauteils über ±0,00, aus Geschoss, Versatz und Oberkante.
    pub fn vertical_extent(&self, storey: ElementId, base_offset: f64, top: Top) -> Option<(f64, f64)> {
        let s = self.storey(storey)?;
        let bottom = s.elevation + base_offset;
        let top = match top {
            Top::Height(h) => bottom + h,
            Top::Storey { offset } => s.elevation + s.height + offset,
        };
        Some((bottom, top))
    }

    /// Unter- und Oberkante einer Wand über ±0,00.
    pub fn wall_extent(&self, wall: &Wall) -> Option<(f64, f64)> {
        self.vertical_extent(wall.storey, wall.base_offset, wall.top)
    }

    /// Neues, leeres Projekt mit einem Erdgeschoss und üblichen Wandaufbauten.
    /// Die Aufbauten sind ein Vorschlag; der Bürostandard ersetzt sie später (2.08).
    pub fn template(name: impl Into<String>) -> Self {
        use LayerFunction::*;
        let layer = |material: &str, thickness: f64, function: LayerFunction| Layer {
            material: material.into(),
            thickness,
            function,
        };
        let mut p = Project::new(name);
        p.elements.insert(ElementId(1), Element::Storey(Storey { name: "EG".into(), elevation: 0.0, height: 2.75 }));
        let types = [
            (
                "Außenwand WDVS 36",
                vec![
                    layer("putz", 0.01, Finish),
                    layer("daemmung_hart", 0.16, Insulation),
                    layer("kalksandstein", 0.175, Structure),
                    layer("putz", 0.015, Finish),
                ],
            ),
            (
                "Innenwand tragend KS 17,5",
                vec![
                    layer("putz", 0.015, Finish),
                    layer("kalksandstein", 0.175, Structure),
                    layer("putz", 0.015, Finish),
                ],
            ),
            (
                "Innenwand nichttragend KS 11,5",
                vec![
                    layer("putz", 0.015, Finish),
                    layer("kalksandstein", 0.115, Structure),
                    layer("putz", 0.015, Finish),
                ],
            ),
            (
                "Trockenbauwand 10",
                vec![
                    layer("gipskarton", 0.0125, Finish),
                    layer("daemmung", 0.075, Service),
                    layer("gipskarton", 0.0125, Finish),
                ],
            ),
            ("Stahlbetonwand 25", vec![layer("stahlbeton", 0.25, Structure)]),
        ];
        for (i, (name, layers)) in types.into_iter().enumerate() {
            p.elements.insert(ElementId(10 + i as u64), Element::WallType(WallType { name: name.into(), layers }));
        }
        p
    }

    /// Alle Wandaufbauten, nach Kennung.
    pub fn wall_types(&self) -> impl Iterator<Item = (ElementId, &WallType)> {
        self.elements.iter().filter_map(|(id, e)| match e {
            Element::WallType(t) => Some((*id, t)),
            _ => None,
        })
    }

    /// Beispielprojekt für M0: eine 5 m lange Wand mit einem Fenster.
    /// Entspricht `testprojekte/m0-wand/projekt.ron`.
    pub fn sample() -> Self {
        let mut p = Project::new("M0 Wand mit Fenster");
        p.elements.insert(ElementId(1), Element::Storey(Storey { name: "EG".into(), elevation: 0.0, height: 2.75 }));
        p.elements.insert(
            ElementId(2),
            Element::WallType(WallType {
                name: "Kalksandstein 30".into(),
                layers: vec![Layer {
                    material: "kalksandstein".into(),
                    thickness: 0.3,
                    function: LayerFunction::Structure,
                }],
            }),
        );
        p.elements.insert(
            ElementId(3),
            Element::Wall(Wall {
                storey: ElementId(1),
                wall_type: ElementId(2),
                start: Point2::new(0.0, 0.0),
                end: Point2::new(5.0, 0.0),
                alignment: WallAlignment::Center,
                base_offset: 0.0,
                top: Top::Height(2.6),
            }),
        );
        p.elements.insert(
            ElementId(4),
            Element::Opening(Opening { host: ElementId(3), offset: 1.75, width: 1.5, height: 1.3, sill_height: 0.9 }),
        );
        p
    }
}
