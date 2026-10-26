# Wände: Vergleich und Vorschlag

Quellen und Einzelheiten: `recherche-waende.md`. Was Lot heute kann: `lot-heute.md`.

## Kurzvergleich

| Thema | Revit | Archicad | Vectorworks | Allplan | Lot heute |
|---|---|---|---|---|---|
| Bezugslinie | 6 Lagen, Wechsel verschiebt nicht, Umdrehen kann springen | 6 Lagen, Wechsel in den Einstellungen verschiebt die Wand, Extra-Befehl nicht | 4 Lagen, Wand oder Kern, Verhalten unklar | frei je Schicht, Änderungsmodus lässt Wand liegen | 3 Lagen, Wechsel verschiebt die Wand |
| Zeichnen | Linie bis Ellipse, aus Linien und Flächen, je Raum | Kette mit Bögen, Rechteck, Polywand, Zauberstab | Polylinie, Rechteck, aus Räumen | Linie, Rechteck, N-Eck, aus 2D | Kette, Bogen, Länge tippen |
| Anschlüsse | Priorität 1 bis 5 je Schicht, Handwerkzeug bis 4 Wände | Baustoffpriorität 0 bis 999 projektweit, Reihenfolge je Wand | über den Kern, Handwerkzeug T, L, X | Priorität je Schicht, nur im Teilbild | automatisch nach Schichtrang, alle Fälle, keine Ausnahmen |
| Aufbau bearbeiten | Typ-Editor, Umschlag an Enden und Öffnungen | Mehrschicht-Attribut | Stile, nach Stil oder Exemplar | Favoriten ohne Verknüpfung | nur in der Projektdatei |
| Höhe | Ebene plus Versatz, an Dach und Decke fixieren | Geschoss plus Versatz, Zuschneiden an Dach | Geschossebenen, je Komponente | je Schicht an beliebige Ebene | Geschoss, feste Höhe oder Dach |
| Teilen, Verbinden | Teilen ja | Teilen, Cmd-Klick | Umformen | teilweise löschen | nein |

**Was Nutzer am meisten ärgert:** unsaubere Anschlüsse mehrschichtiger Wände, besonders T-Stöße mit Putz, in allen vier Programmen [F1, F6, F11, F12]; Anschlusswerkzeuge mit Grenzen (Revit höchstens 4 Wände [R5], Archicad Prioritäten projektweit je Baustoff [A10]); Wände springen beim Umdrehen oder Wechseln der Bezugslinie [F5, A1]. Autodesk empfiehlt 2027 sogar, Bekleidung, Kern und Innenbekleidung als drei getrennte Wände zu zeichnen, damit Anschlüsse stimmen [R4].

**Wo Lot schon besser ist:** Anschlüsse ohne Einstellungen nach Schichtrang für L, T, Kreuzung, Y und Stützen, getestet mit 200 Zufallsfällen (ADR 0017). Das ist genau der Punkt, an dem die anderen scheitern. Darauf bauen die Vorschläge auf.

## Vorschlag für Lot

1. **Die Bezugslinie ist eine Lesart, keine Geometrie.** Sechs Lagen wie Revit und Archicad: Mitte, Außenfläche, Innenfläche, Kernmitte, Kern außen, Kern innen. Ein Wechsel verschiebt nie etwas. Ändert sich der Aufbau, bleibt die Bezugslinie stehen, und eine Vorschau zeigt, nach welcher Seite die Wand wächst. **Umdrehen** tauscht innen und außen, die Wand bleibt liegen. Besser als alle vier, weil keiner beides garantiert.
2. **Zeichnen:** Kette (wie heute), Rechteck, geschlossenes Polygon, Wand aus Linien der Unterlage (DXF, DWG) und aus 2D-Linien; Bezugslinie beim Zeichnen mit einer Taste umschalten (wie Archicad C); Versatz zur geklickten Linie. Zahleneingabe wie in `bearbeiten.md`.
3. **Anschlüsse bleiben automatisch, aber erklärbar und übersteuerbar am Knoten.** Jeder Stoß ist ein Knoten, auswählbar wie ein Bauteil, für beliebig viele Wände. Am Knoten steht, warum es so aussieht („Mauerwerk schneidet Dämmung“), und dort lassen sich Ausnahmen setzen: welche Wand durchläuft, nicht verbinden. Die Ausnahme bleibt am Knoten, auch wenn Wände bewegt werden. Wände verschiedener Bauzustände verbinden sich nicht miteinander (Revit-Ärger [RP4 in recherche-bearbeiten.md]).
4. **Wandaufbau in der App:** Editor für Schichten mit Material, Dicke, Funktion und Kern. Der Aufbau gehört zum Typ (wie Revit), Bezugslinie und Höhen zur einzelnen Wand.
5. **Höhe:** Unterkante und Oberkante gebunden an Geschoss mit Versatz (wie heute), dazu an die Decke darüber, ans Dach (wie heute) oder fest. Später je Schicht (Tragschicht bis Rohdecke, Putz bis Unterkante Decke), wie Allplan und Vectorworks es können.
6. **Teilen und Verbinden:** Teilen an einem Punkt, Öffnungen bleiben auf ihrer Seite; zwei fluchtende Wände gleichen Aufbaus verbinden.
7. **Später, als Idee:** Putz und Fliesen vom Raum aus („diese Raumseite“, „ganzer Raum“, wie Qonic [N5]), gespeichert als Schicht der Wand.
