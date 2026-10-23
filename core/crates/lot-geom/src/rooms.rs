//! Räume: die von Wänden und Stützen umschlossenen Flächen eines Geschosses.
//!
//! Gemessen wird in Fußbodenhöhe an den fertigen Oberflächen, also einschließlich
//! Putz und Bekleidung, und ohne Öffnungen: Türen verbinden keine Räume, Tür- und
//! Fensternischen gehören nicht zum Raum. So verlangen es DIN 277:2021-08 für die
//! Netto-Raumfläche und die Wohnflächenverordnung vom 25.11.2003, § 3 Abs. 1 und
//! Abs. 3 (Vorderkante der Bekleidung, Türnischen bleiben außer Betracht).

use lot_model::{ElementId, Project};

use crate::joins::solid_regions;
use crate::poly::Area;

/// Höhe über OKFF, in der die Wände für die Raumgrenzen geschnitten werden, in Metern.
const FLOOR_CUT: f64 = 0.05;

/// Alle umschlossenen Bereiche eines Geschosses, einmal gerechnet.
pub struct RoomFinder {
    /// Wände und Stützen in Fußbodenhöhe, vereinigt.
    pub footprint: Area,
    /// Jeder umschlossene Bereich, ohne Inseln (freistehende Wände, Stützen).
    pub enclosed: Vec<Area>,
}

impl RoomFinder {
    pub fn new(project: &Project, storey: ElementId) -> Self {
        let z = project.storey(storey).map_or(0.0, |s| s.elevation) + FLOOR_CUT;
        let regions = solid_regions(project, storey, z);
        let footprint = Area::union_all(regions.iter().map(|r| &r.area));
        // Löcher der vereinigten Wandfläche sind die Räume. Clipper liefert sie im
        // Uhrzeigersinn, also mit negativer Fläche.
        let enclosed = footprint
            .rings()
            .into_iter()
            .filter(|r| crate::signed_area(r) < 0.0)
            .map(|r| Area::from_ring(&r).difference(&footprint))
            .filter(|a| !a.is_empty())
            .collect();
        Self { footprint, enclosed }
    }

    /// Der umschlossene Bereich um den Punkt, falls es einen gibt.
    pub fn area_at(&self, p: [f64; 2]) -> Option<&Area> {
        // Bei verschachtelten Bereichen (Hof in einem Raum) gewinnt der kleinste.
        self.enclosed.iter().filter(|a| a.contains(p)).min_by(|a, b| a.area().total_cmp(&b.area()))
    }

    /// Bereiche, in denen noch kein Raum steht.
    pub fn unclaimed(&self, project: &Project, storey: ElementId) -> Vec<&Area> {
        let points: Vec<[f64; 2]> = project.rooms_on(storey).map(|(_, r)| [r.position.x, r.position.y]).collect();
        self.enclosed.iter().filter(|a| !points.iter().any(|p| a.contains(*p))).collect()
    }
}

/// Ein Punkt sicher im Inneren der Fläche, für Raumstempel neu erkannter Räume:
/// der Mittelpunkt des Hüllquaders, wenn er innen liegt, sonst der innere Punkt
/// auf einer waagerechten Linie durch die Mitte, der am weitesten vom Rand entfernt ist.
pub fn interior_point(area: &Area) -> Option<[f64; 2]> {
    let (lo, hi) = area.bounds()?;
    let center = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0];
    if area.contains(center) {
        return Some(center);
    }
    // Schnittpunkte der Mittellinie mit den Rändern, die längste innere Strecke gewinnt.
    let y = center[1];
    let mut xs: Vec<f64> = Vec::new();
    for ring in area.rings() {
        let n = ring.len();
        for i in 0..n {
            let (a, b) = (ring[i], ring[(i + 1) % n]);
            if (a[1] > y) != (b[1] > y) {
                xs.push(a[0] + (y - a[1]) * (b[0] - a[0]) / (b[1] - a[1]));
            }
        }
    }
    xs.sort_by(f64::total_cmp);
    xs.as_chunks::<2>()
        .0
        .iter()
        .max_by(|a, b| (a[1] - a[0]).total_cmp(&(b[1] - b[0])))
        .map(|s| [(s[0] + s[1]) / 2.0, y])
}
