# Zeichnen und Fangen: Vergleich und Vorschlag

Quellen und Einzelheiten: `recherche-zeichnen.md`. Was Lot heute kann: `lot-heute.md`. Die Zahleneingabe beim Bearbeiten steht schon im Paket Bedienung 1 (`bearbeiten.md`, Punkt 2); hier geht es um das Zeichnen neuer Bauteile.

## Kurzvergleich

| Thema | Revit | Archicad | Vectorworks | Allplan | Lot heute |
|---|---|---|---|---|---|
| Fangpunkte | rund 14 Arten, mit lotrecht und tangential | Knoten, Kante, Lot, Teilungen | 7 Objektpunkte, Smart Edge | Ende, Mitte, Teilung, Schnitt, Lotfuß | Wandenden, Achsen, auf Wänden, DXF-Punkte |
| Fang für einen Klick | Kürzel SE, SM, SI, SP | Konstruktionsmethoden | nur Fang aussetzen | Punkt-Assistent | nein |
| mehrere Kandidaten | Tab | Tab (Forum) | Lupe Z | zoomen | nein |
| Spurlinien | gestrichelt, automatisch | Snap Guides, Q setzt Referenz | Smart Points, Datum G | F11, 5 Punkte | nein |
| Richtung | ⇧, Inkremente | ⇧, Alt+X/Y/A | 30/45°, gedrehtes System | rechtwinklig, 15°, Systemwinkel | 15°, ⇧ 90° |
| Länge tippen | nach dem Startpunkt | direkt | Datenzeile | Dialogzeile | ja, Wandlänge |
| Winkel tippen | nicht zuerst | A und Wert | Feld A | Winkel und Länge | nein |
| Relativ, Versatz von Punkt | unbelegt | Snap Reference | Datum mit Versatz | Bezugspunkt | nein |
| Rechnen | unbelegt | ja | unbelegt | ja | nein |
| aus Vorhandenem | Linien und Flächen wählen | Zauberstab | aus Formen und Räumen | Flächensuche | nein |
| Decken, Dächer | Skizzenmodus | direkt | direkt | direkt | „aus Wänden“ |
| PDF fangen | Schalter | erst auflösen | Schalter, nur Vektor | unbelegt | nein (DXF und DWG ja) |

**Was Nutzer am meisten ärgert:** in Revit den Winkel nicht zuerst tippen können [K1]; Fangeinstellungen, die mit der Sitzung verloren gehen [R2]; Spurlinien, die sich drängeln oder träge sind, wenn sie per Verweilen entstehen [K3, K4, K5]; ein Fang, der zu einem ferneren Punkt springt statt zum Punkt unter dem Zeiger [K8]; PDFs, die sich nicht fangen lassen [K6, K7].

## Vorschlag für Lot

1. **Ein Eingabefeld am Zeiger, eine Schreibweise.** Nach dem ersten Punkt einfach tippen: „3,65“ Länge, „<30“ Winkel, „3,65<30“ beides, „@1,20;0,80“ relativ (Semikolon, weil das Komma die Dezimalstelle ist), „2*1,25+0,115“ gerechnet, „365cm“ mit Einheit. Der Winkel darf zuerst kommen. Tab springt zwischen Länge und Winkel. Das vereint Archicads Tracker, Rhinos Schreibweise und Allplans Rechnen und schließt Revits Lücke.
2. **Vorhersagbarer Fang.** Der Punkt unter dem Zeiger gewinnt immer; Art und Name stehen am Zeiger („Ende Wand“, „Mitte“, „Schnittpunkt“). Tab blättert durch die Kandidaten. Revits Fangkürzel (SE, SM, SI, SP, SN, SO) gelten für einen Klick. Neu dazu: Mitte, Schnittpunkt zweier Wände, lotrecht, Wandflächen und Ecken der Außenkanten, Ecken von Decken und Räumen. Die Einstellungen gehören zum Projekt, nicht zur Sitzung.
3. **Bezugspunkte bewusst setzen, nicht per Verweilen.** Eine Taste (Q wie Archicad) setzt sofort einen Bezugspunkt, höchstens drei, nummeriert sichtbar; Esc löscht sie. Von dort laufen Spurlinien waagerecht, senkrecht und im Winkelraster, ihr Schnitt ist fangbar. „Von Punkt“: Bezugspunkt fangen, dann „@0,50;0“ tippen.
4. **Eine Zeichenrichtung je Bereich.** Ein Klick auf eine Kante setzt die Bezugsrichtung; rechte Winkel, Winkelraster, Rechteck und Pfeiltasten folgen ihr (für schiefwinklige Grundstücke und Altbauten).
5. **Zeichenwerkzeuge ohne Skizzenmodus.** Wände als Kette (wie heute), Rechteck, Polygon, aus Linien einer Unterlage oder aus 2D-Linien. Decken, Räume und Dächer per Klick in eine umschlossene Fläche (wie Archicads Zauberstab) oder aus gewählten Wänden. Unterlagen aus Vektor-PDF, DXF und DWG sind sofort fangbar.
6. **Achtelmeter auf Wunsch.** Ein Schalter lässt Längen im Baurichtmaß 12,5 cm einrasten, mit den Rohbaumaßen nach DIN 4172 (Öffnung + 1 cm, Pfeiler und Außenmaß − 1 cm). So entstehen im Mauerwerksbau sofort maßgerechte Wände und Öffnungen.
