# Fremdcode und Abhängigkeiten

Jede Abhängigkeit und jedes übernommene Stück Code steht hier mit Lizenz. Regeln siehe CLAUDE.md, Punkt 12.

Stand M4 (23.10.2026). Die Tabelle nennt die direkten Abhängigkeiten. Insgesamt zieht der Kern 317 Crates nach. Alle stehen unter MIT, Apache-2.0, BSD, Zlib, ISC, BSL-1.0, Unicode-3.0 oder sind gemeinfrei; MPL-2.0 haben UniFFI, acadrust und IFClite (nur als Bibliothek, unverändert). r-efi steht unter MIT oder Apache-2.0 oder LGPL; wir nutzen es unter MIT. GPL oder AGPL kommt nicht vor. Prüfen mit `cargo metadata` (Auswertung im Prüfpaket M0).

## Kern (`core/`)

| Name | Version | Lizenz | Wofür | Art (Bibliothek / übernommener Code) |
|---|---|---|---|---|
| manifold-rust | 0.18.0 | Apache-2.0 | 3D-Booleans (Wand minus Öffnung), Schnitte | Bibliothek |
| clipper2-rust | 1.2.0 | BSL-1.0 | 2D-Booleans der Wandanschlüsse, auch über manifold-rust | Bibliothek |
| lyon_tessellation | 1.0.23 | MIT OR Apache-2.0 | Planansicht auf dem Bildschirm: Linien und Flächen in Dreiecke | Bibliothek |
| wgpu | 30.0.1 | MIT OR Apache-2.0 | Renderer der 3D-Ansicht (Metal auf dem Mac) | Bibliothek |
| raw-window-handle | 0.6.2 | MIT OR Apache-2.0 OR Zlib | Übergabe des NSView an wgpu | Bibliothek |
| glam | 0.34.1 | MIT OR Apache-2.0 | Vektoren und Matrizen der Kamera | Bibliothek |
| bytemuck | 1.25.2 | Zlib OR Apache-2.0 OR MIT | Eckpunktdaten für die Grafikkarte | Bibliothek |
| pollster | 1.0.1 | MIT OR Apache-2.0 | Warten auf wgpu-Aufrufe | Bibliothek |
| uniffi | 0.32.2 | MPL-2.0 | Swift-Anbindung des Kerns | Bibliothek (MPL: nur als Bibliothek, nicht verändert) |
| rusqlite | 0.40.2 | MIT | Projektdatei `.lot` | Bibliothek |
| SQLite (über libsqlite3-sys 0.38.2, `bundled`) | 3.53.2 | gemeinfrei | Projektdatei `.lot` | Bibliothek, statisch eingebaut |
| krilla | 0.8.2 | MIT OR Apache-2.0 | PDF-Ausgabe mit eingebetteter Schrift | Bibliothek |
| tiny-skia | 0.12.0 | BSD-3-Clause | PNG-Ausgabe der Pläne | Bibliothek |
| ttf-parser | 0.25.1 | MIT OR Apache-2.0 | Maße und Umrisse der Planschrift | Bibliothek |
| png | 0.18.1 | MIT OR Apache-2.0 | PNG des 3D-Bilds | Bibliothek |
| serde | 1.0.229 | MIT OR Apache-2.0 | Lesen und Schreiben von Modelldaten | Bibliothek |
| serde_json | 1.0.154 | MIT OR Apache-2.0 | Elemente und Operationen in der Projektdatei | Bibliothek |
| ron | 0.12.2 | MIT OR Apache-2.0 | Testprojekte als Textdateien | Bibliothek |
| thiserror | 2.0.21 | MIT OR Apache-2.0 | Fehlertypen | Bibliothek |
| clap | 4.6.7 | MIT OR Apache-2.0 | Kommandozeile `lot-cli` | Bibliothek |
| acadrust | 0.6.6 | MPL-2.0 | DXF und DWG lesen und schreiben (ADR 0012) | Bibliothek (MPL: nur als Bibliothek, nicht verändert), fest gepinnt |
| ifc-lite-core | 20.4.0 | MPL-2.0 | IFC lesen (ADR 0013) | Bibliothek (MPL: nur als Bibliothek, nicht verändert), fest gepinnt |
| ifc-lite-geometry | 20.4.0 | MPL-2.0 | Geometrie aus IFC, mit Öffnungen | Bibliothek (MPL: nur als Bibliothek, nicht verändert), fest gepinnt |
| rustc-hash | 2.1 | Apache-2.0 OR MIT | Übergabe des Öffnungsverzeichnisses an IFClite | Bibliothek |
| anyhow | 1.0.104 | MIT OR Apache-2.0 | Fehlermeldungen in `lot-cli` | Bibliothek |
| tempfile | 3.27.0 | MIT OR Apache-2.0 | nur in Tests | Bibliothek |

## Schrift

| Name | Version | Lizenz | Wofür | Art |
|---|---|---|---|---|
| osifont (`core/assets/fonts/osifont-lgpl3fe.ttf`) | Stand Repository 16.04.2026, Quelle github.com/hikikomori82/osifont | LGPL-3.0 mit GPL-Font-Ausnahme | Planschrift nach ISO 3098 in PDF, SVG und PNG | Datei, unverändert, wird zur Laufzeit geladen und nicht einkompiliert. In PDFs als Teilmenge eingebettet, das erlaubt die Font-Ausnahme. Mitliefern im Produkt siehe ADR 0011 |

## Mac-App (`app-mac/`)

Keine Fremdabhängigkeiten. `app-mac/Lot/Generated/lot_ffi.swift` und `app-mac/Bridging/lot_ffiFFI.h` erzeugt UniFFI aus unserem eigenen Code.

## Übernommener Code

Keiner.
