import SwiftUI

/// Befehlspalette (8.04): tippen filtert, ⏎ führt aus, Esc schließt.
/// Die Befehle kommen aus dem Kern; dieselben kann später die KI aufrufen.
struct CommandPalette: View {
    let commands: [CommandItem]
    /// Sprungziele zur Suche: Geschosse, Schnitte, Blätter, Räume (ADR 0049).
    let search: (String) -> [CommandItem]
    /// Mit Kennung ausführen, mit `nil` schließen.
    let onDone: (String?) -> Void

    @State private var query = ""
    @State private var selected = 0
    @FocusState private var focused: Bool

    private var matches: [CommandItem] {
        let words = query.lowercased().split(separator: " ")
        guard !words.isEmpty else { return commands + search("") }
        let found = commands.filter { item in
            let text = "\(item.group) \(item.title)".lowercased()
            return words.allSatisfy { text.contains($0) }
        }
        // Ansichten und Räume zuerst, wenn sie passen: ⌘K springt vor allem.
        return search(query) + found
    }

    var body: some View {
        VStack(spacing: 0) {
            TextField("Befehl, Ansicht oder Raum …", text: $query)
                .textFieldStyle(.plain)
                .font(.title3)
                .padding(12)
                .focused($focused)
                .onSubmit { run(at: selected) }
                .onKeyPress(.downArrow) {
                    selected = min(selected + 1, max(matches.count - 1, 0))
                    return .handled
                }
                .onKeyPress(.upArrow) {
                    selected = max(selected - 1, 0)
                    return .handled
                }
                .onKeyPress(.escape) {
                    onDone(nil)
                    return .handled
                }
                .onChange(of: query) { selected = 0 }
            Divider()
            ScrollViewReader { proxy in
                ScrollView {
                    LazyVStack(alignment: .leading, spacing: 0) {
                        ForEach(Array(matches.enumerated()), id: \.element.id) { index, item in
                            HStack {
                                Text(item.group)
                                    .foregroundStyle(.secondary)
                                    .frame(width: 100, alignment: .leading)
                                Text(item.title)
                                Spacer()
                                Text(item.shortcut).foregroundStyle(.secondary)
                            }
                            .padding(.horizontal, 12)
                            .padding(.vertical, 6)
                            .background(index == selected ? Color.accentColor.opacity(0.2) : .clear)
                            .contentShape(Rectangle())
                            .onTapGesture { run(at: index) }
                            .id(index)
                        }
                    }
                }
                .frame(maxHeight: 320)
                .onChange(of: selected) { proxy.scrollTo(selected) }
            }
        }
        .frame(width: 520)
        .background(.regularMaterial, in: RoundedRectangle(cornerRadius: 10))
        .shadow(radius: 12)
        .onAppear { focused = true }
    }

    private func run(at index: Int) {
        guard matches.indices.contains(index) else { return }
        onDone(matches[index].id)
    }
}
