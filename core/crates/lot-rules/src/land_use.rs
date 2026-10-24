//! Maß der baulichen Nutzung: Grundflächenzahl (GRZ, § 19 BauNVO) und
//! Geschossflächenzahl (GFZ, § 20 BauNVO). Die Vollgeschosse bestimmt das
//! Landesrecht (Daten in `landesrecht/`).

use lot_geom::Area;
use lot_model::Project;

use crate::building::{BuildingClassResult, building_outline, storey_outline};
use crate::landesrecht::StateRules;
use crate::report::{Finding, ReportSection, Status, de};

/// Anteil der Grundfläche eines Geschosses mit mindestens `height` lichter Höhe.
fn high_share(project: &Project, storey: lot_model::ElementId, outline: &Area, height: f64) -> f64 {
    let total = outline.area();
    if total <= 0.0 {
        return 0.0;
    }
    let roofs: Vec<_> = project.roofs_on(storey).collect();
    if roofs.is_empty() {
        return 1.0;
    }
    let regions: Vec<Area> =
        roofs.iter().map(|(id, r)| lot_geom::roofs::clear_height_region(project, *id, r, height)).collect();
    // Außerhalb der Dächer gilt die volle Geschosshöhe.
    let ring = |r: &lot_model::Roof| r.outer_outline().iter().map(|p| [p.x, p.y]).collect::<Vec<_>>();
    let covered = Area::union_all(roofs.iter().map(|(_, r)| Area::from_ring(&ring(r))).collect::<Vec<_>>().iter());
    let high = Area::union_all(regions.iter()).union(&outline.difference(&covered));
    outline.intersection(&high).area() / total
}

pub fn land_use(project: &Project, rules: &StateRules, class: &BuildingClassResult) -> ReportSection {
    let r = &rules.land_use;
    let mut f = Vec::new();
    let Some(plot) = crate::setbacks::building_plot(project) else {
        f.push(Finding::new(
            Status::Note,
            "Kein Baugrundstück im Modell: GRZ und GFZ nicht berechnet",
            r.source.clone(),
        ));
        return ReportSection { title: "Maß der baulichen Nutzung".into(), basis: rules.name.clone(), findings: f };
    };
    let site = plot.area();
    let footprint = building_outline(project).area();
    f.push(Finding::new(
        Status::Ok,
        format!(
            "Grundstück {} m², Grundfläche des Gebäudes {} m², GRZ = {}",
            de(site, 2),
            de(footprint, 2),
            de(footprint / site, 2)
        ),
        "§ 19 BauNVO",
    ));
    let mut floor = 0.0;
    let mut full = Vec::new();
    for id in &class.above_ground {
        let outline = storey_outline(project, *id);
        if high_share(project, *id, &outline, r.full_storey_height) + 1e-9 >= r.full_storey_share {
            floor += outline.area();
            full.push(project.storey(*id).map_or("?".to_string(), |s| s.name.clone()));
        }
    }
    f.push(Finding::new(
        Status::Ok,
        format!(
            "Vollgeschosse {} ({}), Geschossfläche {} m², GFZ = {}",
            full.len(),
            full.join(", "),
            de(floor, 2),
            de(floor / site, 2)
        ),
        format!("§ 20 BauNVO, {}", r.source),
    ));
    f.push(Finding::new(
        Status::Note,
        "Zulässige Werte aus dem Bebauungsplan sind nicht im Modell; der Vergleich fehlt",
        "§§ 16 bis 21 BauNVO",
    ));
    ReportSection { title: "Maß der baulichen Nutzung".into(), basis: rules.name.clone(), findings: f }
}
