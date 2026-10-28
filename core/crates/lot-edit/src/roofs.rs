//! Dächer in der App (ADR 0051): aus den Wänden als Sattel-, Walm- oder
//! Flachdach, Kanten umschalten zwischen Traufe und Giebel, Neigung und
//! Überstand je Kante. Giebelwände reichen bis unter das Dach.

use lot_model::{
    Element, ElementId, Fall, FallPurpose, Layer, LayerFunction, Operation, Point2, Project, Roof, RoofEdge, Slab,
    SlabKind, SlabType, Top,
};

/// Form eines neuen Dachs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoofForm {
    /// Traufen an den langen Seiten, Giebel an den kurzen.
    Gable,
    /// Traufen ringsum.
    Hip,
}

/// Neigung neuer Dächer in Grad, Vorschlag.
pub const SLOPE: f64 = 40.0;

/// Dach aus den Wänden des Geschosses; alle Wände bis zum Geschoss reichen danach bis unter das Dach.
pub fn roof_ops(project: &Project, storey_id: ElementId, form: RoofForm) -> Result<Vec<Operation>, String> {
    let storey = project.storey(storey_id).ok_or("Das Geschoss fehlt")?;
    let roof_type = project
        .slab_types()
        .find(|(_, t)| t.name.to_lowercase().contains("dach") && !t.name.to_lowercase().contains("flachdach"))
        .or_else(|| project.slab_types().last())
        .map(|(id, _)| id)
        .ok_or("Im Projekt gibt es noch keinen Dachaufbau")?;
    let finder = lot_geom::spaces::SpaceFinder::new(project, storey_id);
    let outer: Vec<_> = finder.footprint.rings().into_iter().filter(|r| lot_geom::signed_area(r) > 0.0).collect();
    let [ring] = &outer[..] else {
        return Err("Für ein Dach braucht das Geschoss genau einen geschlossenen Umriss aus Wänden".into());
    };
    let outline: Vec<Point2> = ring.iter().map(|p| Point2::new(p[0], p[1])).collect();
    let n = outline.len();
    let dir = |i: usize| {
        let (a, b) = (outline[i], outline[(i + 1) % n]);
        let len = a.distance(b).max(1e-12);
        ((b.x - a.x) / len, (b.y - a.y) / len, len)
    };
    let longest = (0..n).max_by(|a, b| dir(*a).2.total_cmp(&dir(*b).2)).unwrap_or(0);
    let (lx, ly, _) = dir(longest);
    let edges: Vec<RoofEdge> = (0..n)
        .map(|i| {
            let (x, y, _) = dir(i);
            let along_longest = (x * ly - y * lx).abs() < 0.05;
            match form {
                RoofForm::Hip => RoofEdge::Eave { slope: SLOPE, overhang: 0.5 },
                RoofForm::Gable if along_longest => RoofEdge::Eave { slope: SLOPE, overhang: 0.5 },
                RoofForm::Gable => RoofEdge::Verge { overhang: 0.3 },
            }
        })
        .collect();
    let knee = project
        .walls_on(storey_id)
        .filter_map(|(_, w)| match w.top {
            Top::Height(h) => Some(h + w.base_offset),
            _ => None,
        })
        .fold(f64::NEG_INFINITY, f64::max);
    let knee = if knee.is_finite() { knee } else { storey.height };
    let thickness = project.slab_type(roof_type).map_or(0.0, |t| t.thickness());
    let roof = Roof {
        phase: Default::default(),
        storey: storey_id,
        roof_type,
        outline,
        edges,
        eave_height: knee + thickness / SLOPE.to_radians().cos(),
    };
    let mut ops = vec![Operation::Insert { id: project.next_id(), element: Element::Roof(roof) }];
    for (id, w) in project.walls_on(storey_id) {
        if let Top::Storey { offset } = w.top {
            let mut n = w.clone();
            n.top = Top::Roof { offset };
            ops.push(Operation::Replace { id, old: Element::Wall(w.clone()), new: Element::Wall(n) });
        }
    }
    Ok(ops)
}

/// Aufbau eines Flachdachs ab Werk (Vorschlag): Kies, Abdichtung,
/// Gefälledämmung, Stahlbeton, Putz.
fn flat_roof_type() -> SlabType {
    let layer = |material: &str, thickness: f64, function: LayerFunction| Layer {
        material: material.into(),
        thickness,
        function,
    };
    SlabType {
        name: "Flachdach Warmdach StB 20".into(),
        layers: vec![
            layer("kies", 0.05, LayerFunction::Finish),
            layer("abdichtung", 0.01, LayerFunction::Finish),
            layer("daemmung_hart", 0.16, LayerFunction::Insulation),
            layer("stahlbeton", 0.20, LayerFunction::Structure),
            layer("putz", 0.015, LayerFunction::Finish),
        ],
        props: lot_model::Properties { ifc_class: Some("IfcSlab.ROOF".into()), ..Default::default() },
        locked: Default::default(),
    }
}

/// Flachdach über dem Geschoss: eine Decke der Art Flachdach im Geschoss
/// darüber (das bei Bedarf entsteht), die den Wänden folgt, mit 2 % Gefälle
/// in der Dämmung, quer zur längsten Seite.
pub fn flat_roof_ops(project: &Project, storey_id: ElementId) -> Result<(ElementId, Vec<Operation>), String> {
    let storey = project.storey(storey_id).ok_or("Das Geschoss fehlt")?;
    let mut scratch = project.clone();
    let mut ops = Vec::new();
    let insert = |scratch: &mut Project, element: Element, ops: &mut Vec<Operation>| {
        let id = scratch.next_id();
        scratch.elements.insert(id, element.clone());
        ops.push(Operation::Insert { id, element });
        id
    };
    let above = match crate::navigation::storey_step(project, storey_id, true) {
        Some(id) => id,
        None => insert(
            &mut scratch,
            Element::Storey(lot_model::Storey {
                name: "Dach".into(),
                elevation: storey.elevation + storey.height,
                height: storey.height,
            }),
            &mut ops,
        ),
    };
    let roof_type = match project.slab_types().find(|(_, t)| t.name.to_lowercase().contains("flachdach")) {
        Some((id, _)) => id,
        None => insert(&mut scratch, Element::SlabType(flat_roof_type()), &mut ops),
    };
    let rings = lot_geom::slabs::slab_outlines(&scratch, storey_id);
    let ring = rings
        .into_iter()
        .max_by(|a, b| lot_geom::signed_area(a).total_cmp(&lot_geom::signed_area(b)))
        .ok_or("Für ein Flachdach fehlen geschlossene Wände im Geschoss")?;
    let outline: Vec<Point2> = ring.iter().map(|p| Point2::new(p[0], p[1])).collect();
    let layer = scratch
        .slab_type(roof_type)
        .and_then(|t| t.layers.iter().position(|l| l.function == LayerFunction::Insulation))
        .unwrap_or(0);
    // Wasser fließt quer zur längsten Seite, zu ihr hin.
    let n = outline.len();
    let longest = (0..n)
        .max_by(|a, b| {
            outline[*a].distance(outline[(*a + 1) % n]).total_cmp(&outline[*b].distance(outline[(*b + 1) % n]))
        })
        .unwrap_or(0);
    let (a, b) = (outline[longest], outline[(longest + 1) % n]);
    let c = Point2::new(
        outline.iter().map(|p| p.x).sum::<f64>() / n as f64,
        outline.iter().map(|p| p.y).sum::<f64>() / n as f64,
    );
    let len = a.distance(b).max(1e-9);
    let normal = (-(b.y - a.y) / len, (b.x - a.x) / len);
    let toward = if (a.x - c.x) * normal.0 + (a.y - c.y) * normal.1 > 0.0 { normal } else { (-normal.0, -normal.1) };
    let direction = toward.1.atan2(toward.0).to_degrees().round();
    let slab = Slab {
        storey: above,
        slab_type: roof_type,
        outline,
        top_offset: 0.0,
        kind: SlabKind::Roof,
        phase: Default::default(),
        props: Default::default(),
        follows_walls: true,
        boundary_walls: Vec::new(),
        fall: Some(Box::new(Fall {
            purpose: FallPurpose::Roof,
            layer,
            percent: 2.0,
            direction,
            drains: Vec::new(),
            overflows: Vec::new(),
        })),
    };
    let slab = crate::follow::with_boundary(&scratch, slab);
    let id = insert(&mut scratch, Element::Slab(slab), &mut ops);
    Ok((id, ops))
}

/// Die Kante eines Dachs im Geschoss unter dem Zeiger: Dach und Kante.
pub fn edge_at(project: &Project, storey: ElementId, p: Point2, tolerance: f64) -> Option<(ElementId, usize)> {
    project
        .roofs_on(storey)
        .flat_map(|(id, r)| {
            let n = r.outline.len();
            let outer = r.outer_outline();
            (0..n).map(move |i| {
                let d = |ring: &[Point2]| segment_distance(ring[i], ring[(i + 1) % n], p);
                (id, i, d(&r.outline).min(if outer.len() == n { d(&outer) } else { f64::INFINITY }))
            })
        })
        .filter(|(_, _, d)| *d <= tolerance)
        .min_by(|a, b| a.2.total_cmp(&b.2))
        .map(|(id, i, _)| (id, i))
}

fn segment_distance(a: Point2, b: Point2, p: Point2) -> f64 {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let len2 = (dx * dx + dy * dy).max(1e-18);
    let t = (((p.x - a.x) * dx + (p.y - a.y) * dy) / len2).clamp(0.0, 1.0);
    p.distance(Point2::new(a.x + dx * t, a.y + dy * t))
}

/// Kante umschalten: Traufe wird Giebel, Giebel wird Traufe (an der Schmalseite ein Walm).
/// Die Wände des Geschosses reichen danach bis unter das Dach.
pub fn toggle_edge_ops(project: &Project, roof_id: ElementId, edge: usize) -> Vec<Operation> {
    let Some(Element::Roof(old)) = project.elements.get(&roof_id) else { return Vec::new() };
    let Some(e) = old.edges.get(edge).copied() else { return Vec::new() };
    let mut new = old.clone();
    new.edges[edge] = match e {
        RoofEdge::Eave { overhang, .. } => RoofEdge::Verge { overhang: overhang.min(0.3) },
        RoofEdge::Verge { overhang } => {
            // Neigung wie die übrigen Traufen, sonst der Vorschlag.
            let slope = old
                .edges
                .iter()
                .find_map(|x| match x {
                    RoofEdge::Eave { slope, .. } => Some(*slope),
                    _ => None,
                })
                .unwrap_or(SLOPE);
            RoofEdge::Eave { slope, overhang: overhang.max(0.5) }
        }
    };
    let mut ops = vec![Operation::Replace { id: roof_id, old: Element::Roof(old.clone()), new: Element::Roof(new) }];
    for (id, w) in project.walls_on(old.storey) {
        if let Top::Storey { offset } = w.top {
            let mut n = w.clone();
            n.top = Top::Roof { offset };
            ops.push(Operation::Replace { id, old: Element::Wall(w.clone()), new: Element::Wall(n) });
        }
    }
    ops
}

/// Neigung und Überstand einer Kante; Neigung nur bei Traufen.
pub fn edge_ops(
    project: &Project,
    roof_id: ElementId,
    edge: usize,
    slope: Option<f64>,
    overhang: Option<f64>,
) -> Vec<Operation> {
    let Some(Element::Roof(old)) = project.elements.get(&roof_id) else { return Vec::new() };
    let Some(e) = old.edges.get(edge).copied() else { return Vec::new() };
    let mut new = old.clone();
    new.edges[edge] = match e {
        RoofEdge::Eave { slope: s, overhang: o } => {
            RoofEdge::Eave { slope: slope.unwrap_or(s).clamp(0.0, 75.0), overhang: overhang.unwrap_or(o).max(0.0) }
        }
        RoofEdge::Verge { overhang: o } => RoofEdge::Verge { overhang: overhang.unwrap_or(o).max(0.0) },
    };
    if new == *old {
        return Vec::new();
    }
    vec![Operation::Replace { id: roof_id, old: Element::Roof(old.clone()), new: Element::Roof(new) }]
}

/// Neigungsgriffe eines Dachs: je Traufe einer, so weit innen, wie das Dach
/// auf 1 m Höhe steigt. Zieht man ihn zur Traufe, wird das Dach steiler.
pub fn slope_grips(roof: &Roof) -> Vec<(usize, Point2)> {
    let n = roof.outline.len();
    roof.edges
        .iter()
        .enumerate()
        .filter_map(|(i, e)| match *e {
            RoofEdge::Eave { slope, .. } if slope > 0.5 => {
                let (a, b) = (roof.outline[i], roof.outline[(i + 1) % n]);
                let (ix, iy) = roof.inward(i);
                let d = (1.0 / slope.to_radians().tan()).clamp(0.25, 6.0);
                Some((i, Point2::new((a.x + b.x) / 2.0 + ix * d, (a.y + b.y) / 2.0 + iy * d)))
            }
            _ => None,
        })
        .collect()
}

/// Neigung in Grad, wenn der Griff der Traufe `edge` nach `p` gezogen wird; auf 0,5° gerundet.
pub fn slope_from_grip(roof: &Roof, edge: usize, p: Point2) -> Option<f64> {
    let n = roof.outline.len();
    let (a, b) = (*roof.outline.get(edge)?, roof.outline[(edge + 1) % n]);
    let (ix, iy) = roof.inward(edge);
    let d = (p.x - (a.x + b.x) / 2.0) * ix + (p.y - (a.y + b.y) / 2.0) * iy;
    if d <= 0.05 {
        return Some(75.0);
    }
    let slope = (1.0 / d).atan().to_degrees();
    Some(((slope * 2.0).round() / 2.0).clamp(5.0, 75.0))
}
