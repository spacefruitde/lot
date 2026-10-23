//! Abnahme M4 am Umbau-Testprojekt: Bestand, Abbruch und Neubau sind richtig
//! eingefärbt, Bestandsplan und Neuzustand enthalten das Richtige, und die
//! Unterlage liegt im Projekt.

use std::path::Path;

use lot_model::{Element, ElementId, Phase, PhaseFilter};
use lot_plan::floor_plan::{DEFAULT_CUT_HEIGHT, FloorPlanView, floor_plan};
use lot_plan::standard::OfficeStandard;
use lot_plan::{Color, Fill, PaperPoint, PlanObject, Viewport};

fn load() -> (lot_model::Project, std::path::PathBuf) {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../testprojekte/umbau");
    (lot_io::testproject::load(&dir.join("projekt.ron")).unwrap(), dir)
}

fn fills(p: &lot_model::Project) -> Vec<Color> {
    let view = FloorPlanView { storey: ElementId(1), cut_height: DEFAULT_CUT_HEIGHT, scale: 100.0 };
    let viewport = Viewport { scale: 100.0, origin: PaperPoint::new(0.0, 0.0) };
    floor_plan(p, view, OfficeStandard::default_standard(), viewport)
        .into_iter()
        .filter_map(|o| match o {
            PlanObject::Area(a) => match a.fill {
                Fill::Solid(c) => Some(c),
                _ => None,
            },
            _ => None,
        })
        .collect()
}

#[test]
fn conversion_plan_is_coloured_by_phase() {
    let (project, _) = load();
    let colors = OfficeStandard::default_standard().phases.clone();
    let f = fills(&project);
    for phase in [Phase::Existing, Phase::Demolished, Phase::New] {
        assert!(f.contains(&colors.of(phase)), "{phase:?} fehlt");
    }
    // Bestandsplan ohne Farben und ohne Anbau, aber mit der Wand, die fällt.
    let existing = project.phase_view(PhaseFilter::Existing);
    assert!(!fills(&existing).contains(&colors.of(Phase::Existing)));
    assert!(existing.elements.contains_key(&ElementId(25)));
    assert!(!existing.elements.contains_key(&ElementId(50)));
    // Neuzustand: Wand und Kamin weg, Anbau da, ein Raum Wohnen, Kochen.
    let planned = project.phase_view(PhaseFilter::Planned);
    assert!(!planned.elements.contains_key(&ElementId(25)) && !planned.elements.contains_key(&ElementId(29)));
    assert!(planned.elements.contains_key(&ElementId(50)));
    let report = lot_rules::area_report(&planned);
    let living = report.spaces().find(|s| s.name == "Wohnen, Kochen").unwrap();
    // Wohnzimmer 21,40 + Küche 15,73 + Wand 1,24 (0,27 · 4,6 ohne die Öffnung im Bestand).
    assert!((living.net.unwrap() - 38.37).abs() < 0.01, "{:?}", living.net);
}

#[test]
fn underlay_file_is_in_the_project_folder() {
    let (project, dir) = load();
    let files = lot_io::testproject::attachments(&project, &dir);
    let png = files.get("bestand-eg.png").expect("Unterlage fehlt");
    let r = lot_io::raster::decode_png(png).unwrap();
    let Some(Element::Underlay(u)) = project.elements.get(&ElementId(80)) else { panic!() };
    assert_eq!((r.width as f64, r.height as f64), (u.width, u.height));
    // A4 bei 150 dpi in 1:100: ein Bildpunkt sind 1,69 cm.
    assert!((u.scale - 0.0254 / 150.0 * 100.0).abs() < 1e-6);
}

#[test]
fn reference_model_appears_in_the_geometry_and_the_plan() {
    let (project, dir) = load();
    let files = lot_io::testproject::attachments(&project, &dir);
    let g = lot_cli::geometry(&project, &files).unwrap();
    let parts: Vec<_> = g.elements.iter().filter(|e| e.material == lot_io::ifc::REFERENCE_MATERIAL).collect();
    assert_eq!(parts.len(), 3);
    // Die Stütze wird in der Schnittebene des EG geschnitten.
    let mut objects = Vec::new();
    let viewport = Viewport { scale: 100.0, origin: PaperPoint::new(0.0, 0.0) };
    let refs: Vec<lot_geom::ElementGeometry> = parts.into_iter().cloned().collect();
    lot_plan::reference::draw_reference_cut(&mut objects, &refs, 1.0, viewport);
    assert!(objects.iter().any(|o| matches!(o, PlanObject::Area(_))));
}
