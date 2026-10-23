//! Flächenaufstellung nach DIN 277:2021-08 und WoFlV (4.01, 4.02, 4.03) als
//! Blatt und als CSV für die Tabellenkalkulation.

use lot_rules::AreaReport;

use crate::{Anchor, PaperPoint, Pen, Sheet, format_de, rect_points};

const WIDTH: f64 = 210.0;
const HEIGHT: f64 = 297.0;
const MARGIN: f64 = 10.0;
const ROW: f64 = 5.5;
const TEXT: f64 = 2.5;
/// Spalten: Nummer, Raum, Nutzung (links), NRF und Wohnfläche (rechts bündig).
const COL_NUMBER: f64 = 22.0;
const COL_NAME: f64 = 42.0;
const COL_USAGE: f64 = 112.0;
const COL_NET: f64 = 162.0;
const COL_LIVING: f64 = 196.0;

enum Row {
    Heading(String),
    Columns,
    Space { number: String, name: String, usage: String, net: String, living: String },
    Sum { label: String, net: String, living: String },
    Value { label: String, value: String },
    Note(String),
    Gap,
}

fn m2(v: f64) -> String {
    format_de(v, 2)
}

fn rows(report: &AreaReport) -> Vec<Row> {
    let mut rows = Vec::new();
    for s in &report.storeys {
        rows.push(Row::Heading(format!("Geschoss {}", s.name)));
        rows.push(Row::Columns);
        for r in &s.spaces {
            rows.push(Row::Space {
                number: r.number.clone(),
                name: r.name.clone(),
                usage: r.usage.code().to_string(),
                net: r.net.map_or_else(|| "offen".to_string(), m2),
                living: r.living.map_or_else(String::new, m2),
            });
        }
        if s.unassigned > 0.005 {
            rows.push(Row::Sum { label: "Umschlossen ohne Raum".into(), net: m2(s.unassigned), living: String::new() });
        }
        let living: f64 = s.spaces.iter().filter_map(|r| r.living).sum();
        rows.push(Row::Sum {
            label: format!("Summe {}", s.name),
            net: m2(s.net),
            living: if living > 0.0 { m2(living) } else { String::new() },
        });
        rows.push(Row::Value {
            label: "Konstruktions-Grundfläche KGF".into(),
            value: format!("{} m²", m2(s.construction)),
        });
        rows.push(Row::Value { label: "Brutto-Grundfläche BGF".into(), value: format!("{} m²", m2(s.gross)) });
        rows.push(Row::Gap);
    }
    rows.push(Row::Heading("Zusammenstellung nach DIN 277:2021-08".into()));
    for (usage, area) in report.net_by_usage() {
        rows.push(Row::Value {
            label: format!("{} {}", usage.code(), usage.title()),
            value: format!("{} m²", m2(area)),
        });
    }
    rows.push(Row::Value { label: "Nutzungsfläche NUF".into(), value: format!("{} m²", m2(report.usable())) });
    rows.push(Row::Value { label: "Netto-Raumfläche NRF".into(), value: format!("{} m²", m2(report.net())) });
    rows.push(Row::Value {
        label: "Konstruktions-Grundfläche KGF".into(),
        value: format!("{} m²", m2(report.construction())),
    });
    rows.push(Row::Value { label: "Brutto-Grundfläche BGF".into(), value: format!("{} m²", m2(report.gross())) });
    rows.push(Row::Value {
        label: "Brutto-Rauminhalt BRI (vorläufig: BGF mal Geschosshöhe)".into(),
        value: format!("{} m³", m2(report.volume())),
    });
    rows.push(Row::Gap);
    rows.push(Row::Value { label: "Wohnfläche nach WoFlV".into(), value: format!("{} m²", m2(report.living())) });
    rows.push(Row::Note("Ohne Dachschrägen (folgen mit M3) zählen alle Räume mit voller Höhe.".into()));
    if !report.warnings.is_empty() {
        rows.push(Row::Gap);
        rows.push(Row::Heading("Hinweise".into()));
        rows.extend(report.warnings.iter().cloned().map(Row::Note));
    }
    rows
}

/// Die Flächenaufstellung auf A4 hoch, so viele Seiten wie nötig.
pub fn area_sheets(project_name: &str, report: &AreaReport) -> Vec<Sheet> {
    let rows = rows(report);
    let top = HEIGHT - MARGIN - 18.0;
    let bottom = MARGIN + 8.0;
    let per_page = ((top - bottom) / ROW).floor() as usize;
    let pages: Vec<&[Row]> = rows.chunks(per_page.max(1)).collect();
    let count = pages.len();
    pages
        .into_iter()
        .enumerate()
        .map(|(i, page)| {
            let mut sheet = Sheet {
                title: format!("{project_name}, Flächenaufstellung"),
                width_mm: WIDTH,
                height_mm: HEIGHT,
                objects: Vec::new(),
            };
            sheet.polygon(rect_points(MARGIN, MARGIN, WIDTH - 2.0 * MARGIN, HEIGHT - 2.0 * MARGIN), Pen::mm(0.35));
            sheet.text(COL_NUMBER, HEIGHT - MARGIN - 10.0, "Flächenaufstellung", 5.0, Anchor::Left);
            sheet.text(COL_LIVING, HEIGHT - MARGIN - 10.0, project_name, TEXT, Anchor::Right);
            sheet.text(COL_LIVING, MARGIN + 4.0, format!("Seite {} von {count}", i + 1), 1.8, Anchor::Right);
            sheet.text(
                COL_NUMBER,
                MARGIN + 4.0,
                "Flächen in m², gemessen an den fertigen Wandoberflächen",
                1.8,
                Anchor::Left,
            );
            let mut y = top;
            for row in page {
                draw_row(&mut sheet, row, y);
                y -= ROW;
            }
            sheet
        })
        .collect()
}

fn draw_row(sheet: &mut Sheet, row: &Row, y: f64) {
    let base = y - TEXT;
    let rule = |sheet: &mut Sheet, pen: f64| {
        sheet.line(
            vec![PaperPoint::new(COL_NUMBER, y - ROW + 1.0), PaperPoint::new(COL_LIVING, y - ROW + 1.0)],
            Pen::mm(pen),
        );
    };
    match row {
        Row::Heading(text) => sheet.text(COL_NUMBER, base, text.as_str(), 3.5, Anchor::Left),
        Row::Columns => {
            for (x, text, anchor) in [
                (COL_NUMBER, "Nr.", Anchor::Left),
                (COL_NAME, "Raum", Anchor::Left),
                (COL_USAGE, "Nutzung", Anchor::Left),
                (COL_NET, "NRF", Anchor::Right),
                (COL_LIVING, "Wohnfläche", Anchor::Right),
            ] {
                sheet.text(x, base, text, 1.8, anchor);
            }
            rule(sheet, 0.25);
        }
        Row::Space { number, name, usage, net, living } => {
            sheet.text(COL_NUMBER, base, number.as_str(), TEXT, Anchor::Left);
            sheet.text(COL_NAME, base, name.as_str(), TEXT, Anchor::Left);
            sheet.text(COL_USAGE, base, usage.as_str(), TEXT, Anchor::Left);
            sheet.text(COL_NET, base, net.as_str(), TEXT, Anchor::Right);
            sheet.text(COL_LIVING, base, living.as_str(), TEXT, Anchor::Right);
        }
        Row::Sum { label, net, living } => {
            sheet.line(vec![PaperPoint::new(COL_USAGE, y + 0.5), PaperPoint::new(COL_LIVING, y + 0.5)], Pen::mm(0.18));
            sheet.text(COL_NAME, base, label.as_str(), TEXT, Anchor::Left);
            sheet.text(COL_NET, base, net.as_str(), TEXT, Anchor::Right);
            sheet.text(COL_LIVING, base, living.as_str(), TEXT, Anchor::Right);
        }
        Row::Value { label, value } => {
            sheet.text(COL_NAME, base, label.as_str(), TEXT, Anchor::Left);
            sheet.text(COL_LIVING, base, value.as_str(), TEXT, Anchor::Right);
        }
        Row::Note(text) => sheet.text(COL_NAME, base, text.as_str(), 1.8, Anchor::Left),
        Row::Gap => {}
    }
}

/// Raumliste als CSV: Semikolon getrennt, Dezimalkomma, wie es deutsche
/// Tabellenkalkulationen erwarten.
pub fn area_csv(report: &AreaReport) -> String {
    let mut out = String::from("Geschoss;Nr.;Raum;Nutzung;NRF m²;Wohnfläche m²\n");
    for s in &report.storeys {
        for r in &s.spaces {
            let field = |v: Option<f64>| v.map_or_else(String::new, m2);
            out.push_str(&format!(
                "{};{};{};{};{};{}\n",
                csv_text(&s.name),
                csv_text(&r.number),
                csv_text(&r.name),
                r.usage.code(),
                field(r.net),
                field(r.living)
            ));
        }
    }
    out
}

fn csv_text(text: &str) -> String {
    if text.contains([';', '"', '\n']) { format!("\"{}\"", text.replace('"', "\"\"")) } else { text.to_string() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csv_quotes_and_uses_decimal_commas() {
        let report = AreaReport {
            storeys: vec![lot_rules::StoreyAreas {
                storey: lot_model::ElementId(1),
                name: "EG".into(),
                spaces: vec![lot_rules::SpaceArea {
                    id: lot_model::ElementId(2),
                    number: "0.01".into(),
                    name: "Wohnen; Essen".into(),
                    usage: lot_model::SpaceUsage::Living,
                    net: Some(24.345),
                    living: Some(24.5),
                }],
                gross: 30.0,
                net: 24.345,
                unassigned: 0.0,
                construction: 5.655,
                height: 2.75,
            }],
            warnings: Vec::new(),
        };
        let csv = area_csv(&report);
        assert_eq!(csv.lines().nth(1), Some("EG;0.01;\"Wohnen; Essen\";NUF 1;24,35;24,50"));
        assert_eq!(area_sheets("Test", &report).len(), 1);
    }
}
