//! SVG-Ausgabe eines Planblatts. Einheit ist der Millimeter.
//!
//! Das SVG dient vor allem den Golden-Tests: Zahlen werden mit fester
//! Genauigkeit geschrieben, damit sich Unterschiede im Text-Diff lesen lassen.

use std::fmt::Write;

use lot_plan::{Anchor, Fill, Gray, Line, LineStyle, PaperPoint, PlanObject, Sheet, hatch_segments};

use crate::plan_font::FAMILY;

pub fn to_svg(sheet: &Sheet) -> String {
    let (w, h) = (sheet.width_mm, sheet.height_mm);
    let y = |p: &PaperPoint| h - p.y;
    let mut s = String::new();
    let _ =
        writeln!(s, r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}mm" height="{h}mm" viewBox="0 0 {w} {h}">"#);
    let _ = writeln!(s, "<title>{}</title>", escape(&sheet.title));
    let _ = writeln!(s, r#"<rect width="{w}" height="{h}" fill="white"/>"#);
    for object in &sheet.objects {
        match object {
            PlanObject::Line(line) => write_line(&mut s, line, &y),
            PlanObject::Area(area) => match &area.fill {
                Fill::Solid(gray) => {
                    let mut d = String::new();
                    for ring in &area.rings {
                        d.push_str(&path_data(ring, true, &y));
                    }
                    let _ = writeln!(s, r#"<path d="{d}" fill="{}" fill-rule="evenodd"/>"#, color(*gray));
                }
                Fill::Hatch { angle_deg, spacing_mm, pen } => {
                    let _ =
                        writeln!(s, r#"<g stroke="black" stroke-width="{}" stroke-linecap="butt">"#, num(pen.width_mm));
                    for [a, b] in hatch_segments(&area.rings, *angle_deg, *spacing_mm) {
                        let _ = writeln!(
                            s,
                            r#"<line x1="{}" y1="{}" x2="{}" y2="{}"/>"#,
                            num(a.x),
                            num(y(&a)),
                            num(b.x),
                            num(y(&b))
                        );
                    }
                    s.push_str("</g>\n");
                }
            },
            PlanObject::Text(text) => {
                let anchor = match text.anchor {
                    Anchor::Left => "start",
                    Anchor::Center => "middle",
                    Anchor::Right => "end",
                };
                let size = crate::plan_font::plan_font().size_for_cap_height(text.height_mm);
                // SVG zählt y nach unten, eine Drehung gegen den Uhrzeigersinn ist dort negativ.
                let rotate = if text.rotation == 0.0 {
                    String::new()
                } else {
                    format!(
                        r#" transform="rotate({} {} {})""#,
                        num(-text.rotation),
                        num(text.position.x),
                        num(y(&text.position))
                    )
                };
                let _ = writeln!(
                    s,
                    r#"<text x="{}" y="{}" font-family="{FAMILY}" font-size="{}" text-anchor="{anchor}"{rotate}>{}</text>"#,
                    num(text.position.x),
                    num(y(&text.position)),
                    num(size),
                    escape(&text.text)
                );
            }
        }
    }
    s.push_str("</svg>\n");
    s
}

fn write_line(s: &mut String, line: &Line, y: &impl Fn(&PaperPoint) -> f64) {
    let dash = match &line.style {
        LineStyle::Solid => String::new(),
        LineStyle::Dashed(pattern) => {
            let values: Vec<String> = pattern.iter().map(|v| num(*v)).collect();
            format!(r#" stroke-dasharray="{}""#, values.join(" "))
        }
    };
    let _ = writeln!(
        s,
        r#"<path d="{}" fill="none" stroke="{}" stroke-width="{}" stroke-linecap="butt" stroke-linejoin="miter"{dash}/>"#,
        path_data(&line.points, line.closed, y),
        color(line.color),
        num(line.pen.width_mm)
    );
}

fn path_data(points: &[PaperPoint], closed: bool, y: &impl Fn(&PaperPoint) -> f64) -> String {
    let mut d = String::new();
    for (i, p) in points.iter().enumerate() {
        let _ = write!(d, "{}{} {} ", if i == 0 { "M" } else { "L" }, num(p.x), num(y(p)));
    }
    if closed {
        d.push('Z');
    }
    d.trim_end().to_string()
}

fn color(gray: Gray) -> String {
    let v = (gray.0.clamp(0.0, 1.0) * 255.0).round() as u8;
    format!("#{v:02x}{v:02x}{v:02x}")
}

/// Zahl mit höchstens drei Nachkommastellen, ohne überflüssige Nullen.
fn num(v: f64) -> String {
    let s = format!("{v:.3}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s == "-0" { "0".into() } else { s.into() }
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use lot_plan::Pen;

    #[test]
    fn line_width_is_in_mm() {
        let mut sheet = Sheet::a4_landscape("t");
        sheet.line(vec![PaperPoint::new(0.0, 0.0), PaperPoint::new(50.0, 0.0)], Pen::mm(0.5));
        let svg = to_svg(&sheet);
        assert!(svg.contains(r#"viewBox="0 0 297 210""#));
        assert!(svg.contains(r##"d="M0 210 L50 210" fill="none" stroke="#000000" stroke-width="0.5""##));
    }

    #[test]
    fn numbers_are_stable() {
        assert_eq!(num(0.1 + 0.2), "0.3");
        assert_eq!(num(-0.0001), "0");
        assert_eq!(num(12.5), "12.5");
    }
}
