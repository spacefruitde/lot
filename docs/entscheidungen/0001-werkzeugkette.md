# 0001 Werkzeugkette und Aufbau des Workspace

Status: umgesetzt in M0 · 22.10.2026

Ben zu rustup: „ich folge deinen beiden empfehlungen“

## Kontext

Der Kern soll auf dem Mac und unter Linux bauen (Cloud-Sessions vom iPhone). Auf Bens Mac war Rust 1.85 über Homebrew installiert, ohne rustup. Aktuelle Bibliotheken brauchen neuere Versionen: wgpu 30 ab 1.87, krilla 0.8 ab 1.92.

## Entscheidung

- **Rust über rustup** (Homebrew-Formel `rustup`), Homebrew-Rust entfernt. Der Suchpfad steht in `~/.zshrc`.
- Version fest in `core/rust-toolchain.toml`: 1.99.1 (stabile Version vom 13.10.2026) mit clippy und rustfmt. Cloud-Sessions mit rustup holen dieselbe Version.
- Mindestversion im Workspace: `rust-version = "1.92"`.
- Workspace `core/` mit den Crates aus PLAN.md, Abschnitt 4: `lot-model`, `lot-geom`, `lot-plan`, `lot-rules` (noch leer), `lot-io`, `lot-render`, `lot-ffi`, `lot-cli`, dazu `uniffi-bindgen` als Werkzeug.
- Rust Edition 2024, `resolver = "3"`.
- Mac-App `app-mac/Lot.xcodeproj` ohne Zusatzwerkzeuge (kein XcodeGen): Der Ordner `app-mac/Lot/` ist als synchronisierte Gruppe eingebunden, neue Swift-Dateien erscheinen ohne Projektänderung. Die erste Build-Phase ruft `app-mac/scripts/build-core.sh` auf.

## Alternativen

- **Homebrew-Rust:** zuerst genutzt. Bringt nur das Ziel Apple Silicon mit, seine Standardbibliothek ist für macOS 26 gebaut, und es ignoriert `rust-toolchain.toml`.
- **XcodeGen oder Tuist:** erzeugen das Xcode-Projekt aus einer Textdatei. Eine Abhängigkeit mehr; mit synchronisierten Ordnern (seit Xcode 16) nicht nötig.

## Folgen

- Die App läuft ab macOS 14. Ein Release-Build für `generic/platform=macOS` enthält Apple Silicon und Intel; das Build-Skript baut den Kern für jede Architektur und fügt sie mit `lipo` zusammen.
- Fehlt ein Apple-Ziel, installiert es das Build-Skript über `rustup target add`.
