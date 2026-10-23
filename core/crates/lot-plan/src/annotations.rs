//! 2D-Elemente (3.09) in Grundrissen, Schnitten und auf Planblättern.

use lot_model::{AnnotationShape, AnnotationView, Point2, Project};

use crate::standard::{CutPattern, OfficeStandard, ScaleStyle};
use crate::{Anchor, Area, Fill, Gray, Line, LineStyle, PaperPoint, Pen, PlanObject, Text, Viewport};

/// Strich und Lücke gestrichelter 2D-Linien, in mm.
const DASH: [f64; 2] = [2.0, 1.0];

/// Zeichnet die 2D-Elemente einer Ansicht. `to_paper` bildet ihre Koordinaten
/// auf das Blatt ab (Modellmeter oder bei Planblättern Millimeter).
pub fn draw_annotations(
    objects: &mut Vec<PlanObject>,
    project: &Project,
    view: AnnotationView,
    standard: &OfficeStandard,
    style: &ScaleStyle,
    to_paper: impl Fn(Point2) -> PaperPoint,
) {
    for (_, a) in project.annotations_in(view) {
        let pen = Pen::mm(a.pen);
        let line_style = if a.dashed { LineStyle::Dashed(DASH.to_vec()) } else { LineStyle::Solid };
        let line = |points: Vec<PaperPoint>, closed: bool| {
            PlanObject::Line(Line { points, closed, pen, style: line_style.clone(), color: Gray::BLACK })
        };
        match &a.shape {
            AnnotationShape::Polyline(points) => {
                objects.push(line(points.iter().map(|p| to_paper(*p)).collect(), false))
            }
            AnnotationShape::Polygon { points, hatch } => {
                let ring: Vec<PaperPoint> = points.iter().map(|p| to_paper(*p)).collect();
                if let Some(key) = hatch {
                    for fill in fills(&standard.material(key).cut, style) {
                        objects.push(PlanObject::Area(Area { rings: vec![ring.clone()], fill }));
                    }
                }
                objects.push(line(ring, true));
            }
            AnnotationShape::Circle { center, radius } => {
                let ring: Vec<PaperPoint> = (0..48)
                    .map(|i| {
                        let t = i as f64 / 48.0 * std::f64::consts::TAU;
                        to_paper(Point2::new(center.x + radius * t.cos(), center.y + radius * t.sin()))
                    })
                    .collect();
                objects.push(line(ring, true));
            }
            AnnotationShape::Text { position, text, height, rotation } => {
                objects.push(PlanObject::Text(Text {
                    position: to_paper(*position),
                    text: text.clone(),
                    height_mm: *height,
                    anchor: Anchor::Left,
                    rotation: *rotation,
                }));
            }
        }
    }
}

/// Füllungen einer Schraffur aus dem Bürostandard.
fn fills(pattern: &CutPattern, style: &ScaleStyle) -> Vec<Fill> {
    let pen = Pen::mm(style.pens.hatch);
    match *pattern {
        CutPattern::Solid { gray } => vec![Fill::Solid(Gray(gray))],
        CutPattern::Lines { angle, spacing } => vec![Fill::Hatch { angle_deg: angle, spacing_mm: spacing, pen }],
        CutPattern::Cross { angle, spacing } => vec![
            Fill::Hatch { angle_deg: angle, spacing_mm: spacing, pen },
            Fill::Hatch { angle_deg: angle + 90.0, spacing_mm: spacing, pen },
        ],
        _ => Vec::new(),
    }
}

/// Abbildung für Grundrisse und Schnitte: Modellmeter über den Viewport.
pub fn model(viewport: Viewport) -> impl Fn(Point2) -> PaperPoint {
    move |p| viewport.to_paper(p.x, p.y)
}

#[cfg(test)]
mod tests {
    use lot_model::{Annotation, Element, ElementId};

    use super::*;

    #[test]
    fn annotations_appear_only_in_their_view() {
        let mut p = Project::sample();
        let storey = p.storeys()[0].0;
        let view = AnnotationView::FloorPlan { storey };
        p.elements.insert(
            ElementId(900),
            Element::Annotation(Annotation {
                view,
                shape: AnnotationShape::Polygon {
                    points: vec![Point2::new(0.0, 0.0), Point2::new(2.0, 0.0), Point2::new(2.0, 1.0)],
                    hatch: Some("beton".into()),
                },
                pen: 0.35,
                dashed: true,
            }),
        );
        p.elements.insert(
            ElementId(901),
            Element::Annotation(Annotation {
                view,
                shape: AnnotationShape::Text {
                    position: Point2::new(1.0, 1.0),
                    text: "Terrasse".into(),
                    height: 2.5,
                    rotation: 0.0,
                },
                pen: 0.25,
                dashed: false,
            }),
        );
        let standard = OfficeStandard::default_standard();
        let style = standard.style_for(100.0);
        let viewport = Viewport { scale: 100.0, origin: PaperPoint::new(0.0, 0.0) };
        let mut objects = Vec::new();
        draw_annotations(&mut objects, &p, view, standard, style, model(viewport));
        assert!(objects.iter().any(|o| matches!(o, PlanObject::Text(t) if t.text == "Terrasse")));
        assert!(objects.iter().any(|o| matches!(o, PlanObject::Line(l) if l.closed && l.pen.width_mm == 0.35)));
        let mut other = Vec::new();
        draw_annotations(
            &mut other,
            &p,
            AnnotationView::Section { view: ElementId(1) },
            standard,
            style,
            model(viewport),
        );
        assert!(other.is_empty());
    }
}
