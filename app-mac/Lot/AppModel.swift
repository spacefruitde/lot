import AppKit
import Observation
import UniformTypeIdentifiers

/// Zustand des geöffneten Projekts. Alle Änderungen am Modell laufen über den
/// Rust-Kern; die App hält nur, was sie für die Anzeige braucht.
@MainActor
@Observable
final class AppModel {
    private(set) var document: LotDocument
    private(set) var fileURL: URL?
    private(set) var canUndo = false
    private(set) var canRedo = false
    /// Parameter der gewählten Elemente.
    private(set) var parameters: [ParamValue] = []
    /// Gewählte Räume mit Flächen.
    private(set) var rooms: [RoomInfo] = []
    /// Gewählte Türen.
    private(set) var doors: [DoorInfo] = []
    /// Flächen je Geschoss, für die Übersicht im Inspektor.
    private(set) var storeyAreas: [StoreyAreaInfo] = []
    /// Nutzungsgruppen nach DIN 277.
    static let usages = roomUsages()
    /// Werkzeug, Geschoss, Maßstab, Auswahl und Hinweis aus dem Kern.
    private(set) var viewState: ViewState?
    private(set) var commands: [CommandItem] = []
    /// Zählt neue Dokumente; die Ansicht lädt dann neu.
    private(set) var documentGeneration = 0
    var stats = ViewportStatsDisplay()
    var errorMessage: String?
    var showPalette = false

    /// Die Ansicht, sobald ihr NSView steht.
    private(set) weak var viewport: Viewport?
    /// Bittet die Ansicht, neu zu zeichnen.
    var requestRedraw: (() -> Void)?

    static let fileType = UTType(filenameExtension: "lot") ?? .data

    init() {
        document = LotDocument.newEmpty()
        refresh()
    }

    var title: String {
        fileURL?.deletingPathExtension().lastPathComponent ?? document.name()
    }

    var isPlan: Bool { viewState?.view != .model }

    // MARK: Ansicht

    func attach(_ viewport: Viewport) {
        self.viewport = viewport
        stats.rebuildMs = viewport.setDocument(document: document)
        updateViewState()
    }

    /// Ergebnis eines Ereignisses in der Ansicht verarbeiten.
    func handle(_ result: EventResult) {
        if let error = result.error {
            errorMessage = error
        }
        if result.modelChanged {
            refresh()
        }
        updateViewState()
    }

    func run(_ id: String) {
        guard let viewport else { return }
        handle(viewport.runCommand(id: id))
        requestRedraw?()
    }

    func setView(_ view: ViewKind) {
        viewport?.setView(view: view)
        updateViewState()
        requestRedraw?()
    }

    func zoomToFit() {
        viewport?.zoomToFit()
        requestRedraw?()
    }

    /// Bezeichnung eines Elements für den Inspektor.
    func label(for element: UInt64) -> String {
        document.elementLabel(element: element)
    }

    private func updateViewState() {
        guard let viewport else { return }
        let state = viewport.state()
        viewState = state
        commands = viewport.commands()
        parameters = state.selection.isEmpty ? [] : document.parameters(ids: state.selection)
        rooms = state.selection.isEmpty ? [] : document.rooms(ids: state.selection)
        doors = state.selection.isEmpty ? [] : document.doors(ids: state.selection)
    }

    // MARK: Modell

    func set(_ value: Double, for parameter: ParamValue) {
        do {
            try document.setParam(element: parameter.element, kind: parameter.kind, value: value)
        } catch {
            errorMessage = Self.message(for: error)
        }
        if let viewport {
            stats.rebuildMs = viewport.refresh()
        }
        refresh()
        updateViewState()
        requestRedraw?()
    }

    /// Ändert Stempeltext, Nutzung oder Wohnflächen-Merkmal eines Raums.
    func setRoom(_ room: RoomInfo, number: String? = nil, name: String? = nil, usage: UInt32? = nil, livingSpace: Bool? = nil) {
        do {
            try document.setRoom(
                element: room.element,
                number: number ?? room.number,
                name: name ?? room.name,
                usage: usage ?? room.usage,
                livingSpace: livingSpace ?? room.livingSpace
            )
        } catch {
            errorMessage = Self.message(for: error)
        }
        if let viewport {
            stats.rebuildMs = viewport.refresh()
        }
        refresh()
        updateViewState()
        requestRedraw?()
    }

    func undo() {
        if viewport != nil { run("Undo") } else { _ = try? document.undo(); refresh() }
    }

    func redo() {
        if viewport != nil { run("Redo") } else { _ = try? document.redo(); refresh() }
    }

    func newProject() {
        replace(with: LotDocument.newEmpty(), url: nil)
    }

    func openWithPanel() {
        let panel = NSOpenPanel()
        panel.allowedContentTypes = [Self.fileType]
        panel.allowsOtherFileTypes = true
        guard panel.runModal() == .OK, let url = panel.url else { return }
        do {
            replace(with: try LotDocument.open(path: url.path), url: url)
        } catch {
            errorMessage = Self.message(for: error)
        }
    }

    /// Öffnet ein Testprojekt (Ordner mit projekt.ron), etwa das EFH zum Ausprobieren.
    func openTestprojectWithPanel() {
        let panel = NSOpenPanel()
        panel.canChooseDirectories = true
        panel.canChooseFiles = false
        panel.message = "Ordner eines Testprojekts wählen (mit projekt.ron)"
        guard panel.runModal() == .OK, let url = panel.url else { return }
        do {
            replace(with: try LotDocument.openTestproject(path: url.path), url: nil)
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

    private func replace(with document: LotDocument, url: URL?) {
        self.document = document
        fileURL = url
        documentGeneration += 1
        if let viewport {
            stats.rebuildMs = viewport.setDocument(document: document)
        }
        refresh()
        updateViewState()
        requestRedraw?()
    }

    private func write(to url: URL) {
        do {
            try document.save(path: url.path)
            fileURL = url
        } catch {
            errorMessage = Self.message(for: error)
        }
    }

    private func refresh() {
        canUndo = document.canUndo()
        canRedo = document.canRedo()
        storeyAreas = document.storeyAreas()
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

/// Messwerte der Ansicht für die Statuszeile.
struct ViewportStatsDisplay: Equatable {
    var frameMs: Double = 0
    var rebuildMs: Double = 0
    var triangles: UInt32 = 0
}
