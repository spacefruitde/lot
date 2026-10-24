//! Präsentationspläne (3.11), Vorschlag: Wände als dunkle Fläche ohne
//! Schraffur, Räume farbig nach Nutzung, Raumname und Fläche groß, keine
//! Maße und Achsen, dazu Nordpfeil und Maßstabsleiste. Farben im Bürostandard.

use lot_geom::Area;
use lot_model::{ElementId, Project};

use crate::floor_plan::{DEFAULT_CUT_HEIGHT, format_area};
use crate::standard::OfficeStandard;
use crate::{Anchor, Color, Fill, Line, LineStyle, PaperPoint, Pen, PlanObject, Text, Viewport};

fn rings(area: &Area, viewport: Viewport) -> Vec<Vec<PaperPoint>> {
    area.rings().iter().map(|r| r.iter().map(|p| viewport.to_paper(p[0], p[1])).collect()).collect()
}

fn rgb(c: [u8; 3]) -> Color {
    Color::rgb8(c[0], c[1], c[2])
}

/// Präsentationsgrundriss eines Geschosses.
pub fn presentation_plan(
    project: &Project,
    storey: ElementId,
    scale: f64,
    standard: &OfficeStandard,
    viewport: Viewport,
) -> Vec<PlanObject> {
    let mut out = Vec::new();
    let Some(s) = project.storey(storey) else { return out };
    let style = standard.style_for(scale);
    let p = &standard.presentation;
    let z = s.elevation + DEFAULT_CUT_HEIGHT;

    // Räume farbig.
    let finder = lot_geom::spaces::SpaceFinder::new(project, storey);
    let spaces: Vec<_> = project.spaces_on(storey).filter(|(_, r)| r.phase != lot_model::Phase::Demolished).collect();
    for (_, space) in &spaces {
        let Some(area) = finder.area_at([space.position.x, space.position.y]) else { continue };
        if let Some(c) = p.spaces.get(space.usage.code()) {
            out.push(PlanObject::Area(crate::Area { rings: rings(area, viewport), fill: Fill::Solid(rgb(*c)) }));
        }
    }

    // Wände dunkel, ohne Schichten und Schraffur.
    let cut = lot_geom::layer_regions(project, storey, z);
    let walls = Area::union_all(cut.iter().map(|r| &r.area));
    let wall_rings = rings(&walls, viewport);
    out.push(PlanObject::Area(crate::Area { rings: wall_rings.clone(), fill: Fill::Solid(rgb(p.walls)) }));
    for ring in wall_rings {
        out.push(PlanObject::Line(Line {
            points: ring,
            closed: true,
            pen: Pen::mm(style.pens.view),
            style: LineStyle::Solid,
            color: rgb(p.walls),
        }));
    }

    // Öffnungen, Treppen, Aufzüge wie im Grundriss.
    crate::floor_plan::draw_openings(&mut out, project, storey, z, style, viewport);
    crate::stairs::draw_stairs(&mut out, project, storey, DEFAULT_CUT_HEIGHT, style, viewport);
    crate::elevators::draw_elevators(&mut out, project, storey, style, viewport);

    // Raumname und Fläche, größer als im Werkplan.
    for (_, space) in &spaces {
        if crate::elevators::labels_space(project, storey, space.position) {
            continue;
        }
        let region = finder.area_at([space.position.x, space.position.y]);
        let area = region.map(|a| a.area());
        let c = viewport.to_paper(space.position.x, space.position.y);
        // Name so groß, dass er in den Raum passt (Zeichenbreite etwa 0,6 der Höhe).
        let room_width =
            region.and_then(|a| a.bounds()).map_or(f64::INFINITY, |(lo, hi)| (hi[0] - lo[0]) * 1000.0 / scale);
        let fit = room_width * 0.9 / (0.6 * space.name.chars().count().max(1) as f64);
        let size = (style.space_name * 1.3).min(fit);
        if size < 1.2 {
            continue;
        }
        out.push(PlanObject::Text(Text {
            position: PaperPoint::new(c.x, c.y + 0.4),
            text: space.name.clone(),
            height_mm: size,
            anchor: Anchor::Center,
            rotation: 0.0,
        }));
        if let Some(a) = area {
            out.push(PlanObject::Text(Text {
                position: PaperPoint::new(c.x, c.y - size * 1.1),
                text: format_area(a),
                height_mm: style.space_text,
                anchor: Anchor::Center,
                rotation: 0.0,
            }));
        }
    }

    // Maßstabsleiste und Nordpfeil links unter dem Grundriss.
    if let Some((lo, _)) = walls.bounds() {
        let at = viewport.to_paper(lo[0], lo[1]);
        out.extend(scale_bar(PaperPoint::new(at.x, at.y - 8.0), scale));
        out.extend(crate::site_plan::north_arrow_at(
            PaperPoint::new(at.x - 14.0, at.y + 6.0),
            project.info().north,
            Pen::mm(0.25),
        ));
    }
    out
}

/// Maßstabsleiste mit abwechselnd gefüllten Feldern, 0 bis 10 m (bei 1:200 und
/// kleiner bis 20 m).
fn scale_bar(at: PaperPoint, scale: f64) -> Vec<PlanObject> {
    let (step, count) = if scale > 150.0 { (5.0, 4) } else { (1.0, 5) };
    let f = 1000.0 / scale;
    let h = 1.5;
    let mut out = Vec::new();
    for i in 0..count {
        let x0 = at.x + i as f64 * step * f;
        let x1 = x0 + step * f;
        let ring = vec![
            PaperPoint::new(x0, at.y),
            PaperPoint::new(x1, at.y),
            PaperPoint::new(x1, at.y + h),
            PaperPoint::new(x0, at.y + h),
        ];
        if i % 2 == 0 {
            out.push(PlanObject::Area(crate::Area { rings: vec![ring.clone()], fill: Fill::Solid(Color::BLACK) }));
        }
        out.push(PlanObject::Line(Line {
            points: ring,
            closed: true,
            pen: Pen::mm(0.18),
            style: LineStyle::Solid,
            color: Color::BLACK,
        }));
    }
    for i in [0, count] {
        out.push(PlanObject::Text(Text {
            position: PaperPoint::new(at.x + i as f64 * step * f, at.y - 3.0),
            text: if i == 0 { "0".into() } else { format!("{} m", (i as f64 * step) as i64) },
            height_mm: 1.8,
            anchor: Anchor::Center,
            rotation: 0.0,
        }));
    }
    out
}
