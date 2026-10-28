//! Befehle: Jede Funktion ist ein Befehl mit Parametern (PLAN.md, Abschnitt 3,
//! „KI-Assistent“). Die Befehlspalette (8.04) zeigt sie, später ruft die KI
//! dieselben Befehle auf.

use lot_model::{JoinRule, Phase, Property, ViewContent, WallAlignment, WallEnd, WallSide};
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
    /// Füllung der gewählten Öffnungen: 0 Durchgang, 1 Nische, 2 Tür, 3 Fenster (ADR 0047).
    SetOpeningFill(u32),
    /// Seite und Tiefe der gewählten Nischen.
    SetRecess(WallSide, f64),
    /// Unterer Anschluss am Typ der gewählten Öffnungen, in Metern.
    SetSillConnection(f64),
    /// Textwert am Typ setzen, leer löscht ihn (ADR 0048).
    SetTypeProperty(u64, Property, String),
    /// Rohbaumaß am Tür- oder Fenstertyp; `None`: die Öffnungen bestimmen es selbst.
    SetTypeSize(u64, Property, Option<f64>),
    /// Wert im Typ festlegen (Bauteile dürfen nicht abweichen) oder freigeben.
    SetTypeLocked(u64, Property, bool),
    /// Textwert an den gewählten Bauteilen; gleich dem Typwert heißt „vom Typ“.
    SetElementProperty(Property, String),
    /// Gewählte Bauteile übernehmen den Wert wieder vom Typ.
    ResetToType(Property),
    /// Wert des gewählten Bauteils in seinen Typ übernehmen.
    ApplyToType(Property),
    /// Typ kopieren; die Kopie ist danach gewählt im Inspektor der Übersicht.
    DuplicateType(u64),
    RenameType(u64, String),
    /// Typ löschen, seine Bauteile bekommen den Ersatz.
    DeleteType(u64, Option<u64>),
    /// Gewählten Bauteilen einen Typ geben; bei Öffnungen auch keinen.
    AssignType(Option<u64>),
    /// Alle Bauteile eines Typs bekommen einen anderen.
    ReplaceType(u64, u64),
    /// Für gewählte Öffnungen ohne Typ einen Typ aus ihrem Rohbaumaß anlegen.
    CreateOpeningTypes,
    /// Oberkante der gewählten Wände: Index in `Top::TITLES` (Geschoss, Decke, Dach, fest).
    SetWallTop(u32),
    /// Ausnahme am Anschluss eines Wandendes: Wand, Ende, Regel (ADR 0045).
    SetJoinRule(u64, WallEnd, JoinRule),
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
    /// Schnitt im Grundriss zeichnen (SC): zwei Klicks, Leertaste dreht die Blickrichtung (ADR 0049).
    SectionTool,
    /// Ansicht im Grundriss zeichnen (AN): wie ein Schnitt, außerhalb des Gebäudes.
    ElevationTool,
    /// Blickrichtung der gewählten Schnitte und Ansichten umdrehen.
    FlipSection,
    /// Blicktiefe der gewählten Schnitte und Ansichten, in Metern.
    SetSectionDepth(f64),
    /// Name und Kennbuchstabe eines Schnitts.
    SetSectionInfo(u64, String, String),
    /// Neues Planblatt A3 mit der nächsten Plannummer; danach ist es offen.
    AddSheet,
    /// Ansicht mit ihrer Mitte an (x, y) in Blatt-mm aufs Blatt legen.
    PlaceView(u64, crate::navigation::ViewRef, f64, f64),
    /// Ansicht auf dem Blatt verschieben: Blatt, Stelle, neue Mitte.
    MovePlacedView(u64, u32, f64, f64),
    /// Ansicht vom Blatt nehmen; die Ansicht selbst bleibt.
    RemovePlacedView(u64, u32),
    SetPlacedViewScale(u64, u32, f64),
    /// Plannummer, Planinhalt und Format.
    SetSheetInfo(u64, String, String, String),
    DeleteSheet(u64),
    /// Heimat eines Schnitts: das Blatt, auf das seine Marke verweist.
    SetHome(u64, u64),
    /// Geschosshöhe; die Geschosse darüber rücken mit (Geschossleiste, ADR 0049).
    SetStoreyHeight(u64, f64),
    /// OKFF eines Geschosses; die Geschosse darüber rücken mit.
    SetStoreyElevation(u64, f64),
    /// Ansichtsvorlage einer Ansicht wählen oder keine (ADR 0050).
    SetViewTemplate(ViewContent, Option<String>),
    /// Eine Einstellung der Ansicht setzen oder wieder übernehmen.
    SetViewField(ViewContent, crate::visibility::ViewField),
    /// Regel des Themas in der Ansicht ein- oder abschalten.
    SetRuleOn(ViewContent, String, bool),
    AddViewRule(ViewContent, lot_model::Rule),
    RemoveViewRule(ViewContent, String),
    /// Gewählte Bauteile in der Ansicht ausblenden.
    HideInView(ViewContent),
    /// Ausgeblendete Bauteile der Ansicht wieder zeigen; leer zeigt alle.
    ShowInView(ViewContent, Vec<u64>),
    /// H: Auswahl vorübergehend ausblenden; ohne Auswahl wieder alles zeigen.
    TempHide,
    /// I: Auswahl isolieren; ohne Auswahl wieder alles zeigen.
    TempIsolate,
    /// Vorübergehend Ausgeblendetes und Isoliertes zurücksetzen.
    TempReset,
    /// Treppe zeichnen (TP): Antritt, dann Richtung; Leertaste wechselt die Seite der Wendung (ADR 0051).
    StairTool(crate::stairs::StairForm),
    /// Dachkante umschalten (DK): Klick auf eine Kante wechselt Traufe und Giebel.
    RoofEdgeTool,
    /// Ablauf (AB) und Notüberlauf (NU) auf eine Decke mit Gefälle setzen.
    DrainTool,
    OverflowTool,
    /// Walmdach über den Wänden des Geschosses: Traufen ringsum.
    HipRoofFromWalls,
    /// Flachdach über dem Geschoss, mit 2 % Gefälle in der Dämmung.
    FlatRoofFromWalls,
    /// Kante eines Dachs umschalten: Dach, Kante.
    ToggleRoofEdge(u64, u32),
    /// Neigung (Grad) und Überstand (m) einer Dachkante; `None` lässt den Wert.
    SetRoofEdge(u64, u32, Option<f64>, Option<f64>),
    /// Gewählte Decken folgen den Wänden oder nicht mehr.
    SetSlabFollows(bool),
    /// Umriss der gewählten Decken neu aus den Wänden übernehmen.
    RefollowSlabs,
    /// Gefälle der gewählten Decken für diese Verwendung; `None` entfernt es.
    SetFallPurpose(Option<lot_model::FallPurpose>),
    SetFallPercent(f64),
    /// Fließrichtung in Grad gegen den Uhrzeigersinn von +x.
    SetFallDirection(f64),
    /// Schicht des Aufbaus, die das Gefälle bildet, von oben.
    SetFallLayer(u32),
    /// Abläufe und Notüberläufe der gewählten Decken entfernen.
    ClearDrains,
    /// Eine Treppe ändern: Form, Wendung, Maße, Vorschlag nach DIN 18065.
    SetStair(u64, crate::stairs::StairEdit),
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
