//! Schnitte und Ansichten (3.02). Vorschlag nach DIN 1356-1:1995-02:
//!
//! - Geschnittene Bauteile wie im Grundriss: Muster je Material und
//!   Detailgrad, Schnittkante breit
//! - Sichtbare Kanten dahinter mittel, verdeckte Kanten entfallen
//! - Gelände als breite Linie, darunter keine Ansichtskanten
//! - Höhenkoten links neben dem Bild: OKFF jedes Geschosses, Traufe und First
//!
//! Verdeckte Kanten werden über einen Tiefenpuffer entfernt: Alle Dreiecke
//! hinter der Schnittebene werden in ein Raster von 2 cm gezeichnet, jede Kante
//! wird abgetastet und nur dort gezeichnet, wo nichts davor liegt.

use std::collections::HashMap;

use lot_geom::{Area, Band, LayerRegion, ModelGeometry, TriMesh};
use lot_model::{Element, ElementId, LayerFunction, Project, SectionKind, SectionView};

use crate::floor_plan::draw_cut_for;
use crate::standard::{OfficeStandard, ScaleStyle};
use crate::{Anchor, Color, Line, LineStyle, PaperPoint, Pen, PlanObject, Text, Viewport};

/// Geländehöhe über ±0,00, bis das Gelände im Modell ist (M4). Vorschlag, Ben prüft.
pub const GROUND: f64 = -0.30;
/// Rasterweite des Tiefenpuffers, in Metern.
const RESOLUTION: f64 = 0.02;
/// Flächen, die um weniger als diesen Winkel abknicken, bilden keine Kante (Facetten von Bögen).
const EDGE_ANGLE: f64 = 20.0;
/// Spielraum, mit dem eine Kante noch als sichtbar gilt, in Metern.
const DEPTH_TOLERANCE: f64 = 0.03;

/// Lage eines Schnitts: Ursprung, Blickrichtung und Richtung nach rechts im Bild.
#[derive(Clone, Copy)]
struct Frame {
    origin: [f64; 2],
    look: [f64; 2],
    right: [f64; 2],
}

impl Frame {
    fn new(view: &SectionView) -> Self {
        let (look, right) = view.frame();
        Self { origin: [view.start.x, view.start.y], look: [look.0, look.1], right: [right.0, right.1] }
    }

    /// Punkt im Grundriss zu (u, w).
    fn point(&self, u: f64, w: f64) -> [f64; 2] {
        [self.origin[0] + self.right[0] * u + self.look[0] * w, self.origin[1] + self.right[1] * u + self.look[1] * w]
    }

    /// (u, v, w): nach rechts im Bild, Höhe, Tiefe hinter der Schnittebene.
    fn uvw(&self, p: [f64; 3]) -> [f64; 3] {
        let d = [p[0] - self.origin[0], p[1] - self.origin[1]];
        [d[0] * self.right[0] + d[1] * self.right[1], p[2], d[0] * self.look[0] + d[1] * self.look[1]]
    }
}

/// Ausdehnung des Bildes in (u, v), mit den Höhenkoten links daneben.
pub fn section_extent(geometry: &ModelGeometry, view: &SectionView) -> Option<([f64; 2], [f64; 2])> {
    let frame = Frame::new(view);
    let mut lo = [f64::INFINITY; 2];
    let mut hi = [f64::NEG_INFINITY; 2];
    // In Ansichten zählt das Erdreich nicht mit, sein Rand liegt weit draußen.
    let terrain = |e: &&lot_geom::ElementGeometry| e.material == lot_geom::terrain::TERRAIN_MATERIAL;
    for e in geometry.elements.iter().filter(|e| view.kind == SectionKind::Section || !terrain(e)) {
        for p in &e.mesh.positions {
            let [u, v, w] = frame.uvw(*p);
            if (-1e-6..=view.depth).contains(&w) {
                lo = [lo[0].min(u), lo[1].min(v)];
                hi = [hi[0].max(u), hi[1].max(v)];
            }
        }
    }
    if !lo[0].is_finite() {
        return None;
    }
    lo[1] = lo[1].min(GROUND);
    let right = if view.kind == SectionKind::Section { WALL_HEIGHT_OFFSET + 1.0 } else { 0.0 };
    Some(([lo[0] - MARK_SPACE, lo[1]], [hi[0] + right, hi[1]]))
}

/// Tiefe der vordersten Fläche des Gebäudes (ohne Erdreich), für die Geländelinie in Ansichten.
fn building_front(meshes: &[(&lot_geom::ElementGeometry, Vec<[f64; 3]>)], depth: f64) -> f64 {
    meshes
        .iter()
        .filter(|(e, _)| e.material != lot_geom::terrain::TERRAIN_MATERIAL)
        .flat_map(|(_, uvw)| uvw.iter().map(|p| p[2]))
        .filter(|w| (0.0..=depth).contains(w))
        .fold(f64::INFINITY, f64::min)
        .min(depth)
}

/// Platz links neben dem Bild für die Höhenkoten und die Wandhöhe, in Metern.
const MARK_SPACE: f64 = 4.5;
/// Abstand der Höhenkoten links vom Bild, in Metern.
const MARK_OFFSET: f64 = 1.6;
/// Abstand der Maßlinie für die Wandhöhe von der Außenfläche, in Metern.
const WALL_HEIGHT_OFFSET: f64 = 0.6;

/// Zeichnet einen Schnitt oder eine Ansicht. `viewport` bildet (u, v) in Metern
/// auf das Blatt ab.
pub fn section(
    project: &Project,
    geometry: &ModelGeometry,
    view: &SectionView,
    standard: &OfficeStandard,
    viewport: Viewport,
) -> Vec<PlanObject> {
    let style = standard.style_for(viewport.scale);
    let frame = Frame::new(view);
    let mut objects = Vec::new();

    // Alle Netze ins System des Schnitts.
    let meshes: Vec<(&lot_geom::ElementGeometry, Vec<[f64; 3]>)> =
        geometry.elements.iter().map(|e| (e, e.mesh.positions.iter().map(|p| frame.uvw(*p)).collect())).collect();

    // 1. Geschnittene Bauteile.
    let cut: Vec<LayerRegion> = if view.kind == SectionKind::Section {
        meshes
            .iter()
            .filter_map(|(e, uvw)| {
                let area = slice(&e.mesh, uvw);
                if area.is_empty() {
                    return None;
                }
                Some(LayerRegion {
                    element: e.id,
                    layer: e.layer,
                    material: e.material.clone(),
                    function: layer_function(project, e.id, e.layer),
                    band: band_of(&area),
                    area,
                })
            })
            .collect()
    } else {
        Vec::new()
    };
    // Erdreich nur so weit, wie die Schnittlinie reicht.
    let end_u = frame.uvw([view.end.x, view.end.y, 0.0])[0];
    let (u0, u1) = (end_u.min(0.0), end_u.max(0.0));
    let cut: Vec<LayerRegion> = {
        let window = Area::from_ring(&[[u0, -1e4], [u1, -1e4], [u1, 1e4], [u0, 1e4]]);
        cut.into_iter()
            .filter_map(|mut r| {
                if r.material == lot_geom::terrain::TERRAIN_MATERIAL {
                    r.area = r.area.intersection(&window);
                }
                (!r.area.is_empty()).then_some(r)
            })
            .collect()
    };
    let cut_all = Area::union_all(cut.iter().map(|r| &r.area));

    // 2. Sichtbare Kanten dahinter.
    let mut buffer = DepthBuffer::new(&meshes, view.depth);
    // Mit Gelände verdeckt das Erdreich selbst, was darunter liegt.
    let terrain = project.terrain();
    buffer.underground = match (terrain, view.kind) {
        // Im Schnitt mit Gelände zeigt das geschnittene Erdreich, was unter dem Gelände liegt.
        (Some(_), SectionKind::Section) => Some((f64::NEG_INFINITY, f64::INFINITY)),
        (Some(_), SectionKind::Elevation) => None,
        (None, _) => cut_all.bounds().map(|(lo, hi)| (lo[0], hi[0])),
    };
    for (e, uvw) in &meshes {
        for t in &e.mesh.triangles {
            buffer.fill(t.map(|i| uvw[i as usize]));
        }
    }
    // Geländelinie: ohne Geländemodell auf -0,30 m; mit Gelände in Ansichten
    // vor dem Gebäude, im Schnitt keine (das Erdreich ist selbst geschnitten).
    let ground: Vec<[f64; 2]> = match (buffer.extent(), terrain) {
        (None, _) => Vec::new(),
        (Some((lo, hi)), None) => vec![[lo[0] - 1.0, GROUND], [hi[0] + 1.0, GROUND]],
        (Some(_), Some(_)) if view.kind == SectionKind::Section => Vec::new(),
        (Some((lo, hi)), Some(t)) => {
            // Nicht über die Schnittlinie hinaus: das Gelände reicht oft weit.
            let (from, to) = ((lo[0] - 1.0).max(u0), (hi[0] + 1.0).min(u1));
            let front = building_front(&meshes, view.depth);
            let tris = t.triangles();
            let steps = ((to - from) / 0.1).ceil().max(1.0) as usize;
            (0..=steps)
                .filter_map(|i| {
                    let u = from + (to - from) * i as f64 / steps as f64;
                    let p = frame.point(u, front);
                    t.height_with(&tris, lot_model::Point2::new(p[0], p[1])).map(|z| [u, z])
                })
                .collect()
        }
    };
    if terrain.is_some() && view.kind == SectionKind::Elevation {
        buffer.ground = Some(ground.clone());
    }
    let view_pen = Pen::mm(style.pens.view);
    // Kanten des Erdreichs zeichnet nur der Schnitt (als Schnittfläche) und die Geländelinie.
    let edge_meshes: Vec<(&lot_geom::ElementGeometry, Vec<[f64; 3]>)> = meshes
        .iter()
        .filter(|(e, _)| e.material != lot_geom::terrain::TERRAIN_MATERIAL)
        .map(|(e, uvw)| (*e, uvw.clone()))
        .collect();
    for [a, b] in feature_edges(&edge_meshes) {
        for [p, q] in buffer.visible_parts(a, b) {
            objects.push(PlanObject::Line(Line {
                points: vec![viewport.to_paper(p[0], p[1]), viewport.to_paper(q[0], q[1])],
                closed: false,
                pen: view_pen,
                style: LineStyle::Solid,
                color: Color::BLACK,
            }));
        }
    }

    // 3. Schnittflächen über den Ansichtskanten.
    draw_cut_for(&mut objects, project, &cut, None, standard, style, viewport);

    // 4. Geländelinie, außerhalb der geschnittenen Bauteile.
    if let Some((lo, _)) = buffer.extent() {
        for w in ground.windows(2) {
            for [a, b] in outside_segments(w[0], w[1], &cut_all) {
                objects.push(PlanObject::Line(Line {
                    points: vec![viewport.to_paper(a[0], a[1]), viewport.to_paper(b[0], b[1])],
                    closed: false,
                    pen: Pen::mm(style.pens.cut),
                    style: LineStyle::Solid,
                    color: Color::BLACK,
                }));
            }
        }
        // 5. Höhenkoten.
        draw_level_marks(&mut objects, project, view.kind, lo[0] - MARK_OFFSET, style, viewport);
    }

    // 7. Angaben für den Bauantrag (MBauVorlV § 8 Abs. 2 Nr. 3): Wandhöhe an den
    //    geschnittenen Außenwänden, lichte Raumhöhen, Steigungsverhältnis der Treppen.
    if view.kind == SectionKind::Section {
        draw_submission_info(&mut objects, project, view, &cut, style, viewport);
    }

    // 6. 2D-Elemente, x entlang der Schnittlinie und y als Höhe.
    if let Some(id) = project.section_views().find(|(_, v)| *v == view).map(|(id, _)| id) {
        let annotations = lot_model::AnnotationView::Section { view: id };
        crate::annotations::draw_annotations(
            &mut objects,
            project,
            annotations,
            standard,
            style,
            crate::annotations::model(viewport),
        );
    }
    objects
}

/// Kante zwischen zwei auf 10 µm gerundeten Eckpunkten.
type EdgeKey = ([i64; 3], [i64; 3]);
/// Eine Kante und die Normalen aller Dreiecke an ihr.
type EdgeFaces = ([[f64; 3]; 2], Vec<[f64; 3]>);

/// Kanten über alle Körper gemeinsam: Eine Kante zählt, wo die Flächen an ihr
/// um mehr als `EDGE_ANGLE` abknicken oder ein Rand ist. Laufen zwei Flächen,
/// auch aus verschiedenen Körpern, bündig weiter (Fassade über zwei Geschosse,
/// Schichten an einer Wandstirn), entsteht keine Linie.
fn feature_edges(meshes: &[(&lot_geom::ElementGeometry, Vec<[f64; 3]>)]) -> Vec<[[f64; 3]; 2]> {
    let quant = |p: [f64; 3]| [(p[0] * 1e5).round() as i64, (p[1] * 1e5).round() as i64, (p[2] * 1e5).round() as i64];
    let mut faces: HashMap<EdgeKey, EdgeFaces> = HashMap::new();
    for (e, uvw) in meshes {
        for t in &e.mesh.triangles {
            let [a, b, c] = t.map(|i| uvw[i as usize]);
            let n = cross(sub(b, a), sub(c, a));
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            if len < 1e-12 {
                continue;
            }
            let n = [n[0] / len, n[1] / len, n[2] / len];
            for (p, q) in [(a, b), (b, c), (c, a)] {
                let (kp, kq) = (quant(p), quant(q));
                let key = if kp <= kq { (kp, kq) } else { (kq, kp) };
                faces.entry(key).or_insert(([p, q], Vec::new())).1.push(n);
            }
        }
    }
    let cos_limit = EDGE_ANGLE.to_radians().cos();
    faces
        .into_values()
        .filter(|(_, normals)| {
            if normals.len() < 2 {
                return true;
            }
            // Bündig weiterlaufende Flächen: zwei Normalen fast gleich.
            let smooth =
                (0..normals.len()).any(|i| (i + 1..normals.len()).any(|j| dot(normals[i], normals[j]) >= cos_limit));
            !smooth
        })
        .map(|(edge, _)| edge)
        .collect()
}

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Funktion der Schicht eines Elements, für die Muster je Detailgrad.
fn layer_function(project: &Project, id: ElementId, layer: usize) -> Option<LayerFunction> {
    match project.elements.get(&id)? {
        Element::Wall(w) => project.wall_type(w.wall_type)?.layers.get(layer).map(|l| l.function),
        Element::Slab(s) => project.slab_type(s.slab_type)?.layers.get(layer).map(|l| l.function),
        Element::Roof(r) => project.slab_type(r.roof_type)?.layers.get(layer).map(|l| l.function),
        Element::Dormer(d) => match layer.checked_sub(lot_geom::dormers::DORMER_WALL_LAYER) {
            Some(j) if j < 10 => project.wall_type(d.wall_type)?.layers.get(j).map(|l| l.function),
            Some(_) => None,
            None => {
                project.roof(d.roof).and_then(|r| project.slab_type(r.roof_type))?.layers.get(layer).map(|l| l.function)
            }
        },
        _ => None,
    }
}

/// Lage einer geschnittenen Schicht für Muster entlang der Schicht (Dämmung):
/// Richtung der längsten Kante, Grenzen quer dazu.
fn band_of(area: &Area) -> Option<Band> {
    let rings = area.rings();
    let ring = rings.iter().max_by(|a, b| lot_geom::signed_area(a).abs().total_cmp(&lot_geom::signed_area(b).abs()))?;
    let n = ring.len();
    let (i, _) = (0..n)
        .map(|i| (i, (ring[(i + 1) % n][0] - ring[i][0]).hypot(ring[(i + 1) % n][1] - ring[i][1])))
        .max_by(|a, b| a.1.total_cmp(&b.1))?;
    let (a, b) = (ring[i], ring[(i + 1) % n]);
    let len = (b[0] - a[0]).hypot(b[1] - a[1]).max(1e-12);
    let d = [(b[0] - a[0]) / len, (b[1] - a[1]) / len];
    let across = |p: &[f64; 2]| -(p[0] - a[0]) * d[1] + (p[1] - a[1]) * d[0];
    let (lo, hi) = ring.iter().map(across).fold((f64::INFINITY, f64::NEG_INFINITY), |(l, h), t| (l.min(t), h.max(t)));
    Some(Band { origin: a, direction: d, left: hi, right: lo, arc: None })
}

/// Schnitt eines geschlossenen Netzes mit der Ebene w = 0, als Fläche in (u, v).
pub(crate) fn slice(mesh: &TriMesh, uvw: &[[f64; 3]]) -> Area {
    // Punkte genau auf der Ebene leicht verschieben, damit jede Kante eindeutig schneidet.
    let w: Vec<f64> = uvw.iter().map(|p| if p[2].abs() < 1e-9 { 1e-9 } else { p[2] }).collect();
    let mut next: HashMap<(u32, u32), (u32, u32)> = HashMap::new();
    let mut points: HashMap<(u32, u32), [f64; 2]> = HashMap::new();
    let key = |a: u32, b: u32| (a.min(b), a.max(b));
    for t in &mesh.triangles {
        // Einheitlich ausgerichtet: von der Kante, an der w negativ wird, zu der,
        // an der es wieder positiv wird. Nachbardreiecke durchlaufen ihre
        // gemeinsame Kante gegenläufig, so passt jedes Ende an genau einen Anfang.
        let mut enter = None;
        let mut leave = None;
        for k in 0..3 {
            let (a, b) = (t[k], t[(k + 1) % 3]);
            let (wa, wb) = (w[a as usize], w[b as usize]);
            if (wa > 0.0) == (wb > 0.0) {
                continue;
            }
            let (pa, pb) = (uvw[a as usize], uvw[b as usize]);
            let s = wa / (wa - wb);
            points.entry(key(a, b)).or_insert([pa[0] + (pb[0] - pa[0]) * s, pa[1] + (pb[1] - pa[1]) * s]);
            if wa > 0.0 {
                enter = Some(key(a, b));
            } else {
                leave = Some(key(a, b));
            }
        }
        if let (Some(e), Some(l)) = (enter, leave) {
            next.insert(e, l);
        }
    }
    // Ringe verketten: jede Kante zeigt auf die nächste.
    let mut rings = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for &start in next.keys() {
        if seen.contains(&start) {
            continue;
        }
        let mut ring = Vec::new();
        let mut k = start;
        while seen.insert(k) {
            ring.push(points[&k]);
            match next.get(&k) {
                Some(&n) => k = n,
                None => break,
            }
        }
        if ring.len() >= 3 {
            rings.push(ring);
        }
    }
    Area::from_rings(&rings)
}

/// Tiefenpuffer in (u, v) für den Bereich hinter der Schnittebene.
struct DepthBuffer {
    origin: [f64; 2],
    size: [usize; 2],
    res: f64,
    depth: Vec<f64>,
    max_w: f64,
    /// Bereich in u, in dem Kanten unter dem Gelände sichtbar sind: im Schnitt
    /// innerhalb des geschnittenen Gebäudes (Kellerräume), in Ansichten nirgends.
    underground: Option<(f64, f64)>,
    /// Geländelinie (u, Höhe) in Ansichten mit Gelände; darunter ist nichts zu sehen.
    /// Ohne sie gilt die feste Linie auf -0,30 m.
    ground: Option<Vec<[f64; 2]>>,
}

impl DepthBuffer {
    fn new(meshes: &[(&lot_geom::ElementGeometry, Vec<[f64; 3]>)], max_w: f64) -> Self {
        let mut lo = [f64::INFINITY; 2];
        let mut hi = [f64::NEG_INFINITY; 2];
        for (_, uvw) in meshes {
            for p in uvw {
                if p[2] >= -1e-6 && p[2] <= max_w {
                    lo = [lo[0].min(p[0]), lo[1].min(p[1])];
                    hi = [hi[0].max(p[0]), hi[1].max(p[1])];
                }
            }
        }
        if !lo[0].is_finite() {
            return Self {
                origin: [0.0, 0.0],
                size: [0, 0],
                res: RESOLUTION,
                depth: Vec::new(),
                max_w,
                underground: None,
                ground: None,
            };
        }
        // Rand für Dreiecke, die erst nach dem Kappen an der Ebene hineinreichen.
        let pad = 1.0;
        let (lo, hi) = ([lo[0] - pad, lo[1] - pad], [hi[0] + pad, hi[1] + pad]);
        let res = RESOLUTION.max((hi[0] - lo[0]).max(hi[1] - lo[1]) / 4000.0);
        let size = [((hi[0] - lo[0]) / res).ceil() as usize + 1, ((hi[1] - lo[1]) / res).ceil() as usize + 1];
        Self {
            origin: lo,
            size,
            res,
            depth: vec![f64::INFINITY; size[0] * size[1]],
            max_w,
            underground: None,
            ground: None,
        }
    }

    /// Ausdehnung der gezeichneten Fläche (ohne Rand).
    fn extent(&self) -> Option<([f64; 2], [f64; 2])> {
        let mut lo = [usize::MAX; 2];
        let mut hi = [0usize; 2];
        for j in 0..self.size[1] {
            for i in 0..self.size[0] {
                if self.depth[j * self.size[0] + i].is_finite() {
                    lo = [lo[0].min(i), lo[1].min(j)];
                    hi = [hi[0].max(i), hi[1].max(j)];
                }
            }
        }
        if lo[0] == usize::MAX {
            return None;
        }
        let at = |i: usize, j: usize| [self.origin[0] + i as f64 * self.res, self.origin[1] + j as f64 * self.res];
        Some((at(lo[0], lo[1]), at(hi[0], hi[1])))
    }

    /// Höhe des Geländes an der Stelle u.
    fn ground_at(&self, u: f64) -> f64 {
        let Some(g) = &self.ground else { return GROUND };
        match g.iter().position(|p| p[0] >= u) {
            Some(0) => g[0][1],
            Some(i) => {
                let (a, b) = (g[i - 1], g[i]);
                let t = if (b[0] - a[0]).abs() < 1e-12 { 0.0 } else { (u - a[0]) / (b[0] - a[0]) };
                a[1] + (b[1] - a[1]) * t
            }
            None => g.last().map_or(GROUND, |p| p[1]),
        }
    }

    /// Zeichnet ein Dreieck, gekappt auf 0 ≤ w ≤ Blicktiefe.
    fn fill(&mut self, t: [[f64; 3]; 3]) {
        if self.size[0] == 0 {
            return;
        }
        let poly = clip(clip(t.to_vec(), |p| p[2]), |p| self.max_w - p[2]);
        for k in 1..poly.len().saturating_sub(1) {
            self.raster([poly[0], poly[k], poly[k + 1]]);
        }
    }

    fn raster(&mut self, t: [[f64; 3]; 3]) {
        let px = |p: [f64; 3]| [(p[0] - self.origin[0]) / self.res, (p[1] - self.origin[1]) / self.res, p[2]];
        let [a, b, c] = t.map(px);
        let area = (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
        if area.abs() < 1e-12 {
            return;
        }
        let (x0, x1) = (a[0].min(b[0]).min(c[0]).floor().max(0.0), a[0].max(b[0]).max(c[0]).ceil());
        let (y0, y1) = (a[1].min(b[1]).min(c[1]).floor().max(0.0), a[1].max(b[1]).max(c[1]).ceil());
        let (x1, y1) = (x1.min(self.size[0] as f64 - 1.0), y1.min(self.size[1] as f64 - 1.0));
        let mut y = y0;
        while y <= y1 {
            let mut x = x0;
            while x <= x1 {
                let (sx, sy) = (x, y);
                let w0 = ((b[0] - sx) * (c[1] - sy) - (b[1] - sy) * (c[0] - sx)) / area;
                let w1 = ((c[0] - sx) * (a[1] - sy) - (c[1] - sy) * (a[0] - sx)) / area;
                let w2 = 1.0 - w0 - w1;
                if w0 >= -1e-9 && w1 >= -1e-9 && w2 >= -1e-9 {
                    let d = w0 * a[2] + w1 * b[2] + w2 * c[2];
                    let k = y as usize * self.size[0] + x as usize;
                    if d < self.depth[k] {
                        self.depth[k] = d;
                    }
                }
                x += 1.0;
            }
            y += 1.0;
        }
    }

    /// Ob ein Punkt (u, v, w) sichtbar ist: Irgendwo in seiner Umgebung liegt
    /// nichts deutlich davor.
    fn visible(&self, p: [f64; 3]) -> bool {
        let i = ((p[0] - self.origin[0]) / self.res).round() as i64;
        let j = ((p[1] - self.origin[1]) / self.res).round() as i64;
        for dj in -1..=1 {
            for di in -1..=1 {
                let (x, y) = (i + di, j + dj);
                if x < 0 || y < 0 || x >= self.size[0] as i64 || y >= self.size[1] as i64 {
                    return true;
                }
                if self.depth[y as usize * self.size[0] + x as usize] >= p[2] - DEPTH_TOLERANCE {
                    return true;
                }
            }
        }
        false
    }

    /// Die sichtbaren Stücke einer Kante, in (u, v), über dem Gelände.
    fn visible_parts(&self, a: [f64; 3], b: [f64; 3]) -> Vec<[[f64; 2]; 2]> {
        // Auf 0 ≤ w ≤ Blicktiefe kappen; Kanten in der Schnittebene zeichnet die Schnittfläche.
        let Some((t0, t1)) = clip_param(a[2], b[2], 1e-4, self.max_w) else { return Vec::new() };
        let at = |t: f64| [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t];
        let (p, q) = (at(t0), at(t1));
        let len = (q[0] - p[0]).hypot(q[1] - p[1]);
        if len < 1e-6 {
            return Vec::new();
        }
        let steps = ((len / (self.res / 2.0)).ceil() as usize).max(1);
        let mut parts = Vec::new();
        let mut run: Option<f64> = None;
        let mut last = 0.0;
        for k in 0..=steps {
            let s = k as f64 / steps as f64;
            let r = [p[0] + (q[0] - p[0]) * s, p[1] + (q[1] - p[1]) * s, p[2] + (q[2] - p[2]) * s];
            let underground = self.underground.is_some_and(|(u0, u1)| r[0] > u0 && r[0] < u1);
            let shown = (r[1] >= self.ground_at(r[0]) - 1e-6 || underground) && self.visible(r);
            match (shown, run) {
                (true, None) => run = Some(s),
                (false, Some(s0)) => {
                    parts.push((s0, last));
                    run = None;
                }
                _ => {}
            }
            last = s;
        }
        if let Some(s0) = run {
            parts.push((s0, 1.0));
        }
        parts
            .into_iter()
            .filter(|(s0, s1)| (s1 - s0) * len > self.res)
            .map(|(s0, s1)| {
                let uv = |s: f64| [p[0] + (q[0] - p[0]) * s, p[1] + (q[1] - p[1]) * s];
                [uv(s0), uv(s1)]
            })
            .collect()
    }
}

/// Polygon an der Halbebene f(p) ≥ 0 kappen (Sutherland-Hodgman).
fn clip(poly: Vec<[f64; 3]>, f: impl Fn(&[f64; 3]) -> f64) -> Vec<[f64; 3]> {
    let n = poly.len();
    let mut out = Vec::with_capacity(n + 2);
    for i in 0..n {
        let (a, b) = (poly[i], poly[(i + 1) % n]);
        let (fa, fb) = (f(&a), f(&b));
        if fa >= 0.0 {
            out.push(a);
        }
        if (fa >= 0.0) != (fb >= 0.0) {
            let s = fa / (fa - fb);
            out.push([a[0] + (b[0] - a[0]) * s, a[1] + (b[1] - a[1]) * s, a[2] + (b[2] - a[2]) * s]);
        }
    }
    out
}

/// Bereich des Parameters t ∈ [0, 1], in dem w zwischen `lo` und `hi` liegt.
fn clip_param(wa: f64, wb: f64, lo: f64, hi: f64) -> Option<(f64, f64)> {
    let (mut t0, mut t1) = (0.0f64, 1.0f64);
    for (bound, sign) in [(lo, 1.0), (hi, -1.0)] {
        // sign · (w(t) − bound) ≥ 0
        let fa = sign * (wa - bound);
        let fb = sign * (wb - bound);
        if fa < 0.0 && fb < 0.0 {
            return None;
        }
        if fa < 0.0 {
            t0 = t0.max(fa / (fa - fb));
        } else if fb < 0.0 {
            t1 = t1.min(fa / (fa - fb));
        }
    }
    (t1 - t0 > 1e-9).then_some((t0, t1))
}

/// Die Teile der Strecke von `a` nach `b` außerhalb der Fläche.
fn outside_segments(a: [f64; 2], b: [f64; 2], area: &Area) -> Vec<[[f64; 2]; 2]> {
    let len = (b[0] - a[0]).hypot(b[1] - a[1]);
    let steps = ((len / 0.01).ceil() as usize).max(1);
    let mut parts = Vec::new();
    let mut start: Option<[f64; 2]> = None;
    let mut last = a;
    for k in 0..=steps {
        let s = k as f64 / steps as f64;
        let p = [a[0] + (b[0] - a[0]) * s, a[1] + (b[1] - a[1]) * s];
        let outside = !area.contains(p);
        match (outside, start) {
            (true, None) => start = Some(p),
            (false, Some(s0)) => {
                parts.push([s0, last]);
                start = None;
            }
            _ => {}
        }
        last = p;
    }
    if let Some(s0) = start {
        parts.push([s0, b]);
    }
    parts
}

/// Bereiche einer Fläche auf der senkrechten Linie bei `u`, als (unten, oben), aufsteigend.
fn vertical_runs(area: &Area, u: f64) -> Vec<(f64, f64)> {
    let mut hits: Vec<f64> = Vec::new();
    for ring in area.rings() {
        for i in 0..ring.len() {
            let (a, b) = (ring[i], ring[(i + 1) % ring.len()]);
            if (a[0] <= u && u < b[0]) || (b[0] <= u && u < a[0]) {
                hits.push(a[1] + (u - a[0]) / (b[0] - a[0]) * (b[1] - a[1]));
            }
        }
    }
    hits.sort_by(f64::total_cmp);
    hits.as_chunks::<2>().0.iter().map(|p| (p[0], p[1])).collect()
}

/// Wandhöhe H an beiden Außenseiten, lichte Raumhöhe je Geschoss und das
/// Steigungsverhältnis jeder geschnittenen Treppe.
fn draw_submission_info(
    objects: &mut Vec<PlanObject>,
    project: &Project,
    view: &SectionView,
    cut: &[LayerRegion],
    style: &ScaleStyle,
    viewport: Viewport,
) {
    let frame = Frame::new(view);
    let pen = Pen::mm(style.pens.annotation);
    let size = style.space_text;
    let building: Vec<&Area> = cut
        .iter()
        // Ohne Erdreich und ohne Referenzmodelle (Material „referenz“, lot-io).
        .filter(|r| r.material != lot_geom::terrain::TERRAIN_MATERIAL && r.material != "referenz")
        .map(|r| &r.area)
        .collect();
    let building = Area::union_all(building);
    let Some((lo, hi)) = building.bounds() else { return };
    let text = |objects: &mut Vec<PlanObject>, at: PaperPoint, t: String, rotation: f64, anchor: Anchor| {
        objects.push(PlanObject::Text(Text { position: at, text: t, height_mm: size, anchor, rotation }));
    };
    let segment = |objects: &mut Vec<PlanObject>, a: PaperPoint, b: PaperPoint, width: f64| {
        objects.push(PlanObject::Line(Line {
            points: vec![a, b],
            closed: false,
            pen: Pen::mm(width),
            style: LineStyle::Solid,
            color: Color::BLACK,
        }));
    };

    // Wandhöhe vom Gelände bis zur Oberkante der Außenwand (MBO § 6 Abs. 4 Satz 2).
    for (edge, inward, side) in [(lo[0], 1.0, -1.0), (hi[0], -1.0, 1.0)] {
        let probe = edge + inward * 0.02;
        let Some(top) = vertical_runs(&building, probe).last().map(|r| r.1) else { continue };
        let p = frame.point(edge - inward * 0.001, 0.0);
        let ground = project.terrain().and_then(|t| t.height_at(lot_model::Point2::new(p[0], p[1]))).unwrap_or(GROUND);
        if top - ground < 1.0 {
            continue;
        }
        let u = edge + side * WALL_HEIGHT_OFFSET;
        let (a, b) = (viewport.to_paper(u, ground), viewport.to_paper(u, top));
        segment(objects, a, b, pen.width_mm);
        for q in [a, b] {
            segment(
                objects,
                PaperPoint::new(q.x - 0.7, q.y - 0.7),
                PaperPoint::new(q.x + 0.7, q.y + 0.7),
                pen.width_mm * 2.0,
            );
            let e = viewport.to_paper(edge, if q == a { ground } else { top });
            segment(
                objects,
                PaperPoint::new(e.x + side * 1.0, q.y),
                PaperPoint::new(q.x + side * 1.5, q.y),
                pen.width_mm,
            );
        }
        let mid = PaperPoint::new(a.x - 0.8, (a.y + b.y) / 2.0);
        text(objects, mid, format!("H = {} m", crate::format_de(top - ground, 2)), 90.0, Anchor::Center);
    }

    // Lichte Raumhöhe je Geschoss: Fertigfußboden bis Unterseite Decke oder Dach.
    for (_, storey) in project.storeys() {
        let z = storey.elevation;
        let found = (1..20).find_map(|k| {
            let u = lo[0] + (hi[0] - lo[0]) * k as f64 / 20.0;
            let runs = vertical_runs(&building, u);
            let floor = runs.iter().any(|r| r.0 < z - 0.01 && r.1 > z - 0.03);
            let above = runs.iter().filter(|r| r.0 > z + 0.01).map(|r| r.0).reduce(f64::min)?;
            let inside = runs.iter().any(|r| r.0 < z + 0.5 && r.1 > z + 0.05);
            (floor && !inside && (1.8..6.0).contains(&(above - z))).then_some((u, above - z))
        });
        if let Some((u, clear)) = found {
            let at = viewport.to_paper(u, z + 0.25);
            text(
                objects,
                PaperPoint::new(at.x + 1.5, at.y),
                format!("LRH {}", crate::format_de(clear, 2)),
                0.0,
                Anchor::Left,
            );
            let (a, b) = (viewport.to_paper(u, z), viewport.to_paper(u, z + clear));
            segment(objects, a, b, pen.width_mm);
            for q in [a, b] {
                segment(
                    objects,
                    PaperPoint::new(q.x - 0.7, q.y - 0.7),
                    PaperPoint::new(q.x + 0.7, q.y + 0.7),
                    pen.width_mm * 2.0,
                );
            }
        }
    }

    // Steigungsverhältnis über jeder geschnittenen Treppe.
    let mut stairs: Vec<ElementId> = cut
        .iter()
        .filter(|r| matches!(project.elements.get(&r.element), Some(Element::Stair(_))))
        .map(|r| r.element)
        .collect();
    stairs.sort();
    stairs.dedup();
    for id in stairs {
        let Some(Element::Stair(stair)) = project.elements.get(&id) else { continue };
        let Some(layout) = project.stair_layout(stair) else { continue };
        let regions: Vec<&Area> = cut.iter().filter(|r| r.element == id).map(|r| &r.area).collect();
        let Some((slo, shi)) = Area::union_all(regions).bounds() else { continue };
        let at = viewport.to_paper((slo[0] + shi[0]) / 2.0, shi[1] + 0.15);
        let label = format!(
            "{} Stg. {}/{}",
            stair.risers(),
            crate::format_de(layout.rise * 100.0, 1),
            crate::format_de(layout.tread * 100.0, 1)
        );
        text(objects, at, label, 0.0, Anchor::Center);
    }
}

/// Höhe im üblichen Format: „±0,00“, „+2,75“, „-2,50“.
pub fn format_height(z: f64) -> String {
    if z.abs() < 0.0005 {
        "±0,00".into()
    } else if z > 0.0 {
        format!("+{}", crate::format_de(z, 2))
    } else {
        format!("-{}", crate::format_de(-z, 2))
    }
}

/// Höhenkoten bei `u`: OKFF jedes Geschosses (offenes Dreieck, Fertigmaß nach
/// DIN 1356-1), Traufe und First der Dächer.
fn draw_level_marks(
    objects: &mut Vec<PlanObject>,
    project: &Project,
    kind: SectionKind,
    u: f64,
    style: &ScaleStyle,
    viewport: Viewport,
) {
    // In Ansichten nur, was über dem Gelände liegt.
    let mut marks: Vec<(f64, String)> = project
        .storeys()
        .iter()
        .filter(|(_, s)| kind == SectionKind::Section || s.elevation >= GROUND)
        .map(|(_, s)| (s.elevation, format!("OKFF {} {}", s.name, format_height(s.elevation))))
        .collect();
    for (_, roof) in project.elements.iter().filter_map(|(id, e)| match e {
        Element::Roof(r) => Some((*id, r)),
        _ => None,
    }) {
        let Some(storey) = project.storey(roof.storey) else { continue };
        let eave = storey.elevation + roof.eave_height;
        let ridge = storey.elevation
            + roof
                .outline
                .iter()
                .flat_map(|a| {
                    roof.outline.iter().map(move |b| lot_model::Point2::new((a.x + b.x) / 2.0, (a.y + b.y) / 2.0))
                })
                .map(|p| roof.top_at(p))
                .fold(f64::NEG_INFINITY, f64::max);
        // Flachdächer haben keine Traufe, die Höhe ist die Oberkante der Dachhaut.
        let label = if roof.eave_planes().is_empty() { "OK Dach" } else { "Traufe" };
        marks.push((eave, format!("{label} {}", format_height(eave))));
        if ridge > eave + 0.01 {
            marks.push((ridge, format!("First {}", format_height(ridge))));
        }
    }
    let pen = Pen::mm(style.pens.annotation);
    marks.sort_by(|a, b| a.0.total_cmp(&b.0));
    // Liegen zwei Höhen dicht beieinander, rückt die obere Beschriftung nach oben.
    let gap = style.space_text * 1.6;
    let mut last_text = f64::NEG_INFINITY;
    for (z, text) in marks {
        let p = viewport.to_paper(u, z);
        let text_y = (p.y + 0.8).max(last_text + gap);
        last_text = text_y;
        // Waagerechter Strich und offenes Dreieck, das mit der Spitze auf der Höhe steht.
        objects.push(PlanObject::Line(Line {
            points: vec![PaperPoint::new(p.x - 4.0, p.y), PaperPoint::new(p.x + 4.0, p.y)],
            closed: false,
            pen,
            style: LineStyle::Solid,
            color: Color::BLACK,
        }));
        objects.push(PlanObject::Line(Line {
            points: vec![p, PaperPoint::new(p.x - 1.2, p.y + 2.0), PaperPoint::new(p.x + 1.2, p.y + 2.0)],
            closed: true,
            pen,
            style: LineStyle::Solid,
            color: Color::BLACK,
        }));
        // Beschriftung links vom Dreieck, damit sie nicht ins Bild läuft.
        objects.push(PlanObject::Text(Text {
            position: PaperPoint::new(p.x - 2.0, text_y),
            text,
            height_mm: style.space_text,
            anchor: Anchor::Right,
            rotation: 0.0,
        }));
    }
}

#[cfg(test)]
mod tests {
    use lot_model::{Element, ElementId, Point2, SectionKind, SectionView, Top, Wall, WallAlignment, WallSide};

    use super::*;

    fn view(start: (f64, f64), end: (f64, f64), kind: SectionKind) -> SectionView {
        SectionView {
            name: "Test".into(),
            mark: "A".into(),
            start: Point2::new(start.0, start.1),
            end: Point2::new(end.0, end.1),
            look: WallSide::Left,
            depth: 20.0,
            kind,
        }
    }

    #[test]
    fn heights_are_signed() {
        assert_eq!(format_height(0.0), "±0,00");
        assert_eq!(format_height(2.75), "+2,75");
        assert_eq!(format_height(-2.5), "-2,50");
    }

    #[test]
    fn section_cuts_the_wall_across() {
        // Beispielwand 5 m lang, 30 cm dick, 2,60 m hoch; Schnitt quer bei x = 1, Blick nach Westen.
        let p = lot_model::Project::sample();
        let g = lot_geom::build(&p).unwrap();
        let v = view((1.0, -2.0), (1.0, 2.0), SectionKind::Section);
        let frame = Frame::new(&v);
        let e = &g.elements[0];
        let uvw: Vec<[f64; 3]> = e.mesh.positions.iter().map(|q| frame.uvw(*q)).collect();
        let area = slice(&e.mesh, &uvw);
        assert!((area.area() - 0.3 * 2.6).abs() < 1e-6, "{}", area.area());
    }

    #[test]
    fn walls_behind_a_wall_are_hidden() {
        let mut p = lot_model::Project::sample();
        // Eine kürzere Wand 2 m hinter der Beispielwand, ganz verdeckt.
        let id = p.next_id();
        p.elements.insert(
            id,
            Element::Wall(Wall {
                phase: Default::default(),
                storey: ElementId(1),
                wall_type: ElementId(2),
                start: Point2::new(1.0, 2.0),
                end: Point2::new(4.0, 2.0),
                alignment: WallAlignment::Center,
                base_offset: 0.0,
                top: Top::Height(2.0),
                sagitta: 0.0,
            }),
        );
        p.elements.remove(&ElementId(4)); // ohne Fenster, damit nichts durchscheint
        let g = lot_geom::build(&p).unwrap();
        // Ansicht von Süden: Linie bei y = -3 nach Osten, Blick nach Norden.
        let v = view((-1.0, -3.0), (6.0, -3.0), SectionKind::Elevation);
        let viewport = Viewport { scale: 100.0, origin: PaperPoint::new(0.0, 0.0) };
        let objects = section(&p, &g, &v, OfficeStandard::default_standard(), viewport);
        // Ansichtskanten (mittlerer Stift) liegen nur auf dem Rand der vorderen Wand.
        let inside = objects.iter().any(|o| match o {
            PlanObject::Line(l) if l.pen.width_mm == 0.25 => l.points.iter().all(|q| {
                let (u, z) = (q.x / 10.0 - 1.0, q.y / 10.0);
                u > 0.1 && u < 4.9 && z > 0.1 && z < 2.5
            }),
            _ => false,
        });
        assert!(!inside, "Kanten der verdeckten Wand sind sichtbar");
    }
}
