# 0015 Golden-Tests und Snapshot-Tests

Status: umgesetzt in M0 · 22.10.2026

## Kontext

Freigegebene Pläne sind Referenzen. Jede künftige Ausgabe wird damit verglichen (CLAUDE.md, Regel 3). PLAN.md nennt insta für Snapshot-Tests.

## Entscheidung

- **Golden-Tests** ohne Zusatzbibliothek: `core/crates/lot-cli/tests/golden.rs` rechnet jedes Testprojekt mit Ordner `referenz/` neu und vergleicht das SVG zeichengenau. Die Referenzen sind normale SVG-Dateien, die Ben ansehen kann.
- Eine Referenz entsteht nur nach Bens Freigabe: Ausgabe aus `out/<name>/` nach `testprojekte/<name>/referenz/` kopieren.
- **insta** (1.49, Apache-2.0) nehmen wir dazu, sobald Unit-Tests größere Strukturen vergleichen, etwa Wandknoten in M1. In M0 noch nicht gebraucht.

## Alternativen

insta auch für die Golden-Tests: Dessen eigenes Dateiformat wäre für Ben schlechter lesbar als ein SVG, und insta schlägt beim Abweichen das Überschreiben vor. Das passt nicht zur Regel, dass nur Ben freigibt.

## Folgen

Ein Bildvergleich (PNG) kommt dazu, wenn die ersten Referenzen freigegeben sind.
