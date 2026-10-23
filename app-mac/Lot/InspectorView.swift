import SwiftUI

/// Eigenschaften der gewählten Elemente. Jede Änderung ist eine Operation im
/// Kern und lässt sich widerrufen.
struct InspectorView: View {
    @Environment(AppModel.self) private var model

    var body: some View {
        Form {
            if model.parameters.isEmpty && model.spaces.isEmpty {
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
                ForEach(groups, id: \.0) { element, parameters in
                    Section(model.label(for: element)) {
                        ForEach(parameters, id: \.kind) { ParameterRow(parameter: $0) }
                        if let door = model.doors.first(where: { $0.element == element }) {
                            DoorRows(door: door)
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
                    Text("W: Wand, B: Bogenwand, T: Tür, F: Fenster, R: Raum, A: Achse, S: Stütze, M: Bemaßen, D: Deckendurchbruch, V: Auswählen, ⌘K: Befehle")
                    Text("Zeichnen: klicken, Länge tippen und ⏎, ⇧ rechtwinklig")
                    Text("Grundriss: zwei Finger verschieben, spreizen zoomt")
                    Text("3D: zwei Finger drehen, mit ⇧ verschieben")
                }
                .font(.callout)
                .foregroundStyle(.secondary)
            }
        }
        .formStyle(.grouped)
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
                Text("m").foregroundStyle(.secondary)
                Stepper("", value: value, step: 0.05)
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
        case .wallTopOffset: "Oberkante über Geschoss"
        case .baseOffset: "Unterkante über OKFF"
        case .openingOffset: "Abstand vom Wandanfang"
        case .openingWidth: "Breite"
        case .openingHeight: "Höhe"
        case .openingSillHeight: "Brüstungshöhe"
        case .storeyElevation: "OKFF über ±0,00"
        case .storeyHeight: "Geschosshöhe"
        case .columnWidth: "Breite"
        case .columnDepth: "Tiefe"
        }
    }
}
