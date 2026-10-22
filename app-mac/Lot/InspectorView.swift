import SwiftUI

/// Eigenschaften der gewählten Elemente. Jede Änderung ist eine Operation im
/// Kern und lässt sich widerrufen.
struct InspectorView: View {
    @Environment(AppModel.self) private var model

    var body: some View {
        Form {
            if model.parameters.isEmpty {
                Section {
                    Text("Nichts gewählt. Eine Wand anklicken zeigt ihre Maße.")
                        .foregroundStyle(.secondary)
                }
            } else {
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
                    Text("W: Wand, A: Achse, S: Stütze, V: Auswählen, ⌘K: Befehle")
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
