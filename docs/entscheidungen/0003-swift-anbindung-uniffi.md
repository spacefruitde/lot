# 0003 Swift-Anbindung mit UniFFI

Status: umgesetzt in M0 · 22.10.2026

## Kontext

Die Mac-App in Swift ruft den Rust-Kern auf. PLAN.md fragt nach der Leistung bei vielen kleinen Aufrufen.

## Entscheidung

UniFFI 0.32.2 (Mozilla, MPL-2.0, nur als Bibliothek) mit Proc-Macros statt UDL. Die Swift-Dateien erzeugt `uniffi-bindgen` im Library-Modus aus der gebauten Bibliothek. Die App linkt `liblot_ffi.a` statisch und bindet die C-Seite über einen Bridging-Header ein.

Die Schnittstelle in M0: `LotDocument` (Parameter, Undo/Redo, Speichern, Laden) und `Viewport` (3D-Ansicht, Gesten, Zeichnen).

## Messwerte (M1 Max, Release)

| Fall | Zeit |
|---|---|
| 1 Mio. Aufrufe mit Zahlen | 6 bis 12 ns je Aufruf |
| Rückgabe einer Liste mit einem Datensatz | 1,3 µs je Aufruf |
| `Vec<f32>` mit 1 Mio. Werten | 290 ms (jedes Element einzeln) |
| `Vec<u8>` mit 4 MB | 2,7 ms |
| Swift nach Rust, `&[u8]` mit 64 MB (ohne Kopie) | 9,7 ms |

Daraus folgt: Viele kleine Aufrufe mit Zahlen sind unkritisch, eine Trackpad-Geste kostet Nanosekunden. Große Datenmengen gehen als Bytes über die Schnittstelle, nie als Listen von Zahlen oder Datensätzen. Die Geometrie bleibt ohnehin im Kern, weil auch der Renderer dort läuft.

## Alternativen

- **BoltFFI** 0.31 (MIT): jung, sehr aktiv, verspricht weniger Aufwand je Aufruf. Nicht getestet.
- **swift-bridge** 0.1 (MIT/Apache): langsam gepflegt, vor 1.0.
- Eigene C-Schnittstelle: volle Kontrolle, viel Handarbeit.

## Folgen

Die erzeugten Dateien `app-mac/Lot/Generated/lot_ffi.swift` und `app-mac/Bridging/lot_ffiFFI.h` liegen im Repository, damit das Projekt ohne Vorlauf in Xcode öffnet. Das Build-Skript aktualisiert sie bei jedem Build.
