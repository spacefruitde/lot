import AppKit
import PDFKit
import UniformTypeIdentifiers

/// Unterlagen (6.05): alte Pläne als PDF oder Bild unter dem Grundriss. Die
/// Datei liegt im Projekt; PDF-Seiten und Bilder rendert die App mit den
/// Mitteln des Systems und gibt die Bildpunkte an den Kern (ADR 0016).
extension AppModel {
    /// Auflösung, mit der PDF-Seiten für die Anzeige gerendert werden.
    private static let underlayDPI: CGFloat = 150

    /// Fragt nach einer Datei und legt sie als Unterlage ins aktuelle Geschoss.
    /// Danach ist „Unterlage kalibrieren“ gewählt.
    func importUnderlayWithPanel() {
        let panel = NSOpenPanel()
        let cad = ["dxf", "dwg"].compactMap { UTType(filenameExtension: $0) }
        panel.allowedContentTypes = [.pdf, .png, .jpeg, .tiff] + cad
        panel.message = "Bestandsplan als PDF, Bild, DXF oder DWG wählen"
        guard panel.runModal() == .OK, let url = panel.url else { return }
        do {
            let data = try Data(contentsOf: url)
            let name = url.lastPathComponent
            if ["dxf", "dwg"].contains(url.pathExtension.lowercased()) {
                // Linien liest der Kern selbst; Maßstab aus den Einheiten der Datei.
                document.addAttachment(name: name, data: data)
                guard let viewport else { return }
                _ = try viewport.addCadUnderlay(source: name)
                refreshViews()
                loadMissingUnderlays()
                return
            }
            guard let size = Self.underlaySize(data: data, name: name) else {
                errorMessage = "\(name) lässt sich nicht als Unterlage lesen"
                return
            }
            document.addAttachment(name: name, data: data)
            guard let viewport else { return }
            _ = try viewport.addUnderlay(source: name, page: 0, width: size.width, height: size.height, scale: size.scale)
            loadMissingUnderlays()
            run("CalibrateUnderlayTool")
        } catch {
            errorMessage = Self.message(for: error)
        }
    }

    /// Fragt nach einer IFC-Datei und legt sie als Referenzmodell ins Projekt (6.02).
    func importReferenceWithPanel() {
        let panel = NSOpenPanel()
        panel.allowedContentTypes = [UTType(filenameExtension: "ifc")].compactMap { $0 }
        panel.message = "IFC-Modell eines Fachplaners wählen"
        guard panel.runModal() == .OK, let url = panel.url else { return }
        do {
            let data = try Data(contentsOf: url)
            document.addAttachment(name: url.lastPathComponent, data: data)
            guard let viewport else { return }
            _ = try viewport.addReferenceModel(source: url.lastPathComponent)
            refreshViews()
            loadMissingUnderlays()
        } catch {
            errorMessage = Self.message(for: error)
        }
    }

    /// Gibt dem Kern die Bilder aller Unterlagen, die er nicht selbst lesen kann.
    func loadMissingUnderlays() {
        for viewport in viewports {
            for ref in viewport.missingUnderlayImages() {
                guard let data = document.attachment(name: ref.source),
                      let image = Self.rasterize(data: data, name: ref.source, page: Int(ref.page))
                else { continue }
                viewport.setUnderlayImage(
                    source: ref.source, page: ref.page, width: UInt32(image.width), height: UInt32(image.height), rgba: image.rgba
                )
            }
        }
        requestRedraw?()
    }

    /// Größe in Einheiten der Vorlage und geschätzter Maßstab (Meter je Einheit),
    /// bis kalibriert ist: PDF in Punkten als 1:100, Bilder als 150 dpi in 1:100.
    private static func underlaySize(data: Data, name: String) -> (width: Double, height: Double, scale: Double)? {
        if name.lowercased().hasSuffix(".pdf") {
            guard let page = PDFDocument(data: data)?.page(at: 0) else { return nil }
            let box = page.bounds(for: .mediaBox)
            return (Double(box.width), Double(box.height), 0.0254 / 72 * 100)
        }
        guard let source = CGImageSourceCreateWithData(data as CFData, nil),
              let image = CGImageSourceCreateImageAtIndex(source, 0, nil)
        else { return nil }
        return (Double(image.width), Double(image.height), 0.0254 / 150 * 100)
    }

    /// Bildpunkte RGBA, Zeilen von oben, auf weißem Grund.
    private static func rasterize(data: Data, name: String, page: Int) -> (width: Int, height: Int, rgba: Data)? {
        var draw: ((CGContext, Int, Int) -> Void)?
        var size = (0, 0)
        if name.lowercased().hasSuffix(".pdf") {
            guard let pdfPage = PDFDocument(data: data)?.page(at: page) else { return nil }
            let box = pdfPage.bounds(for: .mediaBox)
            let scale = underlayDPI / 72
            size = (Int(box.width * scale), Int(box.height * scale))
            draw = { context, _, _ in
                context.scaleBy(x: scale, y: scale)
                context.translateBy(x: -box.origin.x, y: -box.origin.y)
                pdfPage.draw(with: .mediaBox, to: context)
            }
        } else {
            guard let source = CGImageSourceCreateWithData(data as CFData, nil),
                  let image = CGImageSourceCreateImageAtIndex(source, 0, nil)
            else { return nil }
            size = (image.width, image.height)
            draw = { context, w, h in context.draw(image, in: CGRect(x: 0, y: 0, width: w, height: h)) }
        }
        let (w, h) = size
        guard w > 0, h > 0, let draw,
              let context = CGContext(
                  data: nil, width: w, height: h, bitsPerComponent: 8, bytesPerRow: w * 4,
                  space: CGColorSpace(name: CGColorSpace.sRGB)!,
                  bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue
              )
        else { return nil }
        context.setFillColor(CGColor(red: 1, green: 1, blue: 1, alpha: 1))
        context.fill(CGRect(x: 0, y: 0, width: w, height: h))
        context.saveGState()
        draw(context, w, h)
        context.restoreGState()
        guard let pointer = context.data else { return nil }
        // CGContext legt die erste Zeile oben ab, wie der Kern es erwartet.
        return (w, h, Data(bytes: pointer, count: w * h * 4))
    }
}
