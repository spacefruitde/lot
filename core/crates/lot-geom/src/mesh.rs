use std::collections::HashMap;

/// Geschlossenes Dreiecksnetz mit gemeinsamen Eckpunkten, in Metern.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TriMesh {
    pub positions: Vec<[f64; 3]>,
    pub triangles: Vec<[u32; 3]>,
}

/// Eckpunkt für die flache Schattierung: jedes Dreieck mit eigener Normale.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FlatVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
}

impl TriMesh {
    pub fn is_empty(&self) -> bool {
        self.triangles.is_empty()
    }

    fn corners(&self, t: [u32; 3]) -> [[f64; 3]; 3] {
        t.map(|i| self.positions[i as usize])
    }

    fn normal(&self, t: [u32; 3]) -> [f64; 3] {
        let [a, b, c] = self.corners(t);
        normalize(cross(sub(b, a), sub(c, a)))
    }

    /// Achsparalleler Hüllquader (min, max).
    pub fn bounds(&self) -> Option<([f64; 3], [f64; 3])> {
        let mut it = self.positions.iter();
        let first = *it.next()?;
        Some(it.fold((first, first), |(lo, hi), p| {
            ([lo[0].min(p[0]), lo[1].min(p[1]), lo[2].min(p[2])], [hi[0].max(p[0]), hi[1].max(p[1]), hi[2].max(p[2])])
        }))
    }

    /// Eckpunkte für die flache Schattierung, drei je Dreieck.
    pub fn flat_vertices(&self) -> Vec<FlatVertex> {
        let mut out = Vec::with_capacity(self.triangles.len() * 3);
        for &t in &self.triangles {
            let n = self.normal(t).map(|v| v as f32);
            for p in self.corners(t) {
                out.push(FlatVertex { position: p.map(|v| v as f32), normal: n });
            }
        }
        out
    }

    /// Kanten, an denen die Flächen um mehr als `angle_deg` abknicken, und offene Ränder.
    /// Die Diagonalen, die ebene Flächen in Dreiecke teilen, fallen weg.
    pub fn feature_edges(&self, angle_deg: f64) -> Vec<[[f32; 3]; 2]> {
        let mut faces: HashMap<(u32, u32), Vec<[f64; 3]>> = HashMap::new();
        for &t in &self.triangles {
            let n = self.normal(t);
            for i in 0..3 {
                let (a, b) = (t[i], t[(i + 1) % 3]);
                faces.entry((a.min(b), a.max(b))).or_default().push(n);
            }
        }
        let cos_limit = angle_deg.to_radians().cos();
        let mut edges: Vec<(u32, u32)> = faces
            .into_iter()
            .filter(|(_, normals)| normals.len() != 2 || dot(normals[0], normals[1]) < cos_limit)
            .map(|(k, _)| k)
            .collect();
        edges.sort_unstable();
        edges.into_iter().map(|(a, b)| [a, b].map(|i| self.positions[i as usize].map(|v| v as f32))).collect()
    }
}

/// Vorzeichenbehaftete Fläche eines Rings, positiv gegen den Uhrzeigersinn.
pub fn signed_area(ring: &[[f64; 2]]) -> f64 {
    let n = ring.len();
    (0..n)
        .map(|i| {
            let (a, b) = (ring[i], ring[(i + 1) % n]);
            a[0] * b[1] - b[0] * a[1]
        })
        .sum::<f64>()
        / 2.0
}

pub(crate) fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

pub(crate) fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

pub(crate) fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn normalize(v: [f64; 3]) -> [f64; 3] {
    let len = dot(v, v).sqrt();
    if len > 0.0 { v.map(|c| c / len) } else { v }
}
