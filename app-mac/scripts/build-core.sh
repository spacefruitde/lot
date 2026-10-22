#!/bin/sh
# Baut den Rust-Kern (lot-ffi) als statische Bibliothek und erzeugt die
# Swift-Anbindung. Läuft als erste Build-Phase in Xcode und lässt sich auch
# von Hand aufrufen:
#
#   app-mac/scripts/build-core.sh [debug|release]
#
# Ergebnis:
#   core/target/apple/<profil>/liblot_ffi.a   wird von Xcode gelinkt, enthält
#                                         alle Architekturen aus $ARCHS
#   app-mac/Lot/Generated/lot_ffi.swift   Swift-Seite der Anbindung
#   app-mac/Bridging/lot_ffiFFI.h         C-Seite, über den Bridging-Header
set -eu

# Xcode startet Skripte mit knappem PATH. cargo kommt von rustup (ADR 0001).
export PATH="$HOME/.cargo/bin:/opt/homebrew/opt/rustup/bin:/opt/homebrew/bin:/usr/local/bin:$PATH"

APP_DIR=$(cd "$(dirname "$0")/.." && pwd)
CORE_DIR=$(cd "$APP_DIR/../core" && pwd)

PROFILE="${1:-}"
if [ -z "$PROFILE" ]; then
    if [ "${CONFIGURATION:-Debug}" = "Release" ]; then PROFILE=release; else PROFILE=debug; fi
fi
case "$PROFILE" in
    debug) CARGO_PROFILE_FLAG="" ;;
    release) CARGO_PROFILE_FLAG="--release" ;;
    *) echo "Unbekanntes Profil: $PROFILE (debug oder release)" >&2; exit 1 ;;
esac

# Architekturen: von Xcode ($ARCHS), sonst die des eigenen Rechners.
ARCHS="${ARCHS:-$(uname -m)}"

cd "$CORE_DIR"
LIBS=""
FIRST_TRIPLE=""
for ARCH in $ARCHS; do
    case "$ARCH" in
        arm64) TRIPLE=aarch64-apple-darwin ;;
        x86_64) TRIPLE=x86_64-apple-darwin ;;
        *) echo "Unbekannte Architektur: $ARCH" >&2; exit 1 ;;
    esac
    if ! rustup target list --installed | grep -qx "$TRIPLE"; then
        rustup target add "$TRIPLE"
    fi
    # shellcheck disable=SC2086
    cargo build -p lot-ffi --target "$TRIPLE" $CARGO_PROFILE_FLAG
    LIBS="$LIBS target/$TRIPLE/$PROFILE/liblot_ffi.a"
    FIRST_TRIPLE="${FIRST_TRIPLE:-$TRIPLE}"
done
mkdir -p "target/apple/$PROFILE"
# shellcheck disable=SC2086
lipo -create $LIBS -output "target/apple/$PROFILE/liblot_ffi.a"

GEN_DIR=$(mktemp -d)
trap 'rm -rf "$GEN_DIR"' EXIT
cargo run -q -p uniffi-bindgen -- generate \
    --library "target/$FIRST_TRIPLE/$PROFILE/liblot_ffi.dylib" \
    --language swift \
    --out-dir "$GEN_DIR"

# Nur ersetzen, wenn sich etwas geändert hat, damit Xcode nicht unnötig neu übersetzt.
copy_if_changed() {
    mkdir -p "$(dirname "$2")"
    if ! cmp -s "$1" "$2"; then
        cp "$1" "$2"
        echo "aktualisiert: $2"
    fi
}
copy_if_changed "$GEN_DIR/lot_ffi.swift" "$APP_DIR/Lot/Generated/lot_ffi.swift"
copy_if_changed "$GEN_DIR/lot_ffiFFI.h" "$APP_DIR/Bridging/lot_ffiFFI.h"
