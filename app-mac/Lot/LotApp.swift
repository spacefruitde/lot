import SwiftUI

@main
struct LotApp: App {
    @State private var model: AppModel

    init() {
        // Planschrift aus dem App-Bundle, bevor der Kern sie zum ersten Mal braucht.
        if let font = Bundle.main.path(forResource: "osifont-lgpl3fe", ofType: "ttf") {
            setenv("LOT_PLAN_FONT", font, 1)
        }
        _model = State(initialValue: AppModel())
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
            CommandMenu("Werkzeug") {
                Button("Befehle …") { model.showPalette.toggle() }
                    .keyboardShortcut("k")
                Divider()
                Button("Auswählen") { model.run("SelectTool") }
                Button("Wand zeichnen") { model.run("WallTool") }
                Button("Bogenwand zeichnen") { model.run("ArcWallTool") }
                Button("Achse zeichnen") { model.run("GridTool") }
                Button("Stütze setzen") { model.run("ColumnTool") }
                Button("Tür setzen") { model.run("DoorTool") }
                Button("Fenster setzen") { model.run("WindowTool") }
                Button("Raum setzen") { model.run("SpaceTool") }
                Button("Räume erkennen") { model.run("DetectSpaces") }
                Button("Gewählte Wände bemaßen") { model.run("DimensionWalls") }
                Button("Decke aus Wänden") { model.run("SlabFromWalls") }
                Button("Deckendurchbruch zeichnen") { model.run("SlabOpeningTool") }
                Button("Satteldach über dem Geschoss") { model.run("GableRoofFromWalls") }
                Button("Linie zeichnen (2D)") { model.run("LineTool") }
                Button("Text setzen (2D)") { model.run("TextTool") }
                Button("Unterlage kalibrieren") { model.run("CalibrateUnderlayTool") }
                Button("Unterlage ausrichten") { model.run("AlignUnderlayTool") }
                Divider()
                Button("Alles auswählen") { model.run("SelectAll") }
                    .keyboardShortcut("a")
                Button("Löschen") { model.run("DeleteSelection") }
            }
        }
    }
}
