//! Sichtbarkeit (ADR 0050) an MFH und EFH: Vorlagen ab Werk wirken, Regeln
//! färben und blenden aus, Blätter zeigen die Ansicht wie eingestellt. Mit
//! `--ignored` schreibt `pruefbilder` die Bilder fürs Prüfpaket nach `LOT_OUT`.

use std::path::Path;

use lot_model::{Element, ElementId, Project, ViewContent, ViewSettings};
use lot_plan::standard::OfficeStandard;
use lot_plan::views::{self, Temporary};
use lot_plan::{Color, PaperPoint, PlanObject, Sheet, Viewport};

fn load(name: &str) -> Project {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../../testprojekte/{name}/projekt.ron"));
    lot_io::testproject::load(&path).unwrap()
}

fn storey(project: &Project, name: &str) -> ElementId {
    project.storeys().into_iter().find(|(_, s)| s.name == name).map(|(id, _)| id).expect("Geschoss")
}

/// Feuerwiderstand als Beispielwerte an den Wandaufbauten des MFH (nicht geprüft).
fn mfh_with_fire_ratings() -> Project {
    let mut p = load("mfh");
    for e in p.elements.values_mut() {
        if let Element::WallType(t) = e {
            let rating = match t.name.as_str() {
                n if n.starts_with("Treppenraum") => Some("F90"),
                n if n.starts_with("Wohnungstrennwand") => Some("F60"),
                n if n.starts_with("Innenwand tragend") => Some("F30"),
                _ => None,
            };
            t.props.fire_rating = rating.map(String::from);
        }
    }
    p
}

fn with_template(mut p: Project, view: ViewContent, template: &str) -> Project {
    let mut s = ViewSettings::new(view);
    s.template = Some(template.into());
    let id = p.next_id();
    p.elements.insert(id, Element::ViewSettings(s));
    p
}

fn drawn(p: &Project, view: ViewContent, storey: ElementId) -> Vec<PlanObject> {
    let standard = OfficeStandard::default_standard();
    let look = views::look(p, standard, view);
    let shown = views::apply(p, &look, &Temporary::default());
    let viewport = Viewport { scale: look.scale.value, origin: PaperPoint::new(0.0, 0.0) };
    let fp = lot_plan::floor_plan::FloorPlanView { storey, cut_height: look.cut_height.value, scale: look.scale.value };
    lot_plan::floor_plan::floor_plan_with(&shown.project, fp, standard, viewport, &shown.paint)
}

fn has_fill(objects: &[PlanObject], c: Color) -> bool {
    objects.iter().any(|o| matches!(o, PlanObject::Area(a) if a.fill == lot_plan::Fill::Solid(c)))
}

#[test]
fn mfh_fire_plan_colors_walls_by_rating_and_hides_dimensions() {
    let p = mfh_with_fire_ratings();
    let eg = storey(&p, "EG");
    let view = ViewContent::FloorPlan { storey: eg };
    let p = with_template(p, view, "Brandschutzplan");
    let look = views::look(&p, OfficeStandard::default_standard(), view);
    let shown = views::apply(&p, &look, &Temporary::default());
    let count = |name: &str| shown.counts[look.rules.iter().position(|r| r.rule.name == name).unwrap()];
    assert!(count("F90") > 0 && count("F60") > 0 && count("F30") > 0);
    assert!(count("Übrige Wände grau") > 0);
    // Maßketten fehlen im Brandschutzplan.
    assert!(shown.project.dimensions_on(eg).next().is_none() || count("Maßketten ausblenden") > 0);
    let objects = drawn(&p, view, eg);
    assert!(has_fill(&objects, Color::rgb8(226, 35, 26)));
}

#[test]
fn efh_area_plan_colors_spaces_and_sheets_follow_the_view() {
    let p = load("efh");
    let eg = storey(&p, "EG");
    let view = ViewContent::FloorPlan { storey: eg };
    let plain = drawn(&p, view, eg);
    let p = with_template(p, view, "Flächenplan");
    let objects = drawn(&p, view, eg);
    assert!(has_fill(&objects, Color::rgb8(250, 228, 160)), "Nutzungsflächen gelb");
    assert!(!has_fill(&plain, Color::rgb8(250, 228, 160)));
    // Das Blatt mit dem EG zeigt den Flächenplan, ohne dass am Blatt etwas zu tun ist.
    let g = lot_geom::build(&p).unwrap();
    let (_, sheet) =
        p.plan_sheets().into_iter().find(|(_, s)| s.views.iter().any(|v| v.content == view)).expect("Blatt mit dem EG");
    let drawn_sheet = lot_plan::layout::plan_sheet(&p, &g, sheet, OfficeStandard::default_standard());
    assert!(has_fill(&drawn_sheet.objects, Color::rgb8(250, 228, 160)));
}

/// Objekte mittig auf ein Blatt A3 quer.
fn centered(title: &str, mut objects: Vec<PlanObject>) -> Sheet {
    if let Some((lo, hi)) = lot_plan::bounds(&objects) {
        let (dx, dy) = (210.0 - (lo.x + hi.x) / 2.0, 148.5 - (lo.y + hi.y) / 2.0);
        for o in &mut objects {
            o.translate(dx, dy);
        }
    }
    Sheet { title: title.into(), width_mm: 420.0, height_mm: 297.0, objects }
}

#[test]
#[ignore]
fn pruefbilder() {
    let out = std::env::var("LOT_OUT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../out/sichtbarkeit"));
    std::fs::create_dir_all(&out).unwrap();
    let write = |name: &str, sheet: &Sheet| {
        std::fs::write(out.join(format!("{name}.png")), lot_io::png::to_png(sheet, 150.0).unwrap()).unwrap();
    };
    let p = mfh_with_fire_ratings();
    let eg = storey(&p, "EG");
    let view = ViewContent::FloorPlan { storey: eg };
    let p = with_template(p, view, "Brandschutzplan");
    write("mfh-brandschutzplan-eg", &centered("MFH EG, Vorlage Brandschutzplan", drawn(&p, view, eg)));
    let p = load("efh");
    let eg = storey(&p, "EG");
    let view = ViewContent::FloorPlan { storey: eg };
    let p = with_template(p, view, "Flächenplan");
    write("efh-flaechenplan-eg", &centered("EFH EG, Vorlage Flächenplan", drawn(&p, view, eg)));
}
