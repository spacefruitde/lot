import SwiftUI

@main
struct LotApp: App {
    @State private var model: AppModel

    init() {
        // Planschrift aus dem App-Bundle, bevor der Kern sie zum ersten Mal braucht.
        if let font = Bundle.main.path(forResource: "osifont-lgpl3fe", ofType: "ttf") {
            setenv("LOT_PLAN_FONT", font, 1)
        }
        // Bürostandard des Büros, bevor der erste Plan gezeichnet wird (2.08).
        let standardError = OfficeStandardFile.install()
        let libraryError = OfficeLibraryFile.install()
        let model = AppModel()
        if let standardError {
            model.errorMessage = "Bürostandard: \(standardError)"
        } else if let libraryError {
            model.errorMessage = "Bibliothek: \(libraryError)"
        }
        _model = State(initialValue: model)
    }

    var body: some Scene {
        Window("Lot", id: "main") {
            ContentView()
                .environment(model)
                .frame(minWidth: 800, minHeight: 500)
        }
        .commands {
            CommandGroup(replacing: .newItem) {
                Button("Neues Projekt") { model.newProject() }
                    .keyboardShortcut("n")
                Button("Öffnen …") { model.openWithPanel() }
                    .keyboardShortcut("o")
                Button("Testprojekt öffnen …") { model.openTestprojectWithPanel() }
                    .keyboardShortcut("o", modifiers: [.command, .shift])
                Divider()
                Button("Unterlage laden (PDF, Bild, DXF, DWG) …") { model.importUnderlayWithPanel() }
                Button("Referenzmodell laden (IFC) …") { model.importReferenceWithPanel() }
            }
            CommandGroup(replacing: .saveItem) {
                Button("Sichern") { model.save() }
                    .keyboardShortcut("s")
                Button("Sichern unter …") { model.saveWithPanel() }
                    .keyboardShortcut("s", modifiers: [.command, .shift])
            }
            CommandGroup(replacing: .undoRedo) {
                Button("Widerrufen") { model.undo() }
                    .keyboardShortcut("z")
                    .disabled(!model.canUndo)
                Button("Wiederholen") { model.redo() }
                    .keyboardShortcut("z", modifiers: [.command, .shift])
                    .disabled(!model.canRedo)
            }
            SidebarCommands()
            CommandGroup(before: .toolbar) {
                Button("Grundriss") { model.setView(.plan) }
                    .keyboardShortcut("1")
                Button("3D") { model.setView(.model) }
                    .keyboardShortcut("2")
                Button("Alles zeigen") { model.zoomToFit() }
                    .keyboardShortcut("0")
                Divider()
                Button("Zurück") { model.back() }
                    .keyboardShortcut("[")
                    .disabled(model.viewState?.canGoBack != true)
                Button("Vorwärts") { model.forward() }
                    .keyboardShortcut("]")
                    .disabled(model.viewState?.canGoForward != true)
                Button("Geschoss höher") { model.stepStorey(up: true) }
                    .keyboardShortcut(.upArrow)
                Button("Geschoss tiefer") { model.stepStorey(up: false) }
                    .keyboardShortcut(.downArrow)
                Toggle("Geschoss darunter blass", isOn: Binding(
                    get: { model.viewState?.fadedBelow ?? false },
                    set: { model.setFadedBelow($0) }
                ))
                .disabled(model.viewState?.hasBelow != true)
                Toggle("Nebeneinander", isOn: Binding(get: { model.showSplit }, set: { model.showSplit = $0 }))
                    .keyboardShortcut("n", modifiers: [.command, .option])
                Toggle("Druckbild", isOn: Binding(get: { model.viewState?.printView ?? false }, set: { model.setPrintView($0) }))
                    .keyboardShortcut("p", modifiers: [.command, .option])
                Button("Auswahl vorübergehend ausblenden (H)") { model.run("TempHide") }
                Button("Auswahl isolieren (I)") { model.run("TempIsolate") }
                Button("Ausgeblendetes wieder zeigen") { model.run("TempReset") }
                Divider()
            }
            CommandMenu("Prüfen") {
                Button("Prüfbericht …") { model.runChecks() }
                    .keyboardShortcut("p", modifiers: [.command, .shift])
                Toggle("Hinweise im Grundriss", isOn: Binding(get: { model.showFindings }, set: { _ in model.toggleFindings() }))
                    .keyboardShortcut("h", modifiers: [.command, .shift])
                Divider()
                Button("Bürostandard bearbeiten …") { OfficeStandardFile.edit() }
                Button("Bibliothek des Büros bearbeiten …") { OfficeLibraryFile.edit() }
            }
            CommandMenu("Werkzeug") {
                Button("Befehle …") { model.showPalette.toggle() }
                    .keyboardShortcut("k")
                Button("Typen …") { model.openTypes() }
                    .keyboardShortcut("t", modifiers: [.command, .shift])
                Divider()
                Button("Auswählen (Esc)") { model.run("SelectTool") }
                Button("Wand zeichnen (WA)") { model.run("WallTool") }
                Button("Wände als Rechteck (WR)") { model.run("WallRectTool") }
                Button("Bogenwand zeichnen (BW)") { model.run("ArcWallTool") }
                Button("Achse zeichnen (GR)") { model.run("GridTool") }
                Button("Schnitt zeichnen (SC)") { model.run("SectionTool") }
                Button("Ansicht zeichnen (AN)") { model.run("ElevationTool") }
                Button("Neues Planblatt") { model.run("AddSheet") }
                Button("Stütze setzen (CL)") { model.run("ColumnTool") }
                Button("Tür setzen (DR)") { model.run("DoorTool") }
                Button("Fenster setzen (WN)") { model.run("WindowTool") }
                Button("Raum setzen (RM)") { model.run("SpaceTool") }
                Button("Räume erkennen") { model.run("DetectSpaces") }
                Button("Gewählte Wände bemaßen (DI)") { model.run("DimensionWalls") }
                Button("Decke aus Wänden") { model.run("SlabFromWalls") }
                Button("Deckendurchbruch zeichnen (DB)") { model.run("SlabOpeningTool") }
                Button("Satteldach über dem Geschoss") { model.run("GableRoofFromWalls") }
                Button("Walmdach über dem Geschoss") { model.run("HipRoofFromWalls") }
                Button("Flachdach über dem Geschoss") { model.run("FlatRoofFromWalls") }
                Button("Dachkante umschalten (DK)") { model.run("RoofEdgeTool") }
                Button("Ablauf setzen (AB)") { model.run("DrainTool") }
                Button("Notüberlauf setzen (NU)") { model.run("OverflowTool") }
                Menu("Treppe zeichnen (TP)") {
                    Button("Gerade") { model.run("StairTool(Straight)") }
                    Button("Zweiläufig mit Halbpodest") { model.run("StairTool(HalfLanding)") }
                    Button("L-Form mit Viertelpodest") { model.run("StairTool(QuarterLanding)") }
                    Button("Viertelgewendelt") { model.run("StairTool(QuarterWinder)") }
                    Button("Halbgewendelt") { model.run("StairTool(HalfWinder)") }
                    Button("Spindeltreppe") { model.run("StairTool(Spiral)") }
                }
                Button("Linie zeichnen, 2D (LI)") { model.run("LineTool") }
                Button("Text setzen, 2D (TX)") { model.run("TextTool") }
                Button("Unterlage kalibrieren (KA)") { model.run("CalibrateUnderlayTool") }
                Button("Unterlage ausrichten") { model.run("AlignUnderlayTool") }
                Divider()
                Button("Zeichenrichtung von einer Kante (ZR)") { model.run("DirectionFromEdge") }
                Menu("Fangen für einen Klick") {
                    Button("Nur Endpunkte (SE)") { model.run("SnapOverride(Endpoints)") }
                    Button("Nur Mitten (SM)") { model.run("SnapOverride(Midpoints)") }
                    Button("Nur Schnittpunkte (SI)") { model.run("SnapOverride(Intersections)") }
                    Button("Nur lotrecht (SP)") { model.run("SnapOverride(Perpendicular)") }
                    Button("Nur auf Linien (SN)") { model.run("SnapOverride(Nearest)") }
                    Button("Fangen aus (SO)") { model.run("SnapOverride(Off)") }
                }
                Divider()
                Button("Verschieben (MV)") { model.run("MoveTool") }
                Button("Kopieren (CO)") { model.run("CopyTool") }
                Button("Drehen (RO)") { model.run("RotateTool") }
                Button("Spiegeln (MM)") { model.run("MirrorTool") }
                Button("Reihe (AR)") { model.run("ArrayTool") }
                Button("Versetzen (OF)") { model.run("OffsetTool") }
                Button("Ausrichten (AL)") { model.run("AlignTool") }
                Button("Stutzen und Dehnen (TR)") { model.run("TrimTool") }
                Button("Wand teilen (SL)") { model.run("SplitTool") }
                Divider()
                Button("Alles auswählen") { model.run("SelectAll") }
                    .keyboardShortcut("a")
                Button("Gleiche wählen (SA)") { model.run("SelectSimilar") }
                Button("Vorige Auswahl (⌘← im Plan)") { model.run("PreviousSelection") }
                Button("Unterlage wählen") { model.run("SelectUnderlay") }
                Button("Löschen") { model.run("DeleteSelection") }
            }
        }
    }
}
