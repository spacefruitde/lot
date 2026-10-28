//! Navigation (ADR 0049) am EFH: Seitenleiste und Suche, Bauteile im Schnitt
//! anklicken, das Geschoss darunter blass. Mit `--ignored` schreibt
//! `pruefbilder` die Bilder fürs Prüfpaket nach `LOT_OUT`.

use std::path::Path;

use lot_edit::navigation::{self, Target, ViewRef};
use lot_model::{ElementId, Project, SectionKind, SectionView};
use lot_plan::standard::OfficeStandard;
use lot_plan::{Color, Line, LineStyle, PaperPoint, Pen, PlanObject, Sheet, Viewport};

fn efh() -> Project {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../testprojekte/efh/projekt.ron");
    lot_io::testproject::load(&path).unwrap()
}

fn storey(project: &Project, name: &str) -> ElementId {
    project.storeys().into_iter().find(|(_, s)| s.name == name).map(|(id, _)| id).expect("Geschoss")
}

fn section<'a>(project: &'a Project, name: &str) -> (ElementId, &'a SectionView) {
    project.section_views().find(|(_, v)| v.name == name).expect("Schnitt")
}

#[test]
fn efh_tree_search_and_storeys() {
    let p = efh();
    let tree = navigation::tree(&p);
    let storeys: Vec<&str> = tree[0].entries.iter().map(|e| e.title.as_str()).collect();
    assert_eq!(storeys, ["DG", "EG", "KG"]);
    // Jede Ansicht liegt im EFH auf genau einem Blatt: keine Heimat-Markierung.
    let sheets = tree.iter().find(|g| g.title == "Planblätter").unwrap();
    assert!(sheets.entries.iter().flat_map(|e| &e.children).all(|c| !c.home));
    let (a, _) = section(&p, "Schnitt A-A");
    assert!(navigation::home(&p, ViewRef::Section(a.0)).is_some());
    let hits = navigation::search(&p, "schnitt a");
    assert_eq!(hits[0].target, Target::View(ViewRef::Section(a.0)));
    let eg = storey(&p, "EG");
    assert_eq!(navigation::storey_step(&p, eg, true), Some(storey(&p, "DG")));
    assert_eq!(navigation::storey_step(&p, eg, false), Some(storey(&p, "KG")));
}

#[test]
fn efh_section_picks_what_is_seen_and_shows_the_selection() {
    let p = efh();
    let g = lot_geom::build(&p.variant_view()).unwrap();
    let (_, v) = section(&p, "Schnitt A-A");
    assert_eq!(v.kind, SectionKind::Section);
    // Die Wände des EG sind geschnitten oder zu sehen; ihre Umrisse gibt es im Schnitt.
    let walls: Vec<ElementId> = p.walls_on(storey(&p, "EG")).map(|(id, _)| id).collect();
    let rings = lot_plan::section::section_outlines(&g, v, &walls);
    assert!(!rings.is_empty());
    // Ein Klick in die Mitte eines Umrisses trifft eine dieser Wände.
    let ring = &rings[0];
    let n = ring.len() as f64;
    let c = ring.iter().fold([0.0, 0.0], |a, q| [a[0] + q[0] / n, a[1] + q[1] / n]);
    let picked = lot_plan::section::section_pick(&g, v, c);
    assert!(picked.is_some());
}

#[test]
fn efh_faded_storey_lies_under_the_plan() {
    let p = efh();
    let viewport = Viewport { scale: 100.0, origin: PaperPoint::new(0.0, 0.0) };
    let standard = OfficeStandard::default_standard();
    let faded = lot_plan::floor_plan::faded_storey(&p, storey(&p, "EG"), standard, viewport);
    assert!(faded.len() > 20, "{}", faded.len());
    assert!(faded.iter().all(|o| !matches!(o, PlanObject::Text(_))));
}

/// Blaue Umrisse wie die Auswahl in der App.
fn blue(rings: Vec<Vec<[f64; 2]>>, f: f64, dx: f64, dy: f64) -> Vec<PlanObject> {
    rings
        .into_iter()
        .map(|r| {
            PlanObject::Line(Line {
                points: r.into_iter().map(|q| PaperPoint::new(q[0] * f + dx, q[1] * f + dy)).collect(),
                closed: true,
                pen: Pen::mm(0.5),
                style: LineStyle::Solid,
                color: Color::rgb8(26, 115, 242),
            })
        })
        .collect()
}

/// Objekte mittig auf ein Blatt A3 quer.
fn centered(title: &str, mut objects: Vec<PlanObject>) -> Sheet {
    let mut sheet = Sheet { title: title.into(), width_mm: 420.0, height_mm: 297.0, objects: Vec::new() };
    if let Some((lo, hi)) = lot_plan::bounds(&objects) {
        let (dx, dy) = (210.0 - (lo.x + hi.x) / 2.0, 148.5 - (lo.y + hi.y) / 2.0);
        for o in &mut objects {
            o.translate(dx, dy);
        }
    }
    sheet.objects = objects;
    sheet
}

#[test]
#[ignore]
fn pruefbilder() {
    let out = std::env::var("LOT_OUT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../out/navigation"));
    std::fs::create_dir_all(&out).unwrap();
    let p = efh();
    let g = lot_geom::build(&p.variant_view()).unwrap();
    let standard = OfficeStandard::default_standard();
    let viewport = Viewport { scale: 100.0, origin: PaperPoint::new(0.0, 0.0) };
    let write = |name: &str, sheet: &Sheet| {
        std::fs::write(out.join(format!("{name}.png")), lot_io::png::to_png(sheet, 150.0).unwrap()).unwrap();
    };

    // 1. Grundriss DG, darunter das EG blass.
    let (dg, eg) = (storey(&p, "DG"), storey(&p, "EG"));
    let mut objects = lot_plan::floor_plan::faded_storey(&p, eg, standard, viewport);
    let view = lot_plan::floor_plan::FloorPlanView {
        storey: dg,
        cut_height: lot_plan::floor_plan::DEFAULT_CUT_HEIGHT,
        scale: 100.0,
    };
    objects.extend(lot_plan::floor_plan::floor_plan(&p, view, standard, viewport));
    write("grundriss-dg-eg-blass", &centered("Grundriss DG, EG blass darunter", objects));

    // 2. Schnitt A-A, die Wände des EG gewählt.
    let (_, v) = section(&p, "Schnitt A-A");
    let mut objects = lot_plan::section::section(&p, &g, v, standard, viewport);
    let walls: Vec<ElementId> = p.walls_on(eg).map(|(id, _)| id).collect();
    objects.extend(blue(lot_plan::section::section_outlines(&g, v, &walls), 10.0, 0.0, 0.0));
    write("schnitt-a-auswahl", &centered("Schnitt A-A mit Auswahl", objects));

    // 3. Planblatt 102 wie in der App, Schnitt A-A darauf gewählt.
    let (_, sheet) = p.plan_sheets().into_iter().find(|(_, s)| s.number == "102").expect("Blatt 102");
    let mut drawn = lot_plan::layout::plan_sheet(&p, &g, sheet, standard);
    let content = lot_model::ViewContent::Section { view: section(&p, "Schnitt A-A").0 };
    let index = sheet.views.iter().position(|v| v.content == content).expect("Schnitt A-A auf 102");
    if let Some((_, lo, hi)) =
        lot_plan::layout::placed_extents(&p, &g, sheet, standard).into_iter().find(|(i, _, _)| *i == index)
    {
        drawn.objects.extend(blue(vec![vec![[lo.x, lo.y], [hi.x, lo.y], [hi.x, hi.y], [lo.x, hi.y]]], 1.0, 0.0, 0.0));
    }
    write("planblatt-102", &drawn);
}
