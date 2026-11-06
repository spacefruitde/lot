import SwiftUI

/// Seitenleiste „Projekt“ (ADR 0049): Geschosse als Leiste mit Höhen,
/// Grundrisse, Schnitte und Ansichten, Lageplan, Planblätter mit ihren
/// Ansichten, Listen. Die Suche findet auch Räume. Grundrisse, Schnitte,
/// Ansichten und der Lageplan lassen sich auf ein Planblatt ziehen.
struct ProjectSidebar: View {
    @Environment(AppModel.self) private var model
    @State private var query = ""

    var body: some View {
        List(selection: selection) {
            if query.isEmpty {
                Section {
                    HStack(spacing: 6) {
                        Spacer()
                        Text("OKFF").frame(width: 44, alignment: .trailing)
                        Text("Höhe").frame(width: 44, alignment: .trailing)
                    }
                    .font(.caption2)
                    .foregroundStyle(.secondary)
                    ForEach(model.storeys, id: \.element) { StoreyRowView(row: $0).tag("storey/\($0.element)") }
                    if model.viewState?.hasBelow == true {
                        Toggle("Geschoss darunter blass", isOn: Binding(
                            get: { model.viewState?.fadedBelow ?? false },
                            set: { model.setFadedBelow($0) }
                        ))
                        .toggleStyle(.checkbox)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                    }
                } header: {
                    header("Geschosse", add: ("Geschoss hinzufügen", "AddStorey"))
                }
                ForEach(model.tree.filter { $0.title != "Geschosse" }, id: \.title) { group in
                    Section {
                        ForEach(rows(group), id: \.0.key) { entry, depth in
                            EntryRow(entry: entry, depth: depth).tag(entry.key)
                        }
                    } header: {
                        switch group.title {
                        case "Schnitte und Ansichten": header(group.title, add: ("Schnitt zeichnen (SC)", "SectionTool"))
                        case "Planblätter": header(group.title, add: ("Neues Planblatt", "AddSheet"))
                        default: Text(group.title)
                        }
                    }
                }
            } else {
                Section("Treffer") {
                    ForEach(model.jumps(query), id: \.id) { hit in
                        VStack(alignment: .leading) {
                            Text(hit.title)
                            Text(hit.shortcut).font(.caption).foregroundStyle(.secondary)
                        }
                        .tag("hit/" + hit.id)
                    }
                }
            }
        }
        .listStyle(.sidebar)
        .searchable(text: $query, placement: .sidebar, prompt: "Ansichten und Räume")
    }

    /// Einträge mit den Ansichten auf Planblättern eingerückt darunter.
    private func rows(_ group: NavGroup) -> [(NavEntry, Int)] {
        group.entries.flatMap { [($0, 0)] + $0.children.map { ($0, 1) } }
    }

    /// Markiert ist, was die aktive Ansicht zeigt; ein Klick springt dorthin.
    private var selection: Binding<String?> {
        Binding(
            get: {
                guard let shown = model.viewState?.shown else { return nil }
                if shown.hasPrefix("FloorPlan(") {
                    return "storey/" + shown.dropFirst("FloorPlan(".count).dropLast()
                }
                for group in model.tree {
                    if let e = group.entries.first(where: { $0.view == shown }) { return e.key }
                }
                return nil
            },
            set: { key in
                guard let key else { return }
                if key.hasPrefix("storey/") {
                    model.go("FloorPlan(\(key.dropFirst("storey/".count)))")
                } else if key.hasPrefix("hit/go:") {
                    model.go(String(key.dropFirst("hit/go:".count)))
                } else if let entry = model.tree.flatMap({ rows($0) }).first(where: { $0.0.key == key })?.0 {
                    model.go(entry.view)
                }
            }
        )
    }

    private func header(_ title: String, add: (String, String)) -> some View {
        HStack {
            Text(title)
            Spacer()
            Button { model.run(add.1) } label: { Image(systemName: "plus") }
                .buttonStyle(.borderless)
                .help(add.0)
        }
    }
}

/// Ein Geschoss in der Leiste: Name, OKFF und Geschosshöhe zum Tippen; die
/// Geschosse darüber rücken mit.
private struct StoreyRowView: View {
    @Environment(AppModel.self) private var model
    let row: StoreyRow

    var body: some View {
        HStack(spacing: 6) {
            Text(row.name).lineLimit(1)
            Spacer(minLength: 4)
            MetreField(value: row.elevation, help: "OKFF über ±0,00") {
                model.run("SetStoreyElevation(\(row.element),\($0))")
            }
            MetreField(value: row.height, help: "Geschosshöhe, OKFF bis OKFF darüber") {
                model.run("SetStoreyHeight(\(row.element),\($0))")
            }
        }
    }
}

/// Zahl in Metern, übernommen mit ⏎.
private struct MetreField: View {
    let value: Double
    let help: String
    let onCommit: (Double) -> Void

    var body: some View {
        TextField("", value: Binding(get: { value }, set: { if abs($0 - value) > 1e-9 { onCommit($0) } }),
                  format: .number.precision(.fractionLength(2)))
            .multilineTextAlignment(.trailing)
            .frame(width: 44)
            .font(.callout.monospacedDigit())
            .help(help)
    }
}

/// Eine Ansicht in der Seitenleiste mit Zusatz, Heimat und Kontextmenü.
private struct EntryRow: View {
    @Environment(AppModel.self) private var model
    let entry: NavEntry
    let depth: Int

    var body: some View {
        HStack(spacing: 6) {
            Image(systemName: icon).foregroundStyle(.secondary).frame(width: 16)
            VStack(alignment: .leading, spacing: 1) {
                Text(entry.title).lineLimit(1)
                if !entry.detail.isEmpty {
                    Text(entry.detail).font(.caption).foregroundStyle(.secondary).lineLimit(1)
                }
            }
            if entry.home {
                Spacer()
                Image(systemName: "house").foregroundStyle(.secondary)
                    .help("Heimat: Schnittmarken verweisen auf dieses Blatt")
            }
        }
        .padding(.leading, CGFloat(depth) * 14)
        .modifier(DraggableView(enabled: entry.placeable && entry.index < 0, value: entry.view))
        .contextMenu { menu }
    }

    private var icon: String {
        switch entry.view {
        case let v where v.hasPrefix("FloorPlan"): "square.split.bottomrightquarter"
        case "Model": "cube"
        case let v where v.hasPrefix("Section"): "rectangle.split.1x2"
        case "SitePlan": "map"
        case let v where v.hasPrefix("Sheet"): "doc.richtext"
        default: "tablecells"
        }
    }

    @ViewBuilder
    private var menu: some View {
        Button("Öffnen") { model.go(entry.view) }
        Button("Rechts daneben öffnen") { model.openBeside(entry.view) }
        if entry.placeable && entry.index < 0 && model.viewState?.view == .sheet {
            Button("Auf das gezeigte Blatt legen") { model.dropView(entry.view, pane: model.activePane, x: 0, y: 0) }
        }
        if entry.index >= 0 {
            Divider()
            if entry.view.hasPrefix("Section("), !entry.home {
                let id = entry.view.dropFirst("Section(".count).dropLast()
                Button("Als Heimat wählen") { model.run("SetHome(\(id),\(entry.sheet))") }
            }
            Button("Vom Blatt nehmen") { model.run("RemovePlacedView(\(entry.sheet),\(entry.index))") }
        } else if entry.view.hasPrefix("Section(") {
            Divider()
            let id = entry.view.dropFirst("Section(".count).dropLast()
            Button("Löschen") {
                model.run("SetSelection([\(id)])")
                model.run("DeleteSelection")
            }
        } else if entry.view.hasPrefix("Sheet(") {
            Divider()
            let id = entry.view.dropFirst("Sheet(".count).dropLast()
            Button("Blatt löschen") { model.run("DeleteSheet(\(id))") }
        }
    }
}

/// Ansichten lassen sich aufs Blatt ziehen; die übrigen Zeilen nicht.
private struct DraggableView: ViewModifier {
    let enabled: Bool
    let value: String

    func body(content: Content) -> some View {
        if enabled {
            content.draggable(value)
        } else {
            content
        }
    }
}
