//! Gelände (1.12): Höhenpunkte, dazwischen ein Dreiecksnetz (TIN).
//!
//! Gespeichert werden nur die Punkte; die Dreiecke entstehen durch eine
//! Delaunay-Triangulation (Bowyer-Watson). Außerhalb der Punkte gilt die
//! Höhe des nächsten Randes.

use serde::{Deserialize, Serialize};

use crate::Point2;

/// Gelände aus Höhenpunkten, Höhen in m über ±0,00.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Terrain {
    /// Punkte (x, y, Höhe).
    pub points: Vec<TerrainPoint>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TerrainPoint {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Terrain {
    /// Dreiecke als Indizes in `points`, gegen den Uhrzeigersinn.
    pub fn triangles(&self) -> Vec<[usize; 3]> {
        delaunay(&self.points.iter().map(|p| [p.x, p.y]).collect::<Vec<_>>())
    }

    /// Höhe an der Stelle `p`: im Netz eben zwischen den Eckpunkten des
    /// Dreiecks, außerhalb die Höhe am nächsten Punkt des Randes.
    pub fn height_at(&self, p: Point2) -> Option<f64> {
        self.height_with(&self.triangles(), p)
    }

    /// Wie [`Terrain::height_at`] mit schon berechneten Dreiecken.
    pub fn height_with(&self, triangles: &[[usize; 3]], p: Point2) -> Option<f64> {
        if self.points.is_empty() {
            return None;
        }
        let mut nearest: Option<(f64, f64)> = None;
        for t in triangles {
            let [a, b, c] = t.map(|i| self.points[i]);
            if let Some(z) = barycentric(a, b, c, p) {
                return Some(z);
            }
            for (u, v) in [(a, b), (b, c), (c, a)] {
                let (d, z) = edge_distance(u, v, p);
                if nearest.is_none_or(|(n, _)| d < n) {
                    nearest = Some((d, z));
                }
            }
        }
        if let Some((_, z)) = nearest {
            return Some(z);
        }
        // Weniger als drei Punkte: der nächste Punkt.
        self.points
            .iter()
            .min_by(|a, b| (a.x - p.x).hypot(a.y - p.y).total_cmp(&(b.x - p.x).hypot(b.y - p.y)))
            .map(|q| q.z)
    }

    /// Niedrigster und höchster Punkt.
    pub fn z_range(&self) -> Option<(f64, f64)> {
        let mut it = self.points.iter().map(|p| p.z);
        let first = it.next()?;
        Some(it.fold((first, first), |(lo, hi), z| (lo.min(z), hi.max(z))))
    }
}

fn barycentric(a: TerrainPoint, b: TerrainPoint, c: TerrainPoint, p: Point2) -> Option<f64> {
    let det = (b.y - c.y) * (a.x - c.x) + (c.x - b.x) * (a.y - c.y);
    if det.abs() < 1e-12 {
        return None;
    }
    let l1 = ((b.y - c.y) * (p.x - c.x) + (c.x - b.x) * (p.y - c.y)) / det;
    let l2 = ((c.y - a.y) * (p.x - c.x) + (a.x - c.x) * (p.y - c.y)) / det;
    let l3 = 1.0 - l1 - l2;
    let eps = -1e-9;
    (l1 >= eps && l2 >= eps && l3 >= eps).then_some(l1 * a.z + l2 * b.z + l3 * c.z)
}

/// Abstand von `p` zur Strecke u-v und die Höhe am nächsten Punkt.
fn edge_distance(u: TerrainPoint, v: TerrainPoint, p: Point2) -> (f64, f64) {
    let (dx, dy) = (v.x - u.x, v.y - u.y);
    let len2 = dx * dx + dy * dy;
    let t = if len2 < 1e-12 { 0.0 } else { (((p.x - u.x) * dx + (p.y - u.y) * dy) / len2).clamp(0.0, 1.0) };
    let (x, y) = (u.x + dx * t, u.y + dy * t);
    ((p.x - x).hypot(p.y - y), u.z + (v.z - u.z) * t)
}

/// Delaunay-Triangulation nach Bowyer-Watson; für einige hundert Punkte schnell genug.
fn delaunay(points: &[[f64; 2]]) -> Vec<[usize; 3]> {
    let n = points.len();
    if n < 3 {
        return Vec::new();
    }
    let (mut lo, mut hi) = (points[0], points[0]);
    for p in points {
        lo = [lo[0].min(p[0]), lo[1].min(p[1])];
        hi = [hi[0].max(p[0]), hi[1].max(p[1])];
    }
    let size = (hi[0] - lo[0]).max(hi[1] - lo[1]).max(1.0) * 20.0;
    let mid = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0];
    // Großes Hilfsdreieck um alle Punkte; seine Ecken haben die Indizes n, n+1, n+2.
    let mut all: Vec<[f64; 2]> = points.to_vec();
    all.extend([[mid[0] - size, mid[1] - size], [mid[0] + size, mid[1] - size], [mid[0], mid[1] + size]]);
    let mut tris: Vec<[usize; 3]> = vec![[n, n + 1, n + 2]];
    for i in 0..n {
        let p = all[i];
        let (bad, good): (Vec<[usize; 3]>, Vec<[usize; 3]>) =
            tris.into_iter().partition(|t| in_circumcircle(all[t[0]], all[t[1]], all[t[2]], p));
        // Rand des Lochs: Kanten, die nur ein schlechtes Dreieck hat.
        let mut edges: Vec<[usize; 2]> = Vec::new();
        for t in &bad {
            for e in [[t[0], t[1]], [t[1], t[2]], [t[2], t[0]]] {
                if let Some(k) = edges.iter().position(|f| f[0] == e[1] && f[1] == e[0]) {
                    edges.swap_remove(k);
                } else {
                    edges.push(e);
                }
            }
        }
        tris = good;
        tris.extend(edges.into_iter().map(|e| [e[0], e[1], i]));
    }
    tris.into_iter()
        .filter(|t| t.iter().all(|&k| k < n))
        .map(|t| {
            let [a, b, c] = t.map(|k| all[k]);
            let area = (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
            if area < 0.0 { [t[0], t[2], t[1]] } else { t }
        })
        .filter(|t| {
            let [a, b, c] = t.map(|k| all[k]);
            ((b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])).abs() > 1e-12
        })
        .collect()
}

/// Liegt p im Umkreis des Dreiecks a, b, c (beliebig orientiert)?
fn in_circumcircle(a: [f64; 2], b: [f64; 2], c: [f64; 2], p: [f64; 2]) -> bool {
    let (ax, ay) = (a[0] - p[0], a[1] - p[1]);
    let (bx, by) = (b[0] - p[0], b[1] - p[1]);
    let (cx, cy) = (c[0] - p[0], c[1] - p[1]);
    let det = (ax * ax + ay * ay) * (bx * cy - cx * by) - (bx * bx + by * by) * (ax * cy - cx * ay)
        + (cx * cx + cy * cy) * (ax * by - bx * ay);
    let orient = (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
    if orient > 0.0 { det > 1e-12 } else { det < -1e-12 }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slope() -> Terrain {
        // Hang: vorn (y = 0) auf -0,5, hinten (y = 10) auf +0,5, ein Punkt in der Mitte.
        let p = |x, y, z| TerrainPoint { x, y, z };
        Terrain {
            points: vec![
                p(0.0, 0.0, -0.5),
                p(10.0, 0.0, -0.5),
                p(10.0, 10.0, 0.5),
                p(0.0, 10.0, 0.5),
                p(5.0, 5.0, 0.0),
            ],
        }
    }

    #[test]
    fn five_points_give_four_triangles() {
        let t = slope().triangles();
        assert_eq!(t.len(), 4, "{t:?}");
    }

    #[test]
    fn height_interpolates_inside_and_clamps_outside() {
        let t = slope();
        assert!((t.height_at(Point2::new(5.0, 2.5)).unwrap() + 0.25).abs() < 1e-9);
        assert!((t.height_at(Point2::new(2.0, 9.0)).unwrap() - 0.4).abs() < 1e-9);
        // Vor dem Netz: die Höhe am Rand.
        assert!((t.height_at(Point2::new(5.0, -3.0)).unwrap() + 0.5).abs() < 1e-9);
        assert_eq!(t.z_range(), Some((-0.5, 0.5)));
    }
}
