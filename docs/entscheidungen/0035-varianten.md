# 0035 Varianten

Status: umgesetzt in M4 · 23.10.2026

## Kontext

M4 verlangt Varianten (2.04): eine Planungsfrage mit mehreren Antworten im selben Projekt, etwa eine Küche offen oder abgetrennt.

## Entscheidung

- **Variantengruppe** (`VariantSet`): Name, Varianten mit ihren Bauteilen, eine aktive Variante. Bauteile, die in keiner Variante stehen, gehören zu allen.
- Pläne, 3D und Flächen zeigen die aktive Variante; Bauteile anderer Varianten lassen sich nicht anklicken. Eine Ausgabe kann eine andere Variante zeigen.
- Befehle (Befehlspalette): neue Gruppe mit A und B, Auswahl nur in die aktive Variante, Variante wechseln.

## Alternativen

- **Varianten als Kopien des Projekts**: einfach, aber Änderungen am gemeinsamen Teil müsste man doppelt machen.

## Folgen

- Neu gezeichnete Bauteile gehören zunächst zu allen Varianten; man ordnet sie danach zu.
