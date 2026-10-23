//! 2D-Zeichnen (3.09): Linien, Flächen, Kreise und Texte, die nur in einer
//! Ansicht erscheinen, etwa Terrassen, Nordpfeile oder Hinweise.

use serde::{Deserialize, Serialize};

use crate::{ElementId, Point2};

/// 2D-Element (IfcAnnotation) in einer Ansicht.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Annotation {
    pub view: AnnotationView,
    pub shape: AnnotationShape,
    /// Strichstärke in mm auf dem Papier.
    #[serde(default = "default_pen")]
    pub pen: f64,
    #[serde(default)]
    pub dashed: bool,
}

fn default_pen() -> f64 {
    0.25
}

/// Ansicht, in der ein 2D-Element erscheint, und was seine Koordinaten bedeuten.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnnotationView {
    /// Grundriss eines Geschosses; Koordinaten in m wie das Modell.
    FloorPlan { storey: ElementId },
    /// Schnitt oder Ansicht; x in m entlang der Schnittlinie ab ihrem Anfang,
    /// y Höhe in m über ±0,00.
    Section { view: ElementId },
    /// Planblatt; Koordinaten in mm ab der linken unteren Blattecke.
    Sheet { sheet: ElementId },
}

/// Form eines 2D-Elements.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnnotationShape {
    /// Offener Linienzug.
    Polyline(Vec<Point2>),
    /// Geschlossene Fläche, wahlweise mit der Schraffur eines Materials aus dem Bürostandard.
    Polygon {
        points: Vec<Point2>,
        #[serde(default)]
        hatch: Option<String>,
    },
    Circle {
        center: Point2,
        radius: f64,
    },
    /// Text; Schrifthöhe in mm auf dem Papier, Drehung in Grad gegen den Uhrzeigersinn.
    Text {
        position: Point2,
        text: String,
        #[serde(default = "default_text_height")]
        height: f64,
        #[serde(default)]
        rotation: f64,
    },
}

fn default_text_height() -> f64 {
    2.5
}

impl AnnotationShape {
    /// Alle Punkte der Form (beim Kreis der Mittelpunkt).
    pub fn points(&self) -> Vec<Point2> {
        match self {
            AnnotationShape::Polyline(p) | AnnotationShape::Polygon { points: p, .. } => p.clone(),
            AnnotationShape::Circle { center, .. } => vec![*center],
            AnnotationShape::Text { position, .. } => vec![*position],
        }
    }

    /// Dieselbe Form, um (dx, dy) verschoben.
    pub fn translated(&self, dx: f64, dy: f64) -> AnnotationShape {
        let m = |p: &Point2| Point2::new(p.x + dx, p.y + dy);
        match self {
            AnnotationShape::Polyline(p) => AnnotationShape::Polyline(p.iter().map(m).collect()),
            AnnotationShape::Polygon { points, hatch } => {
                AnnotationShape::Polygon { points: points.iter().map(m).collect(), hatch: hatch.clone() }
            }
            AnnotationShape::Circle { center, radius } => {
                AnnotationShape::Circle { center: m(center), radius: *radius }
            }
            AnnotationShape::Text { position, text, height, rotation } => AnnotationShape::Text {
                position: m(position),
                text: text.clone(),
                height: *height,
                rotation: *rotation,
            },
        }
    }
}
