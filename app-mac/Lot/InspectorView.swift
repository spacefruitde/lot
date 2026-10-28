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
            if let junction = model.junction {
                JunctionSection(junction: junction)
            } else if model.parameters.isEmpty && model.spaces.isEmpty && model.texts.isEmpty {
                Section {
                    Text("Nichts gewählt. Eine Wand oder einen Raumstempel anklicken zeigt die Eigenschaften.")
                        .foregroundStyle(.secondary)
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
            if let state = model.viewState {
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
