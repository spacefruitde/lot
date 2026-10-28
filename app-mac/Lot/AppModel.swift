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
    private(set) var spaces: [SpaceInfo] = []
    /// Gewählte Türen.
    private(set) var doors: [DoorInfo] = []
    /// Gewählte Öffnungen mit beiden Brüstungshöhen (ADR 0047).
    private(set) var openings: [OpeningInfo] = []
    /// Hinweise, die nicht stören sollen, etwa eine gewanderte Rohbaubrüstung.
    var notices: [String] = []
    /// Füllungen einer Öffnung: Durchgang, Tür, Fenster.
    static let fills = openingFills()
    /// Gewählte Wände mit Lage der Bezugslinie.
    private(set) var walls: [WallInfo] = []
    /// Gewählter Anschluss (Doppelklick an einem Wandknoten), ADR 0045.
    private(set) var junction: JunctionInfo?
    /// Typ der gewählten Bauteile, wenn sie alle denselben haben (ADR 0048).
    private(set) var selectedType: TypeInfo?
    /// Werte des gewählten Bauteils, die sein Typ vorgibt, mit Herkunft.
    private(set) var properties: [PropertyInfo] = []
    /// Alle Typen des Projekts, für Auswahl und Übersicht.
    private(set) var types: [TypeInfo] = []
    /// Typen der Büro-Bibliothek im Vergleich zum Projekt.
    private(set) var libraryItems: [LibraryItemInfo] = []
    var showTypes = false
    /// Materialien des Bürostandards und Schichtfunktionen für den Wandaufbau.
    static let materials = Lot.materials()
    static let functions = layerFunctions()
    /// Die sechs Lagen der Bezugslinie.
    static let alignments = wallAlignments()
    /// Arten der Oberkante: Geschoss, Decke, Dach, fest.
    static let tops = wallTops()
    /// Gewählte 2D-Texte.
    private(set) var texts: [TextInfo] = []
    /// Bauzustand der Auswahl: 0 Bestand, 1 Abbruch, 2 Neubau; `nil` ohne Bauteile mit Bauzustand,
    /// -1 bei gemischter Auswahl.
    private(set) var selectionPhase: Int?
    /// Auswahl nach Art und Typ, bei mehr als einem Bauteil (ADR 0043).
    private(set) var groups: [SelectionGroup] = []
    /// Flächen je Geschoss, für die Übersicht im Inspektor.
    private(set) var storeyAreas: [StoreyAreaInfo] = []
    /// Nutzungsgruppen nach DIN 277.
    static let usages = spaceUsages()
    /// Werkzeug, Geschoss, Maßstab, Auswahl und Hinweis aus dem Kern.
    private(set) var viewState: ViewState?
    private(set) var commands: [CommandItem] = []
    /// Zählt neue Dokumente; die Ansicht lädt dann neu.
    private(set) var documentGeneration = 0
    var stats = ViewportStatsDisplay()
    var errorMessage: String?
    var showPalette = false
    /// Prüfbericht (5.01 bis 5.05), auf Wunsch gerechnet.
    var findings: [FindingInfo] = []
    var showReport = false
    /// Hinweise aus dem Prüfbericht im Grundriss.
    var showFindings = false

    /// Die Ansichten, sobald ihr NSView steht: links (0) und, nebeneinander,
    /// rechts (1). Befehle, Seitenleiste und Inspektor gehören zur aktiven (ADR 0049).
    private(set) weak var primary: Viewport?
    private(set) weak var secondary: Viewport?
    private(set) var activePane = 0
    /// Zwei Ansichten nebeneinander mit gemeinsamer Auswahl.
    var showSplit = false
    /// Was die rechte Ansicht zeigen soll, sobald sie steht.
    private var pendingSecond: String?
    /// Titel jeder Ansicht, etwa „Schnitt A-A“.
    private(set) var paneTitles: [Int: String] = [:]
    /// Vorübergehend Ausgeblendetes je Ansicht, etwa „3 ausgeblendet · H zeigt wieder alles“.
    private(set) var paneTemporary: [Int: String] = [:]
    private var redrawers: [Int: () -> Void] = [:]

    /// Die aktive Ansicht.
    var viewport: Viewport? { activePane == 1 ? (secondary ?? primary) : primary }
    var viewports: [Viewport] { [primary, secondary].compactMap { $0 } }

    /// Bittet beide Ansichten, neu zu zeichnen.
    var requestRedraw: (() -> Void)? {
        { [weak self] in self?.redrawers.values.forEach { $0() } }
    }

    /// Seitenleiste „Projekt“ und Geschossleiste.
    private(set) var tree: [NavGroup] = []
    private(set) var storeys: [StoreyRow] = []
    /// Gewählte Schnitte und Ansichten.
    private(set) var sections: [SectionInfo] = []
    /// Auf dem gezeigten Blatt: das Blatt und die gewählte Ansicht darauf.
    private(set) var sheet: SheetInfo?
    private(set) var placed: PlacedInfo?
    static let formats = sheetFormats()
    /// Darstellung der aktiven Ansicht mit Herkunft jeder Einstellung (ADR 0050).
    private(set) var look: LookInfo?
    /// Gewählte Bauteile, die die aktive Ansicht nicht zeigt, mit Gründen.
    private(set) var hiddenInfos: [HiddenInfo] = []
    static let details = detailLevels()
    /// Gewählte Decken, Dächer und Treppen (ADR 0051).
    private(set) var slabInfos: [SlabInfo] = []
    private(set) var roofInfos: [RoofInfo] = []
    private(set) var stairInfos: [StairInfo] = []
    static let fallPurposes = Lot.fallPurposes()
    static let stairForms = Lot.stairForms()
    static let phases = phaseFilters()

    static let fileType = UTType(filenameExtension: "lot") ?? .data

    init() {
        document = LotDocument.newEmpty()
        #if DEBUG
            // Zum Testen ohne Öffnen-Dialog: Testprojekt aus einem Ordner außerhalb von ~/Documents.
            if let path = ProcessInfo.processInfo.environment["LOT_OPEN_TESTPROJECT"],
               let doc = try? LotDocument.openTestproject(path: path)
            {
                document = doc
            }
        #endif
        refresh()
    }

    var title: String {
        fileURL?.deletingPathExtension().lastPathComponent ?? document.name()
    }

    var isPlan: Bool { viewState?.view == .plan }

    // MARK: Ansicht

    func attach(_ viewport: Viewport, pane: Int) {
        if pane == 1 {
            secondary = viewport
        } else {
            primary = viewport
        }
        stats.rebuildMs = viewport.setDocument(document: document)
        loadMissingUnderlays()
        if pane == 1 {
            // Rechts zuerst, was gewünscht ist, sonst der erste Schnitt oder 3D.
            let first = tree.first { $0.title == "Schnitte und Ansichten" }?.entries.first?.view
            _ = viewport.go(target: pendingSecond ?? first ?? "Model")
            pendingSecond = nil
            if let ids = primary?.state().selection { viewport.setSelection(ids: ids) }
        }
        updateViewState()
    }

    func setRedraw(pane: Int, _ redraw: @escaping () -> Void) {
        redrawers[pane] = redraw
    }

    /// Die Ansicht ist weg, etwa nach Schließen der rechten Hälfte.
    func detach(pane: Int) {
        redrawers[pane] = nil
        paneTitles[pane] = nil
        if pane == 1 {
            secondary = nil
            activePane = 0
            updateViewState()
        }
    }

    /// Klick in eine Ansicht macht sie zur aktiven.
    func activate(pane: Int) {
        guard pane != activePane, pane == 0 || secondary != nil else { return }
        activePane = pane
        updateViewState()
    }

    private func viewport(_ pane: Int) -> Viewport? { pane == 1 ? secondary : primary }

    /// Ergebnis eines Ereignisses in der aktiven Ansicht verarbeiten.
    func handle(_ result: EventResult) {
        handle(result, pane: activePane)
    }

    /// Ergebnis eines Ereignisses in einer Ansicht: Modelländerungen und die
    /// Auswahl gehen an die andere, „öffnen“ springt zur Ansicht.
    func handle(_ result: EventResult, pane: Int) {
        if let error = result.error {
            errorMessage = error
        }
        let other = viewport(pane == 1 ? 0 : 1)
        if result.modelChanged {
            other?.refresh()
            refresh()
        }
        if let source = viewport(pane), let other {
            other.setSelection(ids: source.state().selection)
        }
        if let open = result.open, let source = viewport(pane) {
            let r = source.go(target: open)
            if let error = r.error { errorMessage = error }
        }
        updateViewState()
        requestRedraw?()
    }

    func run(_ id: String) {
        if id.hasPrefix("go:") {
            go(String(id.dropFirst(3)))
            return
        }
        guard let viewport else { return }
        handle(viewport.runCommand(id: id))
    }

    func setView(_ view: ViewKind) {
        viewport?.setView(view: view)
        updateViewState()
        requestRedraw?()
    }

    /// Springt zu einer Ansicht oder einem Raum (ADR 0049), in der aktiven Ansicht.
    func go(_ target: String) {
        guard let viewport else { return }
        handle(viewport.go(target: target))
    }

    /// Öffnet eine Ansicht in der rechten Hälfte; teilt das Fenster, falls nötig.
    func openBeside(_ target: String) {
        if let secondary {
            activePane = 1
            handle(secondary.go(target: target), pane: 1)
        } else {
            pendingSecond = target
            showSplit = true
        }
    }

    func back() {
        guard let viewport else { return }
        handle(viewport.back())
    }

    func forward() {
        guard let viewport else { return }
        handle(viewport.forward())
    }

    /// ⌘↑ und ⌘↓: ein Geschoss höher oder tiefer.
    func stepStorey(up: Bool) {
        guard let viewport else { return }
        handle(viewport.stepStorey(up: up))
    }

    func setFadedBelow(_ show: Bool) {
        guard let viewport else { return }
        handle(viewport.setFadedBelow(showIt: show))
    }

    // MARK: Sichtbarkeit (ADR 0050)

    /// Eine Einstellung der aktiven Ansicht, als RON eines `ViewField`, etwa `Scale(Some(50.0))`.
    func setViewField(_ field: String) {
        guard let viewport else { return }
        handle(viewport.setViewField(field: field))
    }

    func setViewTemplate(_ name: String?) {
        guard let viewport else { return }
        handle(viewport.setViewTemplate(name: name))
    }

    func setRule(_ rule: RuleInfo, on: Bool) {
        guard let viewport else { return }
        handle(viewport.setRuleOn(name: rule.name, on: on, own: rule.own))
    }

    func hideInView() {
        guard let viewport else { return }
        handle(viewport.hideInView())
    }

    func showInView(_ ids: [UInt64]) {
        guard let viewport else { return }
        handle(viewport.showInView(ids: ids))
    }

    func setPrintView(_ on: Bool) {
        viewport?.setPrintView(on: on)
        updateViewState()
        requestRedraw?()
    }

    /// Behebt einen Grund, warum ein Bauteil fehlt: Befehl oder Sprung.
    func fix(_ reason: ReasonInfo) {
        run(reason.fix)
    }

    /// Ansicht aus der Seitenleiste auf das Blatt in dieser Hälfte legen; Lage
    /// in Punkten von der Mitte, y nach oben.
    func dropView(_ view: String, pane: Int, x: Double, y: Double) {
        guard let target = viewport(pane) else { return }
        activePane = pane
        handle(target.dropView(view: view, x: Float(x), y: Float(y)), pane: pane)
    }

    /// Treffer für ⌘K: Ansichten und Räume.
    func jumps(_ query: String) -> [CommandItem] {
        document.search(query: query).map {
            CommandItem(id: "go:" + $0.target, title: $0.title, shortcut: $0.detail, group: "Springen")
        }
    }

    func zoomToFit() {
        viewport?.zoomToFit()
        requestRedraw?()
    }

    /// Bezeichnung eines Elements für den Inspektor.
    func label(for element: UInt64) -> String {
        document.elementLabel(element: element)
    }

    func updateViewState() {
        for (pane, v) in [(0, primary), (1, secondary)] {
            let state = v?.state()
            paneTitles[pane] = state?.title
            paneTemporary[pane] = state?.printView == true ? nil : state?.temporary
        }
        guard let viewport else { return }
        let state = viewport.state()
        viewState = state
        sections = state.selection.isEmpty ? [] : document.sections(ids: state.selection)
        placed = viewport.placedInfo()
        look = viewport.look()
        slabInfos = state.selection.isEmpty ? [] : document.slabs(ids: state.selection)
        roofInfos = state.selection.isEmpty ? [] : document.roofs(ids: state.selection)
        stairInfos = state.selection.isEmpty ? [] : document.stairs(ids: state.selection)
        hiddenInfos = state.selection.isEmpty ? [] : viewport.hiddenReasons(ids: state.selection)
        if state.view == .sheet, let id = UInt64(state.shown.dropFirst("Sheet(".count).dropLast()) {
            sheet = document.sheetInfo(sheet: id)
        } else {
            sheet = nil
        }
        commands = viewport.commands()
        parameters = state.selection.isEmpty ? [] : document.parameters(ids: state.selection)
        groups = state.selection.count > 1 ? document.selectionGroups(ids: state.selection) : []
        spaces = state.selection.isEmpty ? [] : document.spaces(ids: state.selection)
        doors = state.selection.isEmpty ? [] : document.doors(ids: state.selection)
        walls = state.selection.isEmpty ? [] : document.walls(ids: state.selection)
        openings = state.selection.isEmpty ? [] : document.openings(ids: state.selection)
        let selectedTypes = Set(state.selection.compactMap { document.elementType(element: $0) })
        let typed = state.selection.allSatisfy { document.elementType(element: $0) != nil }
        selectedType = selectedTypes.count == 1 && typed ? document.typeInfo(element: selectedTypes.first!) : nil
        properties = state.selection.count == 1 ? document.elementProperties(element: state.selection[0]) : []
        junction = viewport.junction()
        let phases = Set(state.selection.compactMap { document.elementPhase(element: $0) })
        selectionPhase = phases.isEmpty ? nil : (phases.count == 1 ? Int(phases.first!) : -1)
        texts = state.selection.compactMap { id in
            guard let text = document.annotationText(element: id) else { return nil }
            return TextInfo(element: id, text: text, height: document.annotationTextHeight(element: id) ?? 2.5)
        }
    }

    // MARK: Modell

    func set(_ value: Double, for parameter: ParamValue) {
        do {
            try document.setParam(element: parameter.element, kind: parameter.kind, value: value)
        } catch {
            errorMessage = Self.message(for: error)
        }
        refreshViews()
        refresh()
        updateViewState()
        requestRedraw?()
    }

    /// Setzt einen Parameter an mehreren Elementen in einem Undo-Schritt.
    func set(_ values: [Double], for elements: [UInt64], kind: ParamKind) {
        do {
            try document.setParams(elements: elements, kind: kind, values: values)
        } catch {
            errorMessage = Self.message(for: error)
        }
        refreshViews()
        refresh()
        updateViewState()
        requestRedraw?()
    }

    /// Setzt die Rohbaubrüstung; Lot rechnet die fertige Brüstung zurück.
    func setRoughSill(_ value: Double, for element: UInt64) {
        do {
            try document.setRoughSill(element: element, value: value)
        } catch {
            errorMessage = Self.message(for: error)
        }
        refreshViews()
        refresh()
        updateViewState()
        requestRedraw?()
    }

    /// Ändert Namen und Schichten eines Wand- oder Deckenaufbaus in einem Undo-Schritt.
    func setTypeLayers(_ info: TypeInfo, name: String? = nil, layers: [LayerInfo]? = nil) {
        do {
            try document.setTypeLayers(element: info.element, name: name ?? info.name, layers: layers ?? info.layers)
        } catch {
            errorMessage = Self.message(for: error)
        }
        refreshViews()
        refresh()
        updateViewState()
        requestRedraw?()
    }

    /// Öffnet die Typen-Übersicht mit dem Vergleich zur Bibliothek.
    func openTypes() {
        types = document.types()
        libraryItems = document.libraryItems()
        showTypes = true
    }

    /// Holt einen Typ aus der Bibliothek: neu oder als Ersatz für den gleichnamigen.
    func importFromLibrary(_ item: LibraryItemInfo, replace: Bool) {
        do {
            _ = try document.importFromLibrary(index: item.index, replace: replace)
        } catch {
            errorMessage = Self.message(for: error)
        }
        refreshViews()
        refresh()
        libraryItems = document.libraryItems()
        updateViewState()
        requestRedraw?()
    }

    /// Legt einen Typ des Projekts in die Bibliothek des Büros.
    func saveToLibrary(_ type: TypeInfo) {
        guard let text = document.libraryWithType(element: type.element) else { return }
        if let error = OfficeLibraryFile.save(text) {
            errorMessage = error
        }
        libraryItems = document.libraryItems()
    }

    /// Wählt genau diese Elemente, etwa eine Gruppe aus dem Inspektor.
    func select(_ ids: [UInt64]) {
        run("SetSelection([\(ids.map(String.init).joined(separator: ","))])")
    }

    /// Ändert Stempeltext, Nutzung oder Wohnflächen-Merkmal eines Raums.
    func setSpace(_ space: SpaceInfo, number: String? = nil, name: String? = nil, usage: UInt32? = nil, livingSpace: Bool? = nil) {
        do {
            try document.setSpace(
                element: space.element,
                number: number ?? space.number,
                name: name ?? space.name,
                usage: usage ?? space.usage,
                livingSpace: livingSpace ?? space.livingSpace
            )
        } catch {
            errorMessage = Self.message(for: error)
        }
        refreshViews()
        refresh()
        updateViewState()
        requestRedraw?()
    }

    /// Setzt den Bauzustand der gewählten Bauteile.
    func setPhase(_ phase: Int) {
        let names = ["Existing", "Demolished", "New"]
        guard names.indices.contains(phase) else { return }
        run("SetPhase(\(names[phase]))")
    }

    /// Ändert Inhalt und Schrifthöhe eines 2D-Texts.
    func setText(_ info: TextInfo, text: String? = nil, height: Double? = nil) {
        do {
            try document.setAnnotationText(element: info.element, text: text ?? info.text, height: height ?? info.height)
        } catch {
            errorMessage = Self.message(for: error)
        }
        refreshViews()
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

    /// Nach einer Änderung außerhalb der Ansichten: beide neu rechnen.
    func refreshViews() {
        for v in viewports {
            let ms = v.refresh()
            if v === viewport { stats.rebuildMs = ms }
        }
    }

    private func replace(with document: LotDocument, url: URL?) {
        self.document = document
        findings = []
        fileURL = url
        documentGeneration += 1
        refresh()
        for v in viewports {
            stats.rebuildMs = v.setDocument(document: document)
        }
        loadMissingUnderlays()
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
        tree = document.projectTree()
        storeys = document.storeyRows()
        canUndo = document.canUndo()
        canRedo = document.canRedo()
        storeyAreas = document.storeyAreas()
        types = document.types()
        if showTypes { libraryItems = document.libraryItems() }
        notices += document.roughSillNotices()
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

/// Ein gewählter 2D-Text für den Inspektor.
struct TextInfo: Equatable {
    let element: UInt64
    let text: String
    /// Schrifthöhe in mm auf dem Papier.
    let height: Double
}
