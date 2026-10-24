//! Das Gebäude als Ganzes: Umriss je Geschoss, Gelände ringsum, Oberkante über
//! jedem Punkt und die Gebäudeklasse. Grundlage für Abstandsflächen,
//! Rettungswege und das Maß der baulichen Nutzung.

use lot_geom::spaces::SpaceFinder;
use lot_geom::{Area, ModelGeometry};
use lot_model::{ElementId, ElementKind, Point2, Project, SpaceUsage};

use crate::landesrecht::StateRules;

/// Äußerer Umriss eines Geschosses: die Wände in Fußbodenhöhe samt allem,
/// was sie umschließen. Leer, wenn das Geschoss keine Wände hat.
pub fn storey_outline(project: &Project, storey: ElementId) -> Area {
    outline_of(&SpaceFinder::new(project, storey).footprint)
}

/// Fläche ohne Löcher.
pub fn outline_of(area: &Area) -> Area {
    let outer: Vec<Area> =
        area.rings().into_iter().filter(|r| lot_geom::signed_area(r) > 0.0).map(|r| Area::from_ring(&r)).collect();
    Area::union_all(outer.iter())
}

/// Umriss des ganzen Gebäudes: alle Geschosse übereinander gelegt.
pub fn building_outline(project: &Project) -> Area {
    let all: Vec<Area> = project.storeys().into_iter().map(|(id, _)| storey_outline(project, id)).collect();
    Area::union_all(all.iter())
}

/// Geländehöhe an einem Punkt über ±0,00; ohne Gelände ±0,00.
pub fn ground_at(project: &Project, p: Point2) -> f64 {
    project.terrain().and_then(|t| t.height_at(p)).unwrap_or(0.0)
}

/// Mittlere Geländehöhe entlang eines Umrisses, alle 0,25 m gemessen.
pub fn mean_ground(project: &Project, outline: &Area) -> f64 {
    let Some(terrain) = project.terrain() else { return 0.0 };
    let triangles = terrain.triangles();
    let (mut sum, mut length) = (0.0, 0.0);
    for ring in outline.rings() {
        for i in 0..ring.len() {
            let (a, b) = (ring[i], ring[(i + 1) % ring.len()]);
            let len = (b[0] - a[0]).hypot(b[1] - a[1]);
            let n = (len / 0.25).ceil().max(1.0) as usize;
            for k in 0..n {
                let t = (k as f64 + 0.5) / n as f64;
                let p = Point2::new(a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t);
                sum += terrain.height_with(&triangles, p).unwrap_or(0.0) * len / n as f64;
            }
            length += len;
        }
    }
    if length > 0.0 { sum / length } else { 0.0 }
}

/// Oberkante des Gebäudes über einem Punkt: der höchste Schnitt einer
/// senkrechten Linie mit Wänden, Decken, Dächern, Stützen und Gauben.
/// Gelände, Referenzmodelle, Treppen und Geländer zählen nicht.
pub struct TopProbe<'a> {
    parts: Vec<Part<'a>>,
}

/// Ein Netz mit seinem Hüllrechteck im Grundriss.
type Part<'a> = (([f64; 2], [f64; 2]), &'a lot_geom::TriMesh);

impl<'a> TopProbe<'a> {
    pub fn new(project: &Project, geometry: &'a ModelGeometry) -> Self {
        let parts = geometry
            .elements
            .iter()
            .filter(|e| {
                matches!(
                    project.elements.get(&e.id).map(|x| x.kind()),
                    Some(
                        ElementKind::Wall
                            | ElementKind::Slab
                            | ElementKind::Roof
                            | ElementKind::Column
                            | ElementKind::Dormer
                            | ElementKind::RoofWindow
                    )
                )
            })
            .filter_map(|e| e.mesh.bounds().map(|(lo, hi)| (([lo[0], lo[1]], [hi[0], hi[1]]), &e.mesh)))
            .collect();
        Self { parts }
    }

    pub fn top(&self, x: f64, y: f64) -> Option<f64> {
        let mut best: Option<f64> = None;
        for ((lo, hi), mesh) in &self.parts {
            if x < lo[0] - 1e-9 || x > hi[0] + 1e-9 || y < lo[1] - 1e-9 || y > hi[1] + 1e-9 {
                continue;
            }
            for t in &mesh.triangles {
                let [a, b, c] = t.map(|i| mesh.positions[i as usize]);
                let det = (b[1] - c[1]) * (a[0] - c[0]) + (c[0] - b[0]) * (a[1] - c[1]);
                if det.abs() < 1e-14 {
                    continue;
                }
                let l1 = ((b[1] - c[1]) * (x - c[0]) + (c[0] - b[0]) * (y - c[1])) / det;
                let l2 = ((c[1] - a[1]) * (x - c[0]) + (a[0] - c[0]) * (y - c[1])) / det;
                let l3 = 1.0 - l1 - l2;
                if l1 >= -1e-9 && l2 >= -1e-9 && l3 >= -1e-9 {
                    let z = l1 * a[2] + l2 * b[2] + l3 * c[2];
                    best = Some(best.map_or(z, |m| m.max(z)));
                }
            }
        }
        best
    }
}

/// Gebäudeklasse und was sie bestimmt.
#[derive(Debug, Clone, PartialEq)]
pub struct BuildingClassResult {
    /// 1 bis 5.
    pub class: u8,
    /// Höhe nach § 2 Abs. 3 Satz 2 MBO: Fußboden des höchsten Geschosses mit
    /// Aufenthaltsraum über dem Gelände im Mittel, in m.
    pub height: f64,
    /// Gelände im Mittel um das Gebäude, über ±0,00.
    pub mean_ground: f64,
    /// Das Geschoss, nach dem die Höhe bemessen ist.
    pub top_storey: Option<ElementId>,
    /// Nutzungseinheiten mit ihrer Grundfläche in m² (ohne Kellergeschosse).
    pub units: Vec<(String, f64)>,
    /// Oberirdische Geschosse, von unten.
    pub above_ground: Vec<ElementId>,
    pub source: String,
    pub notes: Vec<String>,
}

impl BuildingClassResult {
    pub fn roman(&self) -> &'static str {
        ["", "1", "2", "3", "4", "5"][self.class as usize]
    }
}

/// Ob in einem Raum dieser Nutzung ein Aufenthaltsraum möglich ist.
pub fn is_habitable(usage: SpaceUsage) -> bool {
    matches!(
        usage,
        SpaceUsage::Living | SpaceUsage::Office | SpaceUsage::Production | SpaceUsage::Education | SpaceUsage::Care
    )
}

/// Gebäudeklasse nach Landesrecht (MBO § 2 Abs. 3). Das Gebäude gilt als
/// freistehend; Gebäudeklasse 2 (nicht freistehend) ergibt sich so nicht.
pub fn building_class(project: &Project, rules: &StateRules) -> BuildingClassResult {
    let r = &rules.building_classes;
    let outline = building_outline(project);
    let ground = mean_ground(project, &outline);
    let mut notes = Vec::new();
    if project.terrain().is_none() {
        notes.push("Kein Gelände im Modell: Höhen ab ±0,00 gemessen".to_string());
    }
    let storeys = project.storeys();
    let above_ground: Vec<ElementId> = storeys
        .iter()
        .filter(|(_, s)| s.elevation + s.height - ground > r.above_ground + 1e-9)
        .map(|(id, _)| *id)
        .collect();
    let habitable = |id: ElementId| {
        project.spaces_on(id).any(|(_, s)| is_habitable(s.usage) && s.phase != lot_model::Phase::Demolished)
    };
    let top_storey = storeys
        .iter()
        .filter(|(id, _)| habitable(*id))
        .max_by(|a, b| a.1.elevation.total_cmp(&b.1.elevation))
        .or_else(|| {
            storeys
                .iter()
                .filter(|(id, _)| above_ground.contains(id))
                .max_by(|a, b| a.1.elevation.total_cmp(&b.1.elevation))
        })
        .map(|(id, _)| *id);
    let height = top_storey.and_then(|id| project.storey(id)).map_or(0.0, |s| s.elevation - ground);

    // Grundfläche je Nutzungseinheit: Netto-Raumfläche ihrer Räume, hochgerechnet mit
    // dem Verhältnis BGF zu NRF des Geschosses (die MBO meint die Brutto-Grundfläche).
    let areas = crate::area_report(project);
    let mut units: Vec<(String, f64)> = Vec::new();
    for storey in areas.storeys.iter().filter(|s| above_ground.contains(&s.storey)) {
        let factor = if storey.net > 0.0 { storey.gross / storey.net } else { 1.0 };
        for space in &storey.spaces {
            let Some(net) = space.net else { continue };
            let unit = space.unit.clone();
            if unit.is_empty() {
                continue;
            }
            match units.iter_mut().find(|(u, _)| *u == unit) {
                Some((_, a)) => *a += net * factor,
                None => units.push((unit, net * factor)),
            }
        }
    }
    if units.is_empty() {
        // Ohne Angabe: eine Nutzungseinheit aus allen oberirdischen Geschossen.
        let gross = areas.storeys.iter().filter(|s| above_ground.contains(&s.storey)).map(|s| s.gross).sum();
        units.push((String::new(), gross));
        notes.push("Keine Nutzungseinheiten angegeben: das ganze Gebäude als eine gerechnet".to_string());
    }
    let total: f64 = units.iter().map(|(_, a)| a).sum();
    let class = if height <= r.low + 1e-9 {
        if units.len() <= r.small_units && total <= r.small_area + 1e-9 { 1 } else { 3 }
    } else if height <= r.medium + 1e-9 && units.iter().all(|(_, a)| *a <= r.unit_area + 1e-9) {
        4
    } else {
        5
    };
    if class == 1 {
        notes.push("Freistehend angenommen; angebaut wäre es Gebäudeklasse 2".to_string());
    }
    BuildingClassResult {
        class,
        height,
        mean_ground: ground,
        top_storey,
        units,
        above_ground,
        source: r.source.clone(),
        notes,
    }
}

/// Himmelsrichtung einer Richtung im Grundriss, mit Norden aus den Projektangaben.
pub fn compass(project: &Project, dir: [f64; 2]) -> &'static str {
    let north = project.info().north.to_radians();
    // Winkel im Uhrzeigersinn von Norden.
    let (nx, ny) = (-north.sin(), north.cos());
    let (ex, ey) = (ny, -nx);
    let angle = (dir[0] * ex + dir[1] * ey).atan2(dir[0] * nx + dir[1] * ny).to_degrees().rem_euclid(360.0);
    const NAMES: [&str; 8] = ["Nord", "Nordost", "Ost", "Südost", "Süd", "Südwest", "West", "Nordwest"];
    NAMES[(((angle + 22.5) / 45.0).floor() as usize) % 8]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compass_follows_north() {
        let mut p = Project::sample();
        assert_eq!(compass(&p, [0.0, -1.0]), "Süd");
        assert_eq!(compass(&p, [1.0, 0.0]), "Ost");
        assert_eq!(compass(&p, [-1.0, 1.0]), "Nordwest");
        // Norden um 90° gegen den Uhrzeigersinn gedreht: -x zeigt nach Norden, +y nach Osten.
        let info = lot_model::ProjectInfo { north: 90.0, ..Default::default() };
        p.elements.insert(ElementId(9999), lot_model::Element::ProjectInfo(info));
        assert_eq!(compass(&p, [-1.0, 0.0]), "Nord");
        assert_eq!(compass(&p, [0.0, 1.0]), "Ost");
    }

    #[test]
    fn sample_house_is_class_one() {
        let p = Project::sample();
        let (rules, _) = StateRules::for_state("");
        let c = building_class(&p, rules);
        assert_eq!(c.class, 1, "{c:?}");
        assert!(c.height < 7.0);
    }
}
