import SwiftUI

/// Eigenschaften der gewählten Elemente. Jede Änderung ist eine Operation im
/// Kern und lässt sich widerrufen.
struct InspectorView: View {
    @Environment(AppModel.self) private var model

    var body: some View {
        Form {
            if !model.notices.isEmpty {
                Section("Hinweis") {
                    ForEach(model.notices, id: \.self) { Text($0).foregroundStyle(.orange) }
                    Button("Gelesen") { model.notices.removeAll() }
                }
            }
            if let placed = model.placed {
                PlacedSection(info: placed)
            } else if let sheet = model.sheet {
                SheetSection(info: sheet)
            }
            if let junction = model.junction {
                JunctionSection(junction: junction)
            } else if model.parameters.isEmpty && model.spaces.isEmpty && model.texts.isEmpty && model.sections.isEmpty {
                if model.viewState?.view == .plan || model.viewState?.view == .section {
                    Section {
                        Text("Nichts gewählt. Eine Wand oder einen Raumstempel anklicken zeigt die Eigenschaften.")
                            .foregroundStyle(.secondary)
                    }
                }
                if model.storeyAreas.contains(where: { $0.net > 0 }) {
                    Section("Flächen") {
                        ForEach(model.storeyAreas, id: \.name) { s in
                            LabeledContent(s.name) {
                                VStack(alignment: .trailing) {
                                    Text("NRF \(area(s.net)) · BGF \(area(s.gross))")
                                    if s.living > 0 { Text("Wohnfläche \(area(s.living))") }
                                }
                                .font(.callout)
                                .monospacedDigit()
                            }
                        }
                    }
                }
            } else {
                ForEach(model.sections, id: \.element) { SectionSection(info: $0) }
                ForEach(model.spaces, id: \.element) { SpaceSection(space: $0) }
                ForEach(model.texts, id: \.element) { TextSection(info: $0) }
                if let phase = model.selectionPhase {
                    Section("Bauzustand") {
                        Picker("Bauzustand", selection: Binding(get: { phase }, set: { model.setPhase($0) })) {
                            if phase < 0 { Text("gemischt").tag(-1) }
                            Text("Bestand").tag(0)
                            Text("Abbruch").tag(1)
                            Text("Neubau").tag(2)
                        }
                        .pickerStyle(.segmented)
                        .labelsHidden()
                    }
                }
                if !model.walls.isEmpty {
                    WallSection()
                }
                if let type = model.selectedType {
                    TypeSection(info: type)
                }
                if !model.properties.isEmpty {
                    ElementPropertiesSection()
                }
                if model.groups.count > 0 {
                    SelectionGroupsSection()
                }
                if model.groups.isEmpty {
                    ForEach(groups, id: \.0) { element, parameters in
                        Section(model.label(for: element)) {
                            ForEach(parameters, id: \.kind) { ParameterRow(parameter: $0) }
                            if let opening = model.openings.first(where: { $0.element == element }) {
                                OpeningRows(opening: opening)
                            }
                            if let door = model.doors.first(where: { $0.element == element }) {
                                DoorRows(door: door)
                            }
                        }
                    }
                } else {
                    // Mehrfachauswahl: je Gruppe ein Feld je Eigenschaft, „verschieden“ mit Spanne (ADR 0044).
                    ForEach(model.groups, id: \.label) { group in
                        let values = model.parameters.filter { group.ids.contains($0.element) }
                        if !values.isEmpty {
                            Section("\(group.label) · \(group.ids.count)") {
                                ForEach(kinds(of: values), id: \.self) { kind in
                                    MultiParameterRow(kind: kind, values: values.filter { $0.kind == kind })
                                }
                            }
                        }
                    }
                }
            }
            if let state = model.viewState, state.view == .plan {
                Section("Zeichnen") {
                    LabeledContent("Geschoss", value: state.storey)
                    LabeledContent("Wandaufbau", value: state.wallType)
                    LabeledContent("Maßstab", value: "1:\(Int(state.scale))")
                }
            }
            Section("Bedienung") {
                VStack(alignment: .leading, spacing: 4) {
                    Text("Kürzel wie Revit: WA Wand, DR Tür, WN Fenster, RM Raum, GR Achse, CL Stütze, DI Bemaßen, LI Linie, TX Text, SA Gleiche wählen, Esc Auswählen, ⌘K Befehle")
                    Text("Auswählen: Klick, Ziehen ergibt einen Rahmen (nach links auch Angeschnittenes), ⌥ ein Lasso, ⇧ fügt hinzu, ⇥ blättert unter dem Zeiger, ⌘← vorige Auswahl")
                    Text("Bearbeiten: Ziehen verschiebt die Auswahl, mit ⌥ als Kopie, Pfeile 1 cm (⇧ 10 cm), Leertaste dreht um")
                    Text("Wände: Bezugslinie im Inspektor wechseln, die Wand bleibt liegen; Doppelklick auf eine Ecke zeigt den Anschluss mit Ausnahmen")
                    Text("Ändern: MV verschieben, CO kopieren, RO drehen, MM spiegeln, AR Reihe, OF versetzen, AL ausrichten, TR stutzen, SL teilen; ⏎ wiederholt")
                    Text("Fangen: Name am Zeiger, ⇥ nächster Fangpunkt, Q Bezugspunkt mit Spurlinien (Länge tippen setzt den Punkt), SE SM SI SP SN SO für einen Klick, ZR Zeichenrichtung von einer Kante, WR Wandrechteck")
                    Text("Zeichnen: klicken oder tippen, etwa 3,65 · <30 · 3,65<30 · @1,20;0,80 · 2*1,25+0,115, dann ⏎; ⇧ rechtwinklig")
                    Text("Navigation: Seitenleiste links (⌃⌘S), ⌘K springt auch zu Ansichten und Räumen, ⌘[ und ⌘] zurück und vor, ⌘↑ und ⌘↓ Geschoss, ⌥⌘N nebeneinander")
                    Text("Schnitte: SC zeichnen, AN Ansicht, Leertaste dreht den Blick, Doppelklick auf die Marke öffnet den Schnitt; Ansichten aus der Seitenleiste aufs Blatt ziehen")
                    Text("Grundriss: zwei Finger verschieben, spreizen zoomt")
                    Text("3D: zwei Finger drehen, mit ⇧ verschieben")
                }
                .font(.callout)
                .foregroundStyle(.secondary)
            }
        }
        .formStyle(.grouped)
    }

    /// Eigenschaften in der Reihenfolge ihres ersten Auftretens.
    private func kinds(of values: [ParamValue]) -> [ParamKind] {
        var out: [ParamKind] = []
        for v in values where !out.contains(v.kind) { out.append(v.kind) }
        return out
    }

    private var groups: [(UInt64, [ParamValue])] {
        var order: [UInt64] = []
        var by: [UInt64: [ParamValue]] = [:]
        for p in model.parameters {
            if by[p.element] == nil { order.append(p.element) }
            by[p.element, default: []].append(p)
        }
        return order.map { ($0, by[$0] ?? []) }
    }
}

/// Fläche im deutschen Format, „24,35 m²“.
func area(_ m2: Double) -> String {
    m2.formatted(.number.precision(.fractionLength(2)).locale(Locale(identifier: "de_DE"))) + " m²"
}

/// Raumstempel und Flächen eines Raums. Texte werden mit Return, beim
/// Verlassen des Feldes oder mit der nächsten Änderung im Raum übernommen.
private struct SpaceSection: View {
    @Environment(AppModel.self) private var model
    let space: SpaceInfo
    @State private var number = ""
    @State private var name = ""
    @FocusState private var focused: Bool

    var body: some View {
        Section(model.label(for: space.element)) {
            TextField("Nummer", text: $number)
                .focused($focused)
                .onSubmit(commit)
            TextField("Name", text: $name)
                .focused($focused)
                .onSubmit(commit)
            Picker("Nutzung", selection: Binding(get: { space.usage }, set: { model.setSpace(space, number: number, name: name, usage: $0) })) {
                ForEach(Array(AppModel.usages.enumerated()), id: \.offset) { index, usage in
                    Text("\(usage.code) \(usage.title)").tag(UInt32(index))
                }
            }
            Toggle(
                "Zählt zur Wohnfläche",
                isOn: Binding(get: { space.livingSpace }, set: { model.setSpace(space, number: number, name: name, livingSpace: $0) })
            )
            LabeledContent("Netto-Raumfläche", value: space.netArea.map(area) ?? "offen")
            if let living = space.livingArea {
                LabeledContent("Wohnfläche (WoFlV)", value: area(living))
            }
        }
        .onAppear(perform: load)
        .onChange(of: space.element) { load() }
        .onChange(of: space.number) { number = space.number }
        .onChange(of: space.name) { name = space.name }
        .onChange(of: focused) { if !focused { commit() } }
    }

    private func load() {
        number = space.number
        name = space.name
    }

    private func commit() {
        guard number != space.number || name != space.name else { return }
        model.setSpace(space, number: number, name: name)
    }
}

/// Inhalt und Schrifthöhe eines 2D-Texts, übernommen mit Return oder beim Verlassen des Feldes.
private struct TextSection: View {
    @Environment(AppModel.self) private var model
    let info: TextInfo
    @State private var text = ""
    @FocusState private var focused: Bool

    var body: some View {
        Section(model.label(for: info.element)) {
            TextField("Text", text: $text)
                .focused($focused)
                .onSubmit(commit)
            Picker("Schrifthöhe", selection: Binding(get: { info.height }, set: { model.setText(info, text: text, height: $0) })) {
                ForEach([1.8, 2.5, 3.5, 5.0, 7.0], id: \.self) { h in
                    Text(h.formatted(.number.locale(Locale(identifier: "de_DE"))) + " mm").tag(h)
                }
            }
        }
        .onAppear { text = info.text }
        .onChange(of: info.element) { text = info.text }
        .onChange(of: info.text) { text = info.text }
        .onChange(of: focused) { if !focused { commit() } }
    }

    private func commit() {
        guard text != info.text, !text.trimmingCharacters(in: .whitespaces).isEmpty else { return }
        model.setText(info, text: text)
    }
}

/// Bezugslinie und Umdrehen der gewählten Wände (ADR 0045). Ein Wechsel verschiebt keine Wand.
private struct WallSection: View {
    @Environment(AppModel.self) private var model

    var body: some View {
        let lines = Set(model.walls.map(\.alignment))
        let current = lines.count == 1 ? Int(lines.first!) : -1
        Section(model.walls.count == 1 ? "Wand" : "Wände · \(model.walls.count)") {
            Picker("Bezugslinie", selection: Binding(get: { current }, set: { i in
                if i >= 0 { model.run(AppModel.alignments[i].command) }
            })) {
                if current < 0 { Text("verschieden").tag(-1) }
                ForEach(Array(AppModel.alignments.enumerated()), id: \.offset) { index, item in
                    Text(item.title).tag(index)
                }
            }
            let tops = Set(model.walls.map(\.top))
            Picker("Oberkante", selection: Binding(get: { tops.count == 1 ? Int(tops.first!) : -1 }, set: { i in
                if i >= 0 { model.run(AppModel.tops[i].command) }
            })) {
                if tops.count > 1 { Text("verschieden").tag(-1) }
                ForEach(Array(AppModel.tops.enumerated()), id: \.offset) { index, item in
                    Text(item.title).tag(index)
                }
            }
            if model.walls.count == 1, let wall = model.walls.first {
                LabeledContent("Oberkante über OKFF", value: wall.topHeight.formatted(.number.precision(.fractionLength(2)).locale(Locale(identifier: "de_DE"))) + " m")
            }
            LabeledContent("Innen und außen") {
                Button("Umdrehen") { model.run("FlipWalls") }
            }
            Text("Die Bezugslinie ist eine Lesart: Ein Wechsel verschiebt die Wand nicht. Doppelklick auf eine Ecke zeigt den Anschluss.")
                .font(.caption)
                .foregroundStyle(.secondary)
        }
    }
}

/// Typ der gewählten Bauteile (ADR 0048): tauschen, Typwerte direkt ändern
/// („wirkt auf 14 Wände“), Werte festlegen, Schichten bearbeiten (ADR 0045).
private struct TypeSection: View {
    @Environment(AppModel.self) private var model
    let info: TypeInfo
    @State private var name = ""
    @State private var thickness: [String] = []

    var body: some View {
        Section("\(info.kindTitle) · wirkt auf \(info.usesText)") {
            Picker("Typ", selection: Binding(get: { info.element }, set: { model.run("AssignType(Some(\($0)))") })) {
                ForEach(model.types.filter { $0.kind == info.kind }, id: \.element) { Text($0.name).tag($0.element) }
            }
            TextField("Name", text: $name)
                .onSubmit { if name != info.name { model.run("RenameType(\(info.element),\(ron(name)))") } }
            // Eigene Kennungen je Abschnitt: Gleiche Kennungen in einer Form lassen Zeilen verschwinden.
            ForEach(info.properties.map { ("typ-" + $0.property, $0) }, id: \.0) { TypeValueRow(type: info, property: $0.1) }
            if !info.layers.isEmpty {
                layerRows
            }
            Button("Alle Typen …") { model.openTypes() }
                .buttonStyle(.link)
        }
        .onAppear(perform: load)
        .onChange(of: info) { load() }
    }

    @ViewBuilder private var layerRows: some View {
        ForEach(Array(info.layers.enumerated()).map { ("schicht-\($0.offset)", $0.offset, $0.element) }, id: \.0) { _, index, layer in
            VStack(alignment: .leading, spacing: 4) {
                HStack {
                    Picker("Material", selection: Binding(get: { layer.material }, set: { set(index) { $0.material = $1 }($0) })) {
                        if !AppModel.materials.contains(where: { $0.command == layer.material }) {
                            Text(layer.material).tag(layer.material)
                        }
                        ForEach(AppModel.materials, id: \.command) { Text($0.title).tag($0.command) }
                    }
                    .labelsHidden()
                    TextField("", text: Binding(get: { index < thickness.count ? thickness[index] : "" }, set: { if index < thickness.count { thickness[index] = $0 } }))
                        .multilineTextAlignment(.trailing)
                        .frame(width: 56)
                        .onSubmit {
                            guard index < thickness.count, let v = Double(thickness[index].replacingOccurrences(of: ",", with: ".")) else { return }
                            set(index) { $0.thickness = $1 }(v / 100)
                        }
                    Text("cm").foregroundStyle(.secondary)
                }
                HStack {
                    Picker("Funktion", selection: Binding(get: { Int(layer.function) }, set: { set(index) { $0.function = UInt32($1) }($0) })) {
                        ForEach(Array(AppModel.functions.enumerated()), id: \.offset) { Text($0.element).tag($0.offset) }
                    }
                    .labelsHidden()
                    Spacer()
                    Button { move(index, by: -1) } label: { Image(systemName: "arrow.up") }
                        .disabled(index == 0)
                    Button { move(index, by: 1) } label: { Image(systemName: "arrow.down") }
                        .disabled(index == info.layers.count - 1)
                    Button { remove(index) } label: { Image(systemName: "minus.circle") }
                        .disabled(info.layers.count == 1)
                }
                .buttonStyle(.borderless)
            }
        }
        Button("Schicht hinzufügen") {
            var layers = info.layers
            layers.append(LayerInfo(material: "putz", thickness: 0.015, function: 4))
            model.setTypeLayers(info, layers: layers)
        }
        Text("\(info.kind == 0 ? "Von außen (linke Seite in Zeichenrichtung) nach innen" : "Von oben nach unten"). Dicke \(cm(info.layers.reduce(0) { $0 + $1.thickness })) cm.")
            .font(.caption)
            .foregroundStyle(.secondary)
    }

    private func load() {
        name = info.name
        thickness = info.layers.map { cm($0.thickness) }
    }

    private func cm(_ m: Double) -> String {
        (m * 100).formatted(.number.precision(.fractionLength(0...1)).locale(Locale(identifier: "de_DE")))
    }

    private func set<T>(_ index: Int, _ change: @escaping (inout LayerInfo, T) -> Void) -> (T) -> Void {
        { value in
            var layers = info.layers
            change(&layers[index], value)
            model.setTypeLayers(info, layers: layers)
        }
    }

    private func move(_ index: Int, by delta: Int) {
        var layers = info.layers
        layers.swapAt(index, index + delta)
        model.setTypeLayers(info, layers: layers)
    }

    private func remove(_ index: Int) {
        var layers = info.layers
        layers.remove(at: index)
        model.setTypeLayers(info, layers: layers)
    }
}

/// Ein Wert am Typ: direkt änderbar, wirkt auf alle Bauteile, die ihn übernehmen;
/// das Schloss legt ihn fest, sodass kein Bauteil abweichen darf.
private struct TypeValueRow: View {
    @Environment(AppModel.self) private var model
    let type: TypeInfo
    let property: PropertyInfo
    @State private var text = ""

    var body: some View {
        LabeledContent(property.title) {
            HStack(spacing: 4) {
                TextField(property.isSize ? "frei" : "", text: $text)
                    .multilineTextAlignment(.trailing)
                    .frame(width: 110)
                    .onSubmit(commit)
                Button {
                    model.run("SetTypeLocked(\(type.element),\(property.property),\(!property.locked))")
                } label: {
                    Image(systemName: property.locked ? "lock.fill" : "lock.open")
                }
                .buttonStyle(.borderless)
                .help(property.locked ? "Im Typ festgelegt: Bauteile dürfen nicht abweichen" : "Bauteile dürfen abweichen")
            }
        }
        .onAppear { text = shown }
        .onChange(of: property.value) { text = shown }
    }

    /// Maße ohne „m“, damit sie sich tippen lassen.
    private var shown: String {
        property.isSize ? property.value.replacingOccurrences(of: " m", with: "") : property.value
    }

    private func commit() {
        guard text != shown else { return }
        if property.isSize {
            let t = text.trimmingCharacters(in: .whitespaces).replacingOccurrences(of: ",", with: ".")
            let value = t.isEmpty ? "None" : (Double(t).map { "Some(\($0))" } ?? "")
            guard !value.isEmpty else { text = shown; return }
            model.run("SetTypeSize(\(type.element),\(property.property),\(value))")
        } else {
            model.run("SetTypeProperty(\(type.element),\(property.property),\(ron(text)))")
        }
    }
}

/// Werte, die der Typ vorgibt, am gewählten Bauteil: vom Typ oder abweichend,
/// mit Zurücksetzen und „in den Typ übernehmen“ (ADR 0048).
private struct ElementPropertiesSection: View {
    @Environment(AppModel.self) private var model

    var body: some View {
        Section("Werte dieses Bauteils") {
            ForEach(model.properties.map { ("bauteil-" + $0.property, $0) }, id: \.0) { ElementValueRow(property: $0.1) }
            Text("Grau: vom Typ. Orange: weicht ab. Ein Wert gleich dem Typ gilt wieder vom Typ.")
                .font(.caption)
                .foregroundStyle(.secondary)
        }
    }
}

private struct ElementValueRow: View {
    @Environment(AppModel.self) private var model
    let property: PropertyInfo
    @State private var text = ""

    var body: some View {
        LabeledContent(property.title) {
            HStack(spacing: 4) {
                if property.isSize || property.locked {
                    Text(property.value.isEmpty ? "keins" : property.value)
                        .foregroundStyle(property.fromType ? Color.secondary : Color.orange)
                    if property.locked {
                        Image(systemName: "lock.fill").foregroundStyle(.secondary)
                            .help("Im Typ festgelegt")
                    }
                } else {
                    TextField(property.typeValue.isEmpty ? "keins" : property.typeValue, text: $text)
                        .multilineTextAlignment(.trailing)
                        .foregroundStyle(property.fromType ? Color.secondary : Color.orange)
                        .frame(width: 110)
                        .onSubmit {
                            model.run("SetElementProperty(\(property.property),\(ron(text)))")
                        }
                }
                if !property.fromType {
                    Button { model.run("ResetToType(\(property.property))") } label: {
                        Image(systemName: "arrow.uturn.backward")
                    }
                    .help("Zurücksetzen: Wert vom Typ übernehmen")
                    Button { model.run("ApplyToType(\(property.property))") } label: {
                        Image(systemName: "arrow.up.doc")
                    }
                    .help("In den Typ übernehmen: gilt dann für alle")
                }
            }
            .buttonStyle(.borderless)
        }
        .onAppear { text = property.fromType ? "" : property.value }
        .onChange(of: property.value) { text = property.fromType ? "" : property.value }
    }
}

/// Ein Anschluss: wie jede Wand anschließt, welche Schicht welche schneidet,
/// und Ausnahmen je Wandende (ADR 0045).
private struct JunctionSection: View {
    @Environment(AppModel.self) private var model
    let junction: JunctionInfo

    var body: some View {
        Section("Anschluss · \(junction.title)") {
            ForEach(junction.lines, id: \.self) { Text($0) }
            if junction.lines.isEmpty {
                Text("Keine Schichten berühren sich.").foregroundStyle(.secondary)
            }
        }
        ForEach(junction.arms, id: \.wall) { arm in
            Section(arm.label) {
                LabeledContent("Anschluss", value: arm.role)
                if arm.commands.count == 3 {
                    Picker("Ausnahme", selection: Binding(get: { Int(arm.rule) }, set: { model.run(arm.commands[$0]) })) {
                        Text("keine").tag(0)
                        Text("läuft durch").tag(1)
                        Text("nicht verbinden").tag(2)
                    }
                }
            }
        }
    }
}

/// Füllung und die zweite Brüstungshöhe einer Öffnung (ADR 0047): Die fertige
/// BRH über OKFF ist führend, die Rohbaubrüstung über Rohdecke ergibt sich aus
/// Fußbodenaufbau und unterem Anschluss und lässt sich ebenso tippen.
private struct OpeningRows: View {
    @Environment(AppModel.self) private var model
    let opening: OpeningInfo
    @State private var rough = ""
    @State private var connection = ""
    @State private var recess = ""

    var body: some View {
        if opening.typeName.isEmpty {
            let types = model.types.filter { $0.kind == 2 }
            if !types.isEmpty {
                Picker("Typ", selection: Binding(get: { UInt64(0) }, set: { model.run("AssignType(Some(\($0)))") })) {
                    Text("keiner").tag(UInt64(0))
                    ForEach(types, id: \.element) { Text($0.name).tag($0.element) }
                }
            }
            Button("Typ aus dieser Öffnung anlegen") { model.run("CreateOpeningTypes") }
                .buttonStyle(.link)
        }
        Picker("Füllung", selection: Binding(get: { Int(opening.fill) }, set: { model.run(AppModel.fills[$0].command) })) {
            ForEach(Array(AppModel.fills.enumerated()), id: \.offset) { index, item in
                Text(item.title).tag(index)
            }
        }
        if let depth = opening.recessDepth {
            LabeledContent("Nischentiefe") {
                HStack(spacing: 4) {
                    TextField("", text: $recess)
                        .multilineTextAlignment(.trailing)
                        .frame(width: 64)
                        .onSubmit {
                            if let v = number(recess) { model.run("SetRecess(\(opening.recessLeft ? "Left" : "Right"),\(v))") }
                        }
                    Text("m").foregroundStyle(.secondary)
                }
            }
            .onAppear { recess = german(depth) }
            .onChange(of: opening.recessDepth) { recess = german(opening.recessDepth ?? 0) }
            Picker("Nische auf der", selection: Binding(get: { opening.recessLeft }, set: { left in
                model.run("SetRecess(\(left ? "Left" : "Right"),\(depth))")
            })) {
                Text("linken Wandseite").tag(true)
                Text("rechten Wandseite").tag(false)
            }
        }
        if opening.roughSill != nil {
            LabeledContent("Rohbaubrüstung über Rohdecke") {
                HStack(spacing: 4) {
                    TextField("", text: $rough)
                        .multilineTextAlignment(.trailing)
                        .frame(width: 64)
                        .onSubmit {
                            if let v = number(rough) { model.setRoughSill(v, for: opening.element) }
                        }
                    Text("m").foregroundStyle(.secondary)
                }
            }
        }
        LabeledContent("Fußbodenaufbau", value: opening.floorBuildUp.map { german($0) + " m" } ?? "keine Decke davor")
        LabeledContent("Unterer Anschluss") {
            HStack(spacing: 4) {
                TextField("", text: $connection)
                    .multilineTextAlignment(.trailing)
                    .frame(width: 64)
                    .onSubmit {
                        if let v = number(connection) { model.run("SetSillConnection(\(v))") }
                    }
                Text("m").foregroundStyle(.secondary)
            }
        }
        Text(opening.typeName.isEmpty
             ? "Der untere Anschluss gehört zum Typ; ohne Typ legt Lot einen an."
             : "Typ \(opening.typeName), wirkt auf \(opening.typeUses) Öffnung\(opening.typeUses == 1 ? "" : "en")")
            .font(.caption)
            .foregroundStyle(.secondary)
            .onAppear(perform: load)
            .onChange(of: opening.roughSill) { load() }
            .onChange(of: opening.sillConnection) { load() }
    }

    private func load() {
        rough = opening.roughSill.map(german) ?? ""
        connection = german(opening.sillConnection)
    }

    private func number(_ text: String) -> Double? {
        Double(text.trimmingCharacters(in: .whitespaces).replacingOccurrences(of: ",", with: "."))
    }

    private func german(_ v: Double) -> String {
        v.formatted(.number.precision(.fractionLength(2)).locale(Locale(identifier: "de_DE")))
    }
}

/// Anschlag und Aufschlag einer Tür. Die Knöpfe wirken auf alle gewählten Türen.
private struct DoorRows: View {
    @Environment(AppModel.self) private var model
    let door: DoorInfo

    var body: some View {
        LabeledContent("Anschlag") {
            HStack {
                Text(door.dinLeft ? "DIN links" : "DIN rechts")
                Button("Wechseln") { model.run("FlipDoorHinge") }
            }
        }
        LabeledContent("Aufschlag") {
            HStack {
                Text(door.opensLeft ? "zur linken Wandseite" : "zur rechten Wandseite")
                Button("Wechseln") { model.run("FlipDoorSwing") }
            }
        }
    }
}

/// Gruppen einer Mehrfachauswahl mit Anzahl; „nur diese“ wählt eine Gruppe allein, „ohne“ nimmt sie heraus.
private struct SelectionGroupsSection: View {
    @Environment(AppModel.self) private var model

    var body: some View {
        let all = model.groups.flatMap(\.ids)
        Section("Auswahl · \(all.count) Bauteile") {
            ForEach(model.groups, id: \.label) { group in
                HStack {
                    Text(group.label)
                    Spacer()
                    Text("\(group.ids.count)").monospacedDigit().foregroundStyle(.secondary)
                    Button("nur diese") { model.select(group.ids) }
                        .buttonStyle(.borderless)
                    Button("ohne") { model.select(all.filter { !group.ids.contains($0) }) }
                        .buttonStyle(.borderless)
                }
            }
        }
    }
}

/// Eine Eigenschaft für mehrere Bauteile: gleicher Wert oder „verschieden“
/// mit Spanne. Eine Zahl setzt alle, „+0,10“ oder „-0,05“ ändert alle relativ.
private struct MultiParameterRow: View {
    @Environment(AppModel.self) private var model
    let kind: ParamKind
    let values: [ParamValue]
    @State private var text = ""
    @FocusState private var focused: Bool

    private var low: Double { values.map(\.value).min() ?? 0 }
    private var high: Double { values.map(\.value).max() ?? 0 }
    private var same: Bool { high - low < 0.0005 }

    var body: some View {
        LabeledContent(kind.label) {
            HStack(spacing: 4) {
                TextField(same ? "" : "verschieden", text: $text)
                    .multilineTextAlignment(.trailing)
                    .frame(width: 72)
                    .focused($focused)
                    .onSubmit(commit)
                Text(kind == .roofSlope ? "°" : "m").foregroundStyle(.secondary)
            }
        }
        .help(same ? "Gilt für alle \(values.count)" : "\(german(low)) bis \(german(high)); eine Zahl setzt alle, +0,10 ändert alle")
        .onAppear(perform: load)
        .onChange(of: values.map(\.value)) { load() }
        .onChange(of: focused) { if !focused { commit() } }
        if !same {
            Text("\(german(low)) bis \(german(high))")
                .font(.caption)
                .foregroundStyle(.secondary)
        }
    }

    private func load() {
        text = same ? german(low) : ""
    }

    private func commit() {
        let t = text.trimmingCharacters(in: .whitespaces).replacingOccurrences(of: ",", with: ".")
        guard !t.isEmpty, !(same && t == german(low).replacingOccurrences(of: ",", with: ".")) else { return }
        let relative = t.hasPrefix("+") || (t.hasPrefix("-") && !same)
        guard let number = Double(relative ? String(t.dropFirst()) : t) else { load(); return }
        let delta = t.hasPrefix("-") ? -number : number
        let new = values.map { relative ? $0.value + delta : number }
        model.set(new, for: values.map(\.element), kind: kind)
    }

    private func german(_ v: Double) -> String {
        v.formatted(.number.precision(.fractionLength(2)).locale(Locale(identifier: "de_DE")))
    }
}

private struct ParameterRow: View {
    @Environment(AppModel.self) private var model
    let parameter: ParamValue

    var body: some View {
        let value = Binding(
            get: { parameter.value },
            set: { model.set($0, for: parameter) }
        )
        LabeledContent(parameter.kind.label) {
            HStack(spacing: 4) {
                TextField("", value: value, format: .number.precision(.fractionLength(2)))
                    .multilineTextAlignment(.trailing)
                    .frame(width: 64)
                Text(parameter.kind == .roofSlope ? "°" : "m").foregroundStyle(.secondary)
                Stepper("", value: value, step: parameter.kind == .roofSlope ? 1 : 0.05)
                    .labelsHidden()
            }
        }
    }
}

extension ParamKind {
    var label: String {
        switch self {
        case .wallLength: "Länge"
        case .wallSagitta: "Stich (Bogen)"
        case .wallHeight: "Höhe"
        case .wallTopOffset: "Versatz Oberkante"
        case .baseOffset: "Unterkante über OKFF"
        case .openingOffset: "Abstand vom Wandanfang"
        case .openingWidth: "Breite (Rohbau)"
        case .openingHeight: "Höhe (Rohbau)"
        case .openingSillHeight: "BRH über OKFF (fertig)"
        case .storeyElevation: "OKFF über ±0,00"
        case .storeyHeight: "Geschosshöhe"
        case .columnWidth: "Breite"
        case .columnDepth: "Tiefe"
        case .roofEaveHeight: "Traufhöhe über OKFF"
        case .roofSlope: "Neigung (Grad)"
        }
    }
}

/// Gewählter Schnitt oder gewählte Ansicht (ADR 0049).
private struct SectionSection: View {
    @Environment(AppModel.self) private var model
    let info: SectionInfo
    @State private var name = ""
    @State private var mark = ""

    var body: some View {
        Section(info.elevation ? "Ansicht" : "Schnitt") {
            TextField("Name", text: $name).onSubmit(save)
            if !info.elevation {
                TextField("Kennbuchstabe", text: $mark).onSubmit(save)
            }
            LabeledContent("Blicktiefe") {
                HStack(spacing: 4) {
                    TextField("", value: Binding(get: { info.depth }, set: { model.run("SetSectionDepth(\($0))") }),
                              format: .number.precision(.fractionLength(2)))
                        .multilineTextAlignment(.trailing)
                        .frame(width: 64)
                    Text("m").foregroundStyle(.secondary)
                }
            }
            if !info.sheets.isEmpty {
                LabeledContent("Blätter", value: info.sheets)
            }
            HStack {
                Button("Öffnen") { model.go("Section(\(info.element))") }
                Button("Rechts daneben") { model.openBeside("Section(\(info.element))") }
                Button("Blick umdrehen") { model.run("FlipSection") }
            }
        }
        .onAppear { name = info.name; mark = info.mark }
        .onChange(of: info) { name = info.name; mark = info.mark }
    }

    private func save() {
        model.run("SetSectionInfo(\(info.element),\(ron(name)),\(ron(mark)))")
    }
}

/// Das gezeigte Planblatt: Plannummer, Planinhalt, Format.
private struct SheetSection: View {
    @Environment(AppModel.self) private var model
    let info: SheetInfo
    @State private var number = ""
    @State private var title = ""

    var body: some View {
        Section("Planblatt") {
            TextField("Plannummer", text: $number).onSubmit { save(info.format) }
            TextField("Planinhalt", text: $title).onSubmit { save(info.format) }
            Picker("Format", selection: Binding(get: { info.format }, set: { save($0) })) {
                ForEach(AppModel.formats, id: \.self) { Text("\($0) quer").tag($0) }
            }
            Text("Ansichten aus der Seitenleiste aufs Blatt ziehen. Ziehen verschiebt, Doppelklick öffnet, ⌫ nimmt vom Blatt.")
                .font(.caption)
                .foregroundStyle(.secondary)
        }
        .onAppear { number = info.number; title = info.title }
        .onChange(of: info) { number = info.number; title = info.title }
    }

    private func save(_ format: String) {
        model.run("SetSheetInfo(\(info.element),\(ron(number)),\(ron(title)),\(ron(format)))")
    }
}

/// Die gewählte Ansicht auf dem Blatt: Maßstab, Heimat, öffnen, wegnehmen.
private struct PlacedSection: View {
    @Environment(AppModel.self) private var model
    let info: PlacedInfo

    private var scales: [Double] {
        let common: [Double] = [20, 50, 100, 200, 500, 1000]
        return common.contains(info.scale) ? common : (common + [info.scale]).sorted()
    }

    var body: some View {
        Section("Ansicht auf dem Blatt") {
            LabeledContent("Ansicht", value: info.title)
            Picker("Maßstab", selection: Binding(
                get: { info.scale },
                set: { model.run("SetPlacedViewScale(\(info.sheet),\(info.index),\($0))") }
            )) {
                ForEach(scales, id: \.self) { Text("1:\(Int($0))").tag($0) }
            }
            if info.canBeHome {
                Toggle("Heimat: Schnittmarken verweisen auf dieses Blatt", isOn: Binding(
                    get: { info.home },
                    set: { if $0 { model.run("SetHome(\(info.view.dropFirst("Section(".count).dropLast()),\(info.sheet))") } }
                ))
                .disabled(info.home)
            }
            HStack {
                Button("Öffnen") { model.go(info.view) }
                Button("Vom Blatt nehmen") { model.run("RemovePlacedView(\(info.sheet),\(info.index))") }
            }
        }
    }
}
