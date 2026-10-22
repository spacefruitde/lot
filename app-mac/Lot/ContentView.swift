import SwiftUI

struct ContentView: View {
    @Environment(AppModel.self) private var model
    @State private var showInspector = true

    var body: some View {
        ViewportView(
            document: model.document,
            revision: model.revision,
            fitRequest: model.fitRequest,
            onStats: { model.stats = $0 }
        )
        .overlay(alignment: .bottomLeading) { StatusLine(stats: model.stats) }
        .inspector(isPresented: $showInspector) {
            InspectorView()
                .inspectorColumnWidth(min: 240, ideal: 280)
        }
        .toolbar {
            ToolbarItemGroup {
                Button { model.undo() } label: { Label("Widerrufen", systemImage: "arrow.uturn.backward") }
                    .disabled(!model.canUndo)
                Button { model.redo() } label: { Label("Wiederholen", systemImage: "arrow.uturn.forward") }
                    .disabled(!model.canRedo)
                Button { model.zoomToFit() } label: { Label("Alles zeigen", systemImage: "arrow.up.left.and.arrow.down.right") }
                Button { showInspector.toggle() } label: { Label("Eigenschaften", systemImage: "sidebar.right") }
            }
        }
        .navigationTitle(model.title)
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
}

private struct StatusLine: View {
    let stats: ViewportStatsDisplay

    var body: some View {
        Text("Bild \(stats.frameMs, format: .number.precision(.fractionLength(1))) ms · Neuberechnung \(stats.rebuildMs, format: .number.precision(.fractionLength(1))) ms · \(stats.triangles) Dreiecke")
            .font(.caption.monospacedDigit())
            .foregroundStyle(.secondary)
            .padding(6)
            .background(.regularMaterial, in: RoundedRectangle(cornerRadius: 6))
            .padding(8)
    }
}
