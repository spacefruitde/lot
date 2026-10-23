//! Flächen im Grundriss mit Booleans über Clipper2 (ADR 0005).
//!
//! Gerechnet wird mit Ganzzahlen auf einem Raster von 1 µm. Nach außen gehen
//! Ringe in Metern: Außenringe gegen den Uhrzeigersinn, Löcher im Uhrzeigersinn.

use clipper2_rust::{FillRule, Path64, Paths64, Point64};

/// Ganzzahlige Einheiten je Meter (1 µm).
const SCALE: f64 = 1e6;

/// Ring in Metern, ohne Wiederholung des ersten Punkts.
pub type Ring = Vec<[f64; 2]>;

/// Fläche aus beliebig vielen Ringen (Regel gerade/ungerade nach Bereinigung).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Area(Paths64);

fn to_point(p: [f64; 2]) -> Point64 {
    Point64::new((p[0] * SCALE).round() as i64, (p[1] * SCALE).round() as i64)
}

fn from_point(p: &Point64) -> [f64; 2] {
    [p.x as f64 / SCALE, p.y as f64 / SCALE]
}

impl Area {
    pub fn empty() -> Self {
        Self(Vec::new())
    }

    /// Fläche aus einem einfachen Ring (Richtung egal).
    pub fn from_ring(ring: &[[f64; 2]]) -> Self {
        if ring.len() < 3 {
            return Self::empty();
        }
        let path: Path64 = ring.iter().map(|p| to_point(*p)).collect();
        Self(clipper2_rust::union_subjects_64(&vec![path], FillRule::NonZero))
    }

    /// Fläche aus Ringen mit Löchern (Regel gerade/ungerade).
    pub fn from_rings(rings: &[Ring]) -> Self {
        let paths: Paths64 =
            rings.iter().filter(|r| r.len() >= 3).map(|r| r.iter().map(|p| to_point(*p)).collect()).collect();
        Self(clipper2_rust::union_subjects_64(&paths, FillRule::EvenOdd))
    }

    pub fn is_empty(&self) -> bool {
        self.area() < 1e-10
    }

    /// Flächeninhalt in m².
    pub fn area(&self) -> f64 {
        self.0.iter().map(clipper2_rust::area).sum::<f64>() / (SCALE * SCALE)
    }

    pub fn union(&self, other: &Area) -> Area {
        Area(clipper2_rust::union_64(&self.0, &other.0, FillRule::NonZero))
    }

    pub fn intersection(&self, other: &Area) -> Area {
        Area(clipper2_rust::intersect_64(&self.0, &other.0, FillRule::NonZero))
    }

    pub fn difference(&self, other: &Area) -> Area {
        Area(clipper2_rust::difference_64(&self.0, &other.0, FillRule::NonZero))
    }

    pub fn union_all<'a>(areas: impl IntoIterator<Item = &'a Area>) -> Area {
        let paths: Paths64 = areas.into_iter().flat_map(|a| a.0.iter().cloned()).collect();
        Area(clipper2_rust::union_subjects_64(&paths, FillRule::NonZero))
    }

    /// Ringe in Metern. Außenringe gegen, Löcher im Uhrzeigersinn.
    ///
    /// Punkte, die weniger als 2 µm neben der Verbindung ihrer Nachbarn
    /// liegen, fallen weg. So verschwinden haarfeine Schlitze, die beim
    /// Runden auf das µm-Raster entlang schräger Kanten entstehen und sonst
    /// als Spitzen in der Linie erscheinen.
    pub fn rings(&self) -> Vec<Ring> {
        self.0
            .iter()
            .map(|p| clipper2_rust::simplify_path(p, 2.0, true))
            .filter(|p| p.len() >= 3 && clipper2_rust::area(p).abs() > 1.0)
            .map(|p| p.iter().map(from_point).collect())
            .collect()
    }

    /// Ob der Punkt in der Fläche liegt (gerade/ungerade über alle Ringe).
    pub fn contains(&self, p: [f64; 2]) -> bool {
        let q = to_point(p);
        let (x, y) = (q.x as f64, q.y as f64);
        let mut inside = false;
        for path in &self.0 {
            let n = path.len();
            for i in 0..n {
                let (a, b) = (&path[i], &path[(i + 1) % n]);
                let (ax, ay, bx, by) = (a.x as f64, a.y as f64, b.x as f64, b.y as f64);
                if (ay > y) != (by > y) && x < ax + (y - ay) * (bx - ax) / (by - ay) {
                    inside = !inside;
                }
            }
        }
        inside
    }

    /// Gleiche Form, auch wenn Ringe in anderer Reihenfolge oder mit anderem
    /// Anfangspunkt vorliegen.
    pub fn same_shape(&self, other: &Area) -> bool {
        self == other || canonical(&self.0) == canonical(&other.0)
    }

    /// Fläche um `delta` Meter vergrößert (negativ: verkleinert), mit spitzen
    /// Ecken. Bei achsparallelen Kanten ist das Verkleinern genau das Abtragen
    /// eines Quadrats: Übrig bleiben die Mittelpunkte aller Quadrate mit der
    /// Kantenlänge 2·|delta|, die ganz in die Fläche passen.
    pub fn offset(&self, delta: f64) -> Area {
        let paths = clipper2_rust::inflate_paths_64(
            &self.0,
            delta * SCALE,
            clipper2_rust::JoinType::Miter,
            clipper2_rust::EndType::Polygon,
            4.0,
            0.0,
        );
        Area(clipper2_rust::union_subjects_64(&paths, FillRule::NonZero))
    }

    /// Um den Ursprung gedreht, Winkel in Grad gegen den Uhrzeigersinn.
    pub fn rotated(&self, degrees: f64) -> Area {
        let (sin, cos) = degrees.to_radians().sin_cos();
        let rings: Vec<Ring> = self
            .rings()
            .into_iter()
            .map(|r| r.into_iter().map(|p| [p[0] * cos - p[1] * sin, p[0] * sin + p[1] * cos]).collect())
            .collect();
        let paths: Paths64 = rings.iter().map(|r| r.iter().map(|p| to_point(*p)).collect()).collect();
        Area(clipper2_rust::union_subjects_64(&paths, FillRule::NonZero))
    }

    /// Achsparalleler Hüllquader (min, max).
    pub fn bounds(&self) -> Option<([f64; 2], [f64; 2])> {
        let mut points = self.0.iter().flatten().map(from_point);
        let first = points.next()?;
        Some(points.fold((first, first), |(lo, hi), p| {
            ([lo[0].min(p[0]), lo[1].min(p[1])], [hi[0].max(p[0]), hi[1].max(p[1])])
        }))
    }
}

/// Ringe mit dem kleinsten Punkt vorn, sortiert.
fn canonical(paths: &Paths64) -> Vec<Vec<(i64, i64)>> {
    let mut rings: Vec<Vec<(i64, i64)>> = paths
        .iter()
        .map(|p| {
            let mut r: Vec<(i64, i64)> = p.iter().map(|q| (q.x, q.y)).collect();
            if let Some(i) = r.iter().enumerate().min_by_key(|(_, q)| **q).map(|(i, _)| i) {
                r.rotate_left(i);
            }
            r
        })
        .collect();
    rings.sort();
    rings
}

/// Streifen entlang einer Geraden durch `origin` in Richtung `dir` (Einheitsvektor):
/// längs von `along.0` bis `along.1`, quer (positiv nach links) von `across.0` bis `across.1`.
pub fn strip(origin: [f64; 2], dir: [f64; 2], along: (f64, f64), across: (f64, f64)) -> Area {
    let normal = [-dir[1], dir[0]];
    let at = |s: f64, t: f64| [origin[0] + dir[0] * s + normal[0] * t, origin[1] + dir[1] * s + normal[1] * t];
    Area::from_ring(&[at(along.0, across.0), at(along.1, across.0), at(along.1, across.1), at(along.0, across.1)])
}

/// Halbebene links der gerichteten Geraden von `a` nach `b`, begrenzt auf ein
/// Quadrat mit halber Kantenlänge `reach` um `a`.
pub fn left_half_plane(a: [f64; 2], b: [f64; 2], reach: f64) -> Area {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let len = dx.hypot(dy);
    if len < 1e-12 {
        return Area::empty();
    }
    strip(a, [dx / len, dy / len], (-reach, reach), (0.0, reach))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_area_and_booleans() {
        let a = strip([0.0, 0.0], [1.0, 0.0], (0.0, 5.0), (0.15, -0.15));
        assert!((a.area() - 1.5).abs() < 1e-9);
        let b = strip([2.0, -1.0], [0.0, 1.0], (0.0, 2.0), (0.5, -0.5));
        assert!((a.intersection(&b).area() - 0.3).abs() < 1e-9);
        assert!((a.union(&b).area() - (1.5 + 2.0 - 0.3)).abs() < 1e-9);
        assert!((a.difference(&b).area() - 1.2).abs() < 1e-9);
    }

    #[test]
    fn offset_shrinks_and_grows_with_square_corners() {
        let sq = Area::from_ring(&[[0.0, 0.0], [3.0, 0.0], [3.0, 2.0], [0.0, 2.0]]);
        assert!((sq.offset(-0.75).area() - 1.5 * 0.5).abs() < 1e-6);
        assert!(sq.offset(-1.0).is_empty());
        assert!((sq.offset(0.5).area() - 4.0 * 3.0).abs() < 1e-6);
        let turned = sq.rotated(90.0);
        assert!((turned.area() - 6.0).abs() < 1e-6);
        assert!(turned.contains([-1.0, 1.5]));
    }

    #[test]
    fn half_plane_cuts_square() {
        let sq = Area::from_ring(&[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]);
        let left = left_half_plane([0.0, 0.0], [1.0, 1.0], 10.0);
        assert!((sq.intersection(&left).area() - 0.5).abs() < 1e-9);
    }
}
