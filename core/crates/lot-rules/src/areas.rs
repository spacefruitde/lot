//! Flächen und Rauminhalt.
//!
//! **DIN 277:2021-08** „Grundflächen und Rauminhalte im Hochbau“:
//!
//! - Netto-Raumfläche (NRF): Grundfläche zwischen den fertigen Oberflächen der
//!   umschließenden Bauteile in Fußbodenhöhe. Sie teilt sich in Nutzungsfläche
//!   (NUF 1 bis 7), Technikfläche (TF) und Verkehrsfläche (VF).
//! - Brutto-Grundfläche (BGF): Grundfläche innerhalb der äußeren Maße der
//!   Bauteile, einschließlich Bekleidung (Putz, Dämmung).
//! - Konstruktions-Grundfläche (KGF): BGF minus NRF, also Wände und Stützen.
//! - Brutto-Rauminhalt (BRI): BGF mal Höhe, je Geschoss von OKFF bis OKFF,
//!   im untersten Geschoss zusätzlich die Dicke der Bodenplatte. Bis das Dach im
//!   Modell ist (M3), endet das oberste Geschoss vorläufig an seiner Geschosshöhe.
//!
//! **WoFlV vom 25.11.2003** (BGBl. I S. 2346):
//!
//! - § 2 Abs. 3: Zubehörräume (Keller, Heizungsräume, Abstellräume außerhalb der
//!   Wohnung) zählen nicht. Im Modell entscheidet das Merkmal `living_space` des Raums.
//! - § 3 Abs. 1: gemessen ab der Vorderkante der Bekleidung, wie bei der NRF.
//! - § 3 Abs. 3 Nr. 1: Stützen bis 0,1 m² Grundfläche oder bis 1,50 m Höhe werden nicht abgezogen.
//! - § 3 Abs. 3 Nr. 3: Türnischen bleiben außer Betracht; das ergibt sich, weil
//!   Räume ohne Öffnungen ermittelt werden.
//! - § 4: Anrechnung nach lichter Höhe (unter 1 m gar nicht, 1 bis 2 m zur Hälfte).
//!   Ohne Dachschrägen im Modell (M3) haben alle Räume volle Höhe und zählen ganz.

use lot_geom::spaces::SpaceFinder;
use lot_model::{ElementId, Project, SpaceUsage};

/// Fläche eines Raums.
#[derive(Debug, Clone, PartialEq)]
pub struct SpaceArea {
    pub id: ElementId,
    pub number: String,
    pub name: String,
    pub usage: SpaceUsage,
    /// Netto-Raumfläche in m²; `None`, wenn der Raum nicht von Wänden umschlossen ist.
    pub net: Option<f64>,
    /// Wohnfläche nach WoFlV in m²; `None` bei Zubehörräumen und offenen Räumen.
    pub living: Option<f64>,
}

/// Flächen eines Geschosses.
#[derive(Debug, Clone, PartialEq)]
pub struct StoreyAreas {
    pub storey: ElementId,
    pub name: String,
    pub spaces: Vec<SpaceArea>,
    /// Brutto-Grundfläche, m²
    pub gross: f64,
    /// Netto-Raumfläche aller umschlossenen Bereiche, auch ohne Raum, m²
    pub net: f64,
    /// Teil der NRF in Bereichen ohne Raum, m²
    pub unassigned: f64,
    /// Konstruktions-Grundfläche, m²
    pub construction: f64,
    /// Höhe für den Brutto-Rauminhalt (Geschosshöhe, unten mit Bodenplatte), m
    pub height: f64,
}

impl StoreyAreas {
    /// Brutto-Rauminhalt, m³ (BGF mal Höhe).
    pub fn volume(&self) -> f64 {
        self.gross * self.height
    }
}

/// Flächen aller Geschosse.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AreaReport {
    pub storeys: Vec<StoreyAreas>,
    /// Hinweise, etwa auf Räume ohne geschlossene Wände.
    pub warnings: Vec<String>,
}

impl AreaReport {
    /// Netto-Raumfläche je Nutzungsgruppe über alle Geschosse, in der Reihenfolge der Norm.
    pub fn net_by_usage(&self) -> Vec<(SpaceUsage, f64)> {
        SpaceUsage::ALL
            .iter()
            .map(|u| {
                let sum = self.spaces().filter(|r| r.usage == *u).filter_map(|r| r.net).sum();
                (*u, sum)
            })
            .filter(|(_, a)| *a > 0.0)
            .collect()
    }

    pub fn spaces(&self) -> impl Iterator<Item = &SpaceArea> {
        self.storeys.iter().flat_map(|s| s.spaces.iter())
    }

    pub fn gross(&self) -> f64 {
        self.storeys.iter().map(|s| s.gross).sum()
    }

    pub fn net(&self) -> f64 {
        self.storeys.iter().map(|s| s.net).sum()
    }

    pub fn construction(&self) -> f64 {
        self.storeys.iter().map(|s| s.construction).sum()
    }

    pub fn volume(&self) -> f64 {
        self.storeys.iter().map(StoreyAreas::volume).sum()
    }

    /// Nutzungsfläche (NUF 1 bis 7).
    pub fn usable(&self) -> f64 {
        self.spaces().filter(|r| r.usage.is_usable()).filter_map(|r| r.net).sum()
    }

    /// Wohnfläche nach WoFlV.
    pub fn living(&self) -> f64 {
        self.spaces().filter_map(|r| r.living).sum()
    }
}

/// Rechnet die Flächen aller Geschosse.
pub fn area_report(project: &Project) -> AreaReport {
    let mut report = AreaReport::default();
    for (storey_id, storey) in project.storeys() {
        let finder = SpaceFinder::new(project, storey_id);
        let mut claimed: Vec<(usize, String)> = Vec::new();
        let mut spaces = Vec::new();
        let mut list: Vec<_> = project.spaces_on(storey_id).collect();
        list.sort_by(|a, b| a.1.number.cmp(&b.1.number).then(a.0.cmp(&b.0)));
        for (id, space) in list {
            let p = [space.position.x, space.position.y];
            let label = format!("{} {}", space.number, space.name).trim().to_string();
            let index = finder.position_of(p);
            match index {
                Some(i) => {
                    if let Some((_, other)) = claimed.iter().find(|(j, _)| *j == i) {
                        report.warnings.push(format!("Raum {label} liegt im selben Bereich wie Raum {other}"));
                    } else {
                        claimed.push((i, label.clone()));
                    }
                }
                None => report.warnings.push(format!("Raum {label} ist nicht von Wänden umschlossen")),
            }
            let net = finder.area_at(p).map(|a| a.area());
            // WoFlV § 4 Nr. 1: Ohne Dachschrägen haben alle Räume mindestens 2 m lichte Höhe.
            let living = if space.living_space { finder.living_area_at(p) } else { None };
            spaces.push(SpaceArea {
                id,
                number: space.number.clone(),
                name: space.name.clone(),
                usage: space.usage,
                net,
                living,
            });
        }
        let net: f64 = finder.enclosed.iter().map(|a| a.area()).sum();
        let assigned: f64 = claimed.iter().map(|(i, _)| finder.enclosed[*i].area()).sum();
        let gross = finder.gross_area();
        let base = project
            .slabs_on(storey_id)
            .filter(|(_, s)| s.kind == lot_model::SlabKind::Base)
            .filter_map(|(_, s)| project.slab_type(s.slab_type).map(|t| t.thickness()))
            .fold(0.0, f64::max);
        report.storeys.push(StoreyAreas {
            storey: storey_id,
            name: storey.name.clone(),
            spaces,
            gross,
            net,
            unassigned: net - assigned,
            construction: gross - net,
            height: storey.height + base,
        });
    }
    report
}

#[cfg(test)]
mod tests {
    use lot_model::{
        Column, ColumnProfile, Element, ElementId, Point2, Project, Space, SpaceUsage, Top, Wall, WallAlignment,
    };

    use super::*;

    /// Vorlage mit einem Raum 5 × 4 m (Achsmaß) aus tragenden Innenwänden.
    fn project_with_space() -> (Project, ElementId) {
        let mut p = Project::template("Test");
        let storey = p.storeys()[0].0;
        let wall_type = p.wall_types().find(|(_, t)| t.name.contains("tragend")).unwrap().0;
        for (a, b) in
            [((0.0, 0.0), (5.0, 0.0)), ((5.0, 0.0), (5.0, 4.0)), ((5.0, 4.0), (0.0, 4.0)), ((0.0, 4.0), (0.0, 0.0))]
        {
            let id = p.next_id();
            p.elements.insert(
                id,
                Element::Wall(Wall {
                    storey,
                    wall_type,
                    start: Point2::new(a.0, a.1),
                    end: Point2::new(b.0, b.1),
                    alignment: WallAlignment::Center,
                    base_offset: 0.0,
                    top: Top::Storey { offset: 0.0 },
                    sagitta: 0.0,
                }),
            );
        }
        let id = p.next_id();
        p.elements.insert(
            id,
            Element::Space(Space {
                storey,
                position: Point2::new(2.5, 2.0),
                number: "0.01".into(),
                name: "Wohnen".into(),
                usage: SpaceUsage::Living,
                living_space: true,
            }),
        );
        (p, id)
    }

    #[test]
    fn net_gross_and_construction_add_up() {
        let (p, _) = project_with_space();
        let r = area_report(&p);
        let s = &r.storeys[0];
        // Wand 20,5 cm, Achse mittig: lichte Maße 4,795 × 3,795, Außenmaße 5,205 × 4,205.
        assert!((s.spaces[0].net.unwrap() - 4.795 * 3.795).abs() < 1e-6);
        assert!((s.gross - 5.205 * 4.205).abs() < 1e-6);
        assert!((s.construction - (s.gross - s.net)).abs() < 1e-9);
        assert_eq!(s.unassigned, 0.0);
        assert!((r.volume() - s.gross * 2.75).abs() < 1e-6);
        assert!(r.warnings.is_empty());
    }

    #[test]
    fn small_columns_stay_in_living_area_but_not_in_net_area() {
        let (mut p, _) = project_with_space();
        let storey = p.storeys()[0].0;
        for (x, size) in [(1.5, 0.3), (3.5, 0.4)] {
            let id = p.next_id();
            p.elements.insert(
                id,
                Element::Column(Column {
                    storey,
                    position: Point2::new(x, 2.0),
                    profile: ColumnProfile::Rectangle { width: size, depth: size },
                    rotation: 0.0,
                    material: "stahlbeton".into(),
                    base_offset: 0.0,
                    top: Top::Storey { offset: 0.0 },
                }),
            );
        }
        let space = &area_report(&p).storeys[0].spaces[0];
        let full = 4.795 * 3.795;
        // DIN 277: beide Stützen sind KGF. WoFlV § 3 Abs. 3 Nr. 1: nur die große (0,16 m²) wird abgezogen.
        assert!((space.net.unwrap() - (full - 0.09 - 0.16)).abs() < 1e-6);
        assert!((space.living.unwrap() - (full - 0.16)).abs() < 1e-6);
    }

    #[test]
    fn cellar_spaces_do_not_count_as_living_space() {
        let (mut p, id) = project_with_space();
        if let Some(Element::Space(r)) = p.elements.get_mut(&id) {
            r.living_space = false;
        }
        let r = area_report(&p);
        assert!(r.storeys[0].spaces[0].net.is_some());
        assert_eq!(r.living(), 0.0);
    }

    #[test]
    fn open_and_duplicate_spaces_are_reported() {
        let (mut p, _) = project_with_space();
        let storey = p.storeys()[0].0;
        for (x, name) in [(3.0, "Doppelt"), (9.0, "Draußen")] {
            let id = p.next_id();
            p.elements.insert(
                id,
                Element::Space(Space {
                    storey,
                    position: Point2::new(x, 2.0),
                    number: String::new(),
                    name: name.into(),
                    usage: SpaceUsage::Other,
                    living_space: true,
                }),
            );
        }
        let r = area_report(&p);
        assert_eq!(r.warnings.len(), 2, "{:?}", r.warnings);
        assert!(r.warnings.iter().any(|w| w.contains("Doppelt") && w.contains("0.01 Wohnen")));
        assert!(r.warnings.iter().any(|w| w.contains("Draußen") && w.contains("nicht von Wänden")));
    }
}
