//! Strecken an Flächen beschneiden, für Muster in Schichtflächen.

use crate::PaperPoint;

/// Liegt der Punkt in den Ringen (Regel gerade/ungerade)?
pub fn inside(p: PaperPoint, rings: &[Vec<PaperPoint>]) -> bool {
    let mut inside = false;
    for ring in rings {
        let n = ring.len();
        for i in 0..n {
            let (a, b) = (ring[i], ring[(i + 1) % n]);
            if (a.y > p.y) != (b.y > p.y) {
                let x = a.x + (p.y - a.y) / (b.y - a.y) * (b.x - a.x);
                if x > p.x {
                    inside = !inside;
                }
            }
        }
    }
    inside
}

/// Die Teile der Strecke von `a` nach `b`, die in den Ringen liegen.
pub fn clip_segment(a: PaperPoint, b: PaperPoint, rings: &[Vec<PaperPoint>]) -> Vec<[PaperPoint; 2]> {
    let d = (b.x - a.x, b.y - a.y);
    let mut ts = vec![0.0, 1.0];
    for ring in rings {
        let n = ring.len();
        for i in 0..n {
            let (p, q) = (ring[i], ring[(i + 1) % n]);
            let e = (q.x - p.x, q.y - p.y);
            let denom = d.0 * e.1 - d.1 * e.0;
            if denom.abs() < 1e-15 {
                continue;
            }
            let w = (p.x - a.x, p.y - a.y);
            let t = (w.0 * e.1 - w.1 * e.0) / denom;
            let u = (w.0 * d.1 - w.1 * d.0) / denom;
            if (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u) {
                ts.push(t);
            }
        }
    }
    ts.sort_by(f64::total_cmp);
    let at = |t: f64| PaperPoint::new(a.x + d.0 * t, a.y + d.1 * t);
    ts.windows(2)
        .filter(|w| w[1] - w[0] > 1e-9 && inside(at((w[0] + w[1]) / 2.0), rings))
        .map(|w| [at(w[0]), at(w[1])])
        .collect()
}

/// Beschneidet einen Linienzug an den Ringen.
pub fn clip_polyline(points: &[PaperPoint], rings: &[Vec<PaperPoint>]) -> Vec<[PaperPoint; 2]> {
    points.windows(2).flat_map(|w| clip_segment(w[0], w[1], rings)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rect_points;

    #[test]
    fn segment_through_square_with_hole() {
        let outer = rect_points(0.0, 0.0, 10.0, 10.0);
        let hole = rect_points(4.0, 0.0, 2.0, 10.0);
        let parts = clip_segment(PaperPoint::new(-5.0, 5.0), PaperPoint::new(15.0, 5.0), &[outer, hole]);
        assert_eq!(parts.len(), 2);
        assert!((parts[0][1].x - 4.0).abs() < 1e-9);
        assert!((parts[1][0].x - 6.0).abs() < 1e-9);
    }
}
