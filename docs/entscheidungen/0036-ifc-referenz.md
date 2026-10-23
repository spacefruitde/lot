# 0036 IFC als Referenz

Status: umgesetzt in M4 · 23.10.2026

## Kontext

ADR 0013 merkte IFClite vor. M4 verlangt den IFC-Import als Referenz (6.02): Modelle von Fachplanern sind sichtbar und schneidbar, aber nicht bearbeitbar.

## Entscheidung

- **Referenzmodell** (`ReferenceModel`): Datei (im Projekt), Ursprung, Höhe und Drehung. IFClite (`ifc-lite-core`, `ifc-lite-geometry` 20.4, MPL-2.0, fest gepinnt) liest die Bauteile mit ausgeschnittenen Öffnungen, in Metern.
- **Anzeige**: im 3D, im Schnitt geschnitten (Material `referenz`, hellgrau), im Grundriss in der Schnittebene hellgrau mit dünnem Rand.
- Gelesen werden die üblichen Bauteilklassen (Wände, Decken, Stützen, Träger, Dächer, Treppen, Fenster, Türen, Bekleidungen, TGA-Bauteile); Räume und Öffnungskörper nicht.

## Alternativen

- **IfcOpenShell** (LGPL, C++ mit Open CASCADE): umfassender, aber dynamisch zu linken und groß.
- **IFC in Lot-Bauteile umwandeln**: verlockend, aber dann sind es nicht mehr die Bauteile des Fachplaners.

## Folgen

- Getestet mit einem kleinen, selbst geschriebenen IFC4. Die Beispieldateien von buildingSMART bleiben lokal, bis ihre Lizenz geklärt ist.
- IFC schreiben (6.01) kommt nach Version 1.
