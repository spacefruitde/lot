//! Abnahme M3 am EFH: Grundrisse aller Geschosse, zwei Schnitte und vier
//! Ansichten liegen auf Planblättern mit Plankopf, ohne Überschneidungen.

use std::collections::BTreeSet;
use std::path::Path;

use lot_model::{Element, SectionKind, ViewContent};

#[test]
fn efh_plan_sheets_are_complete() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../testprojekte/efh/projekt.ron");
    let project = lot_io::testproject::load(&path).unwrap();
    let geometry = lot_geom::build(&project).unwrap();
    let standard = lot_plan::standard::OfficeStandard::default_standard();

    let mut storeys = BTreeSet::new();
    let (mut sections, mut elevations) = (0, 0);
    for (_, sheet) in project.plan_sheets() {
        let problems = lot_plan::layout::layout_problems(&project, &geometry, sheet, standard);
        assert!(problems.is_empty(), "{problems:?}");
        for v in &sheet.views {
            match v.content {
                ViewContent::FloorPlan { storey } => {
                    storeys.insert(storey);
                }
                ViewContent::Section { view } => match project.elements.get(&view) {
                    Some(Element::SectionView(s)) if s.kind == SectionKind::Section => sections += 1,
                    Some(Element::SectionView(_)) => elevations += 1,
                    _ => panic!("Ansicht {view:?} fehlt"),
                },
            }
        }
    }
    assert_eq!(storeys.len(), project.storeys().len());
    assert_eq!((sections, elevations), (2, 4));
}
