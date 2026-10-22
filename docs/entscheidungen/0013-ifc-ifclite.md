# 0013 IFC mit IFClite

Status: vorgemerkt für M4 · 22.10.2026

## Kontext

IFC-Import als Referenz kommt in M4, der Export nach Version 1. PLAN.md fragt, ob IFClite nativ ohne WASM einzubinden ist.

## Entscheidung

**IFClite** (`ifc-lite-core` 20.4, MPL-2.0, als Bibliothek) bleibt erster Kandidat. Es übersetzt nativ mit stabilem Rust, nur der WASM-Teil des Projekts braucht Nightly. Es gibt Crates für Geometrie und Export.

Bei M4 neu bewerten: IFClite hatte zwischen Januar und Oktober 2026 sieben Hauptversionen. IFC aus Rust heraus zu schreiben haben wir noch nicht getestet.

## Alternativen

- web-ifc (MPL-2.0, C++/WASM)
- IfcOpenShell (LGPL-3.0, C++ mit Open CASCADE, müsste dynamisch gelinkt werden)

## Folgen

Keine in M0. Die Bauteilklassen folgen schon jetzt IFC (docs/glossar.md).
