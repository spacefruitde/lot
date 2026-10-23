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
pub struct SpaceFinder {
    /// Wände und Stützen in Fußbodenhöhe, vereinigt.
    pub footprint: Area,
    /// Jeder umschlossene Bereich, ohne Inseln (freistehende Wände, Stützen).
    pub enclosed: Vec<Area>,
    /// Derselbe Bereich mit Inseln, je Eintrag in `enclosed`.
    outlines: Vec<Area>,
    /// Stützen, die nach WoFlV nicht abgezogen werden: höchstens 1,50 m hoch
    /// oder höchstens 0,1 m² Grundfläche (§ 3 Abs. 3 Nr. 1).
    small_columns: Area,
}

impl SpaceFinder {
    pub fn new(project: &Project, storey: ElementId) -> Self {
        let z = project.storey(storey).map_or(0.0, |s| s.elevation) + FLOOR_CUT;
        let regions = solid_regions(project, storey, z);
        let footprint = Area::union_all(regions.iter().map(|r| &r.area));
        // Löcher der vereinigten Wandfläche sind die Räume. Clipper liefert sie im
        // Uhrzeigersinn, also mit negativer Fläche.
        let (enclosed, outlines) = footprint
            .rings()
            .into_iter()
            .filter(|r| crate::signed_area(r) < 0.0)
            .map(|r| {
                let outline = Area::from_ring(&r);
                (outline.difference(&footprint), outline)
            })
            .filter(|(a, _)| !a.is_empty())
            .unzip();
        let small: Vec<Area> = project
            .columns_on(storey)
            .filter(|(_, c)| {
                let height = project.vertical_extent(c.storey, c.base_offset, c.top).map_or(0.0, |(lo, hi)| hi - lo);
                let area = crate::column_area(c).area();
                height <= 1.5 || area <= 0.1 + 1e-9
            })
            .map(|(_, c)| crate::column_area(c))
            .collect();
        Self { footprint, enclosed, outlines, small_columns: Area::union_all(small.iter()) }
    }

    /// Fläche des Bereichs um den Punkt nach WoFlV: wie [`SpaceFinder::area_at`],
    /// aber kleine Stützen bleiben in der Fläche (§ 3 Abs. 3 Nr. 1).
    pub fn living_area_at(&self, p: [f64; 2]) -> Option<f64> {
        self.living_region_at(p).map(|a| a.area())
    }

    /// Die Grundfläche nach WoFlV als Fläche: der Bereich mit kleinen Stützen.
    pub fn living_region_at(&self, p: [f64; 2]) -> Option<Area> {
        let i = self.index_at(p)?;
        Some(self.enclosed[i].union(&self.small_columns.intersection(&self.outlines[i])))
    }

    fn index_at(&self, p: [f64; 2]) -> Option<usize> {
        (0..self.enclosed.len())
            .filter(|&i| self.enclosed[i].contains(p) || self.small_columns.intersection(&self.outlines[i]).contains(p))
            .min_by(|&a, &b| self.enclosed[a].area().total_cmp(&self.enclosed[b].area()))
    }

    /// Der umschlossene Bereich um den Punkt, falls es einen gibt. Bei
    /// verschachtelten Bereichen (Hof in einem Raum) gewinnt der kleinste.
    pub fn area_at(&self, p: [f64; 2]) -> Option<&Area> {
        self.enclosed.iter().filter(|a| a.contains(p)).min_by(|a, b| a.area().total_cmp(&b.area()))
    }

    /// Index des Bereichs um den Punkt in `enclosed`, für Prüfungen auf doppelte Räume.
    pub fn position_of(&self, p: [f64; 2]) -> Option<usize> {
        self.index_at(p)
    }

    /// Fläche innerhalb der Außenkanten aller Wände: Räume, Wände und Stützen.
    pub fn gross_area(&self) -> f64 {
        let outer: Vec<Area> = self
            .footprint
            .rings()
            .into_iter()
            .filter(|r| crate::signed_area(r) > 0.0)
            .map(|r| Area::from_ring(&r))
            .collect();
        Area::union_all(outer.iter()).area()
    }

    /// Bereiche, in denen noch kein Raum steht.
    pub fn unclaimed(&self, project: &Project, storey: ElementId) -> Vec<&Area> {
        let points: Vec<[f64; 2]> = project.spaces_on(storey).map(|(_, r)| [r.position.x, r.position.y]).collect();
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
