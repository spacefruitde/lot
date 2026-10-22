# CLAUDE.md

Diese Datei gibt Claude Code die Grundregeln für die Arbeit in diesem Repository. Der ausführliche Plan steht in `docs/PLAN.md`. Lies beide zu Beginn jeder Session.

## Was das ist

**Lot** (Arbeitstitel) ist ein natives BIM-Werkzeug für kleine Architekturbüros. Mac zuerst, Windows später möglich. Version 1 deckt LP 1-4 ab, vom Entwurf bis zum Bauantrag. Zuerst nutzt Ben es für sein eigenes Büro, danach soll es ein Produkt für andere kleine Büros werden.

## Wer was macht

- **Ben**: Architekt mit 15 Jahren Revit-Erfahrung und viel Code-Erfahrung. Er entscheidet über das Produkt und prüft fachlich. Er hat weniger als 5 Stunden pro Woche. **Seine Prüfzeit ist die Engstelle, nicht der Code.**
- **Claude**: schreibt Code, Tests und Doku. Innerhalb des aktuellen Meilensteins arbeitet Claude selbstständig und fragt nur, wenn es fachlich oder produktseitig wirklich offen ist.

## Arbeitsregeln

1. **Jede Lieferung endet mit einem Prüfpaket** unter `pruefpakete/JJJJ-MM-TT-thema/`: PDFs und PNGs der betroffenen Testprojekte plus eine `README.md` mit höchstens 10 konkreten Prüffragen. Ben muss es in 15 Minuten auf dem iPhone durcharbeiten können.
2. **Fachliches nicht raten.** Fragen zu Darstellung, Normen und Planungspraxis kommen als Frage ins Prüfpaket, jeweils mit einem Vorschlag.
3. **Freigegebene Ausgaben sind Referenzen** (`testprojekte/*/referenz/`). Golden-Tests vergleichen jede Planausgabe damit. Eine Referenz wird nur geändert, wenn Ben es ausdrücklich freigibt.
4. **Entscheidungen mit Tragweite** halten wir als kurzen ADR in `docs/entscheidungen/NNNN-titel.md` fest (Kontext, Entscheidung, Alternativen, Folgen). Bens Entscheidungen stehen dort in seinen Worten.
5. **Der Kern bleibt plattformunabhängig.** In `core/` gibt es keinen macOS-spezifischen Code. Alles in `core/` muss unter Linux bauen und testen, damit auch Cloud-Sessions vom iPhone daran arbeiten können.
6. **Geometrie wird immer abgeleitet.** Gespeichert werden nur Parameter, Bezüge und Operationen, nie berechnete Geometrie als Wahrheit.
7. **Jede Modelländerung ist eine Operation.** Darauf bauen Undo/Redo, später der Versionsverlauf und das Arbeiten mit mehreren Personen auf.
8. **Normen und Regeln mit Quelle.** Im Code steht jeweils Norm, Abschnitt und Fassung als Kommentar. Landesrecht (Abstandsflächen, Bauvorlagen) liegt als Daten je Bundesland vor, nicht als fest verdrahteter Code.
9. **Sprache**:
   - Bezeichner im Code sind englisch.
   - Fachbegriffe ohne eindeutige englische Entsprechung (Laibung, Brüstung, Abstandsfläche) stehen in `docs/glossar.md`.
   - Kommentare, Doku, Commit-Nachrichten und UI-Texte sind deutsch.
10. **Texte für Ben ohne Gedankenstriche**, nur normale Bindestriche.
11. **Kleine Commits.** Vor jedem Commit laufen die Tests im Kern.
12. **Lizenzen.** Lot soll später ein geschlossenes Produkt werden können.
    - Code übernehmen wir nur aus Projekten unter MIT, Apache, BSD, BSL oder Zlib und vermerken ihn in `THIRD_PARTY.md`.
    - MPL-2.0- und LGPL-Bibliotheken nutzen wir nur als Bibliothek. LGPL wird dynamisch gelinkt.
    - GPL- und AGPL-Projekte (zum Beispiel Bonsai) dienen nur dazu, Konzepte zu verstehen. Wir kopieren oder übersetzen keinen Code daraus.
    - Jede neue Abhängigkeit kommt mit Lizenz in `THIRD_PARTY.md`.

## Befehle

Angelegt in M0, immer aktuell halten. Rust über rustup, die Version steht in `core/rust-toolchain.toml` (ADR 0001).

- Kern testen: `cd core && cargo test --workspace`
- Kern auf Warnungen prüfen: `cd core && cargo clippy --workspace --all-targets -- -D warnings`
- Formatieren: `cd core && cargo fmt` (Einstellungen in `core/rustfmt.toml`)
- CI: `.github/workflows/pruefen.yml` prüft bei jedem Push Formatierung, clippy und Tests unter Linux und macOS und baut die Mac-App
- Kern unter Linux lokal prüfen, ohne Ordnerfreigabe für Docker: `tar --exclude core/target -cf - core testprojekte | docker run -i --rm rust:1.99 bash -c 'mkdir /lot && tar -C /lot -xf - && cd /lot/core && cargo test --workspace'`
- Testprojekt rechnen und Pläne ausgeben: `cd core && cargo run -p lot-cli -- render ../testprojekte/<name> --out ../out/<name>`
  - schreibt `m0-pruefblatt.pdf/.svg/.png`, `3d.png` und `projekt.lot`
  - `--no-3d` auf Rechnern ohne Grafikkarte (etwa Cloud-Sessions)
  - die Planschrift liegt in `core/assets/fonts/`, ein anderer Ort geht über `LOT_PLAN_FONT`
- Swift-Anbindung neu erzeugen: `app-mac/scripts/build-core.sh` (läuft auch automatisch als erste Build-Phase in Xcode)
- Mac-App bauen: `cd app-mac && xcodebuild -project Lot.xcodeproj -scheme Lot -configuration Debug -derivedDataPath build/DerivedData build`
- Mac-App für Apple Silicon und Intel (ab macOS 14): dasselbe mit `-configuration Release -destination 'generic/platform=macOS'`
- Mac-App starten: `open app-mac/build/DerivedData/Build/Products/Debug/Lot.app`, oder in Xcode `app-mac/Lot.xcodeproj` öffnen und ⌘R
- Golden-Tests: `core/crates/lot-cli/tests/golden.rs` vergleicht jede Referenz in `testprojekte/*/referenz/` zeichengenau mit dem neu gerechneten SVG. Referenzen nur nach Bens Freigabe anlegen oder ändern (Regel 3).

## Aufbau

Siehe `docs/PLAN.md`, Abschnitt 4.

- `core/`: Rust-Workspace (Modell, Geometrie, Pläne, Regeln, Dateien, Renderer, Swift-Schnittstelle, Kommandozeile)
- `app-mac/`: Mac-App in Swift
- `testprojekte/`: Referenzprojekte als lesbare Textdateien, dazu die freigegebenen Referenzpläne
- `pruefpakete/`: Lieferungen zum Prüfen für Ben
- `docs/`: Plan, Entscheidungen, Glossar

## Zentrale

Wenn der Connector „Kommandozentrale“ verfügbar ist:

- Zu Beginn `get_project_brief` aufrufen. Projekt ist vorerst `tools`, bis Lot ein eigenes Projekt hat.
- Neue Aufgaben und Entscheidungen für Ben mit `add_task` eintragen.
- Umgesetzte Entscheidungen mit `mark_applied` zurückmelden.
