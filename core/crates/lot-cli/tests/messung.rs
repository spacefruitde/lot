//! Zeitmessung am EFH gegen den Zielwert aus PLAN.md, Abschnitt 4: Nach einer
//! Wandänderung sind Modell und sichtbare Ansichten in unter 50 ms aktualisiert.
//! Gemessen werden Grundriss mit Raumstempeln und Maßketten, 3D und Flächen.
//!
//! Läuft nur von Hand, weil Zeiten auf den CI-Rechnern schwanken:
//! `cargo test -p lot-cli --test messung -- --ignored --nocapture`

use std::path::Path;
use std::time::Instant;

use lot_model::{Document, Param};
use lot_plan::floor_plan::{DEFAULT_CUT_HEIGHT, FloorPlanView, floor_plan};
use lot_plan::standard::OfficeStandard;
use lot_plan::{PaperPoint, Viewport};

#[test]
#[ignore]
fn efh_wall_change_under_50_ms() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../testprojekte/efh/projekt.ron");
    let mut doc = Document::new(lot_io::testproject::load(&path).unwrap());
    let storey = doc.project().storeys()[1].0;
    let (wall, length) = doc.project().walls_on(storey).map(|(id, w)| (id, w.length())).next().unwrap();
    let standard = OfficeStandard::default_standard();
    let view = FloorPlanView { storey, cut_height: DEFAULT_CUT_HEIGHT, scale: 100.0 };
    let viewport = Viewport { scale: 100.0, origin: PaperPoint::new(0.0, 0.0) };
    // Einmal vorweg, damit Speicher und Caches warm sind.
    lot_geom::build(doc.project()).unwrap();
    let mut worst: f64 = 0.0;
    for i in 0..10 {
        let start = Instant::now();
        doc.set_param(wall, Param::WallLength, length + 0.01 * (i % 2) as f64).unwrap();
        let plan = floor_plan(doc.project(), view, standard, viewport);
        let geometry = lot_geom::build(doc.project()).unwrap();
        let areas = lot_rules::area_report(doc.project());
        let ms = start.elapsed().as_secs_f64() * 1000.0;
        println!(
            "{ms:.1} ms: {} Planobjekte, {} Dreiecke, NRF {:.2} m²",
            plan.len(),
            geometry.triangle_count(),
            areas.net()
        );
        worst = worst.max(ms);
    }
    assert!(worst < 50.0, "langsamste Änderung {worst:.1} ms");
}
