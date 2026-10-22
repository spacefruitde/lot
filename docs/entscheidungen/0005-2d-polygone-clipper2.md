# 0005 2D-Polygone mit clipper2-rust

Status: entschieden für M1, in M0 noch nicht direkt genutzt · 22.10.2026

## Kontext

Wandschichten entstehen durch Versatz der Wandachse, Anschlüsse durch Vereinigung und Differenz von Schichtflächen. PLAN.md fragt nach der Robustheit bei fast parallelen Kanten.

## Entscheidung

**clipper2-rust 1.2** (BSL-1.0, reines Rust, stabile 1.x-Reihe). Kommt ohnehin mit manifold-rust. Gerechnet wird mit Ganzzahlpfaden (`Path64`) auf einem festen Raster von 1 µm, nicht mit den Gleitkomma-Pfaden.

Geprüft mit L-Stoß, Wand minus Tür, Versatz und fast parallelen Kanten (1e-7 rad): richtige Ergebnisse, kein Absturz, Versatz exakt.

## Alternativen

- **i_overlay 9** (MIT/Apache): ebenfalls robust, kann zu jeder Kante mitführen, aus welcher Schicht sie stammt. Aber fünf große Versionssprünge zwischen April und September 2026.
- **cavalier_contours** (MIT/Apache): Versatz mit echten Bögen, interessant für gebogene Wände.
- **geo-clipper:** basiert auf Clipper 1, ruht.

## Folgen

- Die Gleitkomma-Pfade von Clipper2 runden auf ein Zweierpotenz-Raster (mit 6 Stellen wird aus 4,7 der Wert 4,6999998). Deshalb nur `Path64`.
- Sehr schmale Restflächen (unter 1 µm breit) filtern wir aus.
