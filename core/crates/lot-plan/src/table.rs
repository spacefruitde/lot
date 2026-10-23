//! Tabellen auf Blättern: Tür- und Fensterliste (4.05), Prüfbericht und
//! Abstandsflächenberechnung (5.01 bis 5.05). Eine Tabelle läuft über so viele
//! Seiten wie nötig.

use lot_rules::{CheckReport, OpeningRow, OpeningType, Status};

use crate::{Anchor, Color, PaperPoint, Pen, PlanObject, Sheet, Text, format_de, rect_points};

const MARGIN: f64 = 10.0;
const ROW: f64 = 5.0;
const TEXT: f64 = 2.2;
const SMALL: f64 = 1.8;

/// Spalte: Überschrift, linke oder rechte Kante in mm, Bündigkeit.
pub struct Column {
    pub title: &'static str,
    pub x: f64,
    pub anchor: Anchor,
}

pub enum Row {
    Heading(String),
    Columns,
    Cells(Vec<String>),
    /// Text über die ganze Breite, umbrochen; mit Farbe für den Status.
    Text {
        label: String,
        color: Color,
        text: String,
        source: String,
    },
    Note(String),
    Gap,
}

/// Blätter einer Tabelle. `portrait`: A4 hoch, sonst A4 quer.
pub fn table_sheets(
    title: &str,
    project: &str,
    footnote: &str,
    portrait: bool,
    columns: &[Column],
    rows: &[Row],
) -> Vec<Sheet> {
    let (width, height) = if portrait { (210.0, 297.0) } else { (297.0, 210.0) };
    let left = MARGIN + 10.0;
    let right = width - MARGIN - 4.0;
    // Zeilen umbrechen, damit jede Seite weiß, wie viel Platz sie braucht.
    let chars = ((right - left - 28.0) / (TEXT * 0.64)).floor() as usize;
    let mut lines: Vec<(f64, Line)> = Vec::new();
    for row in rows {
        match row {
            Row::Text { label, color, text, source } => {
                let mut parts = wrap(text, chars);
                if !source.is_empty() {
                    parts.push(format!("({source})"));
                }
                for (i, part) in parts.into_iter().enumerate() {
                    let small = i > 0 && part.starts_with('(');
                    lines.push((
                        if small { 3.8 } else { ROW - 0.6 },
                        Line::Status {
                            label: if i == 0 { label.clone() } else { String::new() },
                            color: *color,
                            text: part,
                            small,
                        },
                    ));
                }
                lines.push((1.2, Line::Gap));
            }
            Row::Heading(t) => lines.push((ROW + 2.0, Line::Heading(t.clone()))),
            Row::Columns => lines.push((ROW, Line::Columns)),
            Row::Cells(c) => lines.push((ROW, Line::Cells(c.clone()))),
            Row::Note(t) => {
                for part in wrap(t, (chars as f64 * 1.2) as usize) {
                    lines.push((ROW - 1.0, Line::Note(part)));
                }
            }
            Row::Gap => lines.push((ROW, Line::Gap)),
        }
    }
    let top = height - MARGIN - 18.0;
    let bottom = MARGIN + 8.0;
    let mut pages: Vec<Vec<(f64, Line)>> = vec![Vec::new()];
    let mut used = 0.0;
    for (h, line) in lines {
        if used + h > top - bottom && !pages.last().is_none_or(|p| p.is_empty()) {
            pages.push(Vec::new());
            used = 0.0;
        }
        used += h;
        pages.last_mut().expect("Seite").push((h, line));
    }
    let count = pages.len();
    pages
        .into_iter()
        .enumerate()
        .map(|(i, page)| {
            let mut sheet =
                Sheet { title: format!("{project}, {title}"), width_mm: width, height_mm: height, objects: Vec::new() };
            sheet.polygon(rect_points(MARGIN, MARGIN, width - 2.0 * MARGIN, height - 2.0 * MARGIN), Pen::mm(0.35));
            sheet.text(left, height - MARGIN - 10.0, title, 5.0, Anchor::Left);
            sheet.text(right, height - MARGIN - 10.0, project, 2.5, Anchor::Right);
            sheet.text(right, MARGIN + 4.0, format!("Seite {} von {count}", i + 1), SMALL, Anchor::Right);
            sheet.text(left, MARGIN + 4.0, footnote, SMALL, Anchor::Left);
            let mut y = top;
            for (h, line) in page {
                draw_line(&mut sheet, &line, y, left, right, columns);
                y -= h;
            }
            sheet
        })
        .collect()
}

enum Line {
    Heading(String),
    Columns,
    Cells(Vec<String>),
    Status { label: String, color: Color, text: String, small: bool },
    Note(String),
    Gap,
}

fn draw_line(sheet: &mut Sheet, line: &Line, y: f64, left: f64, right: f64, columns: &[Column]) {
    let base = y - TEXT;
    match line {
        Line::Heading(t) => sheet.text(left, base - 1.0, t.as_str(), 3.5, Anchor::Left),
        Line::Columns => {
            for c in columns {
                sheet.text(c.x, base, c.title, SMALL, c.anchor);
            }
            sheet
                .line(vec![PaperPoint::new(left, y - ROW + 1.0), PaperPoint::new(right, y - ROW + 1.0)], Pen::mm(0.25));
        }
        Line::Cells(cells) => {
            for (c, text) in columns.iter().zip(cells) {
                sheet.text(c.x, base, text.as_str(), TEXT, c.anchor);
            }
        }
        Line::Status { label, color, text, small } => {
            if !label.is_empty() {
                sheet.objects.push(PlanObject::Text(Text {
                    position: PaperPoint::new(left, base),
                    text: label.clone(),
                    height_mm: TEXT,
                    anchor: Anchor::Left,
                    rotation: 0.0,
                }));
                // Farbiger Punkt vor dem Befund.
                let c = PaperPoint::new(left - 3.5, base + TEXT / 2.0);
                let ring: Vec<PaperPoint> = (0..16)
                    .map(|k| {
                        let t = k as f64 / 16.0 * std::f64::consts::TAU;
                        PaperPoint::new(c.x + 1.1 * t.cos(), c.y + 1.1 * t.sin())
                    })
                    .collect();
                sheet
                    .objects
                    .push(PlanObject::Area(crate::Area { rings: vec![ring], fill: crate::Fill::Solid(*color) }));
            }
            sheet.text(left + 26.0, base, text.as_str(), if *small { SMALL } else { TEXT }, Anchor::Left);
        }
        Line::Note(t) => sheet.text(left, base, t.as_str(), SMALL, Anchor::Left),
        Line::Gap => {}
    }
}

/// Text in Zeilen von höchstens `chars` Zeichen, an Leerzeichen umbrochen.
fn wrap(text: &str, chars: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut line = String::new();
    for word in text.split(' ') {
        if !line.is_empty() && line.chars().count() + 1 + word.chars().count() > chars {
            out.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() {
        out.push(line);
    }
    out
}

fn m(v: f64) -> String {
    crate::dimensions::format_dimension(v)
}

/// Tür- und Fensterliste auf A4 quer: Positionen je Geschoss, danach die Typen.
pub fn schedule_sheets(project: &str, rows: &[OpeningRow], types: &[OpeningType]) -> Vec<Sheet> {
    let x = |mm: f64| MARGIN + 10.0 + mm;
    let columns = [
        Column { title: "Pos.", x: x(0.0), anchor: Anchor::Left },
        Column { title: "Typ", x: x(18.0), anchor: Anchor::Left },
        Column { title: "Art", x: x(28.0), anchor: Anchor::Left },
        Column { title: "Raum", x: x(50.0), anchor: Anchor::Left },
        Column { title: "angrenzend", x: x(95.0), anchor: Anchor::Left },
        Column { title: "Einheit", x: x(140.0), anchor: Anchor::Left },
        Column { title: "Breite", x: x(172.0), anchor: Anchor::Right },
        Column { title: "Höhe", x: x(190.0), anchor: Anchor::Right },
        Column { title: "Brüstung", x: x(208.0), anchor: Anchor::Right },
        Column { title: "lichte Breite", x: x(230.0), anchor: Anchor::Right },
        Column { title: "Anschlag", x: x(234.0), anchor: Anchor::Left },
    ];
    let mut out = Vec::new();
    let mut storey = String::new();
    for r in rows {
        if r.storey != storey {
            storey = r.storey.clone();
            if !out.is_empty() {
                out.push(Row::Gap);
            }
            out.push(Row::Heading(format!("Geschoss {storey}")));
            out.push(Row::Columns);
        }
        let handing = match r.handing {
            Some(lot_model::Handing::DinLeft) => "DIN links",
            Some(lot_model::Handing::DinRight) => "DIN rechts",
            None => "",
        };
        out.push(Row::Cells(vec![
            r.mark.clone(),
            r.type_mark.clone(),
            r.class.title().into(),
            r.room.clone(),
            r.other.clone(),
            r.unit.clone(),
            m(r.width),
            m(r.height),
            if matches!(r.class, lot_rules::schedule::OpeningClass::Window) { m(r.sill) } else { String::new() },
            r.clear_width.map_or(String::new(), m),
            handing.into(),
        ]));
    }
    out.push(Row::Gap);
    out.push(Row::Heading("Typen".into()));
    out.push(Row::Columns);
    for t in types {
        let handing =
            if t.left + t.right > 0 { format!("{} links, {} rechts", t.left, t.right) } else { String::new() };
        out.push(Row::Cells(vec![
            format!("{} Stück", t.count),
            t.mark.clone(),
            t.class.title().into(),
            String::new(),
            String::new(),
            String::new(),
            m(t.width),
            m(t.height),
            if matches!(t.class, lot_rules::schedule::OpeningClass::Window) { m(t.sill) } else { String::new() },
            String::new(),
            handing,
        ]));
    }
    let total: usize = types.iter().map(|t| t.count).sum();
    out.push(Row::Note(format!("{total} Öffnungen. Maße sind Rohbaumaße in m; lichte Breite der Türen: Rohbaumaß minus Zarge und Türblatt (Annahme, siehe Prüfbericht).")));
    table_sheets(
        "Tür- und Fensterliste",
        project,
        "Rohbaumaße nach DIN 18100, Brüstung über OKFF",
        false,
        &columns,
        &out,
    )
}

/// Tür- und Fensterliste als CSV (Semikolon, Dezimalkomma).
pub fn schedule_csv(rows: &[OpeningRow]) -> String {
    let mut out = String::from(
        "Pos.;Typ;Geschoss;Art;Raum;angrenzend;Einheit;Breite m;Höhe m;Brüstung m;lichte Breite m;Anschlag;Wand\n",
    );
    let f = |v: f64| format_de(v, 3);
    for r in rows {
        let handing = match r.handing {
            Some(lot_model::Handing::DinLeft) => "DIN links",
            Some(lot_model::Handing::DinRight) => "DIN rechts",
            None => "",
        };
        let cells = [
            r.mark.clone(),
            r.type_mark.clone(),
            r.storey.clone(),
            r.class.title().to_string(),
            r.room.clone(),
            r.other.clone(),
            r.unit.clone(),
            f(r.width),
            f(r.height),
            f(r.sill),
            r.clear_width.map_or(String::new(), f),
            handing.to_string(),
            r.wall.clone(),
        ];
        out.push_str(&cells.map(|c| crate::area_table::csv_field(&c)).join(";"));
        out.push('\n');
    }
    out
}

/// Farbe je Status: grün, gelb, rot.
pub fn status_color(status: Status) -> Color {
    match status {
        Status::Ok => Color::rgb8(46, 139, 87),
        Status::Note => Color::rgb8(230, 170, 0),
        Status::Fail => Color::rgb8(226, 35, 26),
    }
}

/// Prüfbericht auf A4 hoch.
pub fn report_sheets(project: &str, report: &CheckReport) -> Vec<Sheet> {
    let mut rows = Vec::new();
    rows.push(Row::Note(format!(
        "{} erfüllt, {} Hinweise, {} nicht erfüllt. Jeder Befund nennt seine Quelle; Befunde mit Ort erscheinen als Hinweis im Grundriss.",
        report.count(Status::Ok),
        report.count(Status::Note),
        report.count(Status::Fail)
    )));
    rows.push(Row::Gap);
    for s in &report.sections {
        rows.push(Row::Heading(s.title.clone()));
        rows.push(Row::Note(format!("Grundlage: {}", s.basis)));
        for f in &s.findings {
            rows.push(Row::Text {
                label: f.status.label().into(),
                color: status_color(f.status),
                text: f.text.clone(),
                source: f.source.clone(),
            });
        }
        rows.push(Row::Gap);
    }
    table_sheets(
        "Prüfbericht",
        project,
        "Automatische Prüfung, ersetzt keine Prüfung durch Fachleute",
        true,
        &[],
        &rows,
    )
}
