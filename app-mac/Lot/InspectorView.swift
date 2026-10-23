import SwiftUI

/// Eigenschaften der gewählten Elemente. Jede Änderung ist eine Operation im
/// Kern und lässt sich widerrufen.
struct InspectorView: View {
    @Environment(AppModel.self) private var model

    var body: some View {
        Form {
            if model.parameters.isEmpty && model.rooms.isEmpty {
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
                ForEach(model.rooms, id: \.element) { RoomSection(room: $0) }
                ForEach(groups, id: \.0) { element, parameters in
                    Section(model.label(for: element)) {
                        ForEach(parameters, id: \.kind) { ParameterRow(parameter: $0) }
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
                    Text("W: Wand, B: Bogenwand, A: Achse, S: Stütze, R: Raum, V: Auswählen, ⌘K: Befehle")
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
private struct RoomSection: View {
    @Environment(AppModel.self) private var model
    let room: RoomInfo
    @State private var number = ""
    @State private var name = ""
    @FocusState private var focused: Bool

    var body: some View {
        Section(model.label(for: room.element)) {
            TextField("Nummer", text: $number)
                .focused($focused)
                .onSubmit(commit)
            TextField("Name", text: $name)
                .focused($focused)
                .onSubmit(commit)
            Picker("Nutzung", selection: Binding(get: { room.usage }, set: { model.setRoom(room, number: number, name: name, usage: $0) })) {
                ForEach(Array(AppModel.usages.enumerated()), id: \.offset) { index, usage in
                    Text("\(usage.code) \(usage.title)").tag(UInt32(index))
                }
            }
            Toggle(
                "Zählt zur Wohnfläche",
                isOn: Binding(get: { room.livingSpace }, set: { model.setRoom(room, number: number, name: name, livingSpace: $0) })
            )
            LabeledContent("Netto-Raumfläche", value: room.netArea.map(area) ?? "offen")
            if let living = room.livingArea {
                LabeledContent("Wohnfläche (WoFlV)", value: area(living))
            }
        }
        .onAppear(perform: load)
        .onChange(of: room.element) { load() }
        .onChange(of: room.number) { number = room.number }
        .onChange(of: room.name) { name = room.name }
        .onChange(of: focused) { if !focused { commit() } }
    }

    private func load() {
        number = room.number
        name = room.name
    }

    private func commit() {
        guard number != room.number || name != room.name else { return }
        model.setRoom(room, number: number, name: name)
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
