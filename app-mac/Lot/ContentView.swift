import SwiftUI

struct ContentView: View {
    @Environment(AppModel.self) private var model
    @State private var showInspector = true

    var body: some View {
        @Bindable var model = model
        ViewportView(generation: model.documentGeneration, onEvent: { model.handle($0) }, onStats: { model.stats = $0 })
            .overlay(alignment: .bottom) { StatusLine(hint: model.viewState?.hint ?? "", stats: model.stats) }
            .overlay(alignment: .top) {
                if model.showPalette {
                    CommandPalette(commands: model.commands) { id in
                        model.showPalette = false
                        if let id { model.run(id) }
                    }
                    .padding(.top, 12)
                }
            }
            .inspector(isPresented: $showInspector) {
                InspectorView()
                    .inspectorColumnWidth(min: 240, ideal: 280)
            }
            .toolbar { toolbar }
            .navigationTitle(model.title)
            .navigationSubtitle(subtitle)
            .alert(
                "Das ging nicht",
                isPresented: Binding(get: { model.errorMessage != nil }, set: { if !$0 { model.errorMessage = nil } }),
                presenting: model.errorMessage
            ) { _ in
                Button("OK") {}
            } message: { message in
                Text(message)
            }
    }

    private var subtitle: String {
        guard let s = model.viewState else { return "" }
        return "\(s.storey) · 1:\(Int(s.scale)) · \(s.wallType)"
    }

    @ToolbarContentBuilder
    private var toolbar: some ToolbarContent {
        ToolbarItem(placement: .navigation) {
            Picker("Ansicht", selection: Binding(
                get: { model.viewState?.view ?? .plan },
                set: { model.setView($0) }
            )) {
                Text("Grundriss").tag(ViewKind.plan)
                Text("3D").tag(ViewKind.model)
            }
            .pickerStyle(.segmented)
            .help("Ansicht wechseln (⌘1, ⌘2)")
        }
        ToolbarItemGroup {
            toolButton("Auswählen", systemImage: "cursorarrow", tool: "select", command: "SelectTool", key: "V")
            toolButton("Wand", systemImage: "square.split.diagonal.2x2", tool: "wall", command: "WallTool", key: "W")
            toolButton("Bogenwand", systemImage: "point.topleft.down.curvedto.point.bottomright.up", tool: "arc", command: "ArcWallTool", key: "B")
            toolButton("Achse", systemImage: "line.diagonal", tool: "grid", command: "GridTool", key: "A")
            toolButton("Stütze", systemImage: "square.fill", tool: "column", command: "ColumnTool", key: "S")
            toolButton("Tür", systemImage: "door.left.hand.open", tool: "door", command: "DoorTool", key: "T")
            toolButton("Fenster", systemImage: "squareshape.split.2x2", tool: "window", command: "WindowTool", key: "F")
            toolButton("Raum", systemImage: "square.dashed", tool: "room", command: "RoomTool", key: "R")
            commandMenu("Wandaufbau", systemImage: "square.stack.3d.up", group: "Wandaufbau")
            commandMenu("Geschoss", systemImage: "building.2", group: "Geschoss")
        }
        ToolbarItemGroup {
            Button { model.undo() } label: { Label("Widerrufen", systemImage: "arrow.uturn.backward") }
                .disabled(!model.canUndo)
            Button { model.redo() } label: { Label("Wiederholen", systemImage: "arrow.uturn.forward") }
                .disabled(!model.canRedo)
            Button { model.zoomToFit() } label: { Label("Alles zeigen", systemImage: "arrow.up.left.and.arrow.down.right") }
            Button { model.showPalette.toggle() } label: { Label("Befehle", systemImage: "command") }
                .help("Befehlspalette (⌘K)")
            Button { showInspector.toggle() } label: { Label("Eigenschaften", systemImage: "sidebar.right") }
        }
    }

    private func toolButton(_ title: String, systemImage: String, tool: String, command: String, key: String) -> some View {
        let active = model.viewState?.tool == tool
        return Button { model.run(command) } label: {
            Label(title, systemImage: systemImage)
        }
        .foregroundStyle(active ? Color.accentColor : Color.primary)
        .help("\(title) (\(key))")
        .accessibilityLabel(title)
        .disabled(!model.isPlan)
    }

    private func commandMenu(_ title: String, systemImage: String, group: String) -> some View {
        Menu {
            ForEach(model.commands.filter { $0.group == group || ($0.group == "Ansicht" && group == "Geschoss" && $0.id.hasPrefix("scale:")) }, id: \.id) { item in
                Button(item.title) { model.run(item.id) }
            }
        } label: {
            Label(title, systemImage: systemImage)
        }
        .help(title)
        .accessibilityLabel(title)
    }
}

private struct StatusLine: View {
    let hint: String
    let stats: ViewportStatsDisplay

    var body: some View {
        HStack {
            Text(hint)
            Spacer()
            Text("Bild \(stats.frameMs, format: .number.precision(.fractionLength(1))) ms · Neuberechnung \(stats.rebuildMs, format: .number.precision(.fractionLength(1))) ms")
                .monospacedDigit()
        }
        .font(.caption)
        .foregroundStyle(.secondary)
        .padding(.horizontal, 10)
        .padding(.vertical, 6)
        .background(.regularMaterial)
    }
}
