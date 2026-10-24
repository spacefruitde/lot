import AppKit
import SwiftUI

/// Prüfbericht und Hinweise im Grundriss (PLAN.md, Abschnitt 4: „Ergebnisse
/// erscheinen als Hinweise direkt im Modell und als Prüfbericht“).
extension AppModel {
    /// Rechnet den Prüfbericht neu und zeigt ihn.
    func runChecks() {
        findings = document.checkReport()
        showReport = true
    }

    func toggleFindings() {
        showFindings.toggle()
        viewport?.setShowFindings(show: showFindings)
        requestRedraw?()
    }

    /// Springt im Grundriss zum Ort eines Befunds.
    func focus(_ finding: FindingInfo) {
        guard let storey = finding.storey, let viewport else { return }
        setView(.plan)
        if !showFindings {
            showFindings = true
            viewport.setShowFindings(show: true)
        }
        viewport.focus(storey: storey, x: finding.x, y: finding.y)
        requestRedraw?()
    }
}

/// Bürostandard (2.08): Datei mit den Abweichungen des Büros vom eingebauten Standard.
enum OfficeStandardFile {
    static var url: URL {
        let support = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask)[0]
        return support.appendingPathComponent("Lot", isDirectory: true).appendingPathComponent("buerostandard.ron")
    }

    /// Setzt den Bürostandard des Büros beim Start, falls die Datei da ist.
    /// Gibt eine Fehlermeldung zurück, wenn sie nicht lesbar ist.
    static func install() -> String? {
        guard let text = try? String(contentsOf: url, encoding: .utf8) else { return nil }
        do {
            try installOfficeStandard(text: text)
            return nil
        } catch let LotError.Failed(message) {
            return message
        } catch {
            return error.localizedDescription
        }
    }

    /// Öffnet die Datei im Texteditor; legt sie mit einer Vorlage an, falls sie fehlt.
    static func edit() {
        let fm = FileManager.default
        if !fm.fileExists(atPath: url.path) {
            try? fm.createDirectory(at: url.deletingLastPathComponent(), withIntermediateDirectories: true)
            try? template.write(to: url, atomically: true, encoding: .utf8)
        }
        NSWorkspace.shared.open(url)
    }

    static let template = """
    // Bürostandard (2.08): nur, was vom eingebauten Standard von Lot abweicht.
    // Änderungen gelten nach dem nächsten Start von Lot.
    (
        office: Some((
            name: "Name des Büros",
            address: "Straße 1, 12345 Ort",
            contact: "Kammer-Nr., Telefon oder E-Mail",
        )),
        // Unterschriftsfelder für Bauherr und Entwurfsverfasser im Plankopf.
        sheets: Some((signatures: true)),
        // Eigene Layernamen für DXF und DWG: Name in Lot, Name im Büro.
        layers: {
            "A_Bemassung": "A_Bemassung",
        },
    )

    """
}

/// Der Prüfbericht als Liste, nach Abschnitten. Ein Klick auf einen Befund mit
/// Ort zeigt ihn im Grundriss.
struct CheckReportView: View {
    @Environment(AppModel.self) private var model

    private var sections: [String] {
        var seen: [String] = []
        for f in model.findings where !seen.contains(f.section) {
            seen.append(f.section)
        }
        return seen
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            HStack {
                Text("Prüfbericht").font(.title2)
                Spacer()
                Text(summary).foregroundStyle(.secondary)
            }
            .padding()
            List {
                ForEach(sections, id: \.self) { section in
                    Section(section) {
                        ForEach(Array(model.findings.enumerated()).filter { $0.element.section == section }, id: \.offset) { _, f in
                            FindingRow(finding: f)
                                .contentShape(Rectangle())
                                .onTapGesture { model.focus(f) }
                        }
                    }
                }
            }
            HStack {
                Text("Automatische Prüfung, ersetzt keine Prüfung durch Fachleute.")
                    .font(.caption)
                    .foregroundStyle(.secondary)
                Spacer()
                Button("Neu prüfen") { model.runChecks() }
                Button("Schließen") { model.showReport = false }
                    .keyboardShortcut(.defaultAction)
            }
            .padding()
        }
        .frame(minWidth: 640, minHeight: 480)
    }

    private var summary: String {
        let ok = model.findings.filter { $0.status == 0 }.count
        let notes = model.findings.filter { $0.status == 1 }.count
        let fails = model.findings.filter { $0.status == 2 }.count
        return "\(ok) erfüllt · \(notes) Hinweise · \(fails) nicht erfüllt"
    }
}

private struct FindingRow: View {
    let finding: FindingInfo

    var body: some View {
        HStack(alignment: .firstTextBaseline, spacing: 8) {
            Circle()
                .fill(color)
                .frame(width: 9, height: 9)
                .accessibilityLabel(label)
            VStack(alignment: .leading, spacing: 2) {
                Text(finding.text)
                if !finding.source.isEmpty {
                    Text(finding.source).font(.caption).foregroundStyle(.secondary)
                }
            }
            Spacer()
            if finding.storey != nil {
                Image(systemName: "scope").foregroundStyle(.secondary).help("Im Grundriss zeigen")
            }
        }
        .padding(.vertical, 2)
    }

    private var color: Color {
        switch finding.status {
        case 0: .green
        case 1: .orange
        default: .red
        }
    }

    private var label: String {
        switch finding.status {
        case 0: "erfüllt"
        case 1: "Hinweis"
        default: "nicht erfüllt"
        }
    }
}
