import AppKit
import QuartzCore
import SwiftUI

/// 3D-Ansicht: ein NSView mit CAMetalLayer, in den der Rust-Kern über wgpu zeichnet.
struct ViewportView: NSViewRepresentable {
    let document: LotDocument
    let revision: Int
    let fitRequest: Int
    let onStats: (ViewportStatsDisplay) -> Void

    func makeNSView(context: Context) -> MetalViewportNSView {
        let view = MetalViewportNSView(frame: .zero)
        view.onStats = onStats
        return view
    }

    func updateNSView(_ view: MetalViewportNSView, context: Context) {
        view.onStats = onStats
        view.show(document: document, revision: revision)
        view.fit(request: fitRequest)
    }

    static func dismantleNSView(_ view: MetalViewportNSView, coordinator: ()) {
        view.stop()
    }
}

final class MetalViewportNSView: NSView {
    var onStats: ((ViewportStatsDisplay) -> Void)?

    private var viewport: Viewport?
    private var displayLink: CADisplayLink?
    private var needsRender = true

    private var document: LotDocument?
    private var shownDocument: LotDocument?
    private var shownRevision = -1
    private var handledFitRequest = 0
    private var pendingFit = true

    private var stats = ViewportStatsDisplay()
    private var lastStatsUpdate: CFTimeInterval = 0

    override init(frame frameRect: NSRect) {
        super.init(frame: frameRect)
        wantsLayer = true
        layerContentsRedrawPolicy = .never
    }

    @available(*, unavailable)
    required init?(coder: NSCoder) {
        fatalError("init(coder:) wird nicht unterstützt")
    }

    override func makeBackingLayer() -> CALayer {
        let layer = CAMetalLayer()
        layer.isOpaque = true
        return layer
    }

    override var acceptsFirstResponder: Bool { true }

    // MARK: Lebenszyklus

    override func viewDidMoveToWindow() {
        super.viewDidMoveToWindow()
        guard window != nil else {
            stop()
            return
        }
        updateContentsScale()
        createViewportIfNeeded()
        if displayLink == nil {
            let link = displayLink(target: self, selector: #selector(step(_:)))
            link.add(to: .main, forMode: .common)
            displayLink = link
        }
    }

    override func viewDidChangeBackingProperties() {
        super.viewDidChangeBackingProperties()
        updateContentsScale()
        resizeViewport()
    }

    override func setFrameSize(_ newSize: NSSize) {
        super.setFrameSize(newSize)
        resizeViewport()
    }

    func stop() {
        displayLink?.invalidate()
        displayLink = nil
    }

    private var scale: CGFloat { window?.backingScaleFactor ?? 2 }

    private var pixelSize: (UInt32, UInt32) {
        (UInt32(max(1, (bounds.width * scale).rounded())), UInt32(max(1, (bounds.height * scale).rounded())))
    }

    private func updateContentsScale() {
        layer?.contentsScale = scale
    }

    private func createViewportIfNeeded() {
        guard viewport == nil, bounds.width > 0, bounds.height > 0 else { return }
        let (width, height) = pixelSize
        let pointer = UInt64(UInt(bitPattern: Unmanaged.passUnretained(self).toOpaque()))
        do {
            viewport = try Viewport(nsView: pointer, widthPx: width, heightPx: height, scale: Float(scale))
        } catch {
            NSLog("Lot: 3D-Ansicht konnte nicht starten: %@", AppModel.message(for: error))
            return
        }
        NSLog("Lot: 3D-Ansicht bereit, %d x %d Pixel", width, height)
        shownRevision = -1
        if let document { show(document: document, revision: Int.max) }
    }

    private func resizeViewport() {
        guard bounds.width > 0, bounds.height > 0 else { return }
        if viewport == nil {
            createViewportIfNeeded()
            return
        }
        let (width, height) = pixelSize
        viewport?.resize(widthPx: width, heightPx: height, scale: Float(scale))
        needsRender = true
    }

    // MARK: Modell

    func show(document: LotDocument, revision: Int) {
        self.document = document
        guard let viewport else { return }
        let isNewDocument = shownDocument !== document
        guard isNewDocument || revision != shownRevision else { return }
        do {
            stats.rebuildMs = try viewport.show(document: document)
            stats.triangles = viewport.triangleCount()
        } catch {
            NSLog("Lot: Modell konnte nicht angezeigt werden: %@", AppModel.message(for: error))
        }
        shownDocument = document
        shownRevision = revision
        if pendingFit {
            viewport.zoomToFit()
            pendingFit = false
        }
        needsRender = true
    }

    func fit(request: Int) {
        guard request != handledFitRequest else { return }
        handledFitRequest = request
        if let viewport {
            viewport.zoomToFit()
            needsRender = true
        } else {
            pendingFit = true
        }
    }

    // MARK: Zeichnen

    @objc private func step(_ link: CADisplayLink) {
        guard needsRender, let viewport else { return }
        needsRender = false
        let start = CACurrentMediaTime()
        do {
            try viewport.render()
        } catch {
            NSLog("Lot: Zeichnen fehlgeschlagen: %@", AppModel.message(for: error))
        }
        let now = CACurrentMediaTime()
        stats.frameMs = (now - start) * 1000
        if now - lastStatsUpdate > 0.25 {
            lastStatsUpdate = now
            onStats?(stats)
        }
    }

    // MARK: Trackpad und Maus

    override func scrollWheel(with event: NSEvent) {
        guard let viewport else { return }
        let dx = Float(event.scrollingDeltaX)
        let dy = Float(event.scrollingDeltaY)
        if event.hasPreciseScrollingDeltas {
            // Trackpad oder Magic Mouse: zwei Finger drehen, mit Umschalttaste verschieben.
            if event.modifierFlags.contains(.shift) {
                viewport.pan(dx: dx, dy: dy)
            } else {
                viewport.orbit(dx: dx, dy: dy)
            }
        } else {
            // Mausrad: zoomen.
            viewport.zoom(factor: pow(1.1, dy))
        }
        needsRender = true
    }

    override func magnify(with event: NSEvent) {
        viewport?.zoom(factor: Float(1 + event.magnification))
        needsRender = true
    }

    override func mouseDragged(with event: NSEvent) {
        let dx = Float(event.deltaX)
        let dy = Float(event.deltaY)
        if event.modifierFlags.contains(.option) {
            viewport?.pan(dx: dx, dy: dy)
        } else {
            viewport?.orbit(dx: dx, dy: dy)
        }
        needsRender = true
    }

    override func rightMouseDragged(with event: NSEvent) {
        viewport?.pan(dx: Float(event.deltaX), dy: Float(event.deltaY))
        needsRender = true
    }

    override func otherMouseDragged(with event: NSEvent) {
        rightMouseDragged(with: event)
    }

    override func mouseDown(with event: NSEvent) {
        window?.makeFirstResponder(self)
    }
}
