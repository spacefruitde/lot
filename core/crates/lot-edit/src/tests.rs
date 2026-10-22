use lot_model::{Document, ElementId, Opening, Point2, Project};

use super::*;

fn input(x: f64, y: f64) -> Input {
    Input { point: Point2::new(x, y), tolerance: 0.15, shift: false }
}

fn setup() -> (Document, Editor) {
    let doc = Document::new(Project::template("Test"));
    let editor = Editor::new(doc.project());
    (doc, editor)
}

fn click(doc: &mut Document, editor: &mut Editor, x: f64, y: f64) -> Response {
    editor.pointer_moved(doc.project(), input(x, y));
    editor.pointer_down(doc, input(x, y), 1)
}

fn walls(doc: &Document) -> Vec<lot_model::Wall> {
    doc.project().walls().map(|(_, w)| w.clone()).collect()
}

#[test]
fn draws_closed_rectangle_by_clicking() {
    let (mut doc, mut ed) = setup();
    ed.run(&mut doc, &Command::WallTool);
    for (x, y) in [(0.0, 0.0), (0.0, 4.0), (6.0, 4.0), (6.0, 0.0), (0.03, 0.02)] {
        click(&mut doc, &mut ed, x, y);
    }
    ed.key(&mut doc, Key::Escape);
    let w = walls(&doc);
    assert_eq!(w.len(), 4);
    // Der letzte Klick rastet am Anfangspunkt ein, das Rechteck ist geschlossen.
    assert_eq!(w[3].end, Point2::new(0.0, 0.0));
    // Jede Wand ist ein eigener Undo-Schritt.
    assert!(doc.undo().unwrap());
    assert_eq!(walls(&doc).len(), 3);
}

#[test]
fn typed_length_sets_wall_length() {
    let (mut doc, mut ed) = setup();
    ed.run(&mut doc, &Command::WallTool);
    click(&mut doc, &mut ed, 1.0, 1.0);
    ed.pointer_moved(doc.project(), input(3.0, 1.05));
    for c in "4,25".chars() {
        ed.key(&mut doc, Key::Char(c));
    }
    ed.key(&mut doc, Key::Enter);
    let w = &walls(&doc)[0];
    assert!((w.length() - 4.25).abs() < 1e-9);
    // Die Richtung rastet waagerecht ein.
    assert!((w.end.y - 1.0).abs() < 1e-9);
}

#[test]
fn angle_snaps_in_15_degree_steps_and_round_lengths() {
    let p = Project::template("t");
    let s = snap::snap(&p, ElementId(1), Point2::new(3.0, 0.07), 0.1, Some(Point2::new(0.0, 0.0)), false, &[]);
    assert_eq!(s.kind, SnapKind::Angle);
    assert_eq!(s.point, Point2::new(3.0, 0.0));
    // Mit Umschalttaste nur rechte Winkel.
    let s = snap::snap(&p, ElementId(1), Point2::new(3.0, 1.0), 0.1, Some(Point2::new(0.0, 0.0)), true, &[]);
    assert!((s.point.y).abs() < 1e-9);
}

#[test]
fn snaps_to_wall_end_and_wall_line() {
    let (mut doc, mut ed) = setup();
    ed.run(&mut doc, &Command::WallTool);
    click(&mut doc, &mut ed, 0.0, 0.0);
    click(&mut doc, &mut ed, 5.0, 0.0);
    ed.key(&mut doc, Key::Escape);
    let p = doc.project();
    let s = snap::snap(p, ed.storey, Point2::new(4.93, 0.06), 0.15, None, false, &[]);
    assert_eq!((s.kind, s.point), (SnapKind::Endpoint, Point2::new(5.0, 0.0)));
    let s = snap::snap(p, ed.storey, Point2::new(2.5, 0.08), 0.15, None, false, &[]);
    assert_eq!(s.kind, SnapKind::OnWall);
    assert!(s.point.y.abs() < 1e-9);
}

/// Rechteck plus eine Innenwand, die von unten an die obere Wand stößt.
fn house() -> (Document, Editor) {
    let (mut doc, mut ed) = setup();
    ed.run(&mut doc, &Command::WallTool);
    for (x, y) in [(0.0, 0.0), (0.0, 4.0), (6.0, 4.0), (6.0, 0.0), (0.0, 0.0)] {
        click(&mut doc, &mut ed, x, y);
    }
    ed.key(&mut doc, Key::Escape);
    click(&mut doc, &mut ed, 3.0, 0.0);
    click(&mut doc, &mut ed, 3.0, 4.0);
    ed.key(&mut doc, Key::Escape);
    ed.key(&mut doc, Key::Escape);
    assert_eq!(ed.tool, Tool::Select);
    (doc, ed)
}

fn wall_with(doc: &Document, a: Point2, b: Point2) -> ElementId {
    doc.project()
        .walls()
        .find(|(_, w)| (w.start == a && w.end == b) || (w.start == b && w.end == a))
        .map(|(id, _)| id)
        .expect("Wand fehlt")
}

#[test]
fn moving_a_wall_drags_corners_and_tees() {
    let (mut doc, mut ed) = house();
    let top = wall_with(&doc, Point2::new(0.0, 4.0), Point2::new(6.0, 4.0));
    // Obere Wand greifen und um 1 m nach oben ziehen.
    ed.pointer_down(&mut doc, input(1.5, 4.0), 1);
    assert_eq!(ed.selection, vec![top]);
    ed.pointer_dragged(doc.project(), input(1.5, 5.0));
    assert!(ed.preview(doc.project()).is_some());
    ed.pointer_up(&mut doc);
    let p = doc.project();
    assert_eq!(p.wall(top).unwrap().start.y, 5.0);
    // Die Seitenwände sind mitgewachsen, die Innenwand reicht wieder bis an die obere Wand.
    assert!(walls(&doc).iter().any(|w| w.start == Point2::new(0.0, 0.0) && w.end == Point2::new(0.0, 5.0)));
    assert!(walls(&doc).iter().any(|w| w.start == Point2::new(3.0, 0.0) && w.end == Point2::new(3.0, 5.0)));
    // Ein Undo nimmt alles zurück.
    doc.undo().unwrap();
    assert_eq!(doc.project().wall(top).unwrap().start.y, 4.0);
}

#[test]
fn dragging_an_end_moves_connected_end() {
    let (mut doc, mut ed) = house();
    let left = wall_with(&doc, Point2::new(0.0, 0.0), Point2::new(0.0, 4.0));
    ed.pointer_down(&mut doc, input(0.0, 2.0), 1);
    ed.pointer_up(&mut doc);
    assert_eq!(ed.selection, vec![left]);
    // Griff an der oberen Ecke 1 m nach oben ziehen; die obere Wand zieht mit.
    ed.pointer_down(&mut doc, input(0.0, 4.0), 1);
    ed.pointer_dragged(doc.project(), input(0.02, 5.003));
    ed.pointer_up(&mut doc);
    assert_eq!(doc.project().wall(left).unwrap().end, Point2::new(0.0, 5.0));
    assert!(walls(&doc).iter().any(|w| w.start == Point2::new(0.0, 5.0) && w.end == Point2::new(6.0, 4.0)));
}

#[test]
fn delete_removes_wall_with_openings() {
    let (mut doc, mut ed) = house();
    let inner = wall_with(&doc, Point2::new(3.0, 0.0), Point2::new(3.0, 4.0));
    doc.insert(lot_model::Element::Opening(Opening {
        host: inner,
        offset: 1.0,
        width: 1.0,
        height: 2.1,
        sill_height: 0.0,
    }))
    .unwrap();
    ed.pointer_down(&mut doc, input(3.0, 2.0), 1);
    ed.pointer_up(&mut doc);
    let r = ed.key(&mut doc, Key::Delete);
    assert!(r.model_changed, "{:?}", r.error);
    assert!(doc.project().wall(inner).is_none());
    assert_eq!(doc.project().openings_in(inner).count(), 0);
    doc.undo().unwrap();
    assert!(doc.project().wall(inner).is_some());
}

#[test]
fn flip_keeps_openings_in_place() {
    let (mut doc, mut ed) = house();
    let bottom = wall_with(&doc, Point2::new(6.0, 0.0), Point2::new(0.0, 0.0));
    let id = doc
        .insert(lot_model::Element::Opening(Opening {
            host: bottom,
            offset: 1.0,
            width: 1.5,
            height: 1.3,
            sill_height: 0.9,
        }))
        .unwrap();
    ed.selection = vec![bottom];
    ed.run(&mut doc, &Command::FlipWalls);
    let w = doc.project().wall(bottom).unwrap();
    assert_eq!(w.start, Point2::new(0.0, 0.0));
    let o = match &doc.project().elements[&id] {
        lot_model::Element::Opening(o) => o.clone(),
        _ => unreachable!(),
    };
    // Vorher 1,0 bis 2,5 m vom Ostende, jetzt 3,5 bis 5,0 m vom Westende: dieselbe Stelle.
    assert!((o.offset - 3.5).abs() < 1e-9);
}

#[test]
fn wall_type_command_changes_selection() {
    let (mut doc, mut ed) = house();
    let inner = wall_with(&doc, Point2::new(3.0, 0.0), Point2::new(3.0, 4.0));
    ed.selection = vec![inner];
    let r = ed.run(&mut doc, &Command::SetWallType(12));
    assert!(r.model_changed);
    assert_eq!(doc.project().wall(inner).unwrap().wall_type, ElementId(12));
}

#[test]
fn commands_list_includes_types_and_storeys() {
    let (doc, ed) = setup();
    let list = ed.commands(doc.project());
    assert!(list.iter().any(|c| c.title.starts_with("Außenwand WDVS 36")));
    assert!(list.iter().any(|c| c.title == "Geschoss EG ✓"));
    for c in &list {
        assert!(Command::parse(&c.id).is_some(), "{}", c.id);
    }
}

#[test]
fn add_storey_goes_on_top() {
    let (mut doc, mut ed) = setup();
    ed.run(&mut doc, &Command::AddStorey);
    let storeys = doc.project().storeys();
    assert_eq!(storeys.len(), 2);
    assert_eq!(storeys[1].1.elevation, 2.75);
    assert_eq!(ed.storey, storeys[1].0);
}
