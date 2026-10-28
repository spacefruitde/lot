//! Prüfbericht (PLAN.md, Abschnitt 4: „Ergebnisse erscheinen als Hinweise
//! direkt im Modell und als Prüfbericht“). Jede Prüfung liefert Befunde mit
//! Quelle; was einen Ort hat, erscheint im Grundriss als Hinweis.

use lot_geom::ModelGeometry;
use lot_model::{ElementId, Point2, Project};

use crate::landesrecht::StateRules;

/// Ergebnis eines Befunds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Status {
    /// Erfüllt.
    Ok,
    /// Hinweis: nicht geprüft, nur teilweise prüfbar oder mit Bedingung.
    Note,
    /// Nicht erfüllt.
    Fail,
}

impl Status {
    pub fn label(self) -> &'static str {
        match self {
            Status::Ok => "erfüllt",
            Status::Note => "Hinweis",
            Status::Fail => "nicht erfüllt",
        }
    }
}

/// Ein Befund.
#[derive(Debug, Clone, PartialEq)]
pub struct Finding {
    pub status: Status,
    pub text: String,
    /// Norm oder Verordnung mit Abschnitt, etwa „§ 6 Abs. 5 MBO“.
    pub source: String,
    pub storey: Option<ElementId>,
    /// Ort im Grundriss für den Hinweis im Modell.
    pub at: Option<Point2>,
}

impl Finding {
    pub fn new(status: Status, text: impl Into<String>, source: impl Into<String>) -> Self {
        Self { status, text: text.into(), source: source.into(), storey: None, at: None }
    }

    pub fn located(mut self, storey: ElementId, at: Point2) -> Self {
        self.storey = Some(storey);
        self.at = Some(at);
        self
    }
}

/// Ein Abschnitt des Prüfberichts.
#[derive(Debug, Clone, PartialEq)]
pub struct ReportSection {
    pub title: String,
    /// Grundlage, etwa „Musterbauordnung (MBO 2002)“.
    pub basis: String,
    pub findings: Vec<Finding>,
}

impl ReportSection {
    pub fn worst(&self) -> Status {
        self.findings.iter().map(|f| f.status).max().unwrap_or(Status::Ok)
    }
}

/// Der ganze Prüfbericht.
#[derive(Debug, Clone, PartialEq)]
pub struct CheckReport {
    pub sections: Vec<ReportSection>,
}

impl CheckReport {
    pub fn findings(&self) -> impl Iterator<Item = &Finding> {
        self.sections.iter().flat_map(|s| s.findings.iter())
    }

    pub fn count(&self, status: Status) -> usize {
        self.findings().filter(|f| f.status == status).count()
    }
}

/// Zahl im deutschen Format.
pub(crate) fn de(v: f64, decimals: usize) -> String {
    format!("{v:.decimals$}").replace('.', ",")
}

/// Höhe im üblichen Format: „±0,00“, „+2,75“, „-0,20“.
pub(crate) fn height(z: f64) -> String {
    if z.abs() < 0.005 {
        "±0,00".into()
    } else if z > 0.0 {
        format!("+{}", de(z, 2))
    } else {
        format!("-{}", de(-z, 2))
    }
}

/// Alle Prüfungen für den Bauantrag (5.01 bis 5.05) und das Maß der baulichen Nutzung.
pub fn check_project(project: &Project, geometry: &ModelGeometry) -> CheckReport {
    let (rules, fallback) = StateRules::for_state(&project.info().state);
    let class = crate::building_class(project, rules);
    let mut sections = Vec::new();

    // Gebäudeklasse.
    let mut f = vec![Finding::new(
        Status::Ok,
        format!(
            "Gebäudeklasse {}: Höhe {} m (OKFF des obersten Geschosses mit Aufenthaltsräumen über Gelände im Mittel {} m), {} Nutzungseinheiten, {} oberirdische Geschosse",
            class.roman(),
            de(class.height, 2),
            height(class.mean_ground),
            class.units.len(),
            class.above_ground.len()
        ),
        class.source.clone(),
    )];
    if fallback {
        f.push(Finding::new(
            Status::Note,
            format!("Für das Land „{}“ fehlen Daten, geprüft nach {}", project.info().state, rules.name),
            "",
        ));
    }
    f.extend(class.notes.iter().map(|n| Finding::new(Status::Note, n.clone(), class.source.clone())));
    sections.push(ReportSection { title: "Gebäudeklasse".into(), basis: rules.name.clone(), findings: f });

    // Abstandsflächen.
    let setbacks = crate::setbacks(project, geometry, &class);
    let mut f = Vec::new();
    let s = &setbacks.rules.setbacks;
    for a in &setbacks.areas {
        let (h0, h1) = a.height();
        let (t0, t1) = a.depth();
        let span =
            |lo: f64, hi: f64| if hi - lo < 0.005 { de(lo, 2) } else { format!("{} bis {}", de(lo, 2), de(hi, 2)) };
        let rule = match s.small_buildings {
            Some(sb) if setbacks.small_building => format!("{} m", de(sb.depth, 2)),
            _ => format!("{} H, mind. {} m", de(s.factor, 1), de(s.minimum, 2)),
        };
        let avail = a.available.map_or("ohne Grundstück".to_string(), |v| format!("vorhanden {} m", de(v, 2)));
        let mid = Point2::new((a.start.x + a.end.x) / 2.0, (a.start.y + a.end.y) / 2.0);
        let status = if a.available.is_none() {
            Status::Note
        } else if a.ok() {
            Status::Ok
        } else {
            Status::Fail
        };
        f.push(
            Finding::new(
                status,
                format!(
                    "{}, Wand {} m lang: H = {} m, T = {} ({}) m, {}",
                    a.facing,
                    de(a.length(), 2),
                    span(h0, h1),
                    span(t0, t1),
                    rule,
                    avail
                ),
                s.source.clone(),
            )
            .located(a.storey, mid),
        );
    }
    if !setbacks.ok() {
        f.push(Finding::new(
            Status::Fail,
            format!(
                "{} m² Abstandsfläche liegen außerhalb der zulässigen Fläche (im Lageplan rot)",
                de(setbacks.violations.area(), 2)
            ),
            format!("{} Abs. 2", s.source),
        ));
    }
    f.extend(setbacks.notes.iter().map(|n| Finding::new(Status::Note, n.clone(), s.source.clone())));
    sections.push(ReportSection { title: "Abstandsflächen".into(), basis: rules.name.clone(), findings: f });

    // Maß der baulichen Nutzung.
    sections.push(crate::land_use::land_use(project, rules, &class));
    // Barrierefreiheit, Rettungswege, Treppen.
    sections.push(crate::accessible::check_accessibility(project, rules));
    sections.push(crate::escape::check_escape(project, rules, &class));
    let mut f = Vec::new();
    for c in crate::check_stairs(project) {
        let text = format!(
            "Treppe {}: {} Steigungen {}/{} cm, Schrittmaß {} cm, Laufbreite {} m{}",
            project.stair(c.stair).and_then(|st| project.storey(st.storey)).map_or("?".into(), |s| s.name.clone()),
            c.risers,
            de(c.rise * 100.0, 1),
            de(c.tread * 100.0, 1),
            de(c.step * 100.0, 1),
            de(project.stair(c.stair).map_or(0.0, |s| s.width), 2),
            {
                let all: Vec<String> = c.problems.iter().chain(&c.notes).cloned().collect();
                if all.is_empty() { String::new() } else { format!(": {}", all.join("; ")) }
            }
        );
        let status = if !c.problems.is_empty() {
            Status::Fail
        } else if !c.notes.is_empty() {
            Status::Note
        } else {
            Status::Ok
        };
        let mut finding = Finding::new(status, text, "DIN 18065:2020-08, Tabelle 1");
        if let Some(st) = project.stair(c.stair) {
            finding = finding.located(st.storey, st.start);
        }
        f.push(finding);
    }
    if !f.is_empty() {
        sections.push(ReportSection { title: "Treppen".into(), basis: "DIN 18065:2020-08".into(), findings: f });
    }
    // Gefälle von Flachdächern, Balkonen und Duschen (ADR 0051).
    let f = crate::falls::check_falls(project);
    if !f.is_empty() {
        sections.push(ReportSection {
            title: "Gefälle".into(),
            basis: "Flachdachrichtlinie, DIN 18531, DIN 1986-100".into(),
            findings: f,
        });
    }
    CheckReport { sections }
}
