# 0014 B-Rep als Rückfallebene

Status: entschieden, kein B-Rep in Version 1 · 22.10.2026

## Kontext

PLAN.md nennt truck als Rückfallebene, falls ebene Flächen nicht reichen.

## Entscheidung

Kein B-Rep-Kernel in Version 1. Wände, Öffnungen, Decken und Dächer laufen über Netze und Manifold (ADR 0004).

Geprüft: truck 0.6 (Apache-2.0) hat seit September 2024 kein Release. Ein Boolean mit deckungsgleichen Flächen (Öffnung genau so tief wie die Wand) liefert dort kein Ergebnis.

## Alternativen

- monstertruck (Apache-2.0): aktiver Ableger von truck, kaum Nutzer.
- Open CASCADE über `opencascade` (LGPL-2.1): schwerer C++-Build, nur dynamisch.
- fornjot: archiviert.

## Folgen

Falls später STEP-Austausch oder Freiformflächen nötig werden, neu bewerten.
