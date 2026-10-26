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
        let model = AppModel()
        if let standardError {
            model.errorMessage = "Bürostandard: \(standardError)"
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
            CommandGroup(before: .toolbar) {
                Button("Grundriss") { model.setView(.plan) }
                    .keyboardShortcut("1")
                Button("3D") { model.setView(.model) }
                    .keyboardShortcut("2")
                Button("Alles zeigen") { model.zoomToFit() }
                    .keyboardShortcut("0")
                Divider()
            }
            CommandMenu("Prüfen") {
                Button("Prüfbericht …") { model.runChecks() }
                    .keyboardShortcut("p", modifiers: [.command, .shift])
                Toggle("Hinweise im Grundriss", isOn: Binding(get: { model.showFindings }, set: { _ in model.toggleFindings() }))
                    .keyboardShortcut("h", modifiers: [.command, .shift])
                Divider()
                Button("Bürostandard bearbeiten …") { OfficeStandardFile.edit() }
            }
            CommandMenu("Werkzeug") {
                Button("Befehle …") { model.showPalette.toggle() }
                    .keyboardShortcut("k")
                Divider()
                Button("Auswählen (Esc)") { model.run("SelectTool") }
                Button("Wand zeichnen (WA)") { model.run("WallTool") }
                Button("Bogenwand zeichnen (BW)") { model.run("ArcWallTool") }
                Button("Achse zeichnen (GR)") { model.run("GridTool") }
                Button("Stütze setzen (CL)") { model.run("ColumnTool") }
                Button("Tür setzen (DR)") { model.run("DoorTool") }
                Button("Fenster setzen (WN)") { model.run("WindowTool") }
                Button("Raum setzen (RM)") { model.run("SpaceTool") }
                Button("Räume erkennen") { model.run("DetectSpaces") }
                Button("Gewählte Wände bemaßen (DI)") { model.run("DimensionWalls") }
                Button("Decke aus Wänden") { model.run("SlabFromWalls") }
                Button("Deckendurchbruch zeichnen (DB)") { model.run("SlabOpeningTool") }
                Button("Satteldach über dem Geschoss") { model.run("GableRoofFromWalls") }
                Button("Linie zeichnen, 2D (LI)") { model.run("LineTool") }
                Button("Text setzen, 2D (TX)") { model.run("TextTool") }
                Button("Unterlage kalibrieren (KA)") { model.run("CalibrateUnderlayTool") }
                Button("Unterlage ausrichten") { model.run("AlignUnderlayTool") }
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
