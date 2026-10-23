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

    pub fn slabs_on(&self, storey: ElementId) -> impl Iterator<Item = (ElementId, &crate::Slab)> {
        self.elements.iter().filter_map(move |(id, e)| match e {
            Element::Slab(s) if s.storey == storey => Some((*id, s)),
            _ => None,
        })
    }

    /// Durchbrüche in der Decke `slab`.
    pub fn slab_openings_in(&self, slab: ElementId) -> impl Iterator<Item = (ElementId, &crate::SlabOpening)> {
        self.elements.iter().filter_map(move |(id, e)| match e {
            Element::SlabOpening(o) if o.host == slab => Some((*id, o)),
            _ => None,
        })
    }

    pub fn slab_type(&self, id: ElementId) -> Option<&crate::SlabType> {
        match self.elements.get(&id) {
            Some(Element::SlabType(t)) => Some(t),
            _ => None,
        }
    }

    /// Alle Deckenaufbauten, nach Kennung.
    pub fn slab_types(&self) -> impl Iterator<Item = (ElementId, &crate::SlabType)> {
        self.elements.iter().filter_map(|(id, e)| match e {
            Element::SlabType(t) => Some((*id, t)),
            _ => None,
        })
    }

    pub fn dimensions_on(&self, storey: ElementId) -> impl Iterator<Item = (ElementId, &crate::DimensionChain)> {
        self.elements.iter().filter_map(move |(id, e)| match e {
            Element::DimensionChain(d) if d.storey == storey => Some((*id, d)),
            _ => None,
        })
    }

    pub fn spaces_on(&self, storey: ElementId) -> impl Iterator<Item = (ElementId, &crate::Space)> {
        self.elements.iter().filter_map(move |(id, e)| match e {
            Element::Space(r) if r.storey == storey => Some((*id, r)),
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
            Top::Roof { offset } => {
                // Höchster Punkt der Dachunterseite; im 3D wird darunter genau geschnitten.
                let highest = self
                    .roofs_on(storey)
                    .flat_map(|(_, r)| {
                        let t = self.roof_thickness(r);
                        r.outline.clone().into_iter().chain(ridge_samples(r)).map(move |p| r.underside_at(p, t))
                    })
                    .fold(f64::NEG_INFINITY, f64::max);
                if highest.is_finite() { s.elevation + highest + offset } else { s.elevation + s.height + offset }
            }
        };
        Some((bottom, top))
    }

    /// Unter- und Oberkante einer Wand über ±0,00.
    pub fn wall_extent(&self, wall: &Wall) -> Option<(f64, f64)> {
        self.vertical_extent(wall.storey, wall.base_offset, wall.top)
    }

    /// Alle Schnitte und Ansichten, nach Kennung.
    pub fn section_views(&self) -> impl Iterator<Item = (ElementId, &crate::SectionView)> {
        self.elements.iter().filter_map(|(id, e)| match e {
            Element::SectionView(v) => Some((*id, v)),
            _ => None,
        })
    }

    pub fn roofs_on(&self, storey: ElementId) -> impl Iterator<Item = (ElementId, &crate::Roof)> {
        self.elements.iter().filter_map(move |(id, e)| match e {
            Element::Roof(r) if r.storey == storey => Some((*id, r)),
            _ => None,
        })
    }

    /// Dicke eines Dachs aus seinem Aufbau, senkrecht zur Dachfläche.
    pub fn roof_thickness(&self, roof: &crate::Roof) -> f64 {
        self.slab_type(roof.roof_type).map_or(0.0, |t| t.thickness())
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
        let slabs = [
            (
                "Geschossdecke StB 20",
                vec![
                    layer("estrich", 0.05, Finish),
                    layer("daemmung", 0.03, Insulation),
                    layer("stahlbeton", 0.2, Structure),
                    layer("putz", 0.015, Finish),
                ],
            ),
            (
                "Bodenplatte StB 25",
                vec![
                    layer("estrich", 0.06, Finish),
                    layer("daemmung_hart", 0.1, Insulation),
                    layer("stahlbeton", 0.25, Structure),
                ],
            ),
            (
                "Steildach Sparren 22",
                vec![
                    layer("dachziegel", 0.06, Finish),
                    layer("daemmung", 0.22, Insulation),
                    layer("gipskarton", 0.0125, Finish),
                ],
            ),
        ];
        for (i, (name, layers)) in slabs.into_iter().enumerate() {
            p.elements
                .insert(ElementId(20 + i as u64), Element::SlabType(crate::SlabType { name: name.into(), layers }));
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
                sagitta: 0.0,
            }),
        );
        p.elements.insert(
            ElementId(4),
            Element::Opening(Opening {
                host: ElementId(3),
                offset: 1.75,
                width: 1.5,
                height: 1.3,
                sill_height: 0.9,
                kind: crate::OpeningKind::Plain,
            }),
        );
        p
    }
}

/// Punkte, an denen ein Dach am höchsten sein kann: Mitten zwischen allen Eckpaaren.
fn ridge_samples(r: &crate::Roof) -> Vec<Point2> {
    let n = r.outline.len();
    let mut points = Vec::new();
    for i in 0..n {
        for j in i + 1..n {
            let (a, b) = (r.outline[i], r.outline[j]);
            points.push(Point2::new((a.x + b.x) / 2.0, (a.y + b.y) / 2.0));
        }
    }
    let c = r.outline.iter().fold((0.0, 0.0), |acc, p| (acc.0 + p.x / n as f64, acc.1 + p.y / n as f64));
    points.push(Point2::new(c.0, c.1));
    points
}
