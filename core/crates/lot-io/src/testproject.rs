//! Testprojekte als lesbare RON-Textdateien (`testprojekte/<name>/projekt.ron`).
//!
//! Claude kann sie schreiben und ändern, Unterschiede sind im Git-Diff lesbar.
//! Beispiel siehe `testprojekte/m0-wand/projekt.ron`.

use std::path::Path;

use lot_model::Project;
use ron::extensions::Extensions;

use crate::IoError;

/// Dateiname des Modells in einem Testprojektordner.
pub const MODEL_FILE: &str = "projekt.ron";

/// Liest ein Testprojekt. `path` ist der Projektordner oder direkt die RON-Datei.
pub fn load(path: &Path) -> Result<Project, IoError> {
    let file = if path.is_dir() { path.join(MODEL_FILE) } else { path.to_path_buf() };
    let text = std::fs::read_to_string(&file)?;
    from_str(&text)
}

pub fn from_str(text: &str) -> Result<Project, IoError> {
    let project: Project = options().from_str(text)?;
    project.validate()?;
    Ok(project)
}

pub fn to_string(project: &Project) -> Result<String, IoError> {
    let pretty = ron::ser::PrettyConfig::new().struct_names(true).indentor("    ");
    Ok(options().to_string_pretty(project, pretty)?)
}

fn options() -> ron::Options {
    ron::Options::default().with_default_extension(Extensions::UNWRAP_VARIANT_NEWTYPES)
}

/// Dateien, auf die ein Testprojekt verweist (Unterlagen), aus seinem Ordner.
/// Fehlende Dateien fehlen auch im Ergebnis.
pub fn attachments(project: &Project, dir: &Path) -> crate::project_file::Attachments {
    project
        .elements
        .values()
        .filter_map(|e| match e {
            lot_model::Element::Underlay(u) => Some(u.source.clone()),
            _ => None,
        })
        .filter_map(|name| std::fs::read(dir.join(&name)).ok().map(|data| (name, data)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let p = Project::sample();
        let text = to_string(&p).unwrap();
        assert_eq!(from_str(&text).unwrap(), p);
    }

    #[test]
    fn m0_testproject_matches_sample() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../testprojekte/m0-wand");
        assert_eq!(load(&dir).unwrap(), Project::sample());
    }
}
