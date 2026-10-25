# Paket Bedienung 2: Zeichnen und Fangen, Öffnungen, Typen und Eigenschaften

26.10.2026. Etwa 20 Minuten, nur lesen und antworten, kein Mac nötig. Zweites Paket nach deiner Entscheidung vom 25.10.2026 (ADR 0042). Paket 1 (Auswählen, Bearbeiten, Wände) liegt daneben in `pruefpakete/2026-10-25-bedienung-1`.

## Was drin ist

- **`zeichnen.md`**, **`oeffnungen.md`**, **`typen.md`**: je Thema ein Kurzvergleich (Revit, Archicad, Vectorworks, Allplan, Lot heute), was Nutzer dort am meisten ärgert, und mein Vorschlag für Lot. Je eine Seite.
- `lot-heute.md`: was Lot bei diesen Themen heute kann.
- `recherche-zeichnen.md`, `recherche-oeffnungen.md`, `recherche-typen.md`: die Recherche mit Quellen. Nur zum Nachschlagen.

Revit und Archicad habe ich nicht selbst bedient; alles stammt aus den Hilfen und Foren. Wo dein Alltag etwas anderes sagt, gilt dein Alltag. Die Leitlinie ist dieselbe wie in Paket 1: Revit-Stärken übernehmen, belegten Ärger vermeiden, Mac-Konventionen nutzen.

## Fragen

Antwort jeweils „Vorschlag ok“, „wie Revit“, „wie Archicad“ oder ein Satz, was anders sein soll.

**Zeichnen und Fangen** (`zeichnen.md`)

1. **Eingabe am Zeiger:** „3,65“ Länge, „<30“ Winkel, „3,65<30“ beides, „@1,20;0,80“ relativ, „2*1,25+0,115“ gerechnet, „365cm“; der Winkel darf zuerst kommen, Tab springt zwischen Länge und Winkel. Vorschlag ok?
2. **Fangen:** Der Punkt unter dem Zeiger gewinnt immer, Art am Zeiger, Tab blättert; Revits Fangkürzel (SE, SM, SI, SP, SN, SO) für einen Klick; neu Mitte, Schnittpunkt, lotrecht, Wandflächen und Außenecken; Einstellungen im Projekt gespeichert. Vorschlag ok, und welche Fangarten nutzt du wirklich?
3. **Bezugspunkte und Richtung:** Q setzt sofort einen Bezugspunkt (höchstens drei, nummeriert), Spurlinien laufen davon waagerecht, senkrecht und im Winkelraster; ein Klick auf eine Kante setzt die Zeichenrichtung für schiefwinklige Grundrisse. Vorschlag ok?
4. **Werkzeuge ohne Skizzenmodus:** Wände als Kette, Rechteck, Polygon oder aus Linien einer Unterlage; Decken, Räume und Dächer per Klick in eine umschlossene Fläche; dazu auf Wunsch ein Achtelmeter-Fang (12,5 cm, Öffnung + 1 cm, Pfeiler − 1 cm nach DIN 4172). Vermisst du Revits Skizzenmodus, und ist der Achtelmeter-Fang nützlich?

**Öffnungen** (`oeffnungen.md`)

5. **Öffnung zuerst, Füllung später:** Die Wandöffnung mit Rohbaumaß ist das Bauteil; Tür oder Fenster füllen sie, ohne dass sich die Öffnung ändert; dasselbe Bauteil kann Durchgang oder Nische mit Tiefe sein. Passt das zu deinem Ablauf von LP 2 bis LP 4?
6. **Setzen:** Beim Überfahren steht der Abstand zum nächsten Bezug am Zeiger, immer an der Rohbauöffnung gemessen; Tab wechselt den Bezug (Wandende, Achse der Querwand, Nachbaröffnung), „mittig zwischen“ ist ein Fang, eine Zahl setzt den Abstand; die Leertaste schaltet durch die vier Lagen von Aufschlagseite und DIN links/rechts. Vorschlag ok?
7. **Brüstung:** Lot speichert die Brüstung heute über OKFF. Welchen Bezug zeigst du in Grundriss und Maßkette (über OKFF oder über Rohfußboden), und soll Lot beide zeigen? Vorschlag: gespeichert über OKFF, angezeigt nach Bürostandard, im Inspektor beide.

**Typen und Eigenschaften** (`typen.md`)

8. **Typ und Bauteil:** ein Inspektor, jedes Feld zeigt, ob der Wert vom Typ kommt oder überschrieben ist, mit Zurücksetzen und „in den Typ übernehmen“; Typwerte direkt im Inspektor mit „wirkt auf 14 Wände“; der Typ legt fest, was überschreibbar ist (beim Wandaufbau nie die Schichten). Lieber so oder streng wie Revit?
9. **Typen verwalten:** Tabelle aller Typen mit Nutzungszahl, Vorschau, Duplizieren, Tauschen, „unbenutzt“; Löschen nur mit Ersatz; Büro-Bibliothek, aus der man einzelne Typen mit Vorher-Nachher-Vergleich holt. Vorschlag ok?
10. **Eigenschaften und Klassifikation:** Welche eigenen Eigenschaften brauchst du in LP 1 bis 4 wirklich (etwa Kennzeichen, Brandschutz, Schallschutz, U-Wert, DIN-276-Kostengruppe, IFC-Klasse)? Vorschlag: Kennzeichen und IFC-Klasse jetzt, Brandschutz und Schallschutz mit den Typen, DIN 276 mit den Kosten nach V1.

## Wie es weitergeht

Deine Antworten aus beiden Paketen werden ADRs. Dann baue ich die Bedienung nach diesen Entscheidungen, beginnend mit Auswählen, Bearbeiten und Zeichnen, und teste die App selbst per Bildschirmsteuerung. Noch offene Themen für einen dritten Vergleich: Navigation (Projektbrowser, Geschosse, Schnitte, Blätter), Sichtbarkeit, Decken, Dächer und Treppen in der App.
