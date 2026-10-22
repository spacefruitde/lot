import SwiftUI

/// Parameter von Wand und Fenster. Jede Änderung ist eine Operation im Kern
/// und lässt sich widerrufen.
struct InspectorView: View {
    @Environment(AppModel.self) private var model

    var body: some View {
        Form {
            Section("Wand") {
                ForEach(model.parameters.filter { $0.kind.isWall }, id: \.kind) { ParameterRow(parameter: $0) }
            }
            Section("Fenster") {
                ForEach(model.parameters.filter { !$0.kind.isWall }, id: \.kind) { ParameterRow(parameter: $0) }
            }
            Section("Bedienung") {
                VStack(alignment: .leading, spacing: 4) {
                    Text("Zwei Finger ziehen: drehen")
                    Text("Zwei Finger mit ⇧: verschieben")
                    Text("Zwei Finger spreizen: zoomen")
                    Text("Maus: ziehen dreht, mit ⌥ verschieben, Rad zoomt")
                }
                .font(.callout)
                .foregroundStyle(.secondary)
            }
        }
        .formStyle(.grouped)
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
    var isWall: Bool {
        switch self {
        case .wallLength, .wallThickness, .wallHeight: true
        default: false
        }
    }

    var label: String {
        switch self {
        case .wallLength: "Länge"
        case .wallThickness: "Dicke"
        case .wallHeight: "Höhe"
        case .openingOffset: "Abstand vom Wandanfang"
        case .openingWidth: "Breite"
        case .openingHeight: "Höhe"
        case .openingSillHeight: "Brüstungshöhe"
        }
    }
}
