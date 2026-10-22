//! Wandanschlüsse: aus Wänden mit Schichtaufbau werden Schichtflächen im Grundriss.
//!
//! Vorgehen (PLAN.md, Abschnitt 4, „Wandanschlüsse“):
//!
//! 1. **Knoten finden.** Wandenden, die zusammenfallen, bilden einen Knoten
//!    (L-Ecke, Stoß in Flucht, Y). Ein Wandende auf einer durchlaufenden Wand
//!    ist ein T-Stoß. Wände, die sich mittendrin kreuzen, bilden eine Kreuzung.
//! 2. **Eigene Fläche jeder Wand.** Am freien Ende rechtwinklig. Am Knoten
//!    teilen Gehrungslinien die Ecke zwischen benachbarten Wänden. Am T-Stoß
//!    endet die stoßende Wand an der Wandfläche der durchlaufenden Wand.
//! 3. **Schichten greifen ineinander.** Eine Schicht der ankommenden Wand
//!    läuft in die andere Wand hinein, durch deren Schichten mit niedrigerem
//!    Rang hindurch, und endet an der ersten Schicht mit gleichem oder höherem
//!    Rang (`LayerFunction::priority`). So bindet die Tragschicht einer
//!    Innenwand durch den Innenputz bis an die Tragschicht der Außenwand.
//! 4. **Überschneidungen** (Kreuzungen, Restflächen) gewinnt die Schicht mit
//!    höherem Rang, bei Gleichstand die dickere Wand.
//!
//! Die Knoten werden aus allen Wänden des Geschosses bestimmt, unabhängig von
//! der Höhe. So bleibt ein Anschluss von unten bis oben gleich. Treffen sich
//! zwei Wände verschiedener Höhe in einer Ecke, läuft die höhere bis an die
//! Außenfläche der niedrigeren durch, und die niedrigere stößt stumpf an.
//! 5. **Stützen** haben Vorrang vor allen Wandschichten. **Öffnungen** in
//!    Schnitthöhe werden ausgeschnitten.

use std::collections::BTreeMap;

use lot_model::{ColumnProfile, Element, ElementId, LayerFunction, Project, Wall, WallType};

use crate::poly::{Area, strip};

/// Toleranz, mit der Wandenden als verbunden gelten, in Metern.
const JOIN_TOLERANCE: f64 = 0.001;
/// Überlänge der Schichtstreifen, in Metern.
const REACH: f64 = 50.0;
/// Wände, deren Richtungen weniger als dieser Winkel trennt, gelten als parallel.
const PARALLEL_SIN: f64 = 0.0872; // sin 5°
/// Gehrungsgrenze: Spitzen, die weiter als dieses Vielfache der Wanddicke
/// über den Knoten hinausragen, werden gekappt.
const MITER_LIMIT: f64 = 2.0;

/// Fläche einer Schicht (oder einer Stütze) im Grundriss.
#[derive(Debug, Clone, PartialEq)]
pub struct LayerRegion {
    pub element: ElementId,
    /// Index der Schicht im Wandaufbau, bei Stützen 0.
    pub layer: usize,
    pub material: String,
    /// Funktion der Schicht; `None` bei Stützen.
    pub function: Option<LayerFunction>,
    /// Lage der Schicht, für Muster entlang der Wand (Dämmung). `None` bei Stützen.
    pub band: Option<Band>,
    pub area: Area,
}

/// Lage einer Wandschicht: Bezugslinie und Abstände quer dazu.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Band {
    pub origin: [f64; 2],
    pub direction: [f64; 2],
    /// Abstand der linken und rechten Schichtgrenze von der Bezugslinie (links positiv).
    pub left: f64,
    pub right: f64,
}

impl LayerRegion {
    fn priority(&self) -> u8 {
        self.function.map_or(0, LayerFunction::priority)
    }
}

/// Schichtflächen aller Wände und Stützen eines Geschosses in der absoluten Höhe `z`.
pub fn layer_regions(project: &Project, storey: ElementId, z: f64) -> Vec<LayerRegion> {
    let walls: Vec<WallInfo> = project
        .walls_on(storey)
        .filter_map(|(id, w)| {
            let (z0, z1) = project.wall_extent(w)?;
            WallInfo::new(id, w, project.wall_type(w.wall_type)?, z1, z0 <= z && z < z1)
        })
        .collect();

    let nodes = find_nodes(&walls);
    let mut regions = Vec::new();
    let mut natural: BTreeMap<ElementId, Area> = BTreeMap::new();
    for (i, w) in walls.iter().enumerate() {
        if !w.present {
            continue;
        }
        let start = end_edge(&walls, &nodes, i, End::Start);
        let end = end_edge(&walls, &nodes, i, End::End);
        let mut ring = start;
        ring.extend(end.into_iter().rev());
        let mut area = Area::from_ring(&ring);
        for clip in miter_clips(&walls, &nodes, i) {
            area = area.difference(&clip);
        }
        for (k, (left, right)) in w.offsets.iter().enumerate() {
            let layer = &w.wall_type.layers[k];
            let band = strip(w.a, w.dir, (-REACH, w.len + REACH), (*left, *right));
            regions.push(LayerRegion {
                element: w.id,
                layer: k,
                material: layer.material.clone(),
                function: Some(layer.function),
                band: Some(Band { origin: w.a, direction: w.dir, left: *left, right: *right }),
                area: area.intersection(&band),
            });
        }
        natural.insert(w.id, area);
    }

    apply_extensions(&walls, &nodes, &natural, &mut regions);
    resolve_overlaps(&walls, &mut regions);
    let walls: Vec<&WallInfo> = walls.iter().filter(|w| w.present).collect();

    // Stützen haben Vorrang vor allen Wandschichten.
    for (id, column) in project.columns_on(storey) {
        let Some((z0, z1)) = project.vertical_extent(column.storey, column.base_offset, column.top) else { continue };
        if !(z0 <= z && z < z1) {
            continue;
        }
        let area = column_area(column);
        for r in regions.iter_mut() {
            r.area = r.area.difference(&area);
        }
        regions.push(LayerRegion {
            element: id,
            layer: 0,
            material: column.material.clone(),
            function: None,
            band: None,
            area,
        });
    }

    // Öffnungen in Schnitthöhe.
    for w in walls {
        let Some((bottom, _)) = project.wall_extent(w.wall) else { continue };
        for (_, o) in project.openings_in(w.id) {
            let (o0, o1) = (bottom + o.sill_height, bottom + o.sill_height + o.height);
            if o0 <= z && z < o1 {
                let hole = strip(w.a, w.dir, (o.offset, o.offset + o.width), (w.left_face + 0.01, w.right_face - 0.01));
                for r in regions.iter_mut() {
                    if r.function.is_some() {
                        r.area = r.area.difference(&hole);
                    }
                }
            }
        }
    }

    regions.retain(|r| !r.area.is_empty());
    regions
}

/// Querschnitt einer Stütze als Fläche.
pub fn column_area(column: &lot_model::Column) -> Area {
    let (sin, cos) = column.rotation.to_radians().sin_cos();
    let c = [column.position.x, column.position.y];
    let ring: Vec<[f64; 2]> = match column.profile {
        ColumnProfile::Rectangle { width, depth } => [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)]
            .iter()
            .map(|(sx, sy)| {
                let (x, y) = (sx * width / 2.0, sy * depth / 2.0);
                [c[0] + x * cos - y * sin, c[1] + x * sin + y * cos]
            })
            .collect(),
        ColumnProfile::Circle { diameter } => (0..64)
            .map(|i| {
                let a = i as f64 / 64.0 * std::f64::consts::TAU;
                [c[0] + diameter / 2.0 * a.cos(), c[1] + diameter / 2.0 * a.sin()]
            })
            .collect(),
    };
    Area::from_ring(&ring)
}

/// Eine Wand mit allem, was die Anschlüsse brauchen.
struct WallInfo<'a> {
    id: ElementId,
    wall: &'a Wall,
    wall_type: &'a WallType,
    a: [f64; 2],
    b: [f64; 2],
    dir: [f64; 2],
    normal: [f64; 2],
    len: f64,
    /// (links, rechts) je Schicht, von außen nach innen.
    offsets: Vec<(f64, f64)>,
    left_face: f64,
    right_face: f64,
    /// Oberkante über ±0,00.
    top: f64,
    /// Ob die Wand in der betrachteten Höhe vorhanden ist.
    present: bool,
}

impl<'a> WallInfo<'a> {
    fn new(id: ElementId, wall: &'a Wall, wall_type: &'a WallType, top: f64, present: bool) -> Option<Self> {
        let len = wall.length();
        if len < 1e-6 || wall_type.layers.is_empty() {
            return None;
        }
        let (dx, dy) = wall.direction();
        let offsets = wall.layer_offsets(wall_type);
        Some(Self {
            id,
            wall,
            wall_type,
            a: [wall.start.x, wall.start.y],
            b: [wall.end.x, wall.end.y],
            dir: [dx, dy],
            normal: [-dy, dx],
            len,
            left_face: offsets[0].0,
            right_face: offsets[offsets.len() - 1].1,
            offsets,
            top,
            present,
        })
    }

    fn thickness(&self) -> f64 {
        self.left_face - self.right_face
    }

    fn point(&self, end: End) -> [f64; 2] {
        match end {
            End::Start => self.a,
            End::End => self.b,
        }
    }

    /// Richtung vom Knoten weg in die Wand hinein.
    fn outward(&self, end: End) -> [f64; 2] {
        match end {
            End::Start => self.dir,
            End::End => [-self.dir[0], -self.dir[1]],
        }
    }

    /// Lage eines Punkts im System der Wand: (längs, quer).
    fn local(&self, p: [f64; 2]) -> (f64, f64) {
        let d = [p[0] - self.a[0], p[1] - self.a[1]];
        (dot(d, self.dir), dot(d, self.normal))
    }

    /// Punkt auf der Fläche im Abstand `across` von der Bezugslinie, auf Höhe `along`.
    fn at(&self, along: f64, across: f64) -> [f64; 2] {
        [
            self.a[0] + self.dir[0] * along + self.normal[0] * across,
            self.a[1] + self.dir[1] * along + self.normal[1] * across,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum End {
    Start,
    End,
}

/// Wie ein Wandende angeschlossen ist.
#[derive(Debug, Clone)]
enum Joint {
    Free,
    /// Stößt auf die durchlaufende Wand mit diesem Index.
    Tee(usize),
    /// Gehört zu einem Knoten mit anderen Wandenden.
    Node(usize),
    /// Ecke mit einer niedrigeren Wand (Index): Diese Wand läuft bis an deren
    /// abgewandte Fläche durch, die niedrigere stößt stumpf an.
    Over(usize),
}

/// Ein Knoten aus zusammenfallenden Wandenden.
struct Node {
    point: [f64; 2],
    /// (Wandindex, Ende), gegen den Uhrzeigersinn nach Richtung sortiert.
    arms: Vec<(usize, End)>,
}

struct Nodes {
    joints: Vec<[Joint; 2]>,
    nodes: Vec<Node>,
}

impl Nodes {
    fn joint(&self, wall: usize, end: End) -> &Joint {
        &self.joints[wall][end as usize]
    }
}

fn find_nodes(walls: &[WallInfo]) -> Nodes {
    let mut joints: Vec<[Joint; 2]> = walls.iter().map(|_| [Joint::Free, Joint::Free]).collect();
    // T-Stöße: Wandende liegt in der Fläche einer anderen Wand, nicht an deren Ende.
    for (i, w) in walls.iter().enumerate() {
        for end in [End::Start, End::End] {
            let p = w.point(end);
            let through = walls.iter().enumerate().find(|(j, other)| {
                if *j == i || cross(w.dir, other.dir).abs() < PARALLEL_SIN {
                    return false;
                }
                let (s, t) = other.local(p);
                s > JOIN_TOLERANCE
                    && s < other.len - JOIN_TOLERANCE
                    && t <= other.left_face + JOIN_TOLERANCE
                    && t >= other.right_face - JOIN_TOLERANCE
                    && dist(p, other.a) > JOIN_TOLERANCE
                    && dist(p, other.b) > JOIN_TOLERANCE
            });
            if let Some((j, _)) = through {
                joints[i][end as usize] = Joint::Tee(j);
            }
        }
    }
    // Knoten aus zusammenfallenden Enden.
    let mut nodes: Vec<Node> = Vec::new();
    for (i, w) in walls.iter().enumerate() {
        for end in [End::Start, End::End] {
            if matches!(joints[i][end as usize], Joint::Tee(_)) {
                continue;
            }
            let p = w.point(end);
            match nodes.iter().position(|n| dist(n.point, p) <= JOIN_TOLERANCE) {
                Some(k) => nodes[k].arms.push((i, end)),
                None => nodes.push(Node { point: p, arms: vec![(i, end)] }),
            }
        }
    }
    nodes.retain(|n| n.arms.len() >= 2);
    for (k, node) in nodes.iter_mut().enumerate() {
        node.arms.sort_by(|x, y| {
            let ax = angle(walls[x.0].outward(x.1));
            let ay = angle(walls[y.0].outward(y.1));
            ax.total_cmp(&ay)
        });
        for &(i, end) in &node.arms {
            joints[i][end as usize] = Joint::Node(k);
        }
    }
    // Ecken aus zwei Wänden verschiedener Höhe: die höhere läuft durch.
    for node in &nodes {
        if let [(i, ei), (j, ej)] = node.arms[..]
            && (walls[i].top - walls[j].top).abs() > 0.01
            && cross(walls[i].dir, walls[j].dir).abs() >= PARALLEL_SIN
        {
            let ((hi, he), (lo, le)) =
                if walls[i].top > walls[j].top { ((i, ei), (j, ej)) } else { ((j, ej), (i, ei)) };
            joints[hi][he as usize] = Joint::Over(lo);
            joints[lo][le as usize] = Joint::Tee(hi);
        }
    }
    Nodes { joints, nodes }
}

/// Seitenflächen eines Arms vom Knoten aus gesehen: (links, rechts) als Abstand quer zur Wand.
fn arm_faces(w: &WallInfo, end: End) -> (f64, f64) {
    match end {
        End::Start => (w.left_face, w.right_face),
        End::End => (w.right_face, w.left_face),
    }
}

/// Endkante einer Wand, von der rechten zur linken Wandfläche (im Sinn der
/// Zeichenrichtung), als Punktfolge.
fn end_edge(walls: &[WallInfo], nodes: &Nodes, i: usize, end: End) -> Vec<[f64; 2]> {
    let w = &walls[i];
    let s_end = match end {
        End::Start => 0.0,
        End::End => w.len,
    };
    let perpendicular = vec![w.at(s_end, w.right_face), w.at(s_end, w.left_face)];
    match nodes.joint(i, end) {
        Joint::Free => perpendicular,
        Joint::Over(j) => {
            let o = &walls[*j];
            // Bis an die Fläche der niedrigeren Wand, die von dieser Wand abgewandt ist.
            let face = if dot(w.outward(end), o.normal) > 0.0 { o.right_face } else { o.left_face };
            let line_p = o.at(0.0, face);
            let right = intersect_lines(w.at(0.0, w.right_face), w.dir, line_p, o.dir);
            let left = intersect_lines(w.at(0.0, w.left_face), w.dir, line_p, o.dir);
            match (right, left) {
                (Some(r), Some(l)) => vec![r, l],
                _ => perpendicular,
            }
        }
        Joint::Tee(j) => {
            let t = &walls[*j];
            // Die stoßende Wand liegt auf der Seite, in die sie vom Knoten aus weist.
            let face = if dot(w.outward(end), t.normal) > 0.0 { t.left_face } else { t.right_face };
            let line_p = t.at(0.0, face);
            let right = intersect_lines(w.at(0.0, w.right_face), w.dir, line_p, t.dir);
            let left = intersect_lines(w.at(0.0, w.left_face), w.dir, line_p, t.dir);
            match (right, left) {
                (Some(r), Some(l)) => vec![r, l],
                _ => perpendicular,
            }
        }
        Joint::Node(k) => {
            let node = &nodes.nodes[*k];
            let n = node.arms.len();
            let pos = node.arms.iter().position(|&(j, e)| j == i && e == end).expect("Arm gehört zum Knoten");
            let next = node.arms[(pos + 1) % n];
            let prev = node.arms[(pos + n - 1) % n];
            // Linke Ecke: zwischen der linken Fläche dieses Arms und der rechten des nächsten (gegen den Uhrzeigersinn).
            let left_corner = corner(walls, (i, end), next, true);
            let right_corner = corner(walls, prev, (i, end), false);
            // In Zeichenrichtung: am Anfang ist links des Arms links der Wand, am Ende rechts.
            let (wall_right, wall_left) = match end {
                End::Start => (right_corner, left_corner),
                End::End => (left_corner, right_corner),
            };
            if n >= 3 { vec![wall_right, node.point, wall_left] } else { vec![wall_right, wall_left] }
        }
    }
}

/// Eckpunkt zwischen der linken Fläche von Arm `a` und der rechten Fläche von
/// Arm `b`. `for_a` wählt bei parallelen Flächen, für welchen Arm der Punkt gilt.
fn corner(walls: &[WallInfo], a: (usize, End), b: (usize, End), for_a: bool) -> [f64; 2] {
    let (wa, wb) = (&walls[a.0], &walls[b.0]);
    let pa = wa.point(a.1);
    let la = arm_faces(wa, a.1).0;
    let rb = arm_faces(wb, b.1).1;
    let line_a = wa.at(0.0, la);
    let line_b = wb.at(0.0, rb);
    let projected = if for_a { wa.at(wa.local(pa).0, la) } else { wb.at(wb.local(pa).0, rb) };
    if a.0 == b.0 || cross(wa.dir, wb.dir).abs() < PARALLEL_SIN {
        return projected;
    }
    match intersect_lines(line_a, wa.dir, line_b, wb.dir) {
        Some(p) => p,
        None => projected,
    }
}

/// Kappt Gehrungsspitzen, die weiter als `MITER_LIMIT` Wanddicken über den Knoten hinausragen.
fn miter_clips(walls: &[WallInfo], nodes: &Nodes, i: usize) -> Vec<Area> {
    let mut clips = Vec::new();
    for end in [End::Start, End::End] {
        let Joint::Node(k) = nodes.joint(i, end) else { continue };
        let node = &nodes.nodes[*k];
        let n = node.arms.len();
        for idx in 0..n {
            let a = node.arms[idx];
            let b = node.arms[(idx + 1) % n];
            if a.0 != i && b.0 != i {
                continue;
            }
            // Nur die äußere Spitze in einem überstumpfen Sektor wird gekappt,
            // nie die innere Ecke zwischen zwei Wänden.
            let sector =
                (angle(walls[b.0].outward(b.1)) - angle(walls[a.0].outward(a.1))).rem_euclid(std::f64::consts::TAU);
            if n > 1 && sector <= std::f64::consts::PI {
                continue;
            }
            let p = corner(walls, a, b, true);
            let thickness = walls[a.0].thickness().max(walls[b.0].thickness());
            let reach = dist(p, node.point);
            if reach > MITER_LIMIT * thickness {
                // Kappen senkrecht zur Richtung der Spitze, im Abstand der Grenze.
                let d = [(p[0] - node.point[0]) / reach, (p[1] - node.point[1]) / reach];
                let limit = MITER_LIMIT * thickness;
                clips.push(strip(node.point, d, (limit, reach + 1.0), (-REACH, REACH)));
            }
        }
    }
    clips
}

/// Eine Schicht, die über das Ende ihrer Wand hinaus in eine andere Wand läuft.
struct Extension {
    element: ElementId,
    layer: usize,
    priority: u8,
    area: Area,
}

fn apply_extensions(
    walls: &[WallInfo],
    nodes: &Nodes,
    natural: &BTreeMap<ElementId, Area>,
    regions: &mut [LayerRegion],
) {
    let mut extensions = Vec::new();
    for (i, w) in walls.iter().enumerate() {
        for end in [End::Start, End::End] {
            if !w.present {
                continue;
            }
            let others: Vec<usize> = match nodes.joint(i, end) {
                Joint::Free | Joint::Over(_) => continue,
                Joint::Tee(j) => vec![*j],
                Joint::Node(k) => nodes.nodes[*k].arms.iter().map(|a| a.0).filter(|j| *j != i).collect(),
            };
            let p = w.point(end);
            for j in others {
                let o = &walls[j];
                if !o.present || cross(w.dir, o.dir).abs() < PARALLEL_SIN {
                    continue;
                }
                // Die Fläche der anderen Wand, die dieser Wand zugewandt ist.
                let from_left = dot(w.outward(end), o.normal) > 0.0;
                let reach = 3.0 * w.thickness().max(o.thickness()) + 0.5;
                let local_box = strip(p, [1.0, 0.0], (-reach, reach), (-reach, reach));
                for (k, (left, right)) in w.offsets.iter().enumerate() {
                    let priority = w.wall_type.layers[k].function.priority();
                    let depth = penetrable_depth(o, from_left, priority);
                    if depth <= 0.0 {
                        continue;
                    }
                    let across = if from_left {
                        (o.left_face, o.left_face - depth)
                    } else {
                        (o.right_face + depth, o.right_face)
                    };
                    let target = strip(o.a, o.dir, (-REACH, o.len + REACH), across);
                    let own = strip(w.a, w.dir, (-REACH, w.len + REACH), (*left, *right));
                    // Nur in fremdes Gebiet: Wo sich die eigene Fläche mit der anderen
                    // Wand überschneidet (Kreuzung), entscheidet der Rang.
                    let area = own
                        .intersection(&target)
                        .intersection(&natural[&o.id])
                        .intersection(&local_box)
                        .difference(&natural[&w.id]);
                    if !area.is_empty() {
                        extensions.push(Extension { element: w.id, layer: k, priority, area });
                    }
                }
            }
        }
    }
    // Niedriger Rang zuerst, damit höherer Rang zuletzt gewinnt.
    extensions.sort_by_key(|e| std::cmp::Reverse(e.priority));
    for ext in extensions {
        for r in regions.iter_mut() {
            if r.element == ext.element && r.layer == ext.layer {
                r.area = r.area.union(&ext.area);
            } else if r.priority() > ext.priority {
                r.area = r.area.difference(&ext.area);
            }
        }
    }
}

/// Wie tief eine Schicht mit Rang `priority` in die Wand `o` eindringt, von
/// der linken oder rechten Wandfläche aus: durch alle Schichten mit
/// niedrigerem Rang bis zur ersten mit gleichem oder höherem Rang.
fn penetrable_depth(o: &WallInfo, from_left: bool, priority: u8) -> f64 {
    let layers: Box<dyn Iterator<Item = &lot_model::Layer>> =
        if from_left { Box::new(o.wall_type.layers.iter()) } else { Box::new(o.wall_type.layers.iter().rev()) };
    let mut depth = 0.0;
    for layer in layers {
        if layer.function.priority() <= priority {
            break;
        }
        depth += layer.thickness;
    }
    depth
}

/// Überschneidungen: Höherer Rang gewinnt, dann die dickere Wand, dann die
/// kleinere Kennung, innerhalb einer Wand die äußere Schicht.
fn resolve_overlaps(walls: &[WallInfo], regions: &mut [LayerRegion]) {
    let thickness: BTreeMap<ElementId, f64> = walls.iter().map(|w| (w.id, w.thickness())).collect();
    let rank = |r: &LayerRegion| {
        (r.priority(), -(thickness.get(&r.element).copied().unwrap_or(0.0) * 1e6) as i64, r.element, r.layer)
    };
    let mut order: Vec<usize> = (0..regions.len()).collect();
    order.sort_by_key(|&i| rank(&regions[i]));
    for (pos, &winner) in order.iter().enumerate() {
        let Some(wb) = regions[winner].area.bounds() else { continue };
        for &loser in &order[pos + 1..] {
            let Some(lb) = regions[loser].area.bounds() else { continue };
            if lb.0[0] > wb.1[0] || lb.1[0] < wb.0[0] || lb.0[1] > wb.1[1] || lb.1[1] < wb.0[1] {
                continue;
            }
            let overlap = regions[loser].area.intersection(&regions[winner].area);
            if !overlap.is_empty() {
                let rest = regions[loser].area.difference(&regions[winner].area);
                regions[loser].area = rest;
            }
        }
    }
}

/// Alle Wandknoten eines Geschosses, für Tests und Prüfpakete: (Punkt, Anzahl Arme).
pub fn node_summary(project: &Project, storey: ElementId) -> Vec<([f64; 2], usize)> {
    let walls: Vec<WallInfo> = project
        .walls_on(storey)
        .filter_map(|(id, w)| WallInfo::new(id, w, project.wall_type(w.wall_type)?, project.wall_extent(w)?.1, true))
        .collect();
    find_nodes(&walls).nodes.iter().map(|n| (n.point, n.arms.len())).collect()
}

/// Ob das Element eine Wand ist (für Aufrufer, die Regionen nach Art filtern).
pub fn is_wall(project: &Project, id: ElementId) -> bool {
    matches!(project.elements.get(&id), Some(Element::Wall(_)))
}

fn dot(a: [f64; 2], b: [f64; 2]) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}

fn cross(a: [f64; 2], b: [f64; 2]) -> f64 {
    a[0] * b[1] - a[1] * b[0]
}

fn dist(a: [f64; 2], b: [f64; 2]) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

fn angle(d: [f64; 2]) -> f64 {
    d[1].atan2(d[0])
}

/// Schnittpunkt zweier Geraden (Punkt und Richtung).
fn intersect_lines(p: [f64; 2], d: [f64; 2], q: [f64; 2], e: [f64; 2]) -> Option<[f64; 2]> {
    let denom = cross(d, e);
    if denom.abs() < 1e-12 {
        return None;
    }
    let t = cross([q[0] - p[0], q[1] - p[1]], e) / denom;
    Some([p[0] + d[0] * t, p[1] + d[1] * t])
}
