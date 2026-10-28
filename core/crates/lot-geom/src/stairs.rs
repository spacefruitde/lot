//! Treppen und Geländer als Körper.
//!
//! Ein Lauf ist eine Laufplatte mit aufgesetzten Stufen: Das Profil (Stufen
//! oben, schräge Unterseite parallel zur Lauflinie) wird quer zur Laufrichtung
//! über die Laufbreite gezogen. Podeste sind Platten in der Dicke der
//! Laufplatte. Material vorerst Stahlbeton.

use lot_model::{FlightLayout, Point2, Project, Railing, RailingPath, Stair, StairLayout, WallSide};
use manifold_rust::linalg::{Vec2, Vec3};
use manifold_rust::manifold::Manifold;
use manifold_rust::types::OpType;

use crate::{Area, extrude};

/// Material von Treppenläufen und Podesten.
pub const STAIR_MATERIAL: &str = "stahlbeton";
/// Material von Geländern.
pub const RAILING_MATERIAL: &str = "stahl";

/// Körper einer Treppe: Schicht 0, Stahlbeton.
pub fn stair_solids(project: &Project, stair: &Stair) -> Vec<(usize, String, Manifold)> {
    let (Some(layout), Some(storey)) = (project.stair_layout(stair), project.storey(stair.storey)) else {
        return Vec::new();
    };
    let z0 = storey.elevation;
    let mut parts: Vec<Manifold> = Vec::new();
    for (i, f) in layout.flights.iter().enumerate() {
        // Der erste Lauf steht auf dem Boden, die weiteren auf einem Podest.
        let floor = if i == 0 { f.base } else { f.base - stair.thickness };
        parts.push(flight_solid(&layout, f, floor, stair.thickness, z0));
    }
    for (outline, top) in &layout.landings {
        let ring: Vec<[f64; 2]> = outline.iter().map(|p| [p.x, p.y]).collect();
        parts.push(extrude(&Area::from_ring(&ring), z0 + top - stair.thickness, z0 + top));
    }
    let parts: Vec<Manifold> = parts.into_iter().filter(|m| !m.is_empty()).collect();
    if parts.is_empty() {
        return Vec::new();
    }
    vec![(0, STAIR_MATERIAL.to_string(), Manifold::batch_boolean(&parts, OpType::Add))]
}

/// Ein Lauf als Körper. Das Profil liegt in der Ebene aus Laufrichtung (u)
/// und Höhe (z) und wird über die Laufbreite gezogen.
fn flight_solid(layout: &StairLayout, f: &FlightLayout, floor: f64, thickness: f64, z0: f64) -> Manifold {
    let (r, t, n) = (layout.rise, layout.tread, f.risers as usize);
    let u_end = (n - 1) as f64 * t;
    // Senkrechte Dicke der Laufplatte und ihre Unterseite z(u), parallel zur
    // Linie durch die Stufenvorderkanten.
    let cos = t / t.hypot(r);
    let vertical = thickness / cos;
    let under = |u: f64| f.base + r - vertical + r / t * u;
    let mut profile: Vec<Vec2> = Vec::new();
    let u0 = (floor - under(0.0)) * t / r;
    if u0 > 0.0 {
        profile.push(Vec2::new(0.0, floor));
        profile.push(Vec2::new(u0.min(u_end), floor.max(under(u0.min(u_end)))));
    } else {
        profile.push(Vec2::new(0.0, under(0.0).min(f.base)));
    }
    profile.push(Vec2::new(u_end, under(u_end).max(floor).min(f.base + n as f64 * r)));
    // Stufen von oben nach unten zurück.
    profile.push(Vec2::new(u_end, f.base + n as f64 * r));
    for k in (1..n).rev() {
        let u = (k - 1) as f64 * t;
        let z = f.base + k as f64 * r;
        profile.push(Vec2::new(u + t, z));
        profile.push(Vec2::new(u, z));
    }
    // Erste Steigung hinunter zum Boden.
    profile.push(Vec2::new(0.0, f.base));
    profile.dedup_by(|a, b| (a.x - b.x).abs() < 1e-9 && (a.y - b.y).abs() < 1e-9);
    if profile.len() > 1 && (profile[0].x - profile[profile.len() - 1].x).abs() < 1e-9 {
        let (first, last) = (profile[0], profile[profile.len() - 1]);
        if (first.y - last.y).abs() < 1e-9 {
            profile.pop();
        }
    }
    // Gegen den Uhrzeigersinn ausrichten.
    let area: f64 = (0..profile.len())
        .map(|i| {
            let (a, b) = (profile[i], profile[(i + 1) % profile.len()]);
            a.x * b.y - b.x * a.y
        })
        .sum();
    if area < 0.0 {
        profile.reverse();
    }
    let w = layout.width;
    let solid = Manifold::extrude(&vec![profile], w, 0, 0.0, Vec2::new(1.0, 1.0));
    // (u, z, s) → Welt: Antritt + u·Richtung + z·oben + (w/2 - s)·links.
    // Die Abbildung erhält die Orientierung (Determinante +1).
    let (d, left) = (f.dir, Point2::new(-f.dir.y, f.dir.x));
    let origin = f.start;
    solid.warp(move |v: &mut Vec3| {
        let (u, z, s) = (v.x, v.y, v.z);
        let across = w / 2.0 - s;
        *v = Vec3::new(origin.x + u * d.x + across * left.x, origin.y + u * d.y + across * left.y, z0 + z);
    })
}

/// Handlauf und Stäbe eines Geländers: Schicht 0 Handlauf und Pfosten, Schicht 1 Stäbe.
pub fn railing_solids(project: &Project, railing: &Railing) -> Vec<(usize, String, Manifold)> {
    let Some(storey) = project.storey(railing.storey) else { return Vec::new() };
    let z0 = storey.elevation;
    // Segmente mit Höhe unten (Boden oder Stufe) an Anfang und Ende und der
    // Art, wie der Boden darunter verläuft.
    let mut segments: Vec<Segment> = Vec::new();
    match &railing.path {
        RailingPath::Points(points) => {
            for p in points.windows(2) {
                segments.push(Segment { a: p[0], b: p[1], za: 0.0, zb: 0.0, steps: None });
            }
        }
        RailingPath::Flight { stair, flight, side } => {
            let Some(s) = project.stair(*stair) else { return Vec::new() };
            let Some(layout) = project.stair_layout(s) else { return Vec::new() };
            let Some(f) = layout.flights.get(*flight) else { return Vec::new() };
            let [(a, za), (b, zb)] = layout.flight_side(f, *side, RAILING_INSET);
            // Die Vorderkante der ersten Stufe liegt eine Steigung über dem Antritt.
            segments.push(Segment { a, b, za, zb, steps: Some((f.base, layout.rise, layout.tread)) });
        }
    }
    let h = railing.height;
    let mut rails: Vec<Manifold> = Vec::new();
    let mut bars: Vec<Manifold> = Vec::new();
    for s in &segments {
        let len = s.a.distance(s.b);
        if len < 1e-6 {
            continue;
        }
        // Handlauf: Oberkante `h` über der Vorderkantenlinie.
        rails.push(bar_between(s.a, s.b, z0 + s.za + h - HANDRAIL, z0 + s.zb + h - HANDRAIL, HANDRAIL, HANDRAIL));
        // Pfosten an den Enden, Stäbe dazwischen mit höchstens 12 cm Abstand.
        let count = (len / BAR_SPACING).ceil().max(1.0) as usize;
        for i in 0..=count {
            let t = i as f64 / count as f64;
            let p = Point2::new(s.a.x + (s.b.x - s.a.x) * t, s.a.y + (s.b.y - s.a.y) * t);
            let top = s.za + (s.zb - s.za) * t + h - HANDRAIL;
            let bottom = match s.steps {
                // Auf der Stufe, auf der der Stab steht.
                Some((base, rise, tread)) => {
                    let u = t * len;
                    base + ((u / tread).floor() + 1.0) * rise
                }
                None => 0.0,
            };
            let post = i == 0 || i == count;
            let size = if post { POST } else { BAR };
            let solid = post_at(p, z0 + bottom, z0 + top, size);
            if post { rails.push(solid) } else { bars.push(solid) }
        }
    }
    let mut out = Vec::new();
    if !rails.is_empty() {
        out.push((0, RAILING_MATERIAL.to_string(), Manifold::batch_boolean(&rails, OpType::Add)));
    }
    if !bars.is_empty() {
        out.push((1, RAILING_MATERIAL.to_string(), Manifold::batch_boolean(&bars, OpType::Add)));
    }
    out
}

/// Geländerachse innerhalb der Laufseite.
pub const RAILING_INSET: f64 = 0.05;
const HANDRAIL: f64 = 0.04;
const POST: f64 = 0.04;
const BAR: f64 = 0.02;
/// Achsabstand der Stäbe; die lichte Weite bleibt unter 12 cm.
const BAR_SPACING: f64 = 0.12;

struct Segment {
    a: Point2,
    b: Point2,
    /// Höhe der Bezugslinie (Boden oder Stufenvorderkanten) an a und b.
    za: f64,
    zb: f64,
    /// Bei Treppen: Höhe des Antritts, Steigung und Auftritt für den Fuß der Stäbe.
    steps: Option<(f64, f64, f64)>,
}

/// Quader `size` × `size` von `bottom` bis `top`.
fn post_at(p: Point2, bottom: f64, top: f64, size: f64) -> Manifold {
    let h = size / 2.0;
    let ring = [[p.x - h, p.y - h], [p.x + h, p.y - h], [p.x + h, p.y + h], [p.x - h, p.y + h]];
    extrude(&Area::from_ring(&ring), bottom, top.max(bottom + 0.01))
}

/// Stab mit quadratischem Querschnitt von a nach b, Unterkante bei za und zb.
fn bar_between(a: Point2, b: Point2, za: f64, zb: f64, width: f64, height: f64) -> Manifold {
    let len = a.distance(b);
    let (dx, dy) = ((b.x - a.x) / len, (b.y - a.y) / len);
    let (nx, ny) = (-dy * width / 2.0, dx * width / 2.0);
    let points: Vec<Vec3> = [(a, za), (b, zb)]
        .iter()
        .flat_map(|&(p, z)| {
            [
                Vec3::new(p.x + nx, p.y + ny, z),
                Vec3::new(p.x - nx, p.y - ny, z),
                Vec3::new(p.x + nx, p.y + ny, z + height),
                Vec3::new(p.x - nx, p.y - ny, z + height),
            ]
        })
        .collect();
    Manifold::hull(&points)
}

/// Seiten eines Laufs, für die Darstellung im Grundriss.
pub fn flight_outline(layout: &StairLayout, f: &FlightLayout) -> [Point2; 4] {
    let [(a, _), (b, _)] = layout.flight_side(f, WallSide::Right, 0.0);
    let [(c, _), (d, _)] = layout.flight_side(f, WallSide::Left, 0.0);
    [a, b, d, c]
}

#[cfg(test)]
mod tests {
    use lot_model::{Element, ElementId, Landing, StairFlight};

    use super::*;

    fn project_with_stair() -> (Project, Stair) {
        let mut p = Project::sample();
        let storey = p.storeys()[0].0;
        let stair = Stair {
            phase: Default::default(),
            storey,
            start: Point2::new(0.5, 0.0),
            direction: Point2::new(0.0, 1.0),
            width: 1.0,
            tread: 0.27,
            flights: vec![
                StairFlight { risers: 8, landing: Landing::BackRight },
                StairFlight { risers: 8, landing: Landing::Straight },
            ],
            thickness: 0.18,
            necessary: true,
        };
        p.elements.insert(ElementId(900), Element::Stair(stair.clone()));
        (p, stair)
    }

    #[test]
    fn stair_solid_spans_the_storey_height() {
        let (p, stair) = project_with_stair();
        let storey = p.storey(stair.storey).unwrap();
        let solids = stair_solids(&p, &stair);
        assert_eq!(solids.len(), 1);
        let m = crate::to_trimesh(&solids[0].2);
        let (lo, hi) = m.bounds().unwrap();
        assert!((hi[2] - (storey.elevation + storey.height)).abs() < 1e-6, "{hi:?}");
        assert!((lo[2] - storey.elevation).abs() < 1e-6, "{lo:?}");
        // Zwei Läufe nebeneinander, 2 m breit, 1,89 m Lauf plus 1 m Podest.
        assert!((hi[0] - lo[0] - 2.0).abs() < 1e-6 && (hi[1] - lo[1] - 2.89).abs() < 1e-6, "{lo:?} {hi:?}");
        // Volumen: Stufen plus Laufplatte, grob zwischen 0,8 und 1,6 m³.
        let v = solids[0].2.volume();
        assert!((0.8..1.6).contains(&v), "{v}");
    }

    #[test]
    fn railing_follows_the_flight() {
        let (mut p, _) = project_with_stair();
        let storey = p.storeys()[0].0;
        let railing = Railing {
            phase: Default::default(),
            around: None,
            storey,
            path: RailingPath::Flight { stair: ElementId(900), flight: 0, side: WallSide::Left },
            height: 0.9,
        };
        p.elements.insert(ElementId(901), Element::Railing(railing.clone()));
        let solids = railing_solids(&p, &railing);
        assert_eq!(solids.len(), 2);
        let (lo, hi) = crate::to_trimesh(&solids[0].2).bounds().unwrap();
        let z0 = p.storey(storey).unwrap().elevation;
        // Oberkante Handlauf 0,90 m über der Vorderkante der obersten Stufe (Podest 1,375 m).
        assert!((hi[2] - (z0 + 1.375 + 0.9)).abs() < 1e-6, "{hi:?}");
        assert!((lo[0] - (0.05 - 0.02)).abs() < 1e-6, "{lo:?}");
    }
}
