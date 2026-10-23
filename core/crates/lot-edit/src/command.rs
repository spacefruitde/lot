//! Befehle: Jede Funktion ist ein Befehl mit Parametern (PLAN.md, Abschnitt 3,
//! „KI-Assistent“). Die Befehlspalette (8.04) zeigt sie, später ruft die KI
//! dieselben Befehle auf.

use lot_model::WallAlignment;
use serde::{Deserialize, Serialize};

/// Ein Befehl des Editors. Die Kennung ist seine RON-Schreibweise, etwa
/// `WallTool` oder `SetWallType(11)`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Command {
    SelectTool,
    WallTool,
    ArcWallTool,
    GridTool,
    ColumnTool,
    RoomTool,
    DoorTool,
    WindowTool,
    /// Gewählte Türen schlagen zur anderen Wandseite auf.
    FlipDoorSwing,
    /// Gewählte Türen: DIN links und DIN rechts tauschen.
    FlipDoorHinge,
    /// Maßketten an den gewählten Wänden: Öffnungen und Gesamtmaß, außen.
    DimensionWalls,
    /// Einen Raum in jeden umschlossenen Bereich ohne Raum setzen.
    DetectRooms,
    SetWallType(u64),
    SetAlignment(WallAlignment),
    SetStorey(u64),
    /// Neues Geschoss über dem obersten.
    AddStorey,
    DeleteSelection,
    /// Zeichenrichtung der gewählten Wände umkehren: innen und außen tauschen.
    FlipWalls,
    SelectAll,
    Undo,
    Redo,
}

impl Command {
    pub fn id(&self) -> String {
        ron::to_string(self).unwrap_or_default()
    }

    pub fn parse(id: &str) -> Option<Command> {
        ron::from_str(id).ok()
    }
}

/// Eintrag in der Befehlspalette.
#[derive(Debug, Clone, PartialEq)]
pub struct CommandInfo {
    pub id: String,
    pub title: String,
    /// Tastenkürzel zur Anzeige, etwa „W“ oder „⌘Z“.
    pub shortcut: String,
    /// Gruppe zur Anzeige, etwa „Werkzeug“ oder „Wandaufbau“.
    pub group: String,
}

impl CommandInfo {
    pub fn new(command: Command, title: impl Into<String>, shortcut: &str, group: &str) -> Self {
        Self { id: command.id(), title: title.into(), shortcut: shortcut.into(), group: group.into() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_roundtrip() {
        for c in [Command::WallTool, Command::SetWallType(11), Command::SetAlignment(WallAlignment::Exterior)] {
            assert_eq!(Command::parse(&c.id()), Some(c));
        }
        assert_eq!(Command::WallTool.id(), "WallTool");
    }
}
