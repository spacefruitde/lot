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
//!
//! Gebogene Wände werden in kurze gerade Stücke (Facetten) zerlegt, die
//! untereinander auf Gehrung stoßen (PLAN.md, Abschnitt 2: „intern
//! facettiert“). Für die Anschlüsse an andere Wände sind die Facetten
//! gewöhnliche Wände; die Schichtflächen entstehen in einem Stück über alle
//! Facettenecken, damit an den Stößen keine Fugen bleiben.

use std::collections::BTreeMap;

use lot_model::{
    CircularArc, ColumnProfile, Element, ElementId, JoinRule, LayerFunction, Phase, Point2, Project, Wall, WallEnd,
    WallType,
};

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
/// Größter Abstand einer Facette vom Bogen, gemessen an der äußeren Wandfläche, in Metern.
const FACET_TOLERANCE: f64 = 0.002;
/// Höchstzahl der Facetten je Wand.
const MAX_FACETS: usize = 64;

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
    /// Bogen der Bezugslinie bei gebogenen Wänden; dann gelten `origin` und
    /// `direction` nur für die Sehne.
    pub arc: Option<CircularArc>,
}

impl Band {
    /// Punkt im Abstand `along` entlang der Bezugslinie und `across` quer dazu.
    pub fn point(&self, along: f64, across: f64) -> [f64; 2] {
        match self.arc {
            Some(a) => {
                let p = a.point_at(along, across);
                [p.x, p.y]
            }
            None => {
                let (d, o) = (self.direction, self.origin);
                [o[0] + d[0] * along - d[1] * across, o[1] + d[1] * along + d[0] * across]
            }
        }
    }

    /// Lage eines Punkts: (entlang, quer).
    pub fn locate(&self, p: [f64; 2]) -> (f64, f64) {
        match self.arc {
            Some(a) => a.locate(Point2::new(p[0], p[1])),
            None => {
                let (d, q) = (self.direction, [p[0] - self.origin[0], p[1] - self.origin[1]]);
                (q[0] * d[0] + q[1] * d[1], -q[0] * d[1] + q[1] * d[0])
            }
        }
    }
}

impl LayerRegion {
    fn priority(&self) -> u8 {
        self.function.map_or(0, LayerFunction::priority)
    }
}

/// Schichtflächen aller Wände und Stützen eines Geschosses in der absoluten Höhe `z`.
pub fn layer_regions(project: &Project, storey: ElementId, z: f64) -> Vec<LayerRegion> {
    regions_at(project, storey, z, true)
}

/// Wie [`layer_regions`], aber ohne Öffnungen: der volle Wandquerschnitt.
pub fn solid_regions(project: &Project, storey: ElementId, z: f64) -> Vec<LayerRegion> {
    regions_at(project, storey, z, false)
}

fn regions_at(project: &Project, storey: ElementId, z: f64, with_openings: bool) -> Vec<LayerRegion> {
    let walls: Vec<WallInfo> = project
        .walls_on(storey)
        .filter_map(|(id, w)| {
            let (z0, z1) = project.wall_extent(w)?;
            Some(WallInfo::facets(id, w, project.wall_type(w.wall_type)?, z1, z0 <= z && z < z1))
        })
        .flatten()
        .collect();

    let nodes = find_nodes(&walls);
    let mut regions = Vec::new();
    let mut natural: BTreeMap<ElementId, Area> = BTreeMap::new();
    let mut next = 0;
    while next < walls.len() {
        // Die Facetten einer Wand liegen hintereinander.
        let count = walls[next..].iter().take_while(|f| f.id == walls[next].id).count();
        let group = next..next + count;
        next += count;
        let w = &walls[group.start];
        if !w.present {
            continue;
        }
        let mut area = Area::from_ring(&outline(&walls, &nodes, group.clone()));
        for k in group.clone() {
            for clip in miter_clips(&walls, &nodes, k) {
                area = area.difference(&clip);
            }
        }
        for (k, (left, right)) in w.offsets.iter().enumerate() {
            let layer = &w.wall_type.layers[k];
            let band = layer_band(&walls, group.clone(), (*left, *right));
            let (dx, dy) = w.wall.direction();
            regions.push(LayerRegion {
                element: w.id,
                layer: k,
                material: layer.material.clone(),
                function: Some(layer.function),
                band: Some(Band {
                    origin: [w.wall.start.x, w.wall.start.y],
                    direction: [dx, dy],
                    left: *left,
                    right: *right,
                    arc: w.wall.arc(),
                }),
                area: area.intersection(&band),
            });
        }
        natural.insert(w.id, area);
    }

    apply_extensions(&walls, &nodes, &natural, &mut regions);
    resolve_overlaps(&walls, &mut regions);
    let mut present: Vec<(ElementId, &Wall, f64, f64)> = Vec::new();
    for w in walls.iter().filter(|w| w.present) {
        if !present.iter().any(|p| p.0 == w.id) {
            present.push((w.id, w.wall, w.left_face, w.right_face));
        }
    }

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
    for (id, wall, left_face, right_face) in present.into_iter().filter(|_| with_openings) {
        let Some((bottom, _)) = project.wall_extent(wall) else { continue };
        for (_, o) in project.openings_in(id) {
            let (o0, o1) = (bottom + o.sill_height, bottom + o.sill_height + o.height);
            if o0 <= z && z < o1 {
                let hole = wall_section(wall, (o.offset, o.offset + o.width), (left_face + 0.01, right_face - 0.01));
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

/// Fläche einer Wand zwischen `along.0` und `along.1` (entlang der
/// Bezugslinie) und quer zwischen `across.0` und `across.1`. Bei gebogenen
/// Wänden radial begrenzt.
pub fn wall_section(wall: &Wall, along: (f64, f64), across: (f64, f64)) -> Area {
    let n = match wall.arc() {
        Some(a) => {
            let outer = a.radius_at(across.0).abs().max(a.radius_at(across.1).abs());
            facet_count(a.sweep.abs() * (along.1 - along.0).abs() / a.length(), outer)
        }
        None => 1,
    };
    let at = |i: usize, t: f64| {
        let p = wall.point_at(along.0 + (along.1 - along.0) * i as f64 / n as f64, t);
        [p.x, p.y]
    };
    let mut ring: Vec<[f64; 2]> = (0..=n).map(|i| at(i, across.0)).collect();
    ring.extend((0..=n).rev().map(|i| at(i, across.1)));
    Area::from_ring(&ring)
}

/// Zahl der Facetten für einen Bogen mit Winkel `sweep` (Betrag) und äußerem Radius.
fn facet_count(sweep: f64, outer_radius: f64) -> usize {
    if outer_radius <= FACET_TOLERANCE {
        return 1;
    }
    let step = 2.0 * (1.0 - FACET_TOLERANCE / outer_radius).acos();
    ((sweep / step).ceil() as usize).clamp(1, MAX_FACETS)
}

/// Umriss einer Wand aus ihren Facetten `group`, ohne Gehrungskappung:
/// Anfangskante, linke Fläche über alle Facettenecken, Endkante, rechte Fläche zurück.
fn outline(walls: &[WallInfo], nodes: &Nodes, group: std::ops::Range<usize>) -> Vec<[f64; 2]> {
    let inner: Vec<Vec<[f64; 2]>> = (group.start..group.end - 1).map(|k| end_edge(walls, nodes, k, End::End)).collect();
    let mut ring = end_edge(walls, nodes, group.start, End::Start);
    ring.extend(inner.iter().filter_map(|e| e.last().copied()));
    ring.extend(end_edge(walls, nodes, group.end - 1, End::End).into_iter().rev());
    ring.extend(inner.iter().rev().filter_map(|e| e.first().copied()));
    ring
}

/// Streifen einer Schicht zwischen den Abständen `across` (links, rechts) entlang
/// aller Facetten einer Wand, an beiden Enden verlängert.
fn layer_band(walls: &[WallInfo], group: std::ops::Range<usize>, across: (f64, f64)) -> Area {
    let (first, last) = (&walls[group.start], &walls[group.end - 1]);
    if group.len() == 1 {
        return strip(first.a, first.dir, (-REACH, first.len + REACH), across);
    }
    // Bei Bögen nur mäßig verlängern, sonst kreuzen sich die Verlängerungen.
    let ext = 3.0 * first.thickness() + 1.0;
    let side = |t: f64| {
        let mut points = vec![first.at(-ext, t)];
        for k in group.start..group.end - 1 {
            let (f, g) = (&walls[k], &walls[k + 1]);
            points.push(intersect_lines(f.at(0.0, t), f.dir, g.at(0.0, t), g.dir).unwrap_or(f.at(f.len, t)));
        }
        points.push(last.at(last.len + ext, t));
        points
    };
    let mut ring = side(across.0);
    ring.extend(side(across.1).into_iter().rev());
    Area::from_ring(&ring)
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

/// Eine Wand (oder eine Facette einer gebogenen Wand) mit allem, was die Anschlüsse brauchen.
struct WallInfo<'a> {
    id: ElementId,
    wall: &'a Wall,
    wall_type: &'a WallType,
    /// Ob Anfang und Ende dieser Facette auch Anfang und Ende der Wand sind.
    /// Bei geraden Wänden immer beides.
    ends: [bool; 2],
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
    phase: Phase,
}

impl<'a> WallInfo<'a> {
    /// Eine gerade Wand als ein Eintrag, eine gebogene als Folge von Facetten.
    fn facets(id: ElementId, wall: &'a Wall, wall_type: &'a WallType, top: f64, present: bool) -> Vec<Self> {
        if wall.chord() < 1e-6 || wall_type.layers.is_empty() {
            return Vec::new();
        }
        let offsets = wall.layer_offsets(wall_type);
        let (left_face, right_face) = (offsets[0].0, offsets[offsets.len() - 1].1);
        let n = match wall.arc() {
            Some(a) => facet_count(a.sweep.abs(), a.radius_at(left_face).max(a.radius_at(right_face))).max(2),
            None => 1,
        };
        let length = wall.length();
        let points: Vec<[f64; 2]> = (0..=n)
            .map(|i| match i {
                0 => [wall.start.x, wall.start.y],
                i if i == n => [wall.end.x, wall.end.y],
                i => {
                    let p = wall.point_at(length * i as f64 / n as f64, 0.0);
                    [p.x, p.y]
                }
            })
            .collect();
        points
            .windows(2)
            .enumerate()
            .map(|(i, ab)| {
                let (a, b) = (ab[0], ab[1]);
                let len = dist(a, b);
                let dir = [(b[0] - a[0]) / len, (b[1] - a[1]) / len];
                Self {
                    id,
                    wall,
                    wall_type,
                    ends: [i == 0, i == n - 1],
                    a,
                    b,
                    dir,
                    normal: [-dir[1], dir[0]],
                    len,
                    left_face,
                    right_face,
                    offsets: offsets.clone(),
                    top,
                    present,
                    phase: wall.phase,
                }
            })
            .collect()
    }

    /// Ob das Ende dieser Facette ein Ende der Wand ist.
    fn is_wall_end(&self, end: End) -> bool {
        self.ends[end as usize]
    }

    /// Ausnahme am Anschluss dieses Endes; Facettenecken haben keine.
    fn rule(&self, end: End) -> JoinRule {
        if self.is_wall_end(end) { self.wall.joins[end as usize] } else { JoinRule::Auto }
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

    /// Ob der Punkt auf der ganzen Wand liegt, nicht vor ihrem Anfang oder hinter ihrem Ende.
    fn inside_wall(&self, p: [f64; 2]) -> bool {
        let (s, _) = self.wall.locate(Point2::new(p[0], p[1]));
        s > JOIN_TOLERANCE && s < self.wall.length() - JOIN_TOLERANCE
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

/// Ein Wandende an einem Knoten: Index der Wand und welches Ende.
type Arm = (usize, End);

/// Ein Knoten aus zusammenfallenden Wandenden.
struct Node {
    point: [f64; 2],
    /// (Wandindex, Ende), gegen den Uhrzeigersinn nach Richtung sortiert.
    arms: Vec<(usize, End)>,
}

struct Nodes {
    joints: Vec<[Joint; 2]>,
    nodes: Vec<Node>,
    /// Ob die Schichten dieses Wandendes in die andere Wand eingreifen; nicht
    /// bei der Ausnahme „nicht verbinden“ und zwischen Neubau und Bestand.
    interlock: Vec<[bool; 2]>,
    /// Warum ein Ende anders angeschlossen ist als nach der Regel.
    causes: Vec<[Option<Cause>; 2]>,
}

impl Nodes {
    fn joint(&self, wall: usize, end: End) -> &Joint {
        &self.joints[wall][end as usize]
    }
}

/// Grund, warum ein Wandende nicht nach der Regel angeschlossen ist.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cause {
    /// Ausnahme am Knoten, von Hand gesetzt.
    Exception,
    /// Verschiedene Bauzustände (Neubau an Bestand).
    Phase,
    /// Verschiedene Höhen: Die höhere Wand läuft durch.
    Height,
}

fn find_nodes(walls: &[WallInfo]) -> Nodes {
    let mut joints: Vec<[Joint; 2]> = walls.iter().map(|_| [Joint::Free, Joint::Free]).collect();
    let mut interlock: Vec<[bool; 2]> = walls.iter().map(|_| [true, true]).collect();
    let mut causes: Vec<[Option<Cause>; 2]> = walls.iter().map(|_| [None, None]).collect();
    // T-Stöße: Wandende liegt in der Fläche einer anderen Wand, nicht an deren Ende.
    for (i, w) in walls.iter().enumerate() {
        for end in [End::Start, End::End] {
            let p = w.point(end);
            if !w.is_wall_end(end) {
                continue;
            }
            let through = walls.iter().enumerate().find(|(_, other)| {
                if other.id == w.id {
                    return false;
                }
                // Bei gebogenen Wänden zählt die Lage auf dem Bogen: Ein Ende genau
                // zwischen zwei Facetten trifft die Facette, die dort beginnt.
                let (s, t) = other.local(p);
                let on_facet = if other.wall.is_curved() {
                    s >= 0.0 && s < other.len && other.inside_wall(p)
                } else {
                    s > JOIN_TOLERANCE && s < other.len - JOIN_TOLERANCE
                };
                on_facet
                    && cross(w.dir, other.dir).abs() >= PARALLEL_SIN
                    && t <= other.left_face + JOIN_TOLERANCE
                    && t >= other.right_face - JOIN_TOLERANCE
                    && dist(p, [other.wall.start.x, other.wall.start.y]) > JOIN_TOLERANCE
                    && dist(p, [other.wall.end.x, other.wall.end.y]) > JOIN_TOLERANCE
            });
            if let Some((j, other)) = through {
                joints[i][end as usize] = Joint::Tee(j);
                if w.rule(end) == JoinRule::Butt {
                    interlock[i][end as usize] = false;
                    causes[i][end as usize] = Some(Cause::Exception);
                } else if !w.phase.joins_with(other.phase) {
                    interlock[i][end as usize] = false;
                    causes[i][end as usize] = Some(Cause::Phase);
                }
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
    for node in nodes.iter_mut() {
        node.arms.sort_by(|x, y| {
            let ax = angle(walls[x.0].outward(x.1));
            let ay = angle(walls[y.0].outward(y.1));
            ax.total_cmp(&ay)
        });
    }
    // Ausnahmen: Arme, die nicht verbunden werden, verlassen den Knoten und
    // stoßen stumpf an; ebenso Neubau an einem Knoten mit Bestand.
    let mut separate: Vec<(Arm, Cause, Vec<Arm>)> = Vec::new();
    for node in nodes.iter_mut() {
        let built = |a: &(usize, End)| walls[a.0].phase != Phase::New;
        let mixed = node.arms.iter().any(built) && node.arms.iter().any(|a| !built(a));
        let mut kept = Vec::new();
        let mut gone = Vec::new();
        for &(i, end) in &node.arms {
            if walls[i].rule(end) == JoinRule::Butt {
                gone.push(((i, end), Cause::Exception));
            } else if mixed && walls[i].phase == Phase::New {
                gone.push(((i, end), Cause::Phase));
            } else {
                kept.push((i, end));
            }
        }
        if !gone.is_empty() {
            node.arms = kept.clone();
            separate.extend(gone.into_iter().map(|(arm, cause)| (arm, cause, kept.clone())));
        }
    }
    for ((i, end), cause, kept) in separate {
        causes[i][end as usize] = Some(cause);
        interlock[i][end as usize] = false;
        let target = kept.iter().copied().find(|a| cross(walls[a.0].dir, walls[i].dir).abs() >= PARALLEL_SIN);
        let Some((j, ej)) = target else { continue };
        joints[i][end as usize] = Joint::Tee(j);
        // Bleibt nur eine Wand übrig, läuft sie bis an die abgewandte Fläche durch.
        if kept.len() == 1 {
            joints[j][ej as usize] = Joint::Over(i);
            causes[j][ej as usize] = Some(cause);
        }
    }
    nodes.retain(|n| n.arms.len() >= 2);
    for (k, node) in nodes.iter().enumerate() {
        for &(i, end) in &node.arms {
            joints[i][end as usize] = Joint::Node(k);
        }
    }
    // Ecken aus zwei Wänden: Durchlaufen als Ausnahme, sonst läuft bei
    // verschiedenen Höhen die höhere durch.
    for node in &nodes {
        let [(i, ei), (j, ej)] = node.arms[..] else { continue };
        if cross(walls[i].dir, walls[j].dir).abs() < PARALLEL_SIN {
            continue;
        }
        let through = |a: usize, e: End| walls[a].rule(e) == JoinRule::Through;
        let pick = match (through(i, ei), through(j, ej)) {
            (true, false) => Some(((i, ei), (j, ej), Cause::Exception)),
            (false, true) => Some(((j, ej), (i, ei), Cause::Exception)),
            _ if (walls[i].top - walls[j].top).abs() > 0.01 => Some(if walls[i].top > walls[j].top {
                ((i, ei), (j, ej), Cause::Height)
            } else {
                ((j, ej), (i, ei), Cause::Height)
            }),
            _ => None,
        };
        if let Some(((hi, he), (lo, le), cause)) = pick {
            joints[hi][he as usize] = Joint::Over(lo);
            joints[lo][le as usize] = Joint::Tee(hi);
            causes[hi][he as usize] = Some(cause);
            causes[lo][le as usize] = Some(cause);
        }
    }
    Nodes { joints, nodes, interlock, causes }
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
        Joint::Free if w.wall.is_curved() => {
            // Freies Ende einer gebogenen Wand: radial, wie der Bogen es vorgibt.
            let along = if end == End::Start { 0.0 } else { w.wall.length() };
            let (tx, ty) = w.wall.tangent_at(along);
            let p = w.point(end);
            let right = intersect_lines(w.at(0.0, w.right_face), w.dir, p, [-ty, tx]);
            let left = intersect_lines(w.at(0.0, w.left_face), w.dir, p, [-ty, tx]);
            match (right, left) {
                (Some(r), Some(l)) => vec![r, l],
                _ => perpendicular,
            }
        }
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
    // Facetten derselben Wand stoßen immer auf Gehrung, auch bei kleinem Knick.
    let facets = wa.id == wb.id;
    if a.0 == b.0 || (!facets && cross(wa.dir, wb.dir).abs() < PARALLEL_SIN) {
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
            if !w.present || !nodes.interlock[i][end as usize] {
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
                if !o.present || o.id == w.id || cross(w.dir, o.dir).abs() < PARALLEL_SIN {
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
        .filter_map(|(id, w)| {
            Some(WallInfo::facets(id, w, project.wall_type(w.wall_type)?, project.wall_extent(w)?.1, true))
        })
        .flatten()
        .collect();
    // Knoten zwischen Facetten derselben Wand zählen nicht.
    find_nodes(&walls)
        .nodes
        .iter()
        .filter(|n| n.arms.iter().any(|a| walls[a.0].id != walls[n.arms[0].0].id))
        .map(|n| (n.point, n.arms.len()))
        .collect()
}

/// Rolle einer Wand an einem Anschluss.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArmRole {
    /// Trifft die anderen Wände auf Gehrung.
    Mitre,
    /// Läuft durch, die anderen stoßen an.
    Through,
    /// Stößt an, die Schichten greifen nach ihrem Rang ineinander.
    Butts,
    /// Stößt stumpf an, ohne Verbindung.
    Separate,
}

/// Eine Wand an einem Anschluss.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct JunctionArm {
    pub wall: ElementId,
    /// Das Ende am Anschluss; `None` bei der Wand, die an einem T-Stoß durchläuft.
    pub end: Option<WallEnd>,
    pub role: ArmRole,
    /// Warum die Wand anders angeschlossen ist als nach der Regel.
    pub cause: Option<Cause>,
}

/// Ein Anschluss im Grundriss, auswählbar wie ein Bauteil (ADR 0045).
#[derive(Debug, Clone, PartialEq)]
pub struct Junction {
    pub point: Point2,
    pub arms: Vec<JunctionArm>,
}

/// Schicht einer Wand.
#[derive(Debug, Clone, PartialEq)]
pub struct LayerRef {
    pub wall: ElementId,
    pub layer: usize,
    pub material: String,
    pub function: LayerFunction,
}

/// Zwei Schichten verschiedener Wände, die sich an einem Anschluss berühren.
#[derive(Debug, Clone, PartialEq)]
pub struct Contact {
    pub first: LayerRef,
    pub second: LayerRef,
    /// Ob `first` die Schicht `second` schneidet (höherer Rang), sonst stoßen beide aneinander.
    pub cuts: bool,
}

/// Alle Anschlüsse eines Geschosses.
pub fn junctions(project: &Project, storey: ElementId) -> Vec<Junction> {
    let walls: Vec<WallInfo> = project
        .walls_on(storey)
        .filter_map(|(id, w)| {
            Some(WallInfo::facets(id, w, project.wall_type(w.wall_type)?, project.wall_extent(w)?.1, true))
        })
        .flatten()
        .collect();
    let nodes = find_nodes(&walls);
    let mut out: Vec<Junction> = Vec::new();
    let mut add = |p: [f64; 2], arm: JunctionArm| {
        let k = match out.iter().position(|j| dist([j.point.x, j.point.y], p) <= JOIN_TOLERANCE) {
            Some(k) => k,
            None => {
                out.push(Junction { point: Point2::new(p[0], p[1]), arms: Vec::new() });
                out.len() - 1
            }
        };
        if !out[k].arms.iter().any(|a| a.wall == arm.wall && a.end == arm.end) {
            out[k].arms.push(arm);
        }
    };
    let wall_end = |end: End| match end {
        End::Start => WallEnd::Start,
        End::End => WallEnd::End,
    };
    for (i, w) in walls.iter().enumerate() {
        for end in [End::Start, End::End] {
            if !w.is_wall_end(end) {
                continue;
            }
            let p = w.point(end);
            let cause = nodes.causes[i][end as usize];
            let arm = |role| JunctionArm { wall: w.id, end: Some(wall_end(end)), role, cause };
            match nodes.joint(i, end) {
                Joint::Free => {}
                Joint::Node(_) => add(p, arm(ArmRole::Mitre)),
                Joint::Over(_) => add(p, arm(ArmRole::Through)),
                Joint::Tee(j) => {
                    add(p, arm(if nodes.interlock[i][end as usize] { ArmRole::Butts } else { ArmRole::Separate }));
                    let o = &walls[*j];
                    let at_end = [o.wall.start, o.wall.end].iter().any(|q| dist(p, [q.x, q.y]) <= JOIN_TOLERANCE);
                    if !at_end {
                        add(p, JunctionArm { wall: o.id, end: None, role: ArmRole::Through, cause: None });
                    }
                }
            }
        }
    }
    out.retain(|j| j.arms.len() >= 2);
    out
}

/// Der Anschluss nächst `p`, wenn der Punkt höchstens `tolerance` oder eine Wanddicke entfernt liegt.
pub fn junction_at(project: &Project, storey: ElementId, p: Point2, tolerance: f64) -> Option<Junction> {
    let reach = |j: &Junction| {
        j.arms
            .iter()
            .filter_map(|a| project.wall(a.wall).and_then(|w| project.wall_type(w.wall_type)))
            .map(|t| t.thickness())
            .fold(tolerance, f64::max)
    };
    junctions(project, storey)
        .into_iter()
        .filter(|j| j.point.distance(p) <= reach(j))
        .min_by(|a, b| a.point.distance(p).total_cmp(&b.point.distance(p)))
}

/// Welche Schichten sich an einem Anschluss berühren, in einer Höhe, in der alle Wände stehen.
pub fn contacts(project: &Project, storey: ElementId, junction: &Junction) -> Vec<Contact> {
    let ids: Vec<ElementId> = junction.arms.iter().map(|a| a.wall).collect();
    let extents: Vec<(f64, f64)> =
        ids.iter().filter_map(|id| project.wall(*id).and_then(|w| project.wall_extent(w))).collect();
    let (Some(low), Some(high)) =
        (extents.iter().map(|e| e.0).reduce(f64::max), extents.iter().map(|e| e.1).reduce(f64::min))
    else {
        return Vec::new();
    };
    let z = if low < high { (low + high) / 2.0 } else { low + 0.01 };
    let thickness = ids
        .iter()
        .filter_map(|id| project.wall(*id).and_then(|w| project.wall_type(w.wall_type)))
        .map(|t| t.thickness())
        .fold(0.0, f64::max);
    let r = 3.0 * thickness + 0.3;
    let p = [junction.point.x, junction.point.y];
    let local = strip(p, [1.0, 0.0], (-r, r), (-r, r));
    let regions: Vec<LayerRegion> = solid_regions(project, storey, z)
        .into_iter()
        .filter(|g| ids.contains(&g.element) && g.function.is_some())
        .map(|mut g| {
            g.area = g.area.intersection(&local);
            g
        })
        .filter(|g| !g.area.is_empty())
        .collect();
    let loose: Vec<ElementId> = junction.arms.iter().filter(|a| a.role == ArmRole::Separate).map(|a| a.wall).collect();
    let layer = |g: &LayerRegion| LayerRef {
        wall: g.element,
        layer: g.layer,
        material: g.material.clone(),
        function: g.function.unwrap_or(LayerFunction::Structure),
    };
    let mut out = Vec::new();
    for (k, a) in regions.iter().enumerate() {
        let grown = a.area.offset(0.002);
        for b in &regions[k + 1..] {
            if a.element == b.element || grown.intersection(&b.area).area() < 1e-6 {
                continue;
            }
            let free = !loose.contains(&a.element) && !loose.contains(&b.element);
            let (first, second, cuts) = match a.priority().cmp(&b.priority()) {
                std::cmp::Ordering::Less if free => (a, b, true),
                std::cmp::Ordering::Greater if free => (b, a, true),
                _ => (a, b, false),
            };
            out.push(Contact { first: layer(first), second: layer(second), cuts });
        }
    }
    out
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
