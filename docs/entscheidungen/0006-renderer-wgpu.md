# 0006 Renderer mit wgpu im NSView der App

Status: umgesetzt in M0 · 22.10.2026

## Kontext

3D-Ansicht und später die Planansicht zeichnen über denselben Renderer, auf dem Mac über Metal, später auf Windows über DirectX 12 oder Vulkan. PLAN.md fragt, wie sich wgpu in eine Metal-Ebene aus SwiftUI einbetten lässt.

## Entscheidung

- **wgpu 30.0.1** (MIT/Apache-2.0).
- Die App stellt einen `NSView` bereit, dessen Schicht ein `CAMetalLayer` ist (`makeBackingLayer`), und übergibt seine Adresse an den Kern. Der Kern erzeugt daraus über das plattformneutrale `raw-window-handle` eine wgpu-Fläche. wgpu nutzt dann genau diese Schicht, es legt keine zweite an.
- Gezeichnet wird auf dem Main-Thread, angestoßen von `NSView.displayLink` (macOS 14), und nur, wenn sich etwas geändert hat.
- Darstellung in M0: flach schattierte Flächen in warmem Weiß, Kanten über 20° Knick als dunkle Linien, 4-fach Kantenglättung.
- Ohne Fenster zeichnet derselbe Renderer in eine Textur. Daraus entsteht das 3D-Bild im Prüfpaket.

Gemessen: Grafik starten 0,2 s, ein Bild 2400 × 1600 Pixel mit Zurücklesen 2,7 ms.

## Alternativen

- **`SurfaceTargetUnsafe::CoreAnimationLayer`** mit der Adresse der Schicht: ebenso möglich, ist aber nur auf Apple-Systemen übersetzbar und bräuchte deshalb macOS-Code im Kern (CLAUDE.md, Regel 5).
- **Eigener Metal-Renderer in Swift:** schnellster Weg auf dem Mac, aber doppelte Arbeit für Windows.

## Folgen

- Offenes wgpu-Problem #10627: Wer auf einem eigenen Render-Thread zeichnet, bekommt Warnungen des Main-Thread-Checkers. Mit dem Main-Thread-Ansatz tritt das nicht auf.
- Für 120 Hz auf ProMotion-Displays fordert die App 120 Bilder pro Sekunde an und zeichnet nach der letzten Geste noch eine Sekunde lang weiter, sonst senkt macOS die Bildrate. Ben prüft das Gefühl in M0.
- Liegt das Fenster verdeckt oder auf einem anderen Space, liefert wgpu kein Bild. Die App versucht es dann beim nächsten Takt erneut.
- wgpu bringt etwa alle drei Monate eine neue Hauptversion. Updates planen wir bewusst ein.
