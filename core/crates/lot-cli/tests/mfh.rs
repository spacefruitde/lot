//! Abnahme M5 (PLAN.md): Für das MFH-Testprojekt entstehen vollständige
//! Bauantragsunterlagen, die Abstandsflächen sind gegen eine Handrechnung geprüft.
//!
//! Die Handrechnung steht in `pruefpakete/2026-10-24-m5-bauantrag/handrechnung.md`.
//! Gelände z = -0,40 + y / 30, Attika OK +12,10 über ±0,00, MBO § 6: T = 0,4 H, mindestens 3 m.

use std::collections::BTreeSet;
use std::path::Path;

use lot_model::{Element, ElementId, Project, SectionKind, ViewContent};
use lot_rules::{SetbackReport, StateRules};

fn mfh() -> Project {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../testprojekte/mfh");
    lot_io::testproject::load(&dir).expect("Testprojekt MFH lesbar")
}

fn report(project: &Project) -> SetbackReport {
    let geometry = lot_geom::build(project).expect("Geometrie");
    let (rules, _) = StateRules::for_state(&project.info().state);
    let class = lot_rules::building_class(project, rules);
    lot_rules::setbacks(project, &geometry, &class)
}

fn close(a: f64, b: f64, tol: f64) -> bool {
    (a - b).abs() <= tol
}

#[test]
fn building_class_four() {
    let p = mfh().variant_view();
    let (rules, _) = StateRules::for_state("");
    let c = lot_rules::building_class(&p, rules);
    // Gelände im Mittel: Süd -0,40, Nord ±0,00, Ost und West im Mittel -0,20, gewichtet -0,20.
    assert!(close(c.mean_ground, -0.20, 0.005), "{}", c.mean_ground);
    // OKFF 3.OG +8,70 über -0,20: 8,90 m, also über 7 m und bis 13 m.
    assert!(close(c.height, 8.90, 0.005), "{}", c.height);
    assert_eq!(c.class, 4);
    assert_eq!(c.above_ground.len(), 4);
    let units: BTreeSet<&str> = c.units.iter().map(|(u, _)| u.as_str()).collect();
    assert_eq!(units.len(), 8, "{units:?}");
    assert!(c.units.iter().all(|(_, a)| *a < 400.0));
}

#[test]
fn setbacks_match_hand_calculation() {
    let p = mfh().variant_view();
    let r = report(&p);
    assert!(r.ok(), "Abstandsflächen außerhalb: {} m²", r.violations.area());
    assert!(!r.small_building);
    // Die Außenwände der unteren Geschosse liegen in der Flucht der Attika und fallen weg.
    let mut facing: Vec<&str> = r.areas.iter().map(|a| a.facing).collect();
    facing.sort();
    assert_eq!(facing, ["Nord", "Ost", "Süd", "West"]);
    let get = |name: &str| r.areas.iter().find(|a| a.facing == name).unwrap();

    // Süd: H = 12,10 + 0,40 = 12,50 m, T = 0,4 · 12,50 = 5,00 m; Abstand zur Grenze 9,00 m.
    let s = get("Süd");
    assert!(close(s.height().0, 12.50, 0.005) && close(s.height().1, 12.50, 0.005), "{:?}", s.height());
    assert!(close(s.depth().0, 5.00, 0.002) && close(s.depth().1, 5.00, 0.002), "{:?}", s.depth());
    assert!(close(s.available.unwrap(), 9.00, 0.005));
    assert!(close(s.length(), 24.0, 1e-6));

    // Nord: H = 12,10 − 0,00 = 12,10 m, T = 4,84 m; Vorgarten 4,00 m plus halbe Straße 5,00 m.
    let n = get("Nord");
    assert!(close(n.height().0, 12.10, 0.005) && close(n.height().1, 12.10, 0.005), "{:?}", n.height());
    assert!(close(n.depth().0, 4.84, 0.002) && close(n.depth().1, 4.84, 0.002), "{:?}", n.depth());
    assert!(close(n.available.unwrap(), 9.00, 0.005));

    // Ost und West: Gelände von -0,40 (Süd) bis ±0,00 (Nord), H von 12,50 bis 12,10 m,
    // T von 5,00 bis 4,84 m; in der Mitte (y = 6) H = 12,30 m, T = 4,92 m. Grenze 6,00 m.
    for name in ["Ost", "West"] {
        let a = get(name);
        assert!(close(a.height().0, 12.10, 0.005) && close(a.height().1, 12.50, 0.005), "{name} {:?}", a.height());
        assert!(close(a.depth().0, 4.84, 0.002) && close(a.depth().1, 5.00, 0.002), "{name} {:?}", a.depth());
        let mid = a.samples.iter().min_by(|x, y| (x.along - 6.0).abs().total_cmp(&(y.along - 6.0).abs())).unwrap();
        assert!(close(mid.height, 12.30, 0.01) && close(mid.depth, 4.92, 0.004), "{name} {mid:?}");
        assert!(close(a.available.unwrap(), 6.00, 0.005));
        // Fläche: Trapez 12,00 · (4,84 + 5,00) / 2 = 59,04 m².
        assert!(close(a.area.area(), 59.04, 0.02), "{name} {}", a.area.area());
    }
    // Süd und Nord als Rechtecke: 24 · 5,00 = 120,00 m² und 24 · 4,84 = 116,16 m².
    assert!(close(s.area.area(), 120.0, 0.02));
    assert!(close(n.area.area(), 116.16, 0.02));
}

#[test]
fn narrow_plot_shows_red_setback() {
    // Variante „Ostgrenze 4,50 m“: Die Abstandsfläche Ost ragt 0,34 bis 0,50 m über die
    // Grenze, rot ist das Trapez 12,00 · (0,34 + 0,50) / 2 = 5,04 m².
    let p = mfh().with_variant(ElementId(45), 1).variant_view();
    let r = report(&p);
    assert!(!r.ok());
    assert!(close(r.violations.area(), 5.04, 0.02), "{}", r.violations.area());
    let east = r.areas.iter().find(|a| a.facing == "Ost").unwrap();
    assert!(!east.ok());
    assert!(close(east.available.unwrap(), 4.50, 0.005));
    assert!(r.areas.iter().filter(|a| a.facing != "Ost").all(|a| a.ok()));
}

#[test]
fn submission_sheets_are_complete() {
    let p = mfh();
    let geometry = lot_geom::build(&p.variant_view()).unwrap();
    let standard = lot_plan::standard::OfficeStandard::default_standard();
    let mut storeys = BTreeSet::new();
    let (mut sections, mut elevations, mut site) = (0, 0, 0);
    for (_, sheet) in p.plan_sheets() {
        let problems = lot_plan::layout::layout_problems(&p, &geometry, sheet, standard);
        assert!(problems.is_empty(), "{problems:?}");
        for v in &sheet.views {
            match v.content {
                ViewContent::FloorPlan { storey } => {
                    storeys.insert(storey);
                }
                ViewContent::Section { view } => match p.elements.get(&view) {
                    Some(Element::SectionView(s)) if s.kind == SectionKind::Section => sections += 1,
                    Some(Element::SectionView(_)) => elevations += 1,
                    _ => panic!("Ansicht {view:?} fehlt"),
                },
                ViewContent::SitePlan => site += 1,
                ViewContent::Presentation { .. } => {}
            }
        }
    }
    // MBauVorlV § 7 und § 8: Lageplan, Grundrisse aller Geschosse, Schnitte, alle Ansichten.
    assert_eq!(site, 1);
    assert_eq!(storeys.len(), p.storeys().len());
    assert_eq!((sections, elevations), (2, 4));
    let info = p.info();
    assert!(!info.client.is_empty() && !info.architect.is_empty() && !info.parcel.is_empty());
    assert!(info.datum.is_some());
}

#[test]
fn check_report_passes() {
    let p = mfh().variant_view();
    let geometry = lot_geom::build(&p).unwrap();
    let report = lot_rules::check_project(&p, &geometry);
    for s in &report.sections {
        println!("## {} ({})", s.title, s.basis);
        for f in &s.findings {
            println!("  [{}] {} ({})", f.status.label(), f.text, f.source);
        }
    }
    assert_eq!(report.count(lot_rules::Status::Fail), 0);
}
