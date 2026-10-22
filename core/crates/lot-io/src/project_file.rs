//! Projektdatei `.lot`: eine SQLite-Datenbank mit Elementen und Operationsprotokoll.
//!
//! Aufbau (Formatversion 1):
//!
//! - `meta(key, value)`: Projektname und Programmversion
//! - `elements(id, kind, data)`: ein Element je Zeile, Parameter als JSON
//! - `operations(seq, data)`: das Operationsprotokoll in Reihenfolge, als JSON
//!
//! Die Datei trägt `application_id` und `user_version`, damit Lot fremde
//! Datenbanken und neuere Formate erkennt.

use std::path::Path;

use lot_model::{Document, Element, ElementId, Operation, Project};
use rusqlite::{Connection, OptionalExtension, params};

use crate::IoError;

/// „LOT1“ als 32-Bit-Zahl, kennzeichnet Lot-Dateien.
const APPLICATION_ID: i64 = 0x4C4F_5431;
const FORMAT_VERSION: i64 = 1;

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS elements (id INTEGER PRIMARY KEY, kind TEXT NOT NULL, data TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS operations (seq INTEGER PRIMARY KEY, data TEXT NOT NULL);
";

/// Schreibt das Dokument in eine Projektdatei. Eine vorhandene Datei wird ersetzt.
///
/// Geschrieben wird zuerst in eine Nachbardatei, die dann umbenannt wird. Ein
/// Absturz während des Speicherns lässt die alte Datei unversehrt.
pub fn save(doc: &Document, path: &Path) -> Result<(), IoError> {
    let tmp = path.with_extension("lot-tmp");
    if tmp.exists() {
        std::fs::remove_file(&tmp)?;
    }
    {
        let mut conn = Connection::open(&tmp)?;
        conn.pragma_update(None, "application_id", APPLICATION_ID)?;
        conn.pragma_update(None, "user_version", FORMAT_VERSION)?;
        conn.execute_batch(SCHEMA)?;
        let tx = conn.transaction()?;
        tx.execute(
            "INSERT INTO meta (key, value) VALUES ('name', ?1), ('generator', ?2)",
            params![doc.project().name, concat!("Lot ", env!("CARGO_PKG_VERSION"))],
        )?;
        {
            let mut stmt = tx.prepare("INSERT INTO elements (id, kind, data) VALUES (?1, ?2, ?3)")?;
            for (id, element) in &doc.project().elements {
                let kind = format!("{:?}", element.kind());
                stmt.execute(params![id.0 as i64, kind, serde_json::to_string(element)?])?;
            }
            let mut stmt = tx.prepare("INSERT INTO operations (seq, data) VALUES (?1, ?2)")?;
            for (seq, op) in doc.log().iter().enumerate() {
                stmt.execute(params![seq as i64, serde_json::to_string(op)?])?;
            }
        }
        tx.commit()?;
    }
    std::fs::rename(&tmp, path)?;
    Ok(())
}

/// Liest eine Projektdatei.
pub fn load(path: &Path) -> Result<Document, IoError> {
    if !path.is_file() {
        return Err(IoError::File(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("{} gibt es nicht", path.display()),
        )));
    }
    let conn = Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let app_id: i64 =
        conn.pragma_query_value(None, "application_id", |r| r.get(0)).map_err(|_| IoError::NotALotFile)?;
    if app_id != APPLICATION_ID {
        return Err(IoError::NotALotFile);
    }
    let version: i64 = conn.pragma_query_value(None, "user_version", |r| r.get(0))?;
    if version > FORMAT_VERSION {
        return Err(IoError::UnsupportedVersion { found: version, supported: FORMAT_VERSION });
    }

    let name: String =
        conn.query_row("SELECT value FROM meta WHERE key = 'name'", [], |r| r.get(0)).optional()?.unwrap_or_default();
    let mut project = Project::new(name);

    let mut stmt = conn.prepare("SELECT id, data FROM elements ORDER BY id")?;
    let rows = stmt.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))?;
    for row in rows {
        let (id, data) = row?;
        let element: Element = serde_json::from_str(&data)?;
        project.elements.insert(ElementId(id as u64), element);
    }

    let mut stmt = conn.prepare("SELECT data FROM operations ORDER BY seq")?;
    let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
    let mut log = Vec::new();
    for row in rows {
        let op: Operation = serde_json::from_str(&row?)?;
        log.push(op);
    }

    project.validate()?;
    Ok(Document::with_log(project, log))
}

#[cfg(test)]
mod tests {
    use super::*;
    use lot_model::Param;

    #[test]
    fn roundtrip_keeps_elements_and_log() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("wand.lot");
        let mut doc = Document::new(Project::sample());
        doc.set_param(ElementId(2), Param::OpeningWidth, 2.0).unwrap();
        doc.set_param(ElementId(1), Param::WallHeight, 2.75).unwrap();
        doc.undo().unwrap();
        save(&doc, &path).unwrap();

        let loaded = load(&path).unwrap();
        assert_eq!(loaded.project(), doc.project());
        assert_eq!(loaded.log(), doc.log());
    }

    #[test]
    fn save_replaces_existing_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("wand.lot");
        let mut doc = Document::new(Project::sample());
        save(&doc, &path).unwrap();
        doc.set_param(ElementId(2), Param::OpeningWidth, 1.0).unwrap();
        save(&doc, &path).unwrap();
        assert_eq!(load(&path).unwrap().project(), doc.project());
        assert!(!path.with_extension("lot-tmp").exists());
    }

    #[test]
    fn foreign_database_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("fremd.lot");
        Connection::open(&path).unwrap().execute_batch("CREATE TABLE x (a);").unwrap();
        assert!(matches!(load(&path), Err(IoError::NotALotFile)));
    }

    #[test]
    fn newer_format_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("neu.lot");
        save(&Document::new(Project::sample()), &path).unwrap();
        Connection::open(&path).unwrap().pragma_update(None, "user_version", 99).unwrap();
        assert!(matches!(load(&path), Err(IoError::UnsupportedVersion { found: 99, .. })));
    }
}
