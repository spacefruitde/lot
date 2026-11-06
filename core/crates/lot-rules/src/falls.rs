//! Gefälle (ADR 0051): Mindestgefälle je Verwendung, Abläufe und
//! Notüberläufe auf Flachdächern. Werte als Daten in `normen/gefaelle.ron`.

use std::sync::OnceLock;

use lot_model::{Element, FallPurpose, Point2, Project, SlabKind};
use serde::Deserialize;

use crate::report::{Finding, Status};

#[derive(Debug, Clone, Deserialize)]
struct Minimum {
    minimum: f64,
    source: String,
    /// Was es heißt, wenn das Gefälle kleiner ist; leer: nicht erfüllt.
    below: String,
}

#[derive(Debug, Clone, Deserialize)]
struct FallNorm {
    roof: Minimum,
    balcony: Minimum,
    shower: Minimum,
    drains_source: String,
}

fn norm() -> &'static FallNorm {
    static NORM: OnceLock<FallNorm> = OnceLock::new();
    NORM.get_or_init(|| ron::from_str(include_str!("../normen/gefaelle.ron")).expect("Gefällewerte lesbar"))
}

fn centroid(points: &[Point2]) -> Point2 {
    let n = points.len().max(1) as f64;
    Point2::new(points.iter().map(|p| p.x).sum::<f64>() / n, points.iter().map(|p| p.y).sum::<f64>() / n)
}

fn percent(v: f64) -> String {
    format!("{v:.1} %").replace('.', ",")
}

fn slab_name(project: &Project, slab: &lot_model::Slab) -> String {
    let kind = match slab.kind {
        SlabKind::Roof => "Flachdach",
        SlabKind::Base => "Bodenplatte",
        SlabKind::Floor => "Decke",
    };
    match project.slab_type(slab.slab_type) {
        Some(t) => format!("{kind} {}", t.name),
        None => kind.to_string(),
    }
}

/// Gefälle aller Decken und Flachdächer; leer, wenn es keine gibt.
pub fn check_falls(project: &Project) -> Vec<Finding> {
    let n = norm();
    let mut out = Vec::new();
    for e in project.elements.values() {
        let Element::Slab(slab) = e else { continue };
        let name = slab_name(project, slab);
        let at = centroid(&slab.outline);
        let Some(fall) = &slab.fall else {
            if slab.kind == SlabKind::Roof {
                out.push(
                    Finding::new(Status::Note, format!("{name}: Flachdach ohne Gefälle"), n.roof.source.clone())
                        .located(slab.storey, at),
                );
            }
            continue;
        };
        let rule = match fall.purpose {
            FallPurpose::Roof => &n.roof,
            FallPurpose::Balcony => &n.balcony,
            FallPurpose::Shower => &n.shower,
        };
        let status = if fall.percent + 1e-9 >= rule.minimum {
            Status::Ok
        } else if rule.below.is_empty() {
            Status::Fail
        } else {
            Status::Note
        };
        let mut text = format!(
            "{name} ({}): Gefälle {}, mindestens {}",
            fall.purpose.title(),
            percent(fall.percent),
            percent(rule.minimum)
        );
        if status == Status::Note {
            text = format!("{text}; {}", rule.below);
        }
        out.push(Finding::new(status, text, rule.source.clone()).located(slab.storey, at));
        if fall.purpose == FallPurpose::Roof {
            if fall.drains.is_empty() {
                out.push(
                    Finding::new(Status::Note, format!("{name}: kein Ablauf"), n.drains_source.clone())
                        .located(slab.storey, at),
                );
            }
            if fall.overflows.is_empty() {
                out.push(
                    Finding::new(Status::Note, format!("{name}: kein Notüberlauf"), n.drains_source.clone())
                        .located(slab.storey, at),
                );
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use lot_model::{ElementId, Fall, Slab};

    use super::*;

    fn with_fall(fall: Option<Fall>, kind: SlabKind) -> Project {
        let mut p = Project::sample();
        let id = p.next_id();
        let ty = p.next_id().0 + 1;
        p.elements.insert(
            ElementId(ty),
            Element::SlabType(lot_model::SlabType {
                name: "Flachdach".into(),
                layers: vec![],
                props: Default::default(),
                locked: Default::default(),
            }),
        );
        p.elements.insert(
            id,
            Element::Slab(Slab {
                storey: ElementId(1),
                slab_type: ElementId(ty),
                outline: vec![
                    Point2::new(0.0, 0.0),
                    Point2::new(5.0, 0.0),
                    Point2::new(5.0, 4.0),
                    Point2::new(0.0, 4.0),
                ],
                top_offset: 0.0,
                kind,
                phase: Default::default(),
                props: Default::default(),
                follows_walls: false,
                boundary_walls: Vec::new(),
                fall: fall.map(Box::new),
            }),
        );
        p
    }

    #[test]
    fn flat_roof_needs_fall_drain_and_overflow() {
        let f = check_falls(&with_fall(None, SlabKind::Roof));
        assert!(f[0].text.ends_with("Flachdach ohne Gefälle"), "{}", f[0].text);
        let fall = Fall {
            purpose: FallPurpose::Roof,
            layer: 0,
            percent: 1.5,
            direction: 0.0,
            drains: Vec::new(),
            overflows: Vec::new(),
        };
        let f = check_falls(&with_fall(Some(fall.clone()), SlabKind::Roof));
        assert_eq!(f[0].status, Status::Note);
        assert!(f[0].text.contains("Sonderkonstruktion"), "{}", f[0].text);
        assert!(f.iter().any(|x| x.text.ends_with("kein Ablauf")));
        assert!(f.iter().any(|x| x.text.ends_with("kein Notüberlauf")));
        let ok =
            Fall { percent: 2.0, drains: vec![Point2::new(4.5, 2.0)], overflows: vec![Point2::new(5.0, 3.0)], ..fall };
        let f = check_falls(&with_fall(Some(ok), SlabKind::Roof));
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].status, Status::Ok);
    }

    #[test]
    fn shower_below_minimum_fails() {
        let fall = Fall {
            purpose: FallPurpose::Shower,
            layer: 0,
            percent: 1.0,
            direction: 90.0,
            drains: vec![Point2::new(1.0, 1.0)],
            overflows: Vec::new(),
        };
        let f = check_falls(&with_fall(Some(fall), SlabKind::Floor));
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].status, Status::Fail);
    }
}
