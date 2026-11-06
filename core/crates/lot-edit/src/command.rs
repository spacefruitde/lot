//! Befehle: Jede Funktion ist ein Befehl mit Parametern (PLAN.md, Abschnitt 3,
//! „KI-Assistent“). Die Befehlspalette (8.04) zeigt sie, später ruft die KI
//! dieselben Befehle auf.

use lot_model::{Phase, WallAlignment};
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
    SpaceTool,
    DoorTool,
    WindowTool,
    SlabOpeningTool,
    /// 2D-Linie im Grundriss
    LineTool,
    /// 2D-Text im Grundriss
    TextTool,
    /// Unterlage kalibrieren: zwei Punkte, dann den wahren Abstand tippen
    CalibrateUnderlayTool,
    /// Unterlage ausrichten: zwei Punkte, die waagerecht oder senkrecht liegen sollen
    AlignUnderlayTool,
    /// Verschieben, Kopieren, Drehen, Spiegeln, Reihe an der Auswahl (MV, CO, RO, MM, AR).
    MoveTool,
    CopyTool,
    RotateTool,
    MirrorTool,
    ArrayTool,
    /// Versetzen, Ausrichten, Stutzen und Dehnen, Teilen (OF, AL, TR, SL).
    OffsetTool,
    AlignTool,
    TrimTool,
    SplitTool,
    /// Wände als Rechteck in der Zeichenrichtung (WR).
    WallRectTool,
    /// Zeichenrichtung von einer Kante setzen (ZR).
    DirectionFromEdge,
    /// Fangkürzel für den nächsten Klick (SE, SM, SI, SP, SN, SO).
    SnapOverride(crate::snap::SnapOnly),
    /// Gewählte Türen schlagen zur anderen Wandseite auf.
    FlipDoorSwing,
    /// Gewählte Türen: DIN links und DIN rechts tauschen.
    FlipDoorHinge,
    /// Maßketten an den gewählten Wänden: Öffnungen und Gesamtmaß, außen.
    DimensionWalls,
    /// Decke im aktuellen Geschoss aus den Wänden darunter, im untersten Geschoss Bodenplatte.
    SlabFromWalls,
    /// Satteldach über den Wänden des Geschosses; Wände bis zum Geschoss reichen dann bis unters Dach.
    GableRoofFromWalls,
    /// Einen Raum in jeden umschlossenen Bereich ohne Raum setzen.
    DetectSpaces,
    SetWallType(u64),
    SetAlignment(WallAlignment),
    /// Bauzustand der gewählten Bauteile: Bestand, Abbruch oder Neubau.
    SetPhase(Phase),
    SetStorey(u64),
    /// Neues Geschoss über dem obersten.
    AddStorey,
    DeleteSelection,
    /// Neue Variantengruppe mit den Varianten A und B (2.04).
    NewVariantSet,
    /// Gewählte Bauteile in die aktive Variante der Gruppe.
    AddSelectionToVariant(u64),
    /// Nächste Variante der Gruppe zeigen.
    SwitchVariant(u64),
    /// Zeichenrichtung der gewählten Wände umkehren: innen und außen tauschen.
    FlipWalls,
    SelectAll,
    /// Gleiche Bauteile wie die Auswahl: erst im Geschoss, dann im Projekt (SA).
    SelectSimilar,
    /// Die vorige Auswahl zurückholen (⌘←).
    PreviousSelection,
    /// Unterlagen sind nicht anklickbar (ADR 0043); dieser Befehl wählt sie nacheinander.
    SelectUnderlay,
    /// Auswahl in die Zwischenablage (⌘C).
    CopyToClipboard,
    /// Zwischenablage an derselben Stelle ins aktuelle Geschoss einfügen (⌘V).
    Paste,
    /// Genau diese Elemente wählen, etwa eine Gruppe aus dem Inspektor.
    SetSelection(Vec<u64>),
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
