# 0004 3D-Booleans mit manifold-rust

Status: umgesetzt in M0 · 22.10.2026

## Kontext

Öffnungen werden von Wänden abgezogen, später schneiden sich Dächer, Decken und Wände. Das muss robust sein, auch bei deckungsgleichen Flächen. PLAN.md nennt Manifold als native Portierung oder über die C-Schnittstelle.

## Entscheidung

**manifold-rust 0.18.0** (Apache-2.0), eine reine Rust-Portierung von Manifold 3.5, exakt auf diese Version festgelegt. Gekapselt in `lot-geom`, der Rest des Kerns sieht nur `TriMesh` und Ringe.

Geprüft:
- Wand 5,00 × 0,30 × 2,60 m minus Fenster: Volumen exakt, Netz geschlossen (jede Kante an genau zwei Dreiecken).
- Deckungsgleiche Flächen (Öffnung exakt so tief wie die Wand, Tür am Boden): korrekt.
- Schnitt bei +1,00 m: zwei Wandstücke links und rechts des Fensters.
- Wandänderung mit Operation und neuer Geometrie: 0,09 ms. Ziel war unter 50 ms.
- Build ohne C++, ohne cmake, ohne Netzzugriff: 12 s.

## Alternativen

- **manifold-csg 0.4** (Apache/MIT, bindet das C++-Original): identische Ergebnisse, aber 98 s Build mit cmake, und das Build-Skript klont Manifold beim Bauen von GitHub. Ein Versuch scheiterte daran. Bleibt Rückfallebene und Vergleichsreferenz.
- **manifold-rs:** Repository archiviert. **csgrs:** BSP-basiert, seit 15 Monaten kein Release. **boolmesh:** MPL-2.0, sehr jung.

## Folgen

- manifold-rust ist jung (seit Juli 2026, ein Hauptentwickler, häufige Versionssprünge). Deshalb fest gepinnt und nur über `lot-geom` genutzt.
- Manifold rastet Schnitte intern auf ein feines Clipper2-Raster ein (etwa 10 nm). Wir runden Schnittpunkte auf 1 µm (ADR 0002).
