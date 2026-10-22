# 0017 Wandanschlüsse nach Schichtrang

Status: umgesetzt in M1, Ben prüft am Wandknoten-Katalog · 23.10.2026

## Kontext

Wandanschlüsse sind das Herzstück (PLAN.md, Abschnitt 4). Sie sollen ohne Einstellungen so aussehen, wie Ben sie von Hand zeichnet, auch bei verschiedenen Aufbauten, Dicken, Winkeln und Höhen.

## Entscheidung

Ein Verfahren für alle Fälle, in `lot-geom/src/joins.rs`:

1. **Knoten** aus allen Wänden eines Geschosses: zusammenfallende Wandenden (L, Flucht, Y), Wandende in einer durchlaufenden Wand (T), Kreuzung.
2. **Eigene Fläche jeder Wand:** am freien Ende rechtwinklig, am Knoten durch Gehrungslinien zwischen benachbarten Wänden geteilt, am T-Stoß bis an die Wandfläche der durchlaufenden Wand. Gehrungsspitzen über zwei Wanddicken hinaus werden gekappt.
3. **Schichtrang:** Tragschicht 1, Dämmung 2, Luftschicht 3, Installationsebene 4, Putz und Bekleidung 5. Eine Schicht der ankommenden Wand läuft in die andere Wand durch alle Schichten mit niedrigerem Rang und endet an der ersten mit gleichem oder höherem Rang.
4. **Überschneidungen** (Kreuzungen): höherer Rang gewinnt, dann die dickere Wand.
5. **Verschiedene Höhen in einer Ecke:** Die höhere Wand läuft bis an die Außenfläche der niedrigeren durch, die niedrigere stößt stumpf an. Die Knoten gelten über die ganze Geschosshöhe.
6. **Stützen** haben Vorrang vor allen Wandschichten.

3D entsteht aus denselben Flächen, Höhenabschnitt für Höhenabschnitt (an jeder Unter- und Oberkante, Brüstung und jedem Sturz), mit manifold-rust vereinigt.

Abgesichert durch Tests je Fall und einen Zufallstest mit 200 Wandkonstellationen: keine Überlappung über 2 µm Breite.

## Alternativen

- **Gehrung für jede Schicht nach Index** (wie viele Programme bei gleichen Aufbauten): versagt bei verschiedenen Aufbauten und an T-Stößen.
- **Manuelle Anschlusstypen je Knoten** (Revit: Gehrung, stumpf, rechtwinklig): mehr Einstellungen, genau das, was Lot vermeiden soll. Bleibt als spätere Übersteuerung je Knoten.

## Folgen

- Fachliche Feinheiten (Putz an Wandenden, Trockenbau an Putz, spitze Winkel) prüft Ben am Katalog. Regeln lassen sich über den Rang und Sonderfälle nachschärfen, ohne das Verfahren zu ändern.
- Gebogene Wände fehlen noch; der Katalogfall „gebogene Wand an gerader Wand“ folgt mit ihnen.
