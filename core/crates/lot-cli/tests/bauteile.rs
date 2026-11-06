//! Decken, Dächer und Treppen (ADR 0051) an MFH und einem kleinen Haus:
//! Flachdach mit Gefälle, Treppe mit Loch und Umwehrung aus dem Werkzeug.
//! Mit `--ignored` schreibt `pruefbilder` die Bilder fürs Prüfpaket nach `LOT_OUT`.

use std::path::Path;

use lot_edit::{Command, Editor, Input, stairs};
use lot_model::{Document, Element, ElementId, Point2, Project, SectionKind, SectionView, WallSide};
use lot_plan::standard::OfficeStandard;
use lot_plan::{PaperPoint, PlanObject, Sheet, Viewport};

fn load(name: &str) -> Project {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../../testprojekte/{name}/projekt.ron"));
    lot_io::testproject::load(&path).unwrap()
}

fn storey(project: &Project, name: &str) -> ElementId {
    project.storeys().into_iter().find(|(_, s)| s.name == name).map(|(id, _)| id).expect("Geschoss")
}

fn input(x: f64, y: f64) -> Input {
    Input { point: Point2::new(x, y), tolerance: 0.1, shift: false, alt: false }
}

fn click(doc: &mut Document, ed: &mut Editor, x: f64, y: f64) {
    ed.pointer_moved(doc.project(), input(x, y));
    ed.pointer_down(doc, input(x, y), 1);
    ed.pointer_up(doc);
}

/// MFH mit Flachdach über dem 3.OG, Gefälle 2 %, ein Ablauf und ein Notüberlauf.
fn mfh_flat_roof() -> (Document, ElementId, ElementId) {
    let mut doc = Document::new(load("mfh"));
    doc.set_follow(lot_edit::follow::follow_ops);
    let mut ed = Editor::new(doc.project());
    let top = storey(doc.project(), "3.OG");
    ed.run(&mut doc, &Command::SetStorey(top.0));
    // Das Satteldach des MFH weg, das Flachdach kommt an seine Stelle.
    let roofs: Vec<ElementId> = doc.project().roofs_on(top).map(|(id, _)| id).collect();
    if !roofs.is_empty() {
        ed.run(&mut doc, &Command::SetSelection(roofs.iter().map(|r| r.0).collect()));
        ed.run(&mut doc, &Command::DeleteSelection);
    }
    let r = ed.run(&mut doc, &Command::FlatRoofFromWalls);
    assert!(r.error.is_none(), "{:?}", r.error);
    let dach = doc.project().storeys().last().unwrap().0;
    let (slab_id, slab) = doc.project().slabs_on(dach).next().map(|(id, s)| (id, s.clone())).unwrap();
    let fall = slab.fall.as_deref().unwrap();
    let (dx, dy) = fall.flow();
    // Ablauf am tiefen Rand, Notüberlauf daneben.
    let xs: Vec<f64> = slab.outline.iter().map(|p| p.x).collect();
    let ys: Vec<f64> = slab.outline.iter().map(|p| p.y).collect();
    let (cx, cy) = (xs.iter().sum::<f64>() / xs.len() as f64, ys.iter().sum::<f64>() / ys.len() as f64);
    let reach = |d: f64, lo: f64, hi: f64| {
        if d > 0.5 {
            hi - 0.5
        } else if d < -0.5 {
            lo + 0.5
        } else {
            (lo + hi) / 2.0
        }
    };
    let (x0, x1) =
        (xs.iter().cloned().fold(f64::INFINITY, f64::min), xs.iter().cloned().fold(f64::NEG_INFINITY, f64::max));
    let (y0, y1) =
        (ys.iter().cloned().fold(f64::INFINITY, f64::min), ys.iter().cloned().fold(f64::NEG_INFINITY, f64::max));
    let drain = Point2::new(
        if dx.abs() > 0.5 { reach(dx, x0, x1) } else { cx - 3.0 },
        if dy.abs() > 0.5 { reach(dy, y0, y1) } else { cy },
    );
    ed.run(&mut doc, &Command::SetStorey(dach.0));
    ed.run(&mut doc, &Command::DrainTool);
    click(&mut doc, &mut ed, drain.x, drain.y);
    ed.run(&mut doc, &Command::OverflowTool);
    click(&mut doc, &mut ed, drain.x + 4.0 * dy.abs(), drain.y + 4.0 * dx.abs());
    (doc, dach, slab_id)
}

/// Kleines Haus 10 × 6 m mit zwei Geschossen; Treppe mit dem Werkzeug.
fn stair_house() -> (Document, ElementId, ElementId) {
    stair_house_with(stairs::StairForm::HalfLanding, (1.2, 0.8), (1.2, 4.0))
}

fn stair_house_with(form: stairs::StairForm, a: (f64, f64), b: (f64, f64)) -> (Document, ElementId, ElementId) {
    let mut doc = Document::new(Project::template("Treppe"));
    doc.set_follow(lot_edit::follow::follow_ops);
    let mut ed = Editor::new(doc.project());
    ed.run(&mut doc, &Command::WallTool);
    for (x, y) in [(0.0, 0.0), (0.0, 6.0), (10.0, 6.0), (10.0, 0.0), (0.0, 0.0)] {
        click(&mut doc, &mut ed, x, y);
    }
    ed.run(&mut doc, &Command::SelectTool);
    let eg = ed.storey;
    ed.run(&mut doc, &Command::AddStorey);
    let og = ed.storey;
    ed.run(&mut doc, &Command::SlabFromWalls);
    ed.run(&mut doc, &Command::WallTool);
    for (x, y) in [(0.0, 0.0), (0.0, 6.0), (10.0, 6.0), (10.0, 0.0), (0.0, 0.0)] {
        click(&mut doc, &mut ed, x, y);
    }
    ed.run(&mut doc, &Command::SetStorey(eg.0));
    ed.run(&mut doc, &Command::StairTool(form));
    click(&mut doc, &mut ed, a.0, a.1);
    click(&mut doc, &mut ed, b.0, b.1);
    (doc, eg, og)
}

#[test]
fn mfh_flat_roof_has_fall_drain_overflow_and_no_findings() {
    let (doc, _, slab) = mfh_flat_roof();
    let Some(Element::Slab(s)) = doc.project().elements.get(&slab) else { panic!() };
    let fall = s.fall.as_deref().unwrap();
    assert_eq!((fall.drains.len(), fall.overflows.len()), (1, 1));
    assert!(lot_rules::falls::check_falls(doc.project()).iter().all(|f| f.status == lot_rules::Status::Ok));
    // Im 3D steigt die Dämmung zur hohen Seite um 2 % der Tiefe.
    let g = lot_geom::build(doc.project()).unwrap();
    let layer = g.elements.iter().find(|e| e.id == slab && e.layer == fall.layer).unwrap();
    let (lo, hi) = layer.mesh.bounds().unwrap();
    assert!(hi[2] - lo[2] > 0.16 + 0.05, "Dämmung {}..{}", lo[2], hi[2]);
}

#[test]
fn stair_tool_in_a_house_cuts_hole_and_guards_it() {
    let (doc, eg, og) = stair_house();
    let stair =
        doc.project().elements.iter().find_map(|(id, e)| matches!(e, Element::Stair(_)).then_some(*id)).unwrap();
    let check = lot_rules::stairs::check_stair(doc.project(), stair, doc.project().stair(stair).unwrap()).unwrap();
    assert!(check.problems.is_empty(), "{:?}", check.problems);
    assert!(doc.project().railings_on(eg).count() >= 1);
    assert!(doc.project().railings_on(og).any(|(_, r)| r.around.is_some()));
}

fn centered(title: &str, mut objects: Vec<PlanObject>) -> Sheet {
    if let Some((lo, hi)) = lot_plan::bounds(&objects) {
        let (dx, dy) = (210.0 - (lo.x + hi.x) / 2.0, 148.5 - (lo.y + hi.y) / 2.0);
        for o in &mut objects {
            o.translate(dx, dy);
        }
    }
    Sheet { title: title.into(), width_mm: 420.0, height_mm: 297.0, objects }
}

fn plan(p: &Project, storey: ElementId, scale: f64) -> Vec<PlanObject> {
    let viewport = Viewport { scale, origin: PaperPoint::new(0.0, 0.0) };
    let view = lot_plan::floor_plan::FloorPlanView { storey, cut_height: 1.0, scale };
    lot_plan::floor_plan::floor_plan(p, view, OfficeStandard::default_standard(), viewport)
}

#[test]
#[ignore]
fn pruefbilder() {
    let out = std::env::var("LOT_OUT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../out/bauteile"));
    std::fs::create_dir_all(&out).unwrap();
    let write = |name: &str, sheet: &Sheet| {
        std::fs::write(out.join(format!("{name}.png")), lot_io::png::to_png(sheet, 150.0).unwrap()).unwrap();
    };
    let (doc, dach, slab) = mfh_flat_roof();
    let p = doc.project();
    write("mfh-flachdach-aufsicht", &centered("MFH Flachdach, Aufsicht 1:100", plan(p, dach, 100.0)));
    // Schnitt quer zur Fließrichtung des Gefälles durch die Mitte.
    let Some(Element::Slab(s)) = p.elements.get(&slab) else { panic!() };
    let (dx, dy) = s.fall.as_deref().unwrap().flow();
    let n = s.outline.len() as f64;
    let c = Point2::new(s.outline.iter().map(|q| q.x).sum::<f64>() / n, s.outline.iter().map(|q| q.y).sum::<f64>() / n);
    let view = SectionView {
        name: "Schnitt Gefälle".into(),
        mark: "G".into(),
        start: Point2::new(c.x - dx * 20.0 + dy * 1.0, c.y - dy * 20.0 - dx * 1.0),
        end: Point2::new(c.x + dx * 20.0 + dy * 1.0, c.y + dy * 20.0 - dx * 1.0),
        look: WallSide::Left,
        depth: 0.5,
        kind: SectionKind::Section,
        home: None,
    };
    let g = lot_geom::build(p).unwrap();
    let viewport = Viewport { scale: 50.0, origin: PaperPoint::new(0.0, 0.0) };
    let mut objects = lot_plan::section::section(p, &g, &view, OfficeStandard::default_standard(), viewport);
    // Nur der obere Teil: ab 1 m unter der Dachdecke.
    let top = p.storey(dach).unwrap().elevation;
    objects.retain(|o| match o {
        PlanObject::Line(l) => l.points.iter().all(|q| q.y > (top - 1.5) * 20.0),
        PlanObject::Area(a) => a.rings.iter().flatten().all(|q| q.y > (top - 1.5) * 20.0),
        PlanObject::Text(t) => t.position.y > (top - 1.5) * 20.0,
    });
    write("mfh-flachdach-schnitt", &centered("MFH Flachdach, Schnitt durch das Gefälle 1:50", objects));
    let (doc, eg, og) = stair_house();
    write("treppe-eg", &centered("Treppe aus dem Werkzeug, EG 1:50", plan(doc.project(), eg, 50.0)));
    write("treppe-og", &centered("Treppenloch mit Umwehrung, OG 1:50", plan(doc.project(), og, 50.0)));
    for (form, name, a, b) in [
        (stairs::StairForm::QuarterWinder, "treppe-viertelgewendelt", (6.0, 0.8), (6.0, 4.0)),
        (stairs::StairForm::HalfWinder, "treppe-halbgewendelt", (6.0, 0.8), (6.0, 4.0)),
        (stairs::StairForm::Spiral, "spindeltreppe", (5.0, 3.0), (6.4, 3.0)),
    ] {
        let (doc, eg, _) = stair_house_with(form, a, b);
        write(name, &centered(&format!("{}, EG 1:50", form.title()), plan(doc.project(), eg, 50.0)));
    }
}
