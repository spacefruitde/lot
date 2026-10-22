import AppKit
import Observation
import UniformTypeIdentifiers

/// Zustand des geöffneten Projekts. Alle Änderungen am Modell laufen über den
/// Rust-Kern, die App hält nur, was sie für die Anzeige braucht.
@MainActor
@Observable
final class AppModel {
    private(set) var document: LotDocument
    private(set) var parameters: [ParamValue] = []
    private(set) var fileURL: URL?
    private(set) var canUndo = false
    private(set) var canRedo = false
    /// Zählt Änderungen am Modell. Die 3D-Ansicht rechnet neu, wenn sich der Wert ändert.
    private(set) var revision = 0
    /// Zählt Aufforderungen, die Ansicht auf das ganze Modell einzustellen.
    private(set) var fitRequest = 0
    var stats = ViewportStatsDisplay()
    var errorMessage: String?

    static let fileType = UTType(filenameExtension: "lot") ?? .data

    init() {
        document = LotDocument.newSample()
        refresh()
    }

    var title: String {
        fileURL?.deletingPathExtension().lastPathComponent ?? document.name()
    }

    func newSample() {
        document = LotDocument.newSample()
        fileURL = nil
        refresh()
        fitRequest += 1
    }

    func set(_ value: Double, for parameter: ParamValue) {
        perform { try document.setParam(element: parameter.element, kind: parameter.kind, value: value) }
    }

    func undo() {
        perform { _ = try document.undo() }
    }

    func redo() {
        perform { _ = try document.redo() }
    }

    func zoomToFit() {
        fitRequest += 1
    }

    func openWithPanel() {
        let panel = NSOpenPanel()
        panel.allowedContentTypes = [Self.fileType]
        panel.allowsOtherFileTypes = true
        guard panel.runModal() == .OK, let url = panel.url else { return }
        do {
            document = try LotDocument.open(path: url.path)
            fileURL = url
            refresh()
            fitRequest += 1
        } catch {
            errorMessage = Self.message(for: error)
        }
    }

    func save() {
        guard let url = fileURL else {
            saveWithPanel()
            return
        }
        write(to: url)
    }

    func saveWithPanel() {
        let panel = NSSavePanel()
        panel.allowedContentTypes = [Self.fileType]
        panel.nameFieldStringValue = title + ".lot"
        guard panel.runModal() == .OK, let url = panel.url else { return }
        write(to: url)
    }

    private func write(to url: URL) {
        do {
            try document.save(path: url.path)
            fileURL = url
        } catch {
            errorMessage = Self.message(for: error)
        }
    }

    private func perform(_ action: () throws -> Void) {
        do {
            try action()
        } catch {
            errorMessage = Self.message(for: error)
        }
        refresh()
    }

    private func refresh() {
        parameters = document.parameters()
        canUndo = document.canUndo()
        canRedo = document.canRedo()
        revision += 1
    }

    static func message(for error: Error) -> String {
        if let error = error as? LotError {
            switch error {
            case let .Failed(message): return message
            }
        }
        return error.localizedDescription
    }
}

/// Messwerte der 3D-Ansicht für die Statuszeile.
struct ViewportStatsDisplay: Equatable {
    var frameMs: Double = 0
    var rebuildMs: Double = 0
    var triangles: UInt32 = 0
}
