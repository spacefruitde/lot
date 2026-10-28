//! Gewendelte Treppen und Spindeltreppen (ADR 0051), nach DIN 18065:2020-08.
//!
//! Wendelstufen werden nach der Verhältnismethode verzogen (6.2.5): Auf der
//! Lauflinie bleibt der Auftritt gleich, an der inneren Begrenzung nimmt er
//! zur Ecke hin gleichmäßig ab (6.2.2), bis zum Mindestauftritt (6.2.1). Die
//! Lauflinie liegt in der Mitte der Laufbreite, bei mehr als 1,00 m 50 cm von
//! der inneren Begrenzung (8.1, 8.2, Bild A.21). Spindeltreppen: Lauflinie
//! 4/10 der Laufbreite von außen (8.4, Bild A.25).

use crate::stair::{Spiral, Stair, StairLayout, Step, add};
use crate::{Landing, Point2};

/// Mindestauftritt von Wendelstufen an der inneren Begrenzung der nutzbaren
/// Laufbreite: Gebäude im Allgemeinen 10 cm, Wohngebäude mit bis zu zwei
/// Wohnungen 5 cm (DIN 18065:2020-08, Tabelle 1, Nr. 6.2.1).
pub const MIN_INNER_GENERAL: f64 = 0.10;
pub const MIN_INNER_SMALL_RESIDENTIAL: f64 = 0.05;

/// Punkte je Viertelkreis für Bögen.
const ARC_STEPS: usize = 12;

/// Ein Linienzug mit Bogenlänge.
struct Path {
    points: Vec<Point2>,
    /// Bogenlänge bis zu jedem Punkt.
    at: Vec<f64>,
}

impl Path {
    fn new(points: Vec<Point2>) -> Self {
        let mut at = vec![0.0];
        for w in points.windows(2) {
            at.push(at.last().copied().unwrap_or(0.0) + w[0].distance(w[1]));
        }
        Path { points, at }
    }

    fn length(&self) -> f64 {
        self.at.last().copied().unwrap_or(0.0)
    }

    /// Punkt bei Bogenlänge `t`.
    fn point(&self, t: f64) -> Point2 {
        let t = t.clamp(0.0, self.length());
        let i = self.at.windows(2).position(|w| t <= w[1] + 1e-12).unwrap_or(self.at.len().saturating_sub(2));
        let (a, b) = (self.points[i], self.points[i + 1]);
        let len = (self.at[i + 1] - self.at[i]).max(1e-12);
        let f = (t - self.at[i]) / len;
        Point2::new(a.x + (b.x - a.x) * f, a.y + (b.y - a.y) * f)
    }

    /// Die Punkte zwischen den Bogenlängen `t0` und `t1` (ohne die Enden).
    fn between(&self, t0: f64, t1: f64) -> Vec<Point2> {
        self.points.iter().zip(&self.at).filter(|(_, a)| **a > t0 + 1e-9 && **a < t1 - 1e-9).map(|(p, _)| *p).collect()
    }

    /// Erster Schnitt des Strahls von `from` in Richtung `dir` mit dem Linienzug: Punkt und Bogenlänge.
    fn hit(&self, from: Point2, dir: (f64, f64)) -> Option<(Point2, f64)> {
        let mut best: Option<(f64, Point2, f64)> = None;
        for (i, w) in self.points.windows(2).enumerate() {
            let (a, b) = (w[0], w[1]);
            let e = (b.x - a.x, b.y - a.y);
            let denom = dir.0 * e.1 - dir.1 * e.0;
            if denom.abs() < 1e-12 {
                continue;
            }
            let q = (a.x - from.x, a.y - from.y);
            let ray = (q.0 * e.1 - q.1 * e.0) / denom;
            let seg = (q.0 * dir.1 - q.1 * dir.0) / denom;
            if ray > 1e-9 && (-1e-9..=1.0 + 1e-9).contains(&seg) && best.is_none_or(|(r, _, _)| ray < r) {
                let p = Point2::new(from.x + dir.0 * ray, from.y + dir.1 * ray);
                best = Some((ray, p, self.at[i] + seg.clamp(0.0, 1.0) * (self.at[i + 1] - self.at[i])));
            }
        }
        best.map(|(_, p, t)| (p, t))
    }
}

/// Eine Wendelung zwischen zwei Läufen.
struct Turn {
    /// Bogen der Lauflinie: von, bis (Bogenlänge auf der Lauflinie).
    arc: (f64, f64),
    /// Bogenlänge der inneren Begrenzung an der Ecke.
    corner: f64,
    /// Länge der inneren Begrenzung in der Wendelung: das Treppenauge, bei 90° null.
    eye: f64,
}

impl Stair {
    pub(crate) fn winder_layout(&self, height: f64, min_inner: f64) -> StairLayout {
        let n = self.risers().max(1);
        let rise = height / n as f64;
        let a = self.tread;
        let w = self.width;
        let h = w / 2.0;
        // Lauflinie: Mitte, bei mehr als 1,00 m 50 cm von innen.
        let r = h.min(0.5);
        // Innen liegt die Seite der ersten Wendelung.
        let inner_left = self
            .flights
            .iter()
            .find(|f| f.landing.is_winder())
            .is_none_or(|f| matches!(f.landing, Landing::WinderLeft | Landing::WinderBackLeft));
        let len = self.direction.x.hypot(self.direction.y).max(1e-12);
        let mut dir = Point2::new(self.direction.x / len, self.direction.y / len);
        let side = |d: Point2| if inner_left { Point2::new(-d.y, d.x) } else { Point2::new(d.y, -d.x) };
        let mut start = self.start;
        let mut walking = vec![add(start, side(dir), h - r)];
        let mut inner = vec![add(start, side(dir), h)];
        let mut outer = vec![add(start, side(dir), -h)];
        let mut turns: Vec<Turn> = Vec::new();
        let mut walked = 0.0;
        let mut inner_len = 0.0;
        let mut risers_before = 0;
        for (i, f) in self.flights.iter().enumerate() {
            risers_before += f.risers;
            let last = i + 1 == self.flights.len();
            // Gerader Teil: bis zur Winkelhalbierenden der Wendelung liegen `risers_before` Steigungen.
            let back = matches!(f.landing, Landing::WinderBackLeft | Landing::WinderBackRight);
            let phi = if back { std::f64::consts::PI } else { std::f64::consts::FRAC_PI_2 };
            // Bei halbgewendelten Läufen läuft die Lauflinie um das Treppenauge.
            let eye = if back { self.eye.max(0.0) } else { 0.0 };
            let radius = r + eye / 2.0;
            let arc = phi * radius;
            let straight = if last {
                (n - 1) as f64 * a - walked
            } else {
                ((risers_before as f64 - 0.5) * a - arc / 2.0 - walked).max(0.0)
            };
            let end = add(start, dir, straight);
            walked += straight;
            inner_len += straight;
            walking.push(add(end, side(dir), h - r));
            let s = side(dir);
            let corner = add(end, s, h);
            inner.push(corner);
            outer.push(add(end, s, -h));
            if last {
                break;
            }
            // Wendelung: Bogen der Lauflinie um die innere Ecke, äußere Ecken um den Platz des Podests.
            outer.push(add(add(end, s, -h), dir, w));
            if back {
                outer.push(add(add(end, s, h + eye + w), dir, w));
                inner.push(add(corner, s, eye));
            } else {
                outer.push(add(add(end, s, h), dir, w));
            }
            let center = add(corner, s, eye / 2.0);
            let a0 = (-s.y).atan2(-s.x);
            // Drehung vom Vektor -s über die Laufrichtung.
            let cross = (-s.x) * dir.y - (-s.y) * dir.x;
            let sweep = phi * cross.signum();
            let samples = ARC_STEPS * if back { 2 } else { 1 };
            for k in 1..=samples {
                let t = a0 + sweep * k as f64 / samples as f64;
                walking.push(Point2::new(center.x + radius * t.cos(), center.y + radius * t.sin()));
            }
            turns.push(Turn { arc: (walked, walked + arc), corner: inner_len, eye });
            walked += arc;
            inner_len += eye;
            if back {
                dir = Point2::new(-dir.x, -dir.y);
                start = add(end, s, w + eye);
                outer.push(add(end, s, h + eye + w));
            } else {
                start = add(corner, dir, h);
                dir = s;
            }
        }
        let walk = Path::new(walking);
        let inner_path = Path::new(inner);
        let outer_path = Path::new(outer);
        // Innere Lage jeder Steigung: außerhalb der Wendelungen senkrecht, darin verzogen.
        let positions: Vec<f64> = (0..n).map(|k| k as f64 * a).collect();
        let straight_t = |s: f64| {
            let mut t = s;
            for turn in &turns {
                let arc = turn.arc.1 - turn.arc.0;
                if s >= turn.arc.1 {
                    t -= arc - turn.eye;
                } else if s > turn.arc.0 {
                    t = turn.corner + (s - turn.arc.0) / arc * turn.eye;
                }
            }
            t
        };
        let mut inner_t: Vec<f64> = positions.iter().map(|s| straight_t(*s)).collect();
        for turn in &turns {
            let middle = (turn.arc.0 + turn.arc.1) / 2.0;
            // Was die innere Begrenzung in der Wendelung kürzer ist als die Lauflinie.
            let reduce = turn.arc.1 - turn.arc.0 - turn.eye;
            // Ziel an der schmalsten Stelle, mit Reserve für die Sehne über die Ecke (bis √2 kürzer).
            let target = (min_inner * 1.45).min(a);
            let mut k0 = (turn.arc.0 / a).floor() as i64;
            let mut k1 = (turn.arc.1 / a).ceil() as i64;
            loop {
                let (i0, i1) = (k0.max(0) as usize, (k1.max(0) as usize).min(n as usize - 1));
                let m = i1.saturating_sub(i0);
                if m == 0 {
                    break;
                }
                let span = (m as f64 / 2.0 + 0.5) * a;
                let weights: Vec<f64> = (i0..i1)
                    .map(|j| (1.0 - ((positions[j] + positions[j + 1]) / 2.0 - middle).abs() / span).max(0.05))
                    .collect();
                let cuts = distribute(&weights, reduce, a - target);
                let mut t = straight_t(positions[i0]);
                for (j, cut) in cuts.iter().enumerate() {
                    t += a - cut;
                    inner_t[i0 + j + 1] = t;
                }
                let reached = (cuts.iter().sum::<f64>() - reduce).abs() < 1e-6;
                let ok = reached
                    && (i0..i1).all(|j| {
                        inner_path.point(inner_t[j]).distance(inner_path.point(inner_t[j + 1])) >= min_inner - 1e-9
                    });
                if ok || (k0 <= 0 && k1 as usize >= n as usize - 1) {
                    break;
                }
                k0 -= 1;
                k1 += 1;
            }
        }
        // Stufen zwischen den Steigungen.
        let mut lines: Vec<(Point2, Point2, f64, f64)> = Vec::new();
        for k in 0..n as usize {
            let wpt = walk.point(positions[k]);
            let ipt = inner_path.point(inner_t[k]);
            let d = (wpt.x - ipt.x, wpt.y - ipt.y);
            let l = d.0.hypot(d.1).max(1e-12);
            let (opt, ou) = outer_path.hit(wpt, (d.0 / l, d.1 / l)).unwrap_or((wpt, 0.0));
            lines.push((ipt, opt, inner_t[k], ou));
        }
        let mut steps = Vec::new();
        for k in 0..(n as usize).saturating_sub(1) {
            let (i0, o0, t0, u0) = lines[k];
            let (i1, o1, t1, u1) = lines[k + 1];
            let mut outline = vec![i0];
            outline.extend(inner_path.between(t0, t1));
            outline.push(i1);
            outline.push(o1);
            let mut back = outer_path.between(u0.min(u1), u0.max(u1));
            if u1 > u0 {
                back.reverse();
            }
            outline.extend(back);
            outline.push(o0);
            steps.push(Step { outline, top: (k + 1) as f64 * rise, front: (i0, o0), inner: i0.distance(i1) });
        }
        StairLayout {
            width: self.width,
            tread: a,
            rise,
            flights: Vec::new(),
            landings: Vec::new(),
            steps,
            walking: walk.points,
        }
    }

    /// Spindeltreppe. `min_inner`: Mindestauftritt an der inneren Begrenzung der
    /// nutzbaren Laufbreite, die erst dort beginnt (DIN 18065:2020-08, 6.2.4); 0
    /// in Wohngebäuden, dann beginnt sie an der Spindel.
    pub(crate) fn spiral_layout(&self, spiral: Spiral, height: f64, min_inner: f64) -> StairLayout {
        let n = self.risers().max(1);
        let rise = height / n as f64;
        let a = self.tread;
        let c = spiral.center;
        let r_out = spiral.radius.max(spiral.spindle + 0.1);
        let mut r_in = spiral.spindle.max(0.0);
        let mut r_w = r_out;
        let mut step = 0.0;
        // Lauflinie 4/10 der nutzbaren Laufbreite vom äußeren Rand (8.4, Bild A.25);
        // die innere Begrenzung liegt, wo der Auftritt `min_inner` erreicht.
        for _ in 0..8 {
            r_w = r_out - 0.4 * (r_out - r_in);
            step = 2.0 * (a / (2.0 * r_w)).clamp(-1.0, 1.0).asin();
            let at_min = if min_inner > 0.0 { min_inner / (2.0 * (step / 2.0).sin()) } else { 0.0 };
            r_in = spiral.spindle.max(at_min).min(r_out - 0.05);
        }
        let b = r_out - r_in;
        let sign = if spiral.clockwise { -1.0 } else { 1.0 };
        let theta0 = (self.start.y - c.y).atan2(self.start.x - c.x);
        let at = |r: f64, t: f64| Point2::new(c.x + r * t.cos(), c.y + r * t.sin());
        let mut steps = Vec::new();
        for k in 0..(n as usize).saturating_sub(1) {
            let t0 = theta0 + sign * step * k as f64;
            let t1 = theta0 + sign * step * (k + 1) as f64;
            let mut outline = Vec::new();
            for j in 0..=4 {
                outline.push(at(spiral.spindle.max(0.0), t0 + (t1 - t0) * j as f64 / 4.0));
            }
            for j in (0..=4).rev() {
                outline.push(at(r_out, t0 + (t1 - t0) * j as f64 / 4.0));
            }
            steps.push(Step {
                outline,
                top: (k + 1) as f64 * rise,
                front: (at(spiral.spindle.max(0.0), t0), at(r_out, t0)),
                inner: 2.0 * r_in * (step / 2.0).sin(),
            });
        }
        let total = sign * step * (n as f64 - 1.0);
        let samples = ((total.abs() / std::f64::consts::FRAC_PI_2) * ARC_STEPS as f64).ceil().max(1.0) as usize;
        let walking = (0..=samples).map(|j| at(r_w, theta0 + total * j as f64 / samples as f64)).collect();
        StairLayout { width: b, tread: a, rise, flights: Vec::new(), landings: Vec::new(), steps, walking }
    }
}

/// Verteilt die Verkürzung `total` nach Gewichten auf die Stufen, jede höchstens
/// um `cap`: Was über `cap` hinausginge, übernehmen die übrigen (Verhältnismethode
/// mit Mindestauftritt). Reicht es nicht, bleiben alle bei `cap`.
fn distribute(weights: &[f64], total: f64, cap: f64) -> Vec<f64> {
    let mut cuts = vec![0.0; weights.len()];
    let mut capped = vec![false; weights.len()];
    loop {
        let fixed: f64 = capped.iter().filter(|c| **c).count() as f64 * cap;
        let free: f64 = weights.iter().zip(&capped).filter(|(_, c)| !**c).map(|(w, _)| *w).sum();
        if free <= 1e-12 {
            break;
        }
        let lambda = (total - fixed).max(0.0) / free;
        let mut again = false;
        for (j, w) in weights.iter().enumerate() {
            if capped[j] {
                cuts[j] = cap;
            } else if lambda * w > cap {
                capped[j] = true;
                again = true;
            } else {
                cuts[j] = lambda * w;
            }
        }
        if !again {
            break;
        }
    }
    cuts
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ElementId, StairFlight};

    fn stair(flights: Vec<StairFlight>, width: f64, tread: f64) -> Stair {
        Stair {
            phase: Default::default(),
            storey: ElementId(1),
            start: Point2::new(0.0, 0.0),
            direction: Point2::new(0.0, 1.0),
            width,
            tread,
            flights,
            thickness: 0.18,
            necessary: true,
            spiral: None,
            eye: 0.0,
        }
    }

    fn walking_chords(l: &StairLayout) -> Vec<f64> {
        // Schnitt jeder Vorderkante mit der Lauflinie: Abstand der Schnittpunkte.
        let path = Path::new(l.walking.clone());
        let hits: Vec<Point2> = l
            .steps
            .iter()
            .filter_map(|s| {
                let (a, b) = s.front;
                let d = (b.x - a.x, b.y - a.y);
                let len = d.0.hypot(d.1);
                path.hit(Point2::new(a.x - d.0 / len * 0.01, a.y - d.1 / len * 0.01), (d.0 / len, d.1 / len))
                    .map(|(p, _)| p)
            })
            .collect();
        hits.windows(2).map(|w| w[0].distance(w[1])).collect()
    }

    #[test]
    fn quarter_winders_keep_the_tread_on_the_walking_line_and_narrow_inside() {
        let s = stair(
            vec![
                StairFlight { risers: 8, landing: Landing::WinderLeft },
                StairFlight { risers: 8, landing: Landing::Straight },
            ],
            1.0,
            0.27,
        );
        let l = s.layout(2.75);
        assert_eq!(l.steps.len(), 15);
        assert!(l.flights.is_empty() && l.landings.is_empty());
        // Gebäude im Allgemeinen: innen mindestens 10 cm (DIN 18065:2020-08, 6.2.1).
        let inner: Vec<f64> = l.steps.iter().map(|s| s.inner).collect();
        assert!(inner.iter().all(|v| *v >= MIN_INNER_GENERAL - 1e-6), "{inner:?}");
        // Zur Ecke hin gleich oder schmaler, danach wieder breiter (6.2.2).
        let narrowest = inner.iter().enumerate().min_by(|a, b| a.1.total_cmp(b.1)).unwrap().0;
        assert!(inner[..=narrowest].windows(2).all(|w| w[1] <= w[0] + 1e-6), "{inner:?}");
        assert!(inner[narrowest..].windows(2).all(|w| w[1] >= w[0] - 1e-6), "{inner:?}");
        assert!(inner[narrowest] < 0.20, "die Ecke ist verzogen: {inner:?}");
        // Auf der Lauflinie bleibt der Auftritt (als Sehne, im Bogen bis 3 mm kürzer).
        for c in walking_chords(&l) {
            assert!((c - 0.27).abs() < 0.005, "{c}");
        }
        // Höhen steigen gleichmäßig.
        assert!((l.steps[14].top - 15.0 * 2.75 / 16.0).abs() < 1e-9);
        // Im Wohngebäude darf es innen bis 5 cm schmal werden: weniger verzogen.
        let small = s.layout_with(2.75, MIN_INNER_SMALL_RESIDENTIAL);
        let narrowest_small = small.steps.iter().map(|s| s.inner).fold(f64::INFINITY, f64::min);
        assert!(narrowest_small >= MIN_INNER_SMALL_RESIDENTIAL - 1e-6 && narrowest_small < inner[narrowest] + 1e-6);
    }

    #[test]
    fn half_winders_turn_back_beside_the_first_flight() {
        let s = stair(
            vec![
                StairFlight { risers: 8, landing: Landing::WinderBackRight },
                StairFlight { risers: 8, landing: Landing::Straight },
            ],
            0.9,
            0.26,
        );
        let s = Stair { eye: 0.2, ..s };
        let l = s.layout_with(2.75, MIN_INNER_SMALL_RESIDENTIAL);
        assert_eq!(l.steps.len(), 15);
        assert!(l.steps.iter().all(|s| s.inner >= MIN_INNER_SMALL_RESIDENTIAL - 1e-6));
        // Der Austritt liegt rechts neben dem Antritt, eine Laufbreite weiter.
        let last = l.walking.last().unwrap();
        assert!(last.x > 0.5 && last.x < 1.6, "{last:?}");
        for c in walking_chords(&l) {
            assert!((c - 0.26).abs() < 0.006, "{c}");
        }
    }

    #[test]
    fn spiral_steps_turn_around_the_spindle() {
        let mut s = stair(vec![StairFlight { risers: 14, landing: Landing::Straight }], 0.8, 0.236);
        s.start = Point2::new(1.0, 0.0);
        s.spiral = Some(Spiral { center: Point2::new(0.0, 0.0), radius: 0.9, spindle: 0.1, clockwise: false });
        // Wohngebäude: nutzbare Laufbreite ab der Spindel (DIN 18065:2020-08, 6.2.4).
        let l = s.layout_with(2.6, MIN_INNER_SMALL_RESIDENTIAL);
        assert_eq!(l.steps.len(), 13);
        // Lauflinie 4/10 der Laufbreite von außen: r = 0,9 - 0,32 = 0,58.
        assert!(l.walking.iter().all(|p| (p.distance(Point2::new(0.0, 0.0)) - 0.58).abs() < 1e-9));
        for c in walking_chords(&l) {
            assert!((c - 0.236).abs() < 1e-3, "{c}");
        }
        // 2s + a = 2 · 18,6 + 23,6 cm.
        let step = 2.0 * l.rise + l.tread;
        assert!((step - 0.607).abs() < 0.002, "{step}");
        assert!(l.steps[0].inner < 0.05);
        // Sonst beginnt sie erst, wo der Auftritt 10 cm erreicht: schmaler, Lauflinie weiter außen.
        let general = s.layout(2.6);
        assert!(general.steps.iter().all(|st| st.inner >= MIN_INNER_GENERAL - 1e-6));
        assert!(general.width < l.width);
    }
}
