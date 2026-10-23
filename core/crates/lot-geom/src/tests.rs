use lot_model::{
    Column, ColumnProfile, Element, ElementId, Layer, LayerFunction, Point2, Project, Storey, Top, Wall, WallAlignment,
    WallType,
};

use super::*;
use crate::poly::strip;

const STOREY: ElementId = ElementId(1);
const WDVS: ElementId = ElementId(2);
const INNEN: ElementId = ElementId(3);
const TRENN: ElementId = ElementId(4);

fn layer(material: &str, thickness: f64, function: LayerFunction) -> Layer {
    Layer { material: material.into(), thickness, function }
}

/// Projekt mit einem Geschoss und drei Wandaufbauten:
/// Außenwand mit WDVS (36,5 cm), tragende Innenwand (20,5 cm), Trennwand (14,5 cm).
fn project() -> Project {
    use LayerFunction::*;
    let mut p = Project::new("Test");
    p.elements.insert(STOREY, Element::Storey(Storey { name: "EG".into(), elevation: 0.0, height: 2.75 }));
    p.elements.insert(
        WDVS,
        Element::WallType(WallType {
            name: "Außenwand WDVS".into(),
            layers: vec![
                layer("putz", 0.015, Finish),
                layer("daemmung", 0.16, Insulation),
                layer("kalksandstein", 0.175, Structure),
                layer("putz", 0.015, Finish),
            ],
        }),
    );
    p.elements.insert(
        INNEN,
        Element::WallType(WallType {
            name: "Innenwand tragend".into(),
            layers: vec![
                layer("putz", 0.015, Finish),
                layer("kalksandstein", 0.175, Structure),
                layer("putz", 0.015, Finish),
            ],
        }),
    );
    p.elements.insert(
        TRENN,
        Element::WallType(WallType {
            name: "Trennwand".into(),
            layers: vec![
                layer("putz", 0.015, Finish),
                layer("kalksandstein", 0.115, Structure),
                layer("putz", 0.015, Finish),
            ],
        }),
    );
    p
}

fn add_wall(
    p: &mut Project,
    wall_type: ElementId,
    a: (f64, f64),
    b: (f64, f64),
    alignment: WallAlignment,
) -> ElementId {
    let id = p.next_id();
    p.elements.insert(
        id,
        Element::Wall(Wall {
            storey: STOREY,
            wall_type,
            start: Point2::new(a.0, a.1),
            end: Point2::new(b.0, b.1),
            alignment,
            base_offset: 0.0,
            top: Top::Storey { offset: 0.0 },
            sagitta: 0.0,
        }),
    );
    id
}

fn regions(p: &Project) -> Vec<LayerRegion> {
    layer_regions(p, STOREY, 1.0)
}

fn total_area(regions: &[LayerRegion]) -> f64 {
    regions.iter().map(|r| r.area.area()).sum()
}

/// Keine zwei Schichtflächen überlappen sich. Clipper2 rundet neue
/// Schnittpunkte auf das µm-Raster; an langen schrägen Kanten und spitzen
/// Winkeln bleiben deshalb Splitter unter 2 µm Breite. Alles darüber ist ein
/// Fehler. Die Breite eines Rests ist seine mittlere Breite, 2 · Fläche / Umfang.
fn assert_no_overlaps(regions: &[LayerRegion]) {
    for (i, a) in regions.iter().enumerate() {
        for b in &regions[i + 1..] {
            for ring in a.area.intersection(&b.area).rings() {
                let n = ring.len();
                let perimeter: f64 =
                    (0..n).map(|k| (ring[k][0] - ring[(k + 1) % n][0]).hypot(ring[k][1] - ring[(k + 1) % n][1])).sum();
                let width = 2.0 * signed_area(&ring).abs() / perimeter.max(1e-12);
                assert!(
                    width < 2e-6,
                    "Überlappung {width} m breit zwischen {:?}/{} und {:?}/{}: {ring:?}",
                    a.element,
                    a.layer,
                    b.element,
                    b.layer
                );
            }
        }
    }
}

fn region(regions: &[LayerRegion], element: ElementId, layer: usize) -> &LayerRegion {
    regions.iter().find(|r| r.element == element && r.layer == layer).expect("Schicht fehlt")
}

#[test]
fn free_wall_has_exact_layers() {
    let mut p = project();
    let w = add_wall(&mut p, WDVS, (0.0, 0.0), (4.0, 0.0), WallAlignment::Exterior);
    let r = regions(&p);
    assert_eq!(r.len(), 4);
    assert!((region(&r, w, 1).area.area() - 4.0 * 0.16).abs() < 1e-9);
    assert!((total_area(&r) - 4.0 * 0.365).abs() < 1e-9);
}

#[test]
fn l_corner_with_same_build_up_is_mitered() {
    // Außenecke, im Uhrzeigersinn gezeichnet: außen liegt links.
    let mut p = project();
    let a = add_wall(&mut p, WDVS, (-4.0, 0.0), (0.0, 0.0), WallAlignment::Exterior);
    let b = add_wall(&mut p, WDVS, (0.0, 0.0), (0.0, -4.0), WallAlignment::Exterior);
    let r = regions(&p);
    assert_no_overlaps(&r);
    // Gesamtfläche: zwei Streifen 4 m x 0,365 m, das Eckquadrat zählt einmal.
    let t: f64 = 0.365;
    assert!((total_area(&r) - (2.0 * 4.0 * t - t * t)).abs() < 1e-8, "{}", total_area(&r));
    // Die Dämmung läuft um die Ecke: beide Dämmflächen berühren sich auf der Gehrung.
    let da = &region(&r, a, 1).area;
    let db = &region(&r, b, 1).area;
    let joined = da.union(db);
    assert_eq!(joined.rings().len(), 1, "Dämmung ist an der Ecke unterbrochen");
    // Ebenso die Tragschicht.
    let joined = region(&r, a, 2).area.union(&region(&r, b, 2).area);
    assert_eq!(joined.rings().len(), 1, "Tragschicht ist an der Ecke unterbrochen");
}

#[test]
fn tee_structure_binds_through_plaster() {
    // Außenwand von West nach Ost, außen im Norden (links). Innenwand stößt von Süden an.
    let mut p = project();
    let outer = add_wall(&mut p, WDVS, (-4.0, 0.0), (4.0, 0.0), WallAlignment::Exterior);
    let inner = add_wall(&mut p, INNEN, (0.0, -4.0), (0.0, -0.365), WallAlignment::Center);
    let r = regions(&p);
    assert_no_overlaps(&r);
    // Die Tragschicht der Innenwand reicht bis an die Tragschicht der Außenwand (y = -0,35),
    // also durch den Innenputz der Außenwand.
    let (_, hi) = region(&r, inner, 1).area.bounds().unwrap();
    assert!((hi[1] - (-0.35)).abs() < 1e-6, "Tragschicht endet bei y = {}", hi[1]);
    // Der Innenputz der Innenwand endet an der Wandfläche der Außenwand.
    let (_, hi) = region(&r, inner, 0).area.bounds().unwrap();
    assert!((hi[1] - (-0.365)).abs() < 1e-6, "Putz endet bei y = {}", hi[1]);
    // Der Innenputz der Außenwand ist von der Innenwand unterbrochen: zwei Stücke.
    assert_eq!(region(&r, outer, 3).area.rings().len(), 2);
    // Die Dämmung der Außenwand läuft ungestört durch.
    assert!((region(&r, outer, 1).area.area() - 8.0 * 0.16).abs() < 1e-8);
}

#[test]
fn tee_ending_on_reference_line_is_trimmed_to_face() {
    // Die Innenwand ist bis zur Bezugslinie der Außenwand gezeichnet (y = 0) und wird an deren Fläche gekürzt.
    let mut p = project();
    add_wall(&mut p, WDVS, (-4.0, 0.0), (4.0, 0.0), WallAlignment::Exterior);
    let inner = add_wall(&mut p, INNEN, (0.0, -4.0), (0.0, 0.0), WallAlignment::Center);
    let r = regions(&p);
    assert_no_overlaps(&r);
    let (_, hi) = region(&r, inner, 1).area.bounds().unwrap();
    assert!((hi[1] - (-0.35)).abs() < 1e-6, "Tragschicht endet bei y = {}", hi[1]);
}

#[test]
fn crossing_keeps_both_walls_complete() {
    let mut p = project();
    add_wall(&mut p, INNEN, (-3.0, 0.0), (3.0, 0.0), WallAlignment::Center);
    add_wall(&mut p, TRENN, (0.0, -3.0), (0.0, 3.0), WallAlignment::Center);
    let r = regions(&p);
    assert_no_overlaps(&r);
    let expected = strip([-3.0, 0.0], [1.0, 0.0], (0.0, 6.0), (0.1025, -0.1025)).union(&strip(
        [0.0, -3.0],
        [0.0, 1.0],
        (0.0, 6.0),
        (0.0725, -0.0725),
    ));
    assert!((total_area(&r) - expected.area()).abs() < 1e-8);
}

#[test]
fn y_node_partitions_the_center() {
    let mut p = project();
    add_wall(&mut p, INNEN, (0.0, 0.0), (3.0, 0.0), WallAlignment::Center);
    add_wall(&mut p, INNEN, (0.0, 0.0), (-1.5, 2.6), WallAlignment::Center);
    add_wall(&mut p, INNEN, (0.0, 0.0), (-1.5, -2.6), WallAlignment::Center);
    let r = regions(&p);
    assert_no_overlaps(&r);
    assert!(total_area(&r) > 3.0 * 2.8 * 0.205);
}

#[test]
fn opening_cuts_wall_at_cut_height() {
    let p = Project::sample();
    let r = layer_regions(&p, ElementId(1), 1.0);
    assert_eq!(r.len(), 1);
    assert_eq!(r[0].area.rings().len(), 2);
    let below = layer_regions(&p, ElementId(1), 0.5);
    assert_eq!(below[0].area.rings().len(), 1);
}

#[test]
fn column_wins_over_wall() {
    let mut p = project();
    let wall = add_wall(&mut p, INNEN, (0.0, 0.0), (3.0, 0.0), WallAlignment::Center);
    let id = p.next_id();
    p.elements.insert(
        id,
        Element::Column(Column {
            storey: STOREY,
            position: Point2::new(3.0, 0.0),
            profile: ColumnProfile::Rectangle { width: 0.3, depth: 0.3 },
            rotation: 0.0,
            material: "stahlbeton".into(),
            base_offset: 0.0,
            top: Top::Storey { offset: 0.0 },
        }),
    );
    let r = regions(&p);
    assert_no_overlaps(&r);
    // Die Wand endet an der Stützenfläche (x = 2,85).
    let (_, hi) = region(&r, wall, 1).area.bounds().unwrap();
    assert!((hi[0] - 2.85).abs() < 1e-6);
}

#[test]
fn acute_corner_is_capped() {
    let mut p = project();
    add_wall(&mut p, INNEN, (-4.0, 0.0), (0.0, 0.0), WallAlignment::Center);
    let angle = 15f64.to_radians();
    add_wall(&mut p, INNEN, (0.0, 0.0), (-4.0 * angle.cos(), 4.0 * angle.sin()), WallAlignment::Center);
    let r = regions(&p);
    assert_no_overlaps(&r);
    // Ohne Kappung ragte die Spitze 0,79 m über den Knoten hinaus, gekappt etwa zwei Wanddicken.
    let all = Area::union_all(r.iter().map(|x| &x.area));
    let (_, hi) = all.bounds().unwrap();
    assert!(hi[0] < 0.45, "Spitze reicht bis x = {}", hi[0]);
}

#[test]
fn random_constellations_never_overlap() {
    // Eigenschaftstest (PLAN.md, Abschnitt 4): zufällige Wände auf einem Raster,
    // mit Ecken, Stößen und Kreuzungen. Keine Überlappung, keine verlorene Fläche.
    let mut seed: u64 = 0x5eed_1a7e;
    let mut next = move |n: u64| {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed % n
    };
    for _ in 0..200 {
        let mut p = project();
        let points: Vec<(f64, f64)> = (0..5).map(|_| (next(5) as f64 * 1.5, next(5) as f64 * 1.5)).collect();
        for _ in 0..4 {
            let a = points[next(5) as usize];
            let b = points[next(5) as usize];
            if a == b {
                continue;
            }
            let wall_type = [WDVS, INNEN, TRENN][next(3) as usize];
            let alignment = [WallAlignment::Center, WallAlignment::Exterior, WallAlignment::Interior][next(3) as usize];
            add_wall(&mut p, wall_type, a, b, alignment);
        }
        let r = regions(&p);
        if std::env::var("LOT_DEBUG").is_ok() {
            for (id, w) in p.walls() {
                eprintln!("{id:?}: {:?} {:?} -> {:?} {:?}", w.wall_type, w.start, w.end, w.alignment);
            }
        }
        assert_no_overlaps(&r);
        for region in &r {
            assert!(region.area.area() >= 0.0);
        }
    }
}

#[test]
fn sample_volume_is_wall_minus_window() {
    let g = build(&Project::sample()).unwrap();
    assert_eq!(g.elements.len(), 1);
    let mesh = &g.elements[0].mesh;
    let volume: f64 = mesh
        .triangles
        .iter()
        .map(|t| {
            let [a, b, c] = t.map(|i| mesh.positions[i as usize]);
            mesh::dot(a, mesh::cross(b, c)) / 6.0
        })
        .sum();
    let expected = 5.0 * 0.3 * 2.6 - 1.5 * 0.3 * 1.3;
    assert!((volume - expected).abs() < 1e-6, "Volumen {volume}");
}

#[test]
fn walls_of_different_height_meet_in_3d() {
    let mut p = project();
    let a = add_wall(&mut p, INNEN, (0.0, 0.0), (3.0, 0.0), WallAlignment::Center);
    let b = add_wall(&mut p, INNEN, (3.0, 0.0), (3.0, 3.0), WallAlignment::Center);
    if let Some(Element::Wall(w)) = p.elements.get_mut(&b) {
        w.top = Top::Height(1.0);
    }
    let g = build(&p).unwrap();
    let height = |id: ElementId| {
        g.elements
            .iter()
            .filter(|e| e.id == id)
            .filter_map(|e| e.mesh.bounds())
            .map(|(_, hi)| hi[2])
            .fold(0.0, f64::max)
    };
    assert!((height(a) - 2.75).abs() < 1e-9);
    assert!((height(b) - 1.0).abs() < 1e-9);
}

#[test]
fn equal_bands_become_one_box() {
    // Die niedrige Wand teilt das Geschoss bei 1,00 m. Die freie hohe Wand ist
    // in beiden Abschnitten gleich und wird deshalb ein einziger Quader je Schicht.
    let mut p = project();
    let tall = add_wall(&mut p, INNEN, (0.0, 0.0), (3.0, 0.0), WallAlignment::Center);
    let low = add_wall(&mut p, INNEN, (0.0, 2.0), (3.0, 2.0), WallAlignment::Center);
    if let Some(Element::Wall(w)) = p.elements.get_mut(&low) {
        w.top = Top::Height(1.0);
    }
    let g = build(&p).unwrap();
    for e in g.elements.iter().filter(|e| e.id == tall) {
        assert_eq!((e.mesh.positions.len(), e.mesh.triangles.len()), (8, 12), "Schicht {}", e.layer);
    }
}

#[test]
fn taller_wall_runs_through_at_corner() {
    let mut p = project();
    let tall = add_wall(&mut p, INNEN, (0.0, 0.0), (3.0, 0.0), WallAlignment::Center);
    let low = add_wall(&mut p, INNEN, (3.0, 0.0), (3.0, -3.0), WallAlignment::Center);
    if let Some(Element::Wall(w)) = p.elements.get_mut(&low) {
        w.top = Top::Height(1.0);
    }
    // Über der Brüstung: Die hohe Wand reicht bis an die Außenfläche der niedrigen (x = 3,1025).
    let above = layer_regions(&p, STOREY, 2.0);
    let (_, hi) = region(&above, tall, 1).area.bounds().unwrap();
    assert!((hi[0] - 3.1025).abs() < 1e-6, "Tragschicht endet bei x = {}", hi[0]);
    // Darunter genauso; die niedrige Wand stößt stumpf an.
    let below = layer_regions(&p, STOREY, 0.5);
    assert_no_overlaps(&below);
    let (_, hi) = region(&below, tall, 1).area.bounds().unwrap();
    assert!((hi[0] - 3.1025).abs() < 1e-6);
    let (_, hi) = region(&below, low, 1).area.bounds().unwrap();
    assert!(hi[1] <= -0.0875 + 1e-6, "niedrige Wand reicht bis y = {}", hi[1]);
}

fn bend(p: &mut Project, id: ElementId, sagitta: f64) {
    if let Some(Element::Wall(w)) = p.elements.get_mut(&id) {
        w.sagitta = sagitta;
    }
}

/// Bogen von (0, 0) nach (4, 0) mit Stich 1 m nach oben: Radius 2,5 m, Mittelpunkt (2; -1,5).
fn arc_project() -> (Project, ElementId) {
    let mut p = project();
    let arc = add_wall(&mut p, INNEN, (0.0, 0.0), (4.0, 0.0), WallAlignment::Center);
    bend(&mut p, arc, 1.0);
    (p, arc)
}

fn arc_length(p: &Project, id: ElementId) -> f64 {
    p.wall(id).unwrap().length()
}

#[test]
fn free_arc_wall_has_exact_area() {
    let (p, arc) = arc_project();
    let r = regions(&p);
    assert_eq!(r.len(), 3, "eine Fläche je Schicht, nicht je Facette");
    assert_no_overlaps(&r);
    let expected = arc_length(&p, arc) * 0.205;
    assert!((total_area(&r) - expected).abs() < 1e-3 * expected, "{} statt {expected}", total_area(&r));
    // Die äußere Putzschicht liegt außen am Bogen: Ihr höchster Punkt ist die Bogenmitte plus 10,25 cm.
    let (_, hi) = region(&r, arc, 0).area.bounds().unwrap();
    assert!((hi[1] - 1.1025).abs() < 1e-4, "y = {}", hi[1]);
    assert!(region(&r, arc, 0).band.unwrap().arc.is_some());
}

#[test]
fn wall_meets_arc_at_a_facet_corner() {
    // Die Trennwand stößt genau in der Bogenmitte an, dort liegt eine Facettenecke.
    let (mut p, arc) = arc_project();
    let tee = add_wall(&mut p, TRENN, (2.0, 1.0), (2.0, -1.0), WallAlignment::Center);
    let r = regions(&p);
    assert_no_overlaps(&r);
    // Der Bogen läuft durch, nur sein Innenputz fehlt dort, wo die Tragschicht
    // der Trennwand (11,5 cm) hindurchbindet.
    let arc_area: f64 = r.iter().filter(|x| x.element == arc).map(|x| x.area.area()).sum();
    let expected = arc_length(&p, arc) * 0.205 - 0.115 * 0.015;
    assert!((arc_area - expected).abs() < 1e-3 * expected, "{arc_area} statt {expected}");
    // Die Tragschicht der Trennwand bindet durch den Innenputz bis an die Tragschicht
    // des Bogens: Radius 2,5 - 0,0875 um (2; -1,5), also y = 0,9125.
    let (_, hi) = region(&r, tee, 1).area.bounds().unwrap();
    assert!((hi[1] - 0.9125).abs() < 0.003, "Tragschicht endet bei y = {}", hi[1]);
}

#[test]
fn opening_in_arc_is_cut_radially() {
    let (mut p, arc) = arc_project();
    let id = p.next_id();
    p.elements.insert(
        id,
        Element::Opening(lot_model::Opening { host: arc, offset: 1.0, width: 1.0, height: 2.0, sill_height: 0.0 }),
    );
    let r = regions(&p);
    let expected = (arc_length(&p, arc) - 1.0) * 0.205;
    assert!((total_area(&r) - expected).abs() < 1e-3 * expected, "{} statt {expected}", total_area(&r));
    // Die Laibung zeigt zum Mittelpunkt: Die Ecken an Außen- und Innenfläche
    // liegen auf einer Geraden durch (2; -1,5).
    let w = p.wall(arc).unwrap();
    let outer = w.point_at(1.0, 0.1025);
    let inner = w.point_at(1.0, -0.1025);
    let c = w.arc().unwrap().center;
    let cross = (outer.x - c.x) * (inner.y - c.y) - (outer.y - c.y) * (inner.x - c.x);
    assert!(cross.abs() < 1e-9);
    // Die Wandflächen sind Facetten und liegen bis 2 mm innerhalb des Bogens,
    // die Ecken der Laibung aber genau auf dem Radius.
    let radial = |q: Point2| {
        r.iter().flat_map(|x| x.area.rings()).flatten().any(|v| {
            let near = (v[0] - q.x).hypot(v[1] - q.y) < 0.003;
            let off = ((v[0] - c.x) * (q.y - c.y) - (v[1] - c.y) * (q.x - c.x)) / q.distance(c);
            near && off.abs() < 1e-5
        })
    };
    assert!(radial(outer) && radial(inner), "Laibung nicht radial");
}

#[test]
fn arc_and_straight_wall_form_a_corner() {
    let (mut p, arc) = arc_project();
    let straight = add_wall(&mut p, INNEN, (4.0, 0.0), (4.0, -3.0), WallAlignment::Center);
    let r = regions(&p);
    assert_no_overlaps(&r);
    assert_eq!(joins::node_summary(&p, STOREY), vec![([4.0, 0.0], 2)]);
    for id in [arc, straight] {
        assert!(r.iter().any(|x| x.element == id && x.layer == 1 && !x.area.is_empty()));
    }
}

#[test]
fn random_constellations_with_arcs_never_overlap() {
    let mut seed: u64 = 0xa2c5_0b0e;
    let mut next = move |n: u64| {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed % n
    };
    for _ in 0..100 {
        let mut p = project();
        let points: Vec<(f64, f64)> = (0..5).map(|_| (next(5) as f64 * 1.5, next(5) as f64 * 1.5)).collect();
        for _ in 0..4 {
            let a = points[next(5) as usize];
            let b = points[next(5) as usize];
            if a == b {
                continue;
            }
            let wall_type = [WDVS, INNEN, TRENN][next(3) as usize];
            let alignment = [WallAlignment::Center, WallAlignment::Exterior, WallAlignment::Interior][next(3) as usize];
            let id = add_wall(&mut p, wall_type, a, b, alignment);
            bend(&mut p, id, [0.0, 0.0, 0.4, -0.4, 0.7][next(5) as usize]);
        }
        let r = regions(&p);
        if std::env::var("LOT_DEBUG").is_ok() {
            for (id, w) in p.walls() {
                eprintln!(
                    "{id:?}: {:?} {:?} -> {:?} {:?} Stich {}",
                    w.wall_type, w.start, w.end, w.alignment, w.sagitta
                );
            }
        }
        assert_no_overlaps(&r);
    }
}

#[test]
fn every_arc_layer_is_one_piece() {
    // Eine freie Bogenwand: Jede Schicht ist genau ein Ring, ohne Fuge an
    // den Facettenstößen, für beliebige Lage, Länge und Stich.
    let mut seed: u64 = 0x0b09_e11a;
    let mut next = move |n: u64| {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed % n
    };
    for _ in 0..300 {
        let mut p = project();
        let a = (next(2000) as f64 / 100.0 - 10.0, next(2000) as f64 / 100.0 - 10.0);
        let b = (next(2000) as f64 / 100.0 - 10.0, next(2000) as f64 / 100.0 - 10.0);
        let chord = (b.0 - a.0).hypot(b.1 - a.1);
        if chord < 1.0 {
            continue;
        }
        let wall_type = [WDVS, INNEN, TRENN][next(3) as usize];
        let id = add_wall(&mut p, wall_type, a, b, WallAlignment::Center);
        let sagitta = (next(1000) as f64 / 1000.0 - 0.5) * chord;
        bend(&mut p, id, (sagitta * 100.0).round() / 100.0);
        for r in regions(&p) {
            let rings = r.area.rings();
            assert_eq!(
                rings.len(),
                1,
                "Schicht {} von {a:?} nach {b:?}, Stich {sagitta}: {} Ringe",
                r.layer,
                rings.len()
            );
            // Kein Schlitz: Keine Ecke kehrt um.
            let ring = &rings[0];
            let n = ring.len();
            for k in 0..n {
                let (u, v, w) = (ring[(k + n - 1) % n], ring[k], ring[(k + 1) % n]);
                let d1 = [v[0] - u[0], v[1] - u[1]];
                let d2 = [w[0] - v[0], w[1] - v[1]];
                let cos = (d1[0] * d2[0] + d1[1] * d2[1]) / (d1[0].hypot(d1[1]) * d2[0].hypot(d2[1]));
                assert!(cos > -0.99, "Schlitz in Schicht {} bei {v:?}, von {a:?} nach {b:?}, Stich {sagitta}", r.layer);
            }
        }
    }
}

/// Rechteck 6 × 4 m aus tragenden Innenwänden (20,5 cm, Achse mittig), geteilt
/// durch eine Trennwand bei x = 2 mit Tür. Gibt die Trennwand zurück.
fn two_rooms() -> (Project, ElementId) {
    let mut p = project();
    for (a, b) in
        [((0.0, 0.0), (6.0, 0.0)), ((6.0, 0.0), (6.0, 4.0)), ((6.0, 4.0), (0.0, 4.0)), ((0.0, 4.0), (0.0, 0.0))]
    {
        add_wall(&mut p, INNEN, a, b, WallAlignment::Center);
    }
    let split = add_wall(&mut p, TRENN, (2.0, 0.0), (2.0, 4.0), WallAlignment::Center);
    let id = p.next_id();
    p.elements.insert(
        id,
        Element::Opening(lot_model::Opening {
            host: split,
            offset: 1.0,
            width: 0.885,
            height: 2.135,
            sill_height: 0.0,
        }),
    );
    (p, split)
}

#[test]
fn walls_enclose_rooms_and_doors_do_not_connect_them() {
    let (p, _) = two_rooms();
    let finder = rooms::RoomFinder::new(&p, STOREY);
    assert_eq!(finder.enclosed.len(), 2);
    // Lichte Maße an der fertigen Oberfläche: Innenwand 20,5 cm, Trennwand 14,5 cm.
    let left = finder.area_at([1.0, 2.0]).unwrap().area();
    let right = finder.area_at([4.0, 2.0]).unwrap().area();
    let height = 4.0 - 0.205;
    assert!((left - (2.0 - 0.1025 - 0.0725) * height).abs() < 1e-6, "links {left}");
    assert!((right - (4.0 - 0.1025 - 0.0725) * height).abs() < 1e-6, "rechts {right}");
    assert!(finder.area_at([7.0, 2.0]).is_none(), "außen ist kein Raum");
}

#[test]
fn column_in_room_is_cut_out() {
    let (mut p, _) = two_rooms();
    let id = p.next_id();
    p.elements.insert(
        id,
        Element::Column(Column {
            storey: STOREY,
            position: Point2::new(4.0, 2.0),
            profile: ColumnProfile::Rectangle { width: 0.3, depth: 0.3 },
            rotation: 0.0,
            material: "stahlbeton".into(),
            base_offset: 0.0,
            top: Top::Storey { offset: 0.0 },
        }),
    );
    let finder = rooms::RoomFinder::new(&p, STOREY);
    let right = finder.area_at([3.0, 1.0]).unwrap().area();
    assert!((right - ((4.0 - 0.175) * 3.795 - 0.09)).abs() < 1e-6, "{right}");
    // Der Punkt auf der Stütze liegt in keinem Raum.
    assert!(finder.area_at([4.0, 2.0]).is_none());
}

#[test]
fn open_walls_enclose_nothing() {
    let mut p = project();
    for (a, b) in [((0.0, 0.0), (6.0, 0.0)), ((6.0, 0.0), (6.0, 4.0)), ((0.0, 4.0), (0.0, 0.0))] {
        add_wall(&mut p, INNEN, a, b, WallAlignment::Center);
    }
    assert!(rooms::RoomFinder::new(&p, STOREY).enclosed.is_empty());
}

#[test]
fn interior_point_lies_inside_an_l_shape() {
    let l = Area::from_ring(&[[0.0, 0.0], [4.0, 0.0], [4.0, 1.0], [1.0, 1.0], [1.0, 4.0], [0.0, 4.0]]);
    let p = rooms::interior_point(&l).unwrap();
    assert!(l.contains(p), "{p:?}");
}
