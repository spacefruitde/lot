import SwiftUI

@main
struct LotApp: App {
    @State private var model = AppModel()

    var body: some Scene {
        Window("Lot", id: "main") {
            ContentView()
                .environment(model)
                .frame(minWidth: 800, minHeight: 500)
        }
        .commands {
            CommandGroup(replacing: .newItem) {
                Button("Neues Beispielprojekt") { model.newSample() }
                    .keyboardShortcut("n")
                Button("Öffnen …") { model.openWithPanel() }
                    .keyboardShortcut("o")
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
                Button("Alles zeigen") { model.zoomToFit() }
                    .keyboardShortcut("0")
                Divider()
            }
        }
    }
}
