import AppKit
import QuartzCore
import SwiftUI

/// Planansicht und 3D-Ansicht: ein NSView mit CAMetalLayer, in den der
/// Rust-Kern über wgpu zeichnet. Maus, Trackpad und Tasten gehen an den Kern.
struct ViewportView: NSViewRepresentable {
    @Environment(AppModel.self) private var model
    /// Zählt neue Dokumente.
    let generation: Int
    let onEvent: (EventResult) -> Void
    let onStats: (ViewportStatsDisplay) -> Void

    func makeNSView(context: Context) -> MetalViewportNSView {
        let view = MetalViewportNSView(frame: .zero)
        view.model = model
        view.onEvent = onEvent
        view.onStats = onStats
        return view
    }

    func updateNSView(_ view: MetalViewportNSView, context: Context) {
        view.model = model
        view.onEvent = onEvent
        view.onStats = onStats
        view.setNeedsRender()
    }

    static func dismantleNSView(_ view: MetalViewportNSView, coordinator: ()) {
        view.stop()
    }
}

final class MetalViewportNSView: NSView {
    weak var model: AppModel?
    var onEvent: ((EventResult) -> Void)?
    var onStats: ((ViewportStatsDisplay) -> Void)?

    private var viewport: Viewport?
    private var displayLink: CADisplayLink?
    private var needsRender = true
    private var trackingArea: NSTrackingArea?

    private var stats = ViewportStatsDisplay()
    private var lastStatsUpdate: CFTimeInterval = 0

    /// Zeitpunkt der letzten Geste. Kurz danach zeichnet die Ansicht in jedem
    /// Takt weiter, damit macOS auf ProMotion-Displays bei 120 Hz bleibt und
    /// die nächste Bewegung nicht erst mit gesenkter Bildrate erscheint.
    private var lastInteraction: CFTimeInterval = 0
    private static let keepAliveAfterInput: CFTimeInterval = 1.0

    /// Ob die linke Maustaste gerade gedrückt ist.
    private var dragging = false

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

    func setNeedsRender() {
        needsRender = true
    }

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
            link.preferredFrameRateRange = CAFrameRateRange(minimum: 60, maximum: 120, preferred: 120)
            link.add(to: .main, forMode: .common)
            displayLink = link
        }
        window?.makeFirstResponder(self)
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

    override func updateTrackingAreas() {
        super.updateTrackingAreas()
        if let trackingArea { removeTrackingArea(trackingArea) }
        let area = NSTrackingArea(rect: .zero, options: [.mouseMoved, .activeInKeyWindow, .inVisibleRect], owner: self)
        addTrackingArea(area)
        trackingArea = area
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
            let viewport = try Viewport(nsView: pointer, widthPx: width, heightPx: height, scale: Float(scale))
            self.viewport = viewport
            model?.requestRedraw = { [weak self] in self?.setNeedsRender() }
            model?.attach(viewport)
        } catch {
            NSLog("Lot: Ansicht konnte nicht starten: %@", AppModel.message(for: error))
            return
        }
        NSLog("Lot: Ansicht bereit, %d x %d Pixel", width, height)
        needsRender = true
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

    // MARK: Zeichnen

    @objc private func step(_ link: CADisplayLink) {
        let interacting = CACurrentMediaTime() - lastInteraction < Self.keepAliveAfterInput
        guard needsRender || interacting, let viewport else { return }
        let start = CACurrentMediaTime()
        do {
            // Verdeckt oder auf einem anderen Space entsteht kein Bild; beim nächsten Takt erneut versuchen.
            needsRender = try !viewport.render()
        } catch {
            needsRender = false
            NSLog("Lot: Zeichnen fehlgeschlagen: %@", AppModel.message(for: error))
        }
        guard !needsRender else { return }
        let now = CACurrentMediaTime()
        stats.frameMs = (now - start) * 1000
        stats.rebuildMs = model?.stats.rebuildMs ?? 0
        if now - lastStatsUpdate > 0.25 {
            lastStatsUpdate = now
            onStats?(stats)
        }
    }

    // MARK: Ereignisse

    /// Lage des Zeigers in Punkten von der Mitte der Ansicht, y nach oben.
    private func pointerOffset(_ event: NSEvent) -> (Float, Float) {
        let p = convert(event.locationInWindow, from: nil)
        return (Float(p.x - bounds.midX), Float(p.y - bounds.midY))
    }

    private func didInteract() {
        lastInteraction = CACurrentMediaTime()
        needsRender = true
    }

    private func deliver(_ result: EventResult) {
        if result.redraw || result.modelChanged {
            needsRender = true
        }
        onEvent?(result)
    }

    private var isPlan: Bool { model?.viewState?.view != .model }

    override func mouseMoved(with event: NSEvent) {
        guard let viewport, isPlan else { return }
        let (x, y) = pointerOffset(event)
        let result = viewport.pointerMoved(
            x: x, y: y, shift: event.modifierFlags.contains(.shift), alt: event.modifierFlags.contains(.option))
        if result.redraw { deliver(result) }
    }

    override func mouseDown(with event: NSEvent) {
        window?.makeFirstResponder(self)
        guard let viewport else { return }
        dragging = true
        if isPlan {
            let (x, y) = pointerOffset(event)
            deliver(viewport.pointerDown(
                x: x, y: y, clicks: UInt32(event.clickCount),
                shift: event.modifierFlags.contains(.shift), alt: event.modifierFlags.contains(.option)))
        }
    }

    override func mouseDragged(with event: NSEvent) {
        guard let viewport else { return }
        if isPlan {
            // Im Grundriss gehört ⌥ der Auswahl (Lasso, Kopie); verschoben wird mit zwei Fingern.
            let (x, y) = pointerOffset(event)
            deliver(viewport.pointerDragged(
                x: x, y: y, shift: event.modifierFlags.contains(.shift), alt: event.modifierFlags.contains(.option)))
        } else if event.modifierFlags.contains(.option) {
            viewport.pan(dx: Float(event.deltaX), dy: Float(event.deltaY))
        } else {
            viewport.orbit(dx: Float(event.deltaX), dy: Float(event.deltaY))
        }
        didInteract()
    }

    override func mouseUp(with event: NSEvent) {
        guard let viewport, dragging else { return }
        dragging = false
        if isPlan {
            deliver(viewport.pointerUp())
        }
    }

    override func rightMouseDragged(with event: NSEvent) {
        viewport?.pan(dx: Float(event.deltaX), dy: Float(event.deltaY))
        didInteract()
    }

    override func otherMouseDragged(with event: NSEvent) {
        rightMouseDragged(with: event)
    }

    override func scrollWheel(with event: NSEvent) {
        guard let viewport else { return }
        let dx = Float(event.scrollingDeltaX)
        let dy = Float(event.scrollingDeltaY)
        if event.hasPreciseScrollingDeltas {
            // Trackpad: im Grundriss verschieben, in 3D drehen (mit ⇧ verschieben).
            if isPlan || event.modifierFlags.contains(.shift) {
                viewport.pan(dx: dx, dy: dy)
            } else {
                viewport.orbit(dx: dx, dy: dy)
            }
        } else {
            // Mausrad: zum Zeiger zoomen.
            let (x, y) = pointerOffset(event)
            viewport.zoom(factor: pow(1.1, dy), x: x, y: y)
        }
        didInteract()
    }

    override func magnify(with event: NSEvent) {
        let (x, y) = pointerOffset(event)
        viewport?.zoom(factor: Float(1 + event.magnification), x: x, y: y)
        didInteract()
    }

    /// Tasten gehen an den Kern: Kürzel aus zwei Buchstaben (WA, DR, SA …),
    /// Zahlen, Tab, Pfeile und Leertaste (ADR 0043, 0044).
    override func keyDown(with event: NSEvent) {
        guard let viewport else { return super.keyDown(with: event) }
        let mods = event.modifierFlags.intersection([.command, .control, .option])
        if mods == [.command] && event.keyCode == 123 {
            model?.run("PreviousSelection")
            return
        }
        // Auf deutschen Tastaturen liegen @, [ und ] auf ⌥; sie gehören zur Zahleneingabe.
        if mods == [.option], let c = event.characters?.first, "@[]{}|".contains(c) {
            deliver(viewport.key(key: .character(text: String(c))))
            return
        }
        guard mods.isEmpty else { return super.keyDown(with: event) }
        let large = event.modifierFlags.contains(.shift)
        switch event.keyCode {
        case 53: deliver(viewport.key(key: .escape))
        case 36, 76: deliver(viewport.key(key: .enter))
        case 51, 117: deliver(viewport.key(key: .delete))
        case 48: deliver(viewport.key(key: .tab))
        case 123: deliver(viewport.key(key: .arrow(dx: -1, dy: 0, large: large)))
        case 124: deliver(viewport.key(key: .arrow(dx: 1, dy: 0, large: large)))
        case 125: deliver(viewport.key(key: .arrow(dx: 0, dy: -1, large: large)))
        case 126: deliver(viewport.key(key: .arrow(dx: 0, dy: 1, large: large)))
        default:
            guard let c = event.characters?.first else { return super.keyDown(with: event) }
            deliver(viewport.key(key: .character(text: String(c))))
        }
        needsRender = true
    }
}
