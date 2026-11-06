import SwiftUI

struct ContentView: View {
    @Environment(AppModel.self) private var model
    @State private var showInspector = true

    var body: some View {
        @Bindable var model = model
        NavigationSplitView {
            ProjectSidebar()
                .navigationSplitViewColumnWidth(min: 200, ideal: 250, max: 360)
        } detail: {
            workspace
                .overlay(alignment: .bottom) { StatusLine(hint: model.viewState?.hint ?? "", stats: model.stats) }
                .overlay(alignment: .top) {
                    if model.showPalette {
                        CommandPalette(commands: model.commands, search: { model.jumps($0) }) { id in
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
        }
            .sheet(isPresented: $model.showReport) {
                CheckReportView()
            }
            .sheet(isPresented: $model.showTypes) {
                TypesView()
                    .environment(model)
            }
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
        switch s.view {
        case .plan: return "\(s.title) · 1:\(Int(s.scale)) · \(s.wallType)"
        case .section: return "\(s.title) · 1:\(Int(s.scale))"
        default: return s.title
        }
    }

    /// Eine Ansicht oder zwei nebeneinander mit gemeinsamer Auswahl (ADR 0049).
    private var workspace: some View {
        HSplitView {
            pane(0)
            if model.showSplit {
                pane(1)
            }
        }
    }

    private func pane(_ index: Int) -> some View {
        GeometryReader { geo in
            ViewportView(
                pane: index,
                generation: model.documentGeneration,
                onEvent: { model.handle($0, pane: index) },
                onStats: { model.stats = $0 }
            )
            // Ansichten aus der Seitenleiste aufs Blatt ziehen.
            .dropDestination(for: String.self) { items, location in
                guard let view = items.first else { return false }
                model.dropView(view, pane: index, x: location.x - geo.size.width / 2, y: geo.size.height / 2 - location.y)
                return true
            }
        }
        .overlay {
            // H und I: Rahmen und Zähler, nie im Druck (ADR 0050).
            if let text = model.paneTemporary[index], !text.isEmpty {
                ZStack(alignment: .top) {
                    Rectangle().strokeBorder(Color.teal, lineWidth: 3)
                    Text(text)
                        .font(.caption)
                        .padding(.horizontal, 8)
                        .padding(.vertical, 3)
                        .background(Color.teal, in: RoundedRectangle(cornerRadius: 5))
                        .foregroundStyle(.white)
                        .padding(.top, 8)
                }
                .allowsHitTesting(false)
            }
        }
        .overlay(alignment: .topLeading) {
            if model.showSplit, let title = model.paneTitles[index] {
                Text(title)
                    .font(.caption)
                    .padding(.horizontal, 8)
                    .padding(.vertical, 3)
                    .background(model.activePane == index ? Color.accentColor.opacity(0.85) : Color.secondary.opacity(0.5),
                                in: RoundedRectangle(cornerRadius: 5))
                    .foregroundStyle(.white)
                    .padding(8)
            }
        }
        .frame(minWidth: 200)
    }

    @ToolbarContentBuilder
    private var toolbar: some ToolbarContent {
        ToolbarItemGroup(placement: .navigation) {
            Button { model.back() } label: { Label("Zurück", systemImage: "chevron.left") }
                .disabled(model.viewState?.canGoBack != true)
                .help("Zurück (⌘[)")
            Button { model.forward() } label: { Label("Vorwärts", systemImage: "chevron.right") }
                .disabled(model.viewState?.canGoForward != true)
                .help("Vorwärts (⌘])")
        }
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
            toolButton("Auswählen", systemImage: "cursorarrow", tool: "select", command: "SelectTool", key: "Esc")
            toolButton("Wand", systemImage: "square.split.diagonal.2x2", tool: "wall", command: "WallTool", key: "WA")
            toolButton("Bogenwand", systemImage: "point.topleft.down.curvedto.point.bottomright.up", tool: "arc", command: "ArcWallTool", key: "BW")
            toolButton("Achse", systemImage: "line.diagonal", tool: "grid", command: "GridTool", key: "GR")
            toolButton("Stütze", systemImage: "square.fill", tool: "column", command: "ColumnTool", key: "CL")
            toolButton("Tür", systemImage: "door.left.hand.open", tool: "door", command: "DoorTool", key: "DR")
            toolButton("Fenster", systemImage: "squareshape.split.2x2", tool: "window", command: "WindowTool", key: "WN")
            toolButton("Raum", systemImage: "square.dashed", tool: "space", command: "SpaceTool", key: "RM")
            toolButton("Schnitt", systemImage: "rectangle.split.1x2", tool: "section", command: "SectionTool", key: "SC")
            commandMenu("Wandaufbau", systemImage: "square.stack.3d.up", group: "Wandaufbau")
            Button { model.openTypes() } label: { Label("Typen", systemImage: "list.bullet.rectangle") }
                .help("Typen-Übersicht und Bibliothek des Büros (⇧⌘T)")
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
            Button { model.runChecks() } label: { Label("Prüfen", systemImage: "checkmark.seal") }
                .help("Prüfbericht: Abstandsflächen, Barrierefreiheit, Rettungswege (⇧⌘P)")
            Toggle(isOn: Binding(get: { model.viewState?.printView ?? false }, set: { model.setPrintView($0) })) {
                Label("Druckbild", systemImage: "printer")
            }
            .help("Druckbild: nur, was gedruckt wird, ohne Hinweise und vorübergehend Ausgeblendetes (⌥⌘P)")
            Button { model.showSplit.toggle() } label: { Label("Nebeneinander", systemImage: "rectangle.split.2x1") }
                .help("Zwei Ansichten nebeneinander, gemeinsame Auswahl")
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
