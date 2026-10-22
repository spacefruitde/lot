# 0001 Werkzeugkette und Aufbau des Workspace

Status: umgesetzt in M0, ein Punkt offen für Ben (rustup) · 22.10.2026

## Kontext

Der Kern soll auf dem Mac und unter Linux bauen (Cloud-Sessions vom iPhone). Auf Bens Mac war Rust 1.85 über Homebrew installiert, ohne rustup. Aktuelle Bibliotheken brauchen neuere Versionen: wgpu 30 ab 1.87, krilla 0.8 ab 1.92.

## Entscheidung

- Homebrew-Rust auf 1.99.1 aktualisiert (aktuelle stabile Version vom 13.10.2026).
- Mindestversion im Workspace: `rust-version = "1.92"`.
- Workspace `core/` mit den Crates aus PLAN.md, Abschnitt 4: `lot-model`, `lot-geom`, `lot-plan`, `lot-rules` (noch leer), `lot-io`, `lot-render`, `lot-ffi`, `lot-cli`, dazu `uniffi-bindgen` als Werkzeug.
- Rust Edition 2024, `resolver = "3"`.
- Mac-App `app-mac/Lot.xcodeproj` ohne Zusatzwerkzeuge (kein XcodeGen): Der Ordner `app-mac/Lot/` ist als synchronisierte Gruppe eingebunden, neue Swift-Dateien erscheinen ohne Projektänderung. Die erste Build-Phase ruft `app-mac/scripts/build-core.sh` auf.

## Alternativen

- **rustup statt Homebrew-Rust:** üblich, kann weitere Ziele installieren (Intel-Mac, iOS) und hält sich an eine `rust-toolchain.toml`. Ändert aber den Suchpfad in Bens Shell, deshalb nicht ohne Rückfrage umgestellt.
- **XcodeGen oder Tuist:** erzeugen das Xcode-Projekt aus einer Textdatei. Eine Abhängigkeit mehr; mit synchronisierten Ordnern (seit Xcode 16) nicht nötig.

## Folgen

- Homebrew-Rust bringt seine Standardbibliothek für macOS 26 gebaut mit. Die App läuft deshalb vorerst nur ab macOS 26 (`MACOSX_DEPLOYMENT_TARGET = 26.0`), und nur auf Apple Silicon (`ARCHS = arm64`).
- **Offen für Ben:** Mit rustup ließe sich die App für macOS 14 und für Intel-Macs bauen, und Cloud-Sessions nutzen dieselbe Version. Vorschlag: rustup installieren, sobald die App an andere Rechner geht.
