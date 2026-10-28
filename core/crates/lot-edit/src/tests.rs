use lot_model::{Document, ElementId, Opening, Point2, Project};

use super::*;

fn input(x: f64, y: f64) -> Input {
    Input { point: Point2::new(x, y), tolerance: 0.15, shift: false, alt: false }
}

fn setup() -> (Document, Editor) {
    let doc = Document::new(Project::template("Test"));
    let editor = Editor::new(doc.project());
    (doc, editor)
}

fn click(doc: &mut Document, editor: &mut Editor, x: f64, y: f64) -> Response {
    editor.pointer_moved(doc.project(), input(x, y));
    let r = editor.pointer_down(doc, input(x, y), 1);
    let up = editor.pointer_up(doc);
    Response {
        redraw: r.redraw || up.redraw,
        model_changed: r.model_changed || up.model_changed,
        error: r.error.or(up.error),
    }
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
    let s = snap::snap(p, ed.storey, Point2::new(2.0, 0.08), 0.15, None, false, &[]);
    assert_eq!(s.kind, SnapKind::OnWall);
    // Nahe der Mitte gewinnt die Mitte.
    let s = snap::snap(p, ed.storey, Point2::new(2.47, 0.04), 0.15, None, false, &[]);
    assert_eq!((s.kind, s.point), (SnapKind::Midpoint, Point2::new(2.5, 0.0)));
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
    // Obere Wand wählen, greifen und um 1 m nach oben ziehen.
    click(&mut doc, &mut ed, 1.5, 4.0);
    assert_eq!(ed.selection, vec![top]);
    ed.pointer_down(&mut doc, input(1.5, 4.0), 1);
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
    click(&mut doc, &mut ed, 0.0, 2.0);
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
        phase: Default::default(),
        host: inner,
        offset: 1.0,
        width: 1.0,
        height: 2.1,
        sill_height: 0.0,
        kind: lot_model::OpeningKind::Plain,
    }))
    .unwrap();
    // Neben der Öffnung klicken; in der Öffnung wäre die Öffnung gewählt.
    ed.pointer_down(&mut doc, input(3.0, 3.0), 1);
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
            phase: Default::default(),
            host: bottom,
            offset: 1.0,
            width: 1.5,
            height: 1.3,
            sill_height: 0.9,
            kind: lot_model::OpeningKind::Plain,
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

#[test]
fn grid_tool_labels_numbers_and_letters() {
    let (mut doc, mut ed) = setup();
    ed.run(&mut doc, &Command::GridTool);
    click(&mut doc, &mut ed, 0.0, -1.0);
    click(&mut doc, &mut ed, 0.0, 6.0);
    click(&mut doc, &mut ed, 5.0, -1.0);
    click(&mut doc, &mut ed, 5.0, 6.0);
    click(&mut doc, &mut ed, -1.0, 0.0);
    click(&mut doc, &mut ed, 6.0, 0.0);
    let mut labels: Vec<String> = doc.project().grid_lines().map(|(_, g)| g.label.clone()).collect();
    labels.sort();
    assert_eq!(labels, vec!["1", "2", "A"]);
}

#[test]
fn column_tool_places_and_moves_column() {
    let (mut doc, mut ed) = setup();
    ed.run(&mut doc, &Command::ColumnTool);
    click(&mut doc, &mut ed, 2.0, 2.0);
    let (id, c) = doc.project().columns_on(ed.storey).next().map(|(id, c)| (id, c.clone())).unwrap();
    assert_eq!(c.position, Point2::new(2.0, 2.0));
    ed.key(&mut doc, Key::Escape);
    assert_eq!(ed.tool, Tool::Select);
    // Stütze wählen, greifen und verschieben.
    click(&mut doc, &mut ed, 2.1, 2.0);
    assert_eq!(ed.selection, vec![id]);
    ed.pointer_down(&mut doc, input(2.1, 2.0), 1);
    ed.pointer_dragged(doc.project(), input(3.1, 2.0));
    ed.pointer_up(&mut doc);
    let moved = doc.project().columns_on(ed.storey).next().unwrap().1.position;
    assert_eq!(moved, Point2::new(3.0, 2.0));
    // Löschen.
    assert!(ed.key(&mut doc, Key::Delete).model_changed);
    assert_eq!(doc.project().columns_on(ed.storey).count(), 0);
}

#[test]
fn arc_wall_by_three_clicks() {
    let (mut doc, mut ed) = setup();
    ed.run(&mut doc, &Command::ArcWallTool);
    click(&mut doc, &mut ed, 0.0, 0.0);
    click(&mut doc, &mut ed, 4.0, 0.0);
    assert!(walls(&doc).is_empty(), "nach zwei Klicks fehlt noch der Stich");
    // Der Zeiger 0,8 m über der Sehne: Stich 0,80 m nach links.
    click(&mut doc, &mut ed, 1.0, 0.803);
    let w = &walls(&doc)[0];
    assert!((w.sagitta - 0.8).abs() < 1e-9);
    assert_eq!(w.end, Point2::new(4.0, 0.0));
    // Die nächste Bogenwand beginnt am Ende der ersten.
    click(&mut doc, &mut ed, 4.0, -4.0);
    click(&mut doc, &mut ed, 6.0, -2.0);
    assert_eq!(walls(&doc)[1].start, Point2::new(4.0, 0.0));
}

#[test]
fn typed_sagitta_takes_the_pointer_side() {
    let (mut doc, mut ed) = setup();
    ed.run(&mut doc, &Command::ArcWallTool);
    click(&mut doc, &mut ed, 0.0, 0.0);
    click(&mut doc, &mut ed, 4.0, 0.0);
    ed.pointer_moved(doc.project(), input(2.0, -0.3));
    for c in "1,2".chars() {
        ed.key(&mut doc, Key::Char(c));
    }
    ed.key(&mut doc, Key::Enter);
    assert!((walls(&doc)[0].sagitta + 1.2).abs() < 1e-9);
}

#[test]
fn sagitta_is_at_most_a_semicircle() {
    let (mut doc, mut ed) = setup();
    ed.run(&mut doc, &Command::ArcWallTool);
    click(&mut doc, &mut ed, 0.0, 0.0);
    click(&mut doc, &mut ed, 2.0, 0.0);
    click(&mut doc, &mut ed, 1.0, 5.0);
    assert!((walls(&doc)[0].sagitta - 1.0).abs() < 1e-9);
}

#[test]
fn flip_keeps_the_arc_in_place() {
    let (mut doc, mut ed) = setup();
    ed.run(&mut doc, &Command::ArcWallTool);
    for (x, y) in [(0.0, 0.0), (4.0, 0.0), (2.0, 1.0)] {
        click(&mut doc, &mut ed, x, y);
    }
    ed.key(&mut doc, Key::Escape);
    let before = walls(&doc)[0].clone();
    ed.run(&mut doc, &Command::SelectAll);
    ed.run(&mut doc, &Command::FlipWalls);
    let after = &walls(&doc)[0];
    let mid = |w: &lot_model::Wall| w.point_at(w.length() / 2.0, 0.0);
    assert!(mid(&before).distance(mid(after)) < 1e-9);
    assert_eq!(after.start, before.end);
}

#[test]
fn snaps_onto_the_arc() {
    let (mut doc, mut ed) = setup();
    ed.run(&mut doc, &Command::ArcWallTool);
    for (x, y) in [(0.0, 0.0), (4.0, 0.0), (2.0, 1.0)] {
        click(&mut doc, &mut ed, x, y);
    }
    ed.key(&mut doc, Key::Escape);
    ed.key(&mut doc, Key::Escape);
    // Neben der Bogenmitte rastet der Zeiger auf dem Bogen ein, nicht auf der Sehne.
    let s = snap::snap(doc.project(), ed.storey, Point2::new(2.0, 1.08), 0.15, None, false, &[]);
    assert_eq!(s.kind, SnapKind::OnWall);
    assert!(s.point.distance(Point2::new(2.0, 1.0)) < 1e-6, "{:?}", s.point);
    // Mit Richtung: senkrecht von unten auf den Bogen.
    let s = snap::snap(doc.project(), ed.storey, Point2::new(1.0, 0.8), 0.15, Some(Point2::new(1.0, -2.0)), false, &[]);
    let w = &walls(&doc)[0];
    assert!(w.locate(s.point).1.abs() < 1e-6 && (s.point.x - 1.0).abs() < 1e-6, "{:?}", s.point);
}

/// Zeichnet ein Rechteck 6 × 4 m und eine Trennwand bei x = 2,5.
fn two_spaces(doc: &mut Document, ed: &mut Editor) {
    ed.run(doc, &Command::WallTool);
    for (x, y) in [(0.0, 0.0), (6.0, 0.0), (6.0, 4.0), (0.0, 4.0), (0.0, 0.0)] {
        click(doc, ed, x, y);
    }
    ed.key(doc, Key::Escape);
    for (x, y) in [(2.5, 0.0), (2.5, 4.0)] {
        click(doc, ed, x, y);
    }
    ed.key(doc, Key::Escape);
}

fn spaces(doc: &Document) -> Vec<lot_model::Space> {
    doc.project().spaces_on(ElementId(1)).map(|(_, r)| r.clone()).collect()
}

#[test]
fn space_tool_numbers_spaces_per_storey() {
    let (mut doc, mut ed) = setup();
    two_spaces(&mut doc, &mut ed);
    ed.run(&mut doc, &Command::SpaceTool);
    click(&mut doc, &mut ed, 1.2, 2.0);
    click(&mut doc, &mut ed, 4.0, 2.0);
    let r = spaces(&doc);
    assert_eq!(r.iter().map(|r| r.number.as_str()).collect::<Vec<_>>(), ["0.01", "0.02"]);
    assert!(r.iter().all(|r| r.living_space));
}

#[test]
fn detect_spaces_fills_every_enclosed_area_once() {
    let (mut doc, mut ed) = setup();
    two_spaces(&mut doc, &mut ed);
    ed.run(&mut doc, &Command::SpaceTool);
    click(&mut doc, &mut ed, 1.2, 2.0);
    let response = ed.run(&mut doc, &Command::DetectSpaces);
    assert!(response.model_changed);
    let r = spaces(&doc);
    assert_eq!(r.len(), 2);
    assert_eq!(r[1].number, "0.02");
    assert!(r[1].position.x > 2.6, "neuer Raum im rechten Bereich: {:?}", r[1].position);
    // Ein zweites Mal gibt es nichts mehr zu tun, und ein Undo nimmt alle neuen Räume zurück.
    assert!(ed.run(&mut doc, &Command::DetectSpaces).error.is_some());
    doc.undo().unwrap();
    assert_eq!(spaces(&doc).len(), 1);
}

#[test]
fn space_tag_can_be_picked_and_moved() {
    let (mut doc, mut ed) = setup();
    two_spaces(&mut doc, &mut ed);
    ed.run(&mut doc, &Command::SpaceTool);
    click(&mut doc, &mut ed, 4.0, 2.0);
    ed.run(&mut doc, &Command::SelectTool);
    // Klick neben den Stempel wählt den Raum, Ziehen verschiebt ihn.
    click(&mut doc, &mut ed, 4.5, 2.2);
    assert_eq!(ed.selection.len(), 1);
    ed.pointer_down(&mut doc, input(4.5, 2.2), 1);
    ed.pointer_dragged(doc.project(), input(4.5, 3.2));
    ed.pointer_up(&mut doc);
    assert_eq!(spaces(&doc)[0].position, Point2::new(4.0, 3.0));
    // Die Auswahl zeigt die Fläche des Raums.
    let overlay = ed.overlay(doc.project());
    let ring = &overlay.selected[0];
    let xs: Vec<f64> = ring.iter().map(|p| p[0]).collect();
    assert!(xs.iter().cloned().fold(f64::INFINITY, f64::min) > 2.5);
}

#[test]
fn cellar_spaces_are_not_living_space() {
    let (mut doc, mut ed) = setup();
    doc.insert(lot_model::Element::Storey(lot_model::Storey { name: "KG".into(), elevation: -2.5, height: 2.5 }))
        .unwrap();
    let kg = doc.project().storeys()[0].0;
    ed.run(&mut doc, &Command::SetStorey(kg.0));
    ed.run(&mut doc, &Command::SpaceTool);
    click(&mut doc, &mut ed, 1.0, 1.0);
    let r: Vec<_> = doc.project().spaces_on(kg).map(|(_, r)| r.clone()).collect();
    assert_eq!(r[0].number, "-1.01");
    assert!(!r[0].living_space);
}

/// Eine Wand von (0,0) nach (6,0), Außenwand WDVS mit Bezugslinie in der Mitte.
fn one_wall() -> (Document, Editor, ElementId) {
    let (mut doc, mut ed) = setup();
    ed.run(&mut doc, &Command::WallTool);
    click(&mut doc, &mut ed, 0.0, 0.0);
    click(&mut doc, &mut ed, 6.0, 0.0);
    ed.key(&mut doc, Key::Escape);
    let wall = doc.project().walls().next().unwrap().0;
    (doc, ed, wall)
}

fn openings(doc: &Document, wall: ElementId) -> Vec<Opening> {
    doc.project().openings_in(wall).map(|(_, o)| o.clone()).collect()
}

#[test]
fn door_opens_to_the_clicked_side_with_hinges_near_the_closer_end() {
    let (mut doc, mut ed, wall) = one_wall();
    ed.run(&mut doc, &Command::DoorTool);
    // Klick rechts (südlich) der Wand, nahe dem Anfang.
    click(&mut doc, &mut ed, 1.5, -0.1);
    let o = &openings(&doc, wall)[0];
    assert!((o.offset - 1.0).abs() < 1e-9 && (o.width - 1.01).abs() < 1e-9, "mittig zum Klick: {o:?}");
    let lot_model::OpeningKind::Door(d) = o.kind else { panic!("keine Tür") };
    assert_eq!(d.opens_to, lot_model::WallSide::Right);
    assert!(!d.hinge_at_end(), "Bänder am Wandanfang");
}

#[test]
fn window_tool_places_window_and_rejects_overlap() {
    let (mut doc, mut ed, wall) = one_wall();
    ed.run(&mut doc, &Command::WindowTool);
    click(&mut doc, &mut ed, 4.0, 0.05);
    let o = &openings(&doc, wall)[0];
    assert_eq!(o.kind, lot_model::OpeningKind::Window);
    assert!((o.sill_height - 0.885).abs() < 1e-9);
    // Ein zweites Fenster an derselben Stelle überschneidet sich.
    let r = click(&mut doc, &mut ed, 4.2, 0.05);
    assert!(r.error.is_some());
    assert_eq!(openings(&doc, wall).len(), 1);
}

#[test]
fn opening_slides_along_its_wall() {
    let (mut doc, mut ed, wall) = one_wall();
    ed.run(&mut doc, &Command::WindowTool);
    click(&mut doc, &mut ed, 2.0, 0.0);
    ed.run(&mut doc, &Command::SelectTool);
    click(&mut doc, &mut ed, 2.0, 0.0);
    assert_eq!(ed.selection.len(), 1, "Klick in die Öffnung wählt die Öffnung");
    ed.pointer_down(&mut doc, input(2.0, 0.0), 1);
    assert!(matches!(doc.project().elements[&ed.selection[0]], lot_model::Element::Opening(_)));
    // Schräg ziehen: nur der Anteil entlang der Wand zählt, die Wand bleibt.
    ed.pointer_dragged(doc.project(), input(3.0, 0.7));
    ed.pointer_up(&mut doc);
    assert!((openings(&doc, wall)[0].offset - 2.37).abs() < 1e-9, "{:?}", openings(&doc, wall));
    assert_eq!(walls(&doc)[0].start, Point2::new(0.0, 0.0));
}

#[test]
fn door_swing_and_hinge_can_be_flipped() {
    let (mut doc, mut ed, wall) = one_wall();
    ed.run(&mut doc, &Command::DoorTool);
    click(&mut doc, &mut ed, 4.5, 0.1);
    let before = match openings(&doc, wall)[0].kind {
        lot_model::OpeningKind::Door(d) => d,
        _ => panic!(),
    };
    ed.run(&mut doc, &Command::SelectAll);
    ed.selection = doc.project().openings_in(wall).map(|(id, _)| id).collect();
    ed.run(&mut doc, &Command::FlipDoorSwing);
    let after = match openings(&doc, wall)[0].kind {
        lot_model::OpeningKind::Door(d) => d,
        _ => panic!(),
    };
    assert_eq!(after.opens_to, before.opens_to.other());
    assert_eq!(after.hinge_at_end(), before.hinge_at_end(), "die Bänder bleiben am selben Ende");
    ed.run(&mut doc, &Command::FlipDoorHinge);
    let last = match openings(&doc, wall)[0].kind {
        lot_model::OpeningKind::Door(d) => d,
        _ => panic!(),
    };
    assert_ne!(last.hinge_at_end(), before.hinge_at_end());
}

#[test]
fn dimension_chain_follows_the_wall() {
    let (mut doc, mut ed, wall) = one_wall();
    ed.run(&mut doc, &Command::WindowTool);
    click(&mut doc, &mut ed, 2.0, 0.0);
    ed.run(&mut doc, &Command::SelectTool);
    ed.selection = vec![wall];
    let r = ed.run(&mut doc, &Command::DimensionWalls);
    assert!(r.model_changed, "{:?}", r.error);
    let dims: Vec<_> = doc.project().dimensions_on(ElementId(1)).map(|(_, d)| d.clone()).collect();
    assert_eq!(dims.len(), 2, "Öffnungskette und Gesamtmaß");
    let measures = |doc: &Document, d: &lot_model::DimensionChain| -> Vec<f64> {
        let p = lot_geom::dimensions::resolve(doc.project(), d);
        p.windows(2).map(|w| ((w[1].along - w[0].along) * 1000.0).round() / 1000.0).collect()
    };
    assert_eq!(measures(&doc, &dims[0]), vec![1.37, 1.26, 3.37]);
    assert_eq!(measures(&doc, &dims[1]), vec![6.0]);
    // Assoziativ: Die Wand wird länger, die Kette rechnet mit.
    doc.set_param(wall, lot_model::Param::WallLength, 7.0).unwrap();
    assert_eq!(measures(&doc, &dims[1]), vec![7.0]);
    assert_eq!(measures(&doc, &dims[0]), vec![1.37, 1.26, 4.37]);
}

#[test]
fn slab_from_walls_rests_on_the_storey_below() {
    let (mut doc, mut ed) = setup();
    two_spaces(&mut doc, &mut ed);
    // Im untersten Geschoss: Bodenplatte.
    ed.run(&mut doc, &Command::SlabFromWalls);
    let base: Vec<_> = doc.project().slabs_on(ElementId(1)).map(|(_, s)| s.clone()).collect();
    assert_eq!(base.len(), 1);
    assert_eq!(base[0].kind, lot_model::SlabKind::Base);
    // Darüber: Decke auf den Wänden darunter, auch ohne eigene Wände.
    ed.run(&mut doc, &Command::AddStorey);
    let r = ed.run(&mut doc, &Command::SlabFromWalls);
    assert!(r.model_changed, "{:?}", r.error);
    let upper: Vec<_> = doc.project().slabs_on(ed.storey).map(|(_, s)| s.clone()).collect();
    assert_eq!(upper[0].kind, lot_model::SlabKind::Floor);
    assert_eq!(upper[0].outline.len(), 4);
}

#[test]
fn slab_opening_needs_a_slab_and_cuts_it() {
    let (mut doc, mut ed) = setup();
    two_spaces(&mut doc, &mut ed);
    ed.run(&mut doc, &Command::SlabOpeningTool);
    click(&mut doc, &mut ed, 3.0, 1.0);
    let r = click(&mut doc, &mut ed, 4.2, 3.0);
    assert!(r.error.is_some(), "ohne Decke kein Durchbruch");
    ed.run(&mut doc, &Command::SlabFromWalls);
    ed.run(&mut doc, &Command::SlabOpeningTool);
    click(&mut doc, &mut ed, 3.0, 1.0);
    let r = click(&mut doc, &mut ed, 4.2, 3.0);
    assert!(r.model_changed, "{:?}", r.error);
    let (slab_id, slab) = doc.project().slabs_on(ElementId(1)).next().map(|(id, s)| (id, s.clone())).unwrap();
    let full = lot_geom::Area::from_ring(&slab.outline.iter().map(|p| [p.x, p.y]).collect::<Vec<_>>()).area();
    let cut = lot_geom::slabs::slab_area(doc.project(), slab_id, &slab).area();
    assert!((full - cut - 1.2 * 2.0).abs() < 1e-6, "{full} - {cut}");
}

#[test]
fn gable_roof_covers_the_walls_and_raises_them() {
    let (mut doc, mut ed) = setup();
    // 10 × 6 m: Traufen an den langen Seiten.
    ed.run(&mut doc, &Command::WallTool);
    for (x, y) in [(0.0, 0.0), (0.0, 6.0), (10.0, 6.0), (10.0, 0.0), (0.0, 0.0)] {
        click(&mut doc, &mut ed, x, y);
    }
    ed.key(&mut doc, Key::Escape);
    let r = ed.run(&mut doc, &Command::GableRoofFromWalls);
    assert!(r.model_changed, "{:?}", r.error);
    let roof = doc.project().roofs_on(ElementId(1)).next().unwrap().1.clone();
    let eaves = roof.edges.iter().filter(|e| matches!(e, lot_model::RoofEdge::Eave { .. })).count();
    assert_eq!(eaves, 2);
    assert!(walls(&doc).iter().all(|w| matches!(w.top, lot_model::Top::Roof { .. })));
    // Ein Undo nimmt Dach und Wandhöhen zusammen zurück.
    doc.undo().unwrap();
    assert!(doc.project().roofs_on(ElementId(1)).next().is_none());
    assert!(walls(&doc).iter().all(|w| matches!(w.top, lot_model::Top::Storey { .. })));
}

fn annotations(doc: &Document) -> Vec<lot_model::Annotation> {
    doc.project()
        .elements
        .values()
        .filter_map(|e| match e {
            lot_model::Element::Annotation(a) => Some(a.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn line_tool_chains_segments_and_takes_typed_length() {
    let (mut doc, mut ed) = setup();
    ed.run(&mut doc, &Command::LineTool);
    click(&mut doc, &mut ed, 0.0, 0.0);
    click(&mut doc, &mut ed, 2.0, 0.0);
    ed.pointer_moved(doc.project(), input(2.0, 3.0));
    for c in "1,5".chars() {
        ed.key(&mut doc, Key::Char(c));
    }
    ed.key(&mut doc, Key::Enter);
    ed.key(&mut doc, Key::Escape);
    let a = annotations(&doc);
    assert_eq!(a.len(), 2);
    assert_eq!(a[1].shape, lot_model::AnnotationShape::Polyline(vec![Point2::new(2.0, 0.0), Point2::new(2.0, 1.5)]));
    assert_eq!(a[0].view, lot_model::AnnotationView::FloorPlan { storey: ed.storey });
}

#[test]
fn text_tool_places_and_selects_text_which_moves_with_the_selection() {
    let (mut doc, mut ed) = setup();
    ed.run(&mut doc, &Command::TextTool);
    click(&mut doc, &mut ed, 1.0, 1.0);
    assert_eq!(ed.tool, Tool::Select);
    assert_eq!(ed.selection.len(), 1);
    let id = ed.selection[0];
    // Klick in den Text wählt ihn wieder.
    ed.key(&mut doc, Key::Escape);
    click(&mut doc, &mut ed, 1.1, 1.1);
    assert_eq!(ed.selection, vec![id]);
    // Ziehen verschiebt ihn.
    ed.pointer_down(&mut doc, input(1.1, 1.1), 1);
    ed.pointer_dragged(doc.project(), input(2.1, 1.1));
    ed.pointer_up(&mut doc);
    match &annotations(&doc)[0].shape {
        lot_model::AnnotationShape::Text { position, text, .. } => {
            assert_eq!(text, "Text");
            assert!((position.x - 2.0).abs() < 1e-9 && (position.y - 1.0).abs() < 1e-9, "{position:?}");
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn set_phase_marks_selected_walls_and_undoes_in_one_step() {
    let (mut doc, mut ed) = setup();
    ed.run(&mut doc, &Command::WallTool);
    for (x, y) in [(0.0, 0.0), (4.0, 0.0), (4.0, 3.0)] {
        click(&mut doc, &mut ed, x, y);
    }
    ed.key(&mut doc, Key::Escape);
    ed.run(&mut doc, &Command::SelectAll);
    ed.run(&mut doc, &Command::SetPhase(lot_model::Phase::Demolished));
    assert!(walls(&doc).iter().all(|w| w.phase == lot_model::Phase::Demolished));
    assert!(doc.project().has_phases());
    assert!(doc.undo().unwrap());
    assert!(walls(&doc).iter().all(|w| w.phase == lot_model::Phase::New));
    // Die Kennung ist lesbar für Befehlspalette und KI.
    assert_eq!(Command::SetPhase(lot_model::Phase::Existing).id(), "SetPhase(Existing)");
}

fn with_underlay(doc: &mut Document, storey: ElementId) -> ElementId {
    doc.insert(lot_model::Element::Underlay(lot_model::Underlay {
        storey,
        source: "bestand.pdf".into(),
        page: 0,
        origin: Point2::new(0.0, 0.0),
        scale: lot_model::pdf_point_scale(100.0),
        width: 842.0,
        height: 595.0,
        rotation: 0.0,
        opacity: 0.45,
    }))
    .unwrap()
}

#[test]
fn calibrating_the_underlay_stretches_it_about_the_first_point() {
    let (mut doc, mut ed) = setup();
    let id = with_underlay(&mut doc, ed.storey);
    ed.run(&mut doc, &Command::CalibrateUnderlayTool);
    click(&mut doc, &mut ed, 2.0, 1.0);
    click(&mut doc, &mut ed, 6.0, 1.0);
    for c in "5,00".chars() {
        ed.key(&mut doc, Key::Char(c));
    }
    ed.key(&mut doc, Key::Enter);
    let Some(lot_model::Element::Underlay(u)) = doc.project().elements.get(&id) else { panic!() };
    // 4 m gemessen, 5 m wahr: alles um den Faktor 1,25 größer, (2 | 1) bleibt.
    assert!((u.scale - lot_model::pdf_point_scale(100.0) * 1.25).abs() < 1e-12);
    let (s, t) = u.to_source(Point2::new(2.0, 1.0));
    assert!((u.to_model(s, t).distance(Point2::new(2.0, 1.0))) < 1e-9);
    assert!((u.origin.x - (2.0 - 2.0 * 1.25)).abs() < 1e-9);
    // Ein Undo-Schritt.
    assert!(doc.undo().unwrap());
}

#[test]
fn aligning_the_underlay_levels_a_skewed_line() {
    let (mut doc, mut ed) = setup();
    let id = with_underlay(&mut doc, ed.storey);
    ed.run(&mut doc, &Command::AlignUnderlayTool);
    click(&mut doc, &mut ed, 1.0, 1.0);
    click(&mut doc, &mut ed, 9.0, 1.1);
    let Some(lot_model::Element::Underlay(u)) = doc.project().elements.get(&id) else { panic!() };
    assert!((u.rotation + (0.1f64).atan2(8.0).to_degrees()).abs() < 1e-9, "{}", u.rotation);
}

#[test]
fn variants_hide_the_other_variant_and_switch() {
    let (mut doc, mut ed) = setup();
    ed.run(&mut doc, &Command::WallTool);
    click(&mut doc, &mut ed, 0.0, 0.0);
    click(&mut doc, &mut ed, 4.0, 0.0);
    ed.key(&mut doc, Key::Escape);
    let wall = doc.project().walls().next().map(|(id, _)| id).unwrap();
    ed.run(&mut doc, &Command::NewVariantSet);
    let set = doc.project().variant_sets().next().map(|(id, _)| id).unwrap();
    // Wand in Variante A, dann B zeigen: Die Wand ist weg und nicht mehr zu treffen.
    ed.run(&mut doc, &Command::SelectTool);
    ed.selection = vec![wall];
    ed.run(&mut doc, &Command::AddSelectionToVariant(set.0));
    assert!(doc.project().variant_view().wall(wall).is_some());
    ed.run(&mut doc, &Command::SwitchVariant(set.0));
    assert!(doc.project().variant_view().wall(wall).is_none());
    assert!(doc.project().wall(wall).is_some(), "im Modell bleibt sie");
    ed.pointer_down(&mut doc, input(2.0, 0.0), 1);
    assert!(ed.selection.is_empty());
    // Zurück zu A: wieder da.
    ed.run(&mut doc, &Command::SwitchVariant(set.0));
    assert!(doc.project().variant_view().wall(wall).is_some());
    assert!(ed.commands(doc.project()).iter().any(|c| c.title.contains("Variante B zeigen")));
}

fn drag(doc: &mut Document, ed: &mut Editor, from: (f64, f64), to: (f64, f64), shift: bool, alt: bool) {
    let i = |x: f64, y: f64| Input { point: Point2::new(x, y), tolerance: 0.15, shift, alt };
    ed.pointer_moved(doc.project(), i(from.0, from.1));
    ed.pointer_down(doc, i(from.0, from.1), 1);
    ed.pointer_dragged(doc.project(), i((from.0 + to.0) / 2.0, (from.1 + to.1) / 2.0));
    ed.pointer_dragged(doc.project(), i(to.0, to.1));
    ed.pointer_up(doc);
}

#[test]
fn frame_to_the_right_takes_whole_walls_to_the_left_also_crossed_ones() {
    let (mut doc, mut ed) = house();
    let inner = wall_with(&doc, Point2::new(3.0, 0.0), Point2::new(3.0, 4.0));
    let left = wall_with(&doc, Point2::new(0.0, 0.0), Point2::new(0.0, 4.0));
    // Von links nach rechts um die linke Wand: nur sie liegt ganz innen.
    drag(&mut doc, &mut ed, (-1.0, -1.0), (1.0, 5.0), false, false);
    assert_eq!(ed.selection, vec![left]);
    // Von rechts nach links über die Innenwand: auch die angeschnittenen Außenwände oben und unten.
    drag(&mut doc, &mut ed, (3.5, 3.0), (2.5, 1.0), false, false);
    assert_eq!(ed.selection, vec![inner]);
    drag(&mut doc, &mut ed, (3.5, 4.5), (2.5, -0.5), false, false);
    assert_eq!(ed.selection.len(), 3);
    // ⌘← holt die vorige Auswahl zurück.
    ed.run(&mut doc, &Command::PreviousSelection);
    assert_eq!(ed.selection, vec![inner]);
}

#[test]
fn dragging_an_unselected_wall_draws_a_frame_and_moves_nothing() {
    let (mut doc, mut ed) = house();
    let before = walls(&doc);
    drag(&mut doc, &mut ed, (1.5, 4.0), (1.5, 5.0), false, false);
    assert_eq!(walls(&doc), before);
    assert!(ed.selection.is_empty(), "der Rahmen umschließt keine Wand ganz");
}

#[test]
fn shift_click_adds_and_removes() {
    let (mut doc, mut ed) = house();
    let top = wall_with(&doc, Point2::new(0.0, 4.0), Point2::new(6.0, 4.0));
    let left = wall_with(&doc, Point2::new(0.0, 0.0), Point2::new(0.0, 4.0));
    click(&mut doc, &mut ed, 1.5, 4.0);
    let s = |x: f64, y: f64| Input { point: Point2::new(x, y), tolerance: 0.15, shift: true, alt: false };
    ed.pointer_moved(doc.project(), s(0.0, 2.0));
    ed.pointer_down(&mut doc, s(0.0, 2.0), 1);
    ed.pointer_up(&mut doc);
    assert_eq!(ed.selection, vec![top, left]);
    ed.pointer_moved(doc.project(), s(1.5, 4.0));
    ed.pointer_down(&mut doc, s(1.5, 4.0), 1);
    ed.pointer_up(&mut doc);
    assert_eq!(ed.selection, vec![left]);
}

#[test]
fn tab_cycles_to_the_wall_chain() {
    let (mut doc, mut ed) = house();
    ed.pointer_moved(doc.project(), input(1.5, 4.0));
    let tip = ed.overlay(doc.project()).tip.unwrap().1;
    assert!(tip.starts_with("1 von 2: Wand"), "{tip}");
    ed.key(&mut doc, Key::Tab);
    let tip = ed.overlay(doc.project()).tip.unwrap().1;
    assert!(tip.starts_with("2 von 2: Wandkette (4 Wände)"), "{tip}");
    // Die Innenwand stößt nur stumpf an und gehört nicht zur Kette.
    ed.pointer_down(&mut doc, input(1.5, 4.0), 1);
    ed.pointer_up(&mut doc);
    assert_eq!(ed.selection.len(), 4);
}

#[test]
fn select_similar_takes_the_storey_then_the_project() {
    let (mut doc, mut ed) = house();
    ed.run(&mut doc, &Command::AddStorey);
    ed.run(&mut doc, &Command::WallTool);
    click(&mut doc, &mut ed, 0.0, 0.0);
    click(&mut doc, &mut ed, 5.0, 0.0);
    ed.key(&mut doc, Key::Escape);
    ed.key(&mut doc, Key::Escape);
    let ground = doc.project().storeys()[0].0.0;
    ed.run(&mut doc, &Command::SetStorey(ground));
    click(&mut doc, &mut ed, 1.5, 4.0);
    for c in "sa".chars() {
        ed.key(&mut doc, Key::Char(c));
    }
    assert_eq!(ed.selection.len(), 5, "alle Wände des Geschosses");
    for c in "SA".chars() {
        ed.key(&mut doc, Key::Char(c));
    }
    assert_eq!(ed.selection.len(), 6, "dazu die Wand im Obergeschoss");
}

#[test]
fn two_letter_shortcuts_switch_tools_and_enter_repeats() {
    let (mut doc, mut ed) = setup();
    ed.key(&mut doc, Key::Char('w'));
    assert_eq!(ed.tool, Tool::Select);
    assert!(ed.hint().starts_with("Kürzel W…"), "{}", ed.hint());
    ed.key(&mut doc, Key::Char('n'));
    assert_eq!(ed.tool, Tool::Window);
    ed.key(&mut doc, Key::Escape);
    assert_eq!(ed.tool, Tool::Select);
    ed.key(&mut doc, Key::Enter);
    assert_eq!(ed.tool, Tool::Window, "⏎ wiederholt den letzten Befehl");
    // Ein falscher Buchstabe beginnt neu.
    ed.key(&mut doc, Key::Escape);
    for c in "qwa".chars() {
        ed.key(&mut doc, Key::Char(c));
    }
    assert_eq!(ed.tool, Tool::Wall);
}

#[test]
fn arrows_nudge_the_selection() {
    let (mut doc, mut ed) = house();
    let left = wall_with(&doc, Point2::new(0.0, 0.0), Point2::new(0.0, 4.0));
    click(&mut doc, &mut ed, 0.0, 2.0);
    ed.key(&mut doc, Key::Arrow { dx: 1, dy: 0, large: false });
    ed.key(&mut doc, Key::Arrow { dx: 1, dy: 0, large: true });
    let w = doc.project().wall(left).unwrap();
    assert!((w.start.x - 0.11).abs() < 1e-9, "{:?}", w.start);
}

#[test]
fn option_drag_copies_wall_with_its_window_and_selects_the_copy() {
    let (mut doc, mut ed, wall) = one_wall();
    ed.run(&mut doc, &Command::WindowTool);
    click(&mut doc, &mut ed, 2.0, 0.0);
    ed.run(&mut doc, &Command::SelectTool);
    click(&mut doc, &mut ed, 4.5, 0.0);
    assert_eq!(ed.selection, vec![wall]);
    drag(&mut doc, &mut ed, (4.5, 0.0), (4.5, 3.0), false, true);
    let p = doc.project();
    assert_eq!(p.walls().count(), 2);
    let copy = ed.selection[0];
    assert_ne!(copy, wall);
    assert_eq!(p.wall(copy).unwrap().start, Point2::new(0.0, 3.0));
    assert_eq!(p.openings_in(copy).count(), 1, "das Fenster kommt mit");
    assert_eq!(p.wall(wall).unwrap().start, Point2::new(0.0, 0.0), "das Original bleibt");
    // Ein Undo nimmt Wand und Fenster zurück.
    doc.undo().unwrap();
    assert_eq!(doc.project().walls().count(), 1);
}

#[test]
fn space_turns_a_door_through_four_positions() {
    let (mut doc, mut ed, wall) = one_wall();
    ed.run(&mut doc, &Command::DoorTool);
    click(&mut doc, &mut ed, 2.0, 0.3);
    ed.run(&mut doc, &Command::SelectTool);
    click(&mut doc, &mut ed, 2.0, 0.0);
    let door = |doc: &Document| match openings(doc, wall)[0].kind {
        OpeningKind::Door(d) => (d.opens_to, d.hinge_at_end()),
        _ => panic!(),
    };
    let first = door(&doc);
    let mut seen = vec![first];
    for _ in 0..3 {
        ed.key(&mut doc, Key::Char(' '));
        seen.push(door(&doc));
    }
    seen.sort_by_key(|(s, e)| (*s == WallSide::Left, *e));
    seen.dedup();
    assert_eq!(seen.len(), 4, "alle vier Lagen");
    ed.key(&mut doc, Key::Char(' '));
    assert_eq!(door(&doc), first, "nach vier Schritten wieder am Anfang");
}

#[test]
fn underlays_are_not_picked_but_can_be_selected_by_command() {
    let (mut doc, mut ed) = setup();
    let id = with_underlay(&mut doc, ed.storey);
    // Klick auf den Rand der Unterlage wählt nichts.
    click(&mut doc, &mut ed, 0.0, 1.0);
    assert!(ed.selection.is_empty());
    ed.run(&mut doc, &Command::SelectUnderlay);
    assert_eq!(ed.selection, vec![id]);
}

#[test]
fn mirrored_wall_keeps_layers_and_door_side_in_the_mirror() {
    let (mut doc, mut ed, wall) = one_wall();
    ed.run(&mut doc, &Command::DoorTool);
    click(&mut doc, &mut ed, 1.0, 0.3);
    let project = doc.project();
    let t = Transform::mirror(Point2::new(0.0, 2.0), Point2::new(1.0, 2.0)).unwrap();
    let ops = transform::copy_ops(project, &[wall], &t, None);
    let mut p = project.clone();
    for op in ops {
        if let Operation::Insert { id, element } = op {
            p.elements.insert(id, element);
        }
    }
    let (copy, w) = p.walls().find(|(id, _)| *id != wall).map(|(id, w)| (id, w.clone())).unwrap();
    // Gespiegelt an y = 2 und rückwärts gezeichnet: Die linke Seite zeigt weiter vom Spiegel weg.
    let orig = project.wall(wall).unwrap();
    assert_eq!((w.start, w.end), (Point2::new(orig.end.x, 4.0), Point2::new(orig.start.x, 4.0)));
    let (o, c) = (&openings(&doc, wall)[0], p.openings_in(copy).next().unwrap().1);
    // Die Tür liegt gespiegelt an derselben Stelle in x, ihr Abstand zählt vom anderen Ende.
    assert!((c.offset - (orig.length() - o.offset - o.width)).abs() < 1e-9);
    let (OpeningKind::Door(a), OpeningKind::Door(b)) = (o.kind, c.kind) else { panic!() };
    assert_eq!(a.opens_to, b.opens_to);
    assert_ne!(a.hinge, b.hinge);
}

fn typed(doc: &mut Document, ed: &mut Editor, text: &str) -> Response {
    let mut last = Response::default();
    for c in text.chars() {
        last = ed.key(doc, Key::Char(c));
    }
    let enter = ed.key(doc, Key::Enter);
    if enter.error.is_some() { enter } else { Response { error: last.error, ..enter } }
}

#[test]
fn typed_angle_and_relative_points_draw_walls() {
    let (mut doc, mut ed) = setup();
    ed.run(&mut doc, &Command::WallTool);
    click(&mut doc, &mut ed, 0.0, 0.0);
    typed(&mut doc, &mut ed, "4<90");
    typed(&mut doc, &mut ed, "@3;0");
    // Erst den Winkel, dann die Länge (in Revit nicht möglich).
    typed(&mut doc, &mut ed, "<-90");
    typed(&mut doc, &mut ed, "2*1,25+150cm");
    let w = walls(&doc);
    assert_eq!(w.len(), 3);
    assert!(w[0].end.distance(Point2::new(0.0, 4.0)) < 1e-9, "{:?}", w[0].end);
    assert!(w[1].end.distance(Point2::new(3.0, 4.0)) < 1e-9, "{:?}", w[1].end);
    assert!(w[2].end.distance(Point2::new(3.0, 0.0)) < 1e-9, "{:?}", w[2].end);
}

#[test]
fn move_by_typed_distance_keeps_corners() {
    let (mut doc, mut ed) = house();
    let top = wall_with(&doc, Point2::new(0.0, 4.0), Point2::new(6.0, 4.0));
    click(&mut doc, &mut ed, 1.5, 4.0);
    for c in "mv".chars() {
        ed.key(&mut doc, Key::Char(c));
    }
    assert_eq!(ed.tool, Tool::Move);
    click(&mut doc, &mut ed, 1.5, 4.0);
    ed.pointer_moved(doc.project(), input(1.5, 6.0));
    typed(&mut doc, &mut ed, "50cm");
    assert_eq!(ed.tool, Tool::Select);
    assert_eq!(doc.project().wall(top).unwrap().start.y, 4.5);
    assert!(walls(&doc).iter().any(|w| w.start == Point2::new(0.0, 0.0) && w.end == Point2::new(0.0, 4.5)));
}

#[test]
fn rotate_by_typed_angle_about_the_selection_center() {
    let (mut doc, mut ed, wall) = one_wall();
    ed.key(&mut doc, Key::Escape);
    click(&mut doc, &mut ed, 3.0, 0.0);
    ed.run(&mut doc, &Command::RotateTool);
    typed(&mut doc, &mut ed, "90");
    let w = doc.project().wall(wall).unwrap();
    assert!((w.start.x - w.end.x).abs() < 1e-9, "senkrecht: {w:?}");
    assert!((w.start.y + w.end.y).abs() < 0.2, "um die Mitte gedreht: {w:?}");
}

#[test]
fn mirror_copies_by_default() {
    let (mut doc, mut ed, wall) = one_wall();
    ed.key(&mut doc, Key::Escape);
    click(&mut doc, &mut ed, 3.0, 0.0);
    ed.run(&mut doc, &Command::MirrorTool);
    click(&mut doc, &mut ed, 0.0, 2.0);
    click(&mut doc, &mut ed, 5.0, 2.0);
    assert_eq!(walls(&doc).len(), 2);
    assert_eq!(doc.project().wall(wall).unwrap().start, Point2::new(0.0, 0.0));
    assert!(walls(&doc).iter().any(|w| (w.start.y - 4.0).abs() < 1e-9 && (w.end.y - 4.0).abs() < 1e-9));
}

#[test]
fn array_makes_series_counted_like_revit() {
    let (mut doc, mut ed) = setup();
    ed.run(&mut doc, &Command::ColumnTool);
    click(&mut doc, &mut ed, 0.0, 0.0);
    ed.key(&mut doc, Key::Escape);
    click(&mut doc, &mut ed, 0.0, 0.0);
    ed.run(&mut doc, &Command::ArrayTool);
    typed(&mut doc, &mut ed, "4x");
    click(&mut doc, &mut ed, 0.0, 0.0);
    ed.pointer_moved(doc.project(), input(5.0, 0.0));
    typed(&mut doc, &mut ed, "2,5");
    let mut xs: Vec<f64> = doc.project().columns_on(ed.storey).map(|(_, c)| c.position.x).collect();
    xs.sort_by(f64::total_cmp);
    assert_eq!(xs, vec![0.0, 2.5, 5.0, 7.5], "vier Stück einschließlich Original");
    assert!(doc.undo().unwrap());
    assert_eq!(doc.project().columns_on(ed.storey).count(), 1, "ein Undo-Schritt");
    // „3/“: drei Stück, das letzte am Punkt.
    click(&mut doc, &mut ed, 0.0, 0.0);
    ed.run(&mut doc, &Command::ArrayTool);
    typed(&mut doc, &mut ed, "3/");
    click(&mut doc, &mut ed, 0.0, 0.0);
    click(&mut doc, &mut ed, 6.0, 0.0);
    let mut xs: Vec<f64> = doc.project().columns_on(ed.storey).map(|(_, c)| c.position.x).collect();
    xs.sort_by(f64::total_cmp);
    assert_eq!(xs, vec![0.0, 3.0, 6.0]);
}

#[test]
fn split_keeps_openings_on_their_side() {
    let (mut doc, mut ed, wall) = one_wall();
    ed.run(&mut doc, &Command::WindowTool);
    click(&mut doc, &mut ed, 4.5, 0.0);
    ed.run(&mut doc, &Command::SplitTool);
    // Durch die Öffnung geht es nicht.
    assert!(click(&mut doc, &mut ed, 4.5, 0.0).error.is_some());
    click(&mut doc, &mut ed, 2.0, 0.05);
    let p = doc.project();
    assert_eq!(p.walls().count(), 2);
    assert_eq!(p.wall(wall).unwrap().end, Point2::new(2.0, 0.0));
    let (back, _) = p.walls().find(|(id, _)| *id != wall).unwrap();
    let o = p.openings_in(back).next().expect("das Fenster wandert mit dem hinteren Teil").1;
    assert!((o.offset - (4.5 - 0.63 - 2.0)).abs() < 1e-9, "{o:?}");
}

#[test]
fn trim_extends_two_walls_to_a_corner() {
    let (mut doc, mut ed) = setup();
    ed.run(&mut doc, &Command::WallTool);
    click(&mut doc, &mut ed, 0.0, 0.0);
    click(&mut doc, &mut ed, 4.0, 0.0);
    ed.key(&mut doc, Key::Escape);
    click(&mut doc, &mut ed, 5.0, 1.0);
    click(&mut doc, &mut ed, 5.0, 3.0);
    ed.key(&mut doc, Key::Escape);
    ed.run(&mut doc, &Command::TrimTool);
    click(&mut doc, &mut ed, 1.0, 0.0);
    click(&mut doc, &mut ed, 5.0, 2.0);
    let w = walls(&doc);
    assert_eq!(w[0].end, Point2::new(5.0, 0.0));
    assert_eq!(w[1].start, Point2::new(5.0, 0.0));
}

#[test]
fn offset_copies_a_wall_to_the_pointer_side() {
    let (mut doc, mut ed, _) = one_wall();
    ed.run(&mut doc, &Command::OffsetTool);
    assert!(click(&mut doc, &mut ed, 3.0, 0.05).error.is_some(), "erst der Abstand");
    typed(&mut doc, &mut ed, "3,0");
    click(&mut doc, &mut ed, 3.0, 0.05);
    let w = walls(&doc);
    assert_eq!(w.len(), 2);
    assert!((w[1].start.y - 3.0).abs() < 1e-9, "{:?}", w[1]);
}

#[test]
fn align_moves_a_wall_face_onto_a_grid_line() {
    let (mut doc, mut ed, wall) = one_wall();
    ed.run(&mut doc, &Command::GridTool);
    click(&mut doc, &mut ed, -1.0, 2.0);
    click(&mut doc, &mut ed, 8.0, 2.0);
    ed.key(&mut doc, Key::Escape);
    ed.run(&mut doc, &Command::AlignTool);
    click(&mut doc, &mut ed, 3.0, 2.0);
    // Die obere Fläche der Wand anklicken: Sie liegt danach auf der Achse.
    let (left, _) = geometry::faces(doc.project(), doc.project().wall(wall).unwrap());
    click(&mut doc, &mut ed, 3.0, left);
    let w = doc.project().wall(wall).unwrap();
    assert!((w.start.y + left - 2.0).abs() < 1e-9, "{w:?}");
}

#[test]
fn temp_dims_show_clear_widths_and_move_the_wall() {
    let (mut doc, mut ed) = house();
    let inner = wall_with(&doc, Point2::new(3.0, 0.0), Point2::new(3.0, 4.0));
    click(&mut doc, &mut ed, 3.0, 2.0);
    assert_eq!(ed.selection, vec![inner]);
    let dims = tempdims::temp_dims(doc.project(), ed.storey, inner);
    assert_eq!(dims.len(), 2, "lichte Weite zu beiden Seiten");
    let t = {
        let p = doc.project();
        let (l, r) = geometry::faces(p, p.wall(inner).unwrap());
        (l - r) / 2.0
    };
    // Außenwände in der Achse 0 und 6, alle Wände gleich dick: lichte Weite 3 - 2 · halbe Dicke.
    assert!(dims.iter().all(|d| (d.value - (3.0 - 2.0 * t)).abs() < 1e-9), "{dims:?}");
    // Zahl anklicken, neuen Wert tippen: Die Innenwand rückt, die Außenwände bleiben.
    let target = dims.iter().find(|d| d.a.x < 3.0 || d.b.x < 3.0).unwrap().middle();
    click(&mut doc, &mut ed, target.x, target.y);
    typed(&mut doc, &mut ed, "2,0");
    let w = doc.project().wall(inner).unwrap();
    let left_dim = tempdims::temp_dims(doc.project(), ed.storey, inner)
        .into_iter()
        .map(|d| (d.value, d.a.x.min(d.b.x)))
        .fold(f64::INFINITY, |m, (v, x)| if x < w.start.x { v.min(m) } else { m });
    assert!((left_dim - 2.0).abs() < 1e-9, "{left_dim} {w:?}");
    assert_eq!(ed.selection, vec![inner]);
}

#[test]
fn temp_dims_of_an_opening_reach_the_next_wall_face() {
    let (mut doc, mut ed) = house();
    let bottom = wall_with(&doc, Point2::new(0.0, 0.0), Point2::new(6.0, 0.0));
    ed.run(&mut doc, &Command::WindowTool);
    click(&mut doc, &mut ed, 1.5, 0.0);
    let (id, o) = doc.project().openings_in(bottom).next().map(|(id, o)| (id, o.clone())).unwrap();
    assert!((o.width - 1.26).abs() < 1e-9);
    let dims = tempdims::temp_dims(doc.project(), ed.storey, id);
    assert_eq!(dims.len(), 2);
    let (l, r) = {
        let p = doc.project();
        geometry::faces(p, p.wall(bottom).unwrap())
    };
    let half = (l - r) / 2.0;
    // Fenster mittig bei x = 1,5: links bis zur Innenfläche der Außenwand, rechts bis zur Innenwand bei x = 3.
    let mut values: Vec<f64> = dims.iter().map(|d| d.value).collect();
    values.sort_by(f64::total_cmp);
    assert!((values[0] - (1.5 - 0.63 - half)).abs() < 1e-9, "{dims:?}");
    assert!((values[1] - (3.0 - half - 2.13)).abs() < 1e-9, "{dims:?}");
}

#[test]
fn paste_puts_walls_at_the_same_place_in_another_storey() {
    let (mut doc, mut ed) = house();
    ed.run(&mut doc, &Command::SelectAll);
    let count = ed.selection.len();
    ed.run(&mut doc, &Command::CopyToClipboard);
    ed.run(&mut doc, &Command::AddStorey);
    let upper = ed.storey;
    ed.run(&mut doc, &Command::Paste);
    let p = doc.project();
    assert_eq!(p.walls_on(upper).count(), 5);
    assert_eq!(ed.selection.len(), count);
    assert!(p.walls_on(upper).any(|(_, w)| w.start == Point2::new(0.0, 0.0) || w.end == Point2::new(0.0, 0.0)));
    // Ein Undo nimmt alles zurück.
    doc.undo().unwrap();
    assert_eq!(doc.project().walls_on(upper).count(), 0);
}

#[test]
fn tab_cycles_snap_candidates_under_the_pointer() {
    let (mut doc, mut ed) = house();
    ed.run(&mut doc, &Command::LineTool);
    // An der Ecke liegen Wandende, Flächenecken und Linien übereinander.
    ed.pointer_moved(doc.project(), input(0.05, 0.05));
    let first = ed.overlay(doc.project()).tip.unwrap().1;
    assert!(first.starts_with("Ende Wand"), "{first}");
    ed.key(&mut doc, Key::Tab);
    let second = ed.overlay(doc.project()).tip.unwrap().1;
    assert!(second.contains("⇥ 2/"), "{second}");
    // Der Klick nimmt den mit Tab gewählten Punkt.
    let chosen = ed.pointer.unwrap().point;
    ed.pointer_down(&mut doc, input(0.05, 0.05), 1);
    ed.pointer_moved(doc.project(), input(2.0, 2.0));
    ed.pointer_down(&mut doc, input(2.0, 2.0), 1);
    let a = annotations(&doc);
    let lot_model::AnnotationShape::Polyline(pts) = &a[0].shape else { panic!() };
    assert_eq!(pts[0], chosen);
    assert_ne!(chosen, Point2::new(0.0, 0.0));
}

#[test]
fn snap_override_takes_only_midpoints_for_one_click() {
    let (mut doc, mut ed) = house();
    ed.run(&mut doc, &Command::LineTool);
    for c in "sm".chars() {
        ed.key(&mut doc, Key::Char(c));
    }
    // Bei (3 | 4) liegen das Ende der Innenwand und die Mitte der oberen Wand aufeinander.
    ed.pointer_moved(doc.project(), input(3.05, 3.95));
    assert_eq!(ed.pointer.unwrap().kind, SnapKind::Midpoint);
    ed.pointer_down(&mut doc, input(3.05, 3.95), 1);
    ed.pointer_moved(doc.project(), input(3.05, 3.95));
    assert_eq!(ed.pointer.unwrap().kind, SnapKind::Endpoint, "nur für einen Klick");
}

#[test]
fn reference_point_with_tracking_line_places_the_first_point() {
    let (mut doc, mut ed) = house();
    ed.run(&mut doc, &Command::WallTool);
    // Q an der Hausecke, dann an der waagerechten Spurlinie entlang 1,20 tippen.
    ed.pointer_moved(doc.project(), input(6.0, 4.0));
    ed.key(&mut doc, Key::Char('q'));
    assert_eq!(ed.references, vec![Point2::new(6.0, 4.0)]);
    ed.pointer_moved(doc.project(), input(7.5, 4.03));
    assert_eq!(ed.pointer.unwrap().kind, SnapKind::Tracking);
    typed(&mut doc, &mut ed, "1,20");
    ed.pointer_moved(doc.project(), input(7.2, 6.0));
    typed(&mut doc, &mut ed, "2");
    let w = walls(&doc);
    let last = w.last().unwrap();
    assert_eq!(last.start, Point2::new(7.2, 4.0));
    assert!(last.end.distance(Point2::new(7.2, 6.0)) < 1e-9);
    // Zwei Bezugspunkte: Ihre Spurlinien kreuzen sich und lassen sich fangen.
    ed.key(&mut doc, Key::Escape);
    ed.pointer_moved(doc.project(), input(0.0, 0.0));
    ed.key(&mut doc, Key::Char('q'));
    ed.pointer_moved(doc.project(), input(6.02, 0.03));
    assert_eq!(ed.pointer.unwrap().kind, SnapKind::Endpoint);
    ed.pointer_moved(doc.project(), input(7.18, 0.02));
    let s = ed.pointer.unwrap();
    assert!(matches!(s.kind, SnapKind::Tracking | SnapKind::TrackingIntersection), "{s:?}");
}

#[test]
fn drawing_direction_from_an_edge_turns_right_angles_and_rectangles() {
    let (mut doc, mut ed) = setup();
    ed.run(&mut doc, &Command::WallTool);
    // Eine um 10° gedrehte Wand.
    click(&mut doc, &mut ed, 0.0, 0.0);
    typed(&mut doc, &mut ed, "5<10");
    ed.key(&mut doc, Key::Escape);
    for c in "zr".chars() {
        ed.key(&mut doc, Key::Char(c));
    }
    assert_eq!(ed.tool, Tool::PickDirection);
    let mid = walls(&doc)[0].point_at(2.5, 0.0);
    click(&mut doc, &mut ed, mid.x, mid.y);
    assert_eq!(ed.tool, Tool::Wall);
    assert!((ed.direction - 10.0).abs() < 1e-9);
    // Wandrechteck in der Zeichenrichtung, im Uhrzeigersinn.
    ed.run(&mut doc, &Command::WallRectTool);
    click(&mut doc, &mut ed, 10.0, 0.0);
    typed(&mut doc, &mut ed, "@4;3");
    let w = walls(&doc);
    assert_eq!(w.len(), 5);
    let lengths: Vec<f64> = w[1..].iter().map(|w| (w.length() * 1000.0).round() / 1000.0).collect();
    assert!(lengths.contains(&4.0) && lengths.contains(&3.0), "{lengths:?}");
    for wall in &w[1..] {
        let a = wall.direction().1.atan2(wall.direction().0).to_degrees();
        assert!(((a - 10.0) / 90.0 - ((a - 10.0) / 90.0).round()).abs() < 1e-6, "{a}");
    }
    // Ins Leere klicken setzt die Richtung zurück.
    ed.run(&mut doc, &Command::DirectionFromEdge);
    click(&mut doc, &mut ed, 50.0, 50.0);
    assert_eq!(ed.direction, 0.0);
}

/// Fläche aller Wände in Brüstungshöhe, als eine Fläche.
fn wall_body(doc: &Document, storey: ElementId) -> lot_geom::Area {
    let regions = lot_geom::solid_regions(doc.project(), storey, 1.0);
    lot_geom::Area::union_all(regions.iter().map(|r| &r.area))
}

#[test]
fn alignment_change_keeps_walls_and_corners_in_place() {
    let (mut doc, mut ed) = house();
    let body = wall_body(&doc, ed.storey);
    let joins = lot_geom::junctions(doc.project(), ed.storey).len();
    ed.run(&mut doc, &Command::SelectAll);
    ed.run(&mut doc, &Command::SetAlignment(WallAlignment::Exterior));
    let after = wall_body(&doc, ed.storey);
    assert!((after.area() - body.area()).abs() < 1e-6 && after.difference(&body).area() < 1e-6);
    assert_eq!(lot_geom::junctions(doc.project(), ed.storey).len(), joins);
    // Die Ecken liegen jetzt an der Außenkante: Die Wände wurden länger, nicht verschoben.
    let t = doc.project().wall_types().next().unwrap().1.thickness();
    assert!(doc.project().walls().any(|(_, w)| (w.length() - (6.0 + t)).abs() < 1e-9));
    // Kernmitte, und zurück zur Mitte: wieder genau wie vorher.
    ed.run(&mut doc, &Command::SetAlignment(WallAlignment::CoreCenter));
    assert!(wall_body(&doc, ed.storey).difference(&body).area() < 1e-6);
    ed.run(&mut doc, &Command::SetAlignment(WallAlignment::Center));
    assert!(doc.project().walls().any(|(_, w)| w.start == Point2::new(0.0, 0.0)));
    assert!(doc.undo().unwrap());
}

#[test]
fn flip_keeps_the_wall_and_swaps_its_layers() {
    let (mut doc, mut ed) = house();
    ed.run(&mut doc, &Command::SelectAll);
    ed.run(&mut doc, &Command::SetAlignment(WallAlignment::CoreExterior));
    let bottom =
        doc.project().walls().find(|(_, w)| w.start.y.abs() < 1.0 && w.end.y.abs() < 1.0).map(|(id, _)| id).unwrap();
    let door = doc
        .insert(lot_model::Element::Opening(Opening {
            phase: Default::default(),
            host: bottom,
            offset: 1.0,
            width: 1.01,
            height: 2.135,
            sill_height: 0.0,
            kind: lot_model::OpeningKind::Door(lot_model::Door {
                opens_to: lot_model::WallSide::Left,
                hinge: lot_model::Handing::DinLeft,
            }),
        }))
        .unwrap();
    let body = wall_body(&doc, ed.storey);
    let joins = lot_geom::junctions(doc.project(), ed.storey).len();
    let storey = ed.storey;
    let side_of = |doc: &Document| {
        let w = doc.project().wall(bottom).unwrap();
        let regions = lot_geom::solid_regions(doc.project(), storey, 1.0);
        // Dämmung: Schicht 1 des Außenwandaufbaus. Mitte in y.
        let r = regions.iter().find(|r| r.element == bottom && r.layer == 1).unwrap();
        let (lo, hi) = r.area.bounds().unwrap();
        ((lo[1] + hi[1]) / 2.0, w.clone())
    };
    let (before, w0) = side_of(&doc);
    ed.selection = vec![bottom];
    ed.key(&mut doc, Key::Char(' '));
    let (after, w1) = side_of(&doc);
    assert!(wall_body(&doc, ed.storey).difference(&body).area() < 1e-6, "Wand ist verrutscht");
    assert_eq!(lot_geom::junctions(doc.project(), ed.storey).len(), joins);
    assert!(before < 0.0 && after > before, "Dämmung lag bei {before}, jetzt bei {after}");
    assert_eq!(w1.alignment, WallAlignment::CoreInterior);
    assert_eq!(w0.joins, w1.joins);
    // Die Tür schlägt weiter nach Süden auf, mit den Bändern an derselben Stelle.
    let d = match &doc.project().elements[&door] {
        lot_model::Element::Opening(o) => o.clone(),
        _ => unreachable!(),
    };
    let lot_model::OpeningKind::Door(dd) = d.kind else { panic!("keine Tür") };
    let hinge =
        |w: &lot_model::Wall, o: &Opening, at_end: bool| w.point_at(o.offset + if at_end { o.width } else { 0.0 }, 0.0);
    let o0 = Opening { offset: 1.0, width: 1.01, ..d.clone() };
    let before_hinge = hinge(&w0, &o0, true);
    let after_hinge = hinge(&w1, &d, dd.hinge_at_end());
    assert!(before_hinge.distance(after_hinge) < 0.4, "{before_hinge:?} {after_hinge:?}");
    assert_eq!(dd.opens_to, lot_model::WallSide::Right);
}

#[test]
fn double_click_at_a_corner_selects_the_junction() {
    let (mut doc, mut ed) = house();
    ed.pointer_moved(doc.project(), input(0.05, 0.05));
    ed.pointer_down(&mut doc, input(0.05, 0.05), 1);
    ed.pointer_up(&mut doc);
    ed.pointer_down(&mut doc, input(0.05, 0.05), 2);
    ed.pointer_up(&mut doc);
    let j = ed.selected_junction(doc.project()).expect("Anschluss gewählt");
    assert!(ed.selection.is_empty());
    assert_eq!(j.arms.len(), 2);
    assert!(ed.overlay(doc.project()).junction.is_some());
    // Ausnahme: Die erste Wand läuft durch.
    let a = j.arms[0];
    ed.run(&mut doc, &Command::SetJoinRule(a.wall.0, a.end.unwrap(), lot_model::JoinRule::Through));
    let j = ed.selected_junction(doc.project()).unwrap();
    let role = j.arms.iter().find(|x| x.wall == a.wall).unwrap().role;
    assert_eq!(role, lot_geom::ArmRole::Through);
    ed.key(&mut doc, Key::Escape);
    assert!(ed.junction.is_none());
}
