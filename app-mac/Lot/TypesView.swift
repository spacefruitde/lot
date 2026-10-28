import AppKit
import SwiftUI

/// Typen-Übersicht (ADR 0048): alle Typen des Projekts mit Nutzung, und die
/// Bibliothek des Büros, aus der man Typen einzeln holt.
struct TypesView: View {
    @Environment(AppModel.self) private var model
    @Environment(\.dismiss) private var dismiss
    @State private var tab = 0
    @State private var unusedOnly = false

    var body: some View {
        VStack(spacing: 0) {
            HStack {
                Picker("", selection: $tab) {
                    Text("Im Projekt").tag(0)
                    Text("Bibliothek des Büros").tag(1)
                }
                .pickerStyle(.segmented)
                .labelsHidden()
                .frame(width: 320)
                Spacer()
                if tab == 0 {
                    Toggle("Nur unbenutzte", isOn: $unusedOnly)
                } else {
                    Button("Bibliothek bearbeiten …") { OfficeLibraryFile.edit() }
                }
                Button("Fertig") { dismiss() }
                    .keyboardShortcut(.defaultAction)
            }
            .padding(12)
            Divider()
            if tab == 0 {
                projectTypes
            } else {
                libraryList
            }
        }
        .frame(minWidth: 720, minHeight: 480)
    }

    private var projectTypes: some View {
        List {
            ForEach(["Wandaufbau", "Deckenaufbau", "Tür- oder Fenstertyp"], id: \.self) { kind in
                let rows = model.types.filter { $0.kindTitle == kind && (!unusedOnly || $0.uses == 0) }
                if !rows.isEmpty {
                    Section(kind) {
                        ForEach(rows, id: \.element) { TypeRow(info: $0) }
                    }
                }
            }
        }
    }

    private var libraryList: some View {
        List {
            ForEach(["Wandaufbau", "Deckenaufbau", "Tür- oder Fenstertyp"], id: \.self) { kind in
                let rows = model.libraryItems.filter { $0.kindTitle == kind }
                if !rows.isEmpty {
                    Section(kind) {
                        ForEach(rows, id: \.index) { LibraryRow(item: $0) }
                    }
                }
            }
        }
    }
}

/// Ein Typ des Projekts: Vorschau, Name, Kennzeichen, Nutzung und Aktionen.
private struct TypeRow: View {
    @Environment(AppModel.self) private var model
    let info: TypeInfo
    @State private var name = ""

    var body: some View {
        HStack(spacing: 12) {
            LayerPreview(layers: info.layers)
                .frame(width: 56, height: 18)
            TextField("Name", text: $name)
                .onSubmit { if name != info.name { model.run("RenameType(\(info.element),\(ron(name)))") } }
                .frame(minWidth: 220)
            Text(info.properties.first { $0.property == "Mark" }?.value ?? "")
                .foregroundStyle(.secondary)
                .frame(width: 50, alignment: .leading)
            Text(info.usesText)
                .monospacedDigit()
                .foregroundStyle(info.uses == 0 ? .orange : .secondary)
                .frame(width: 140, alignment: .leading)
            Menu {
                Button("Kopieren") { model.run("DuplicateType(\(info.element))") }
                Button("In die Bibliothek des Büros") { model.saveToLibrary(info) }
                let others = model.types.filter { $0.kind == info.kind && $0.element != info.element }
                if info.uses > 0 && !others.isEmpty {
                    Menu("Tauschen gegen") {
                        ForEach(others, id: \.element) { other in
                            Button(other.name) { model.run("ReplaceType(\(info.element),\(other.element))") }
                        }
                    }
                }
                Divider()
                if info.uses == 0 || info.kind == 2 {
                    Button(info.uses == 0 ? "Löschen" : "Löschen, Öffnungen ohne Typ") {
                        model.run("DeleteType(\(info.element),None)")
                    }
                }
                if info.uses > 0 && !others.isEmpty {
                    Menu("Löschen, Ersatz") {
                        ForEach(others, id: \.element) { other in
                            Button(other.name) { model.run("DeleteType(\(info.element),Some(\(other.element)))") }
                        }
                    }
                }
            } label: {
                Image(systemName: "ellipsis.circle")
            }
            .menuStyle(.borderlessButton)
            .frame(width: 32)
        }
        .onAppear { name = info.name }
        .onChange(of: info.name) { name = info.name }
    }
}

/// Ein Typ der Bibliothek mit Vorher-Nachher-Vergleich zum gleichnamigen im Projekt.
private struct LibraryRow: View {
    @Environment(AppModel.self) private var model
    let item: LibraryItemInfo

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack {
                Text(item.name)
                Spacer()
                switch item.status {
                case 0:
                    Text("neu").foregroundStyle(.secondary)
                    Button("Holen") { model.importFromLibrary(item, replace: false) }
                case 1:
                    Text("wie im Projekt").foregroundStyle(.secondary)
                default:
                    Text("weicht ab").foregroundStyle(.orange)
                    Button("Ersetzen, wirkt auf \(item.usesText)") { model.importFromLibrary(item, replace: true) }
                    Button("Als neuen Typ") { model.importFromLibrary(item, replace: false) }
                }
            }
            ForEach(item.differences, id: \.self) { line in
                Text(line).font(.caption).foregroundStyle(.secondary)
            }
            if !item.missingMaterials.isEmpty {
                Text("Fehlt im Bürostandard: \(item.missingMaterials.joined(separator: ", "))")
                    .font(.caption)
                    .foregroundStyle(.orange)
            }
        }
        .padding(.vertical, 2)
    }
}

/// Schichten als Streifen, so breit wie dick; Farbe nach Funktion.
struct LayerPreview: View {
    let layers: [LayerInfo]

    var body: some View {
        GeometryReader { geo in
            let total = max(layers.reduce(0) { $0 + $1.thickness }, 0.001)
            HStack(spacing: 0) {
                ForEach(Array(layers.enumerated()), id: \.offset) { _, layer in
                    Rectangle()
                        .fill(color(layer.function))
                        .frame(width: geo.size.width * layer.thickness / total)
                }
            }
        }
        .overlay(Rectangle().stroke(.secondary.opacity(0.5), lineWidth: 0.5))
    }

    private func color(_ function: UInt32) -> Color {
        switch function {
        case 0: Color(white: 0.45)
        case 1: Color(red: 0.95, green: 0.85, blue: 0.45)
        case 2: Color(white: 0.97)
        case 3: Color(red: 0.75, green: 0.85, blue: 0.95)
        default: Color(white: 0.82)
        }
    }
}

/// Text als RON-Zeichenkette für Befehle, mit Anführungszeichen.
func ron(_ text: String) -> String {
    "\"" + text.replacingOccurrences(of: "\\", with: "\\\\").replacingOccurrences(of: "\"", with: "\\\"") + "\""
}

/// Typen-Bibliothek des Büros (ADR 0048), neben dem Bürostandard.
enum OfficeLibraryFile {
    static var url: URL {
        OfficeStandardFile.url.deletingLastPathComponent().appendingPathComponent("bibliothek.ron")
    }

    /// Setzt die Bibliothek des Büros beim Start, falls die Datei da ist.
    static func install() -> String? {
        guard let text = try? String(contentsOf: url, encoding: .utf8) else { return nil }
        return apply(text)
    }

    /// Schreibt die Bibliothek und setzt sie für die Sitzung.
    static func save(_ text: String) -> String? {
        do {
            try FileManager.default.createDirectory(at: url.deletingLastPathComponent(), withIntermediateDirectories: true)
            try text.write(to: url, atomically: true, encoding: .utf8)
        } catch {
            return error.localizedDescription
        }
        return apply(text)
    }

    /// Öffnet die Datei im Texteditor; legt sie mit der eingebauten Bibliothek an, falls sie fehlt.
    static func edit() {
        if !FileManager.default.fileExists(atPath: url.path) {
            _ = save(builtinLibraryText())
        }
        NSWorkspace.shared.open(url)
    }

    private static func apply(_ text: String) -> String? {
        do {
            try installOfficeLibrary(text: text)
            return nil
        } catch let LotError.Failed(message) {
            return message
        } catch {
            return error.localizedDescription
        }
    }
}
