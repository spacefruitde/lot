# Decken, Dächer und Treppen: Vergleich und Vorschlag

Quellen und Einzelheiten: `recherche-bauteile.md`. Was Lot heute kann: `lot-heute.md`.

## Kurzvergleich

| Thema | Revit | Archicad | Vectorworks | Allplan | Lot heute |
|---|---|---|---|---|---|
| Decke zeichnen | Skizze, Wände wählen | Polygon, Rechteck, Zauberstab | Wände wählen, Klick in Fläche | Polygonzug | „Decke aus Wänden“ |
| folgt den Wänden | ja, löscht bei Fehler, unsichtbar gebunden | unbelegt | ja, je Kante am Kern | unbelegt | nein |
| Gefälle | Formbearbeitung, falsche Dicke | nein | Entwässerungswerkzeug | unbelegt | nein |
| Dach aus Grundriss | Neigung je Kante | Mehrebenendach | aus Wänden, Kanten umschaltbar | Dachebenen | Satteldach-Befehl; Modell kann Traufe und Giebel je Kante |
| Giebelwände | fixieren, oft Ärger | zuschneiden, assoziativ | automatisch | Ebenenbindung | Wände „bis Dach“ |
| Gauben, Dachfenster | Gauben von Hand, Fenster unbelegt | Dachflächenfenster | Dachelemente | Gaube in Dachebene | im Modell, ohne Werkzeug |
| Treppe | nach Komponente | Treppenwerkzeug mit Löser | Treppenobjekt, Dialoge | Modellierer | im Modell, ohne Werkzeug |
| Normprüfung | amerikanische Faustformel | Wertebereiche, keine DIN | Min/Max | ausdrücklich nicht DIN 18065 | DIN 18065 (Prüfbericht) |
| Treppenloch | nein | halbautomatisch, nicht verbunden | unbelegt | unbelegt | von Hand in der Datei |
| Geländer automatisch | ja | ja | per Pick | nein | nein |

**Was Nutzer am meisten ärgert:** Revit schneidet die Decke für die Treppe nicht aus [60] und löscht gebundene Decken ohne Warnung [61, 62]; „Can't create roof“ ohne Ursache [63]; Giebelwände lassen sich nicht ans Dach binden [64]; in Archicad keine geneigte Decke für Gefälle [65]; Treppen gelten in Archicad, Vectorworks und Allplan als mühsam [66 bis 71].

**Wo Lot schon vorn ist:** Lot prüft Treppen nach DIN 18065 (Steigung, Auftritt, Schrittmaß, Laufbreite, Durchgangshöhe), das kann keins der vier Programme. Dächer mit Traufe und Giebel je Kante, Gauben, Dachfenster und Wände „bis unter das Dach“ sind im Modell. Es fehlen die Werkzeuge in der App.

## Vorschlag für Lot

1. **Decke folgt den Wänden, sichtbar und nie zerstörend.** Decke per Klick in eine umschlossene Fläche, aus gewählten Wänden oder als Polygon. Jede Kante zeigt ihren Bezug (Tragschicht außen, Innenfläche, frei) im Grundriss und folgt der Wand. Scheitert eine Anpassung, bleibt die Decke erhalten und wird als Hinweis markiert, statt gelöscht zu werden.
2. **Gefälle als Eigenschaft einer Schicht:** Gefälledämmung oder Gefälleestrich mit Richtung, Prozent und Abläufen; die Dicke im Schnitt und in den Mengen stimmt (Flachdach, Balkon, Bad).
3. **Dachkanten im Grundriss umschalten:** Klick auf eine Kante wechselt Traufe, Giebel, Walm; die Neigung zieht man am Griff oder tippt sie. Giebelwände reichen automatisch bis unter das Dach und bleiben gebunden. Geht eine Form nicht, sagt Lot, welche Kante stört. Flachdach mit Attika als eigene Vorlage. Gauben und Dachfenster per Klick auf die Dachfläche, Lage über temporäre Maße.
4. **Treppenwerkzeug im Grundriss:** Antritt klicken, Richtung zeigen; Lot rechnet die Steigungen aus der Geschosshöhe. Formen gerade, zweiläufig mit Halbpodest, L-Form mit Viertelpodest; Podeste einfügen und Laufbreite ziehen mit Griffen.
5. **DIN 18065 live:** Beim Zeichnen zeigt eine Ampel Steigung, Auftritt, Schrittmaß, Laufbreite und Durchgangshöhe nach Gebäudeart; Lot schlägt passende Werte vor (wie Archicads Löser, aber mit DIN-Werten).
6. **Treppenloch und Geländer automatisch:** Die Treppe schneidet die Decke darüber aus, so groß wie die Durchgangshöhe es verlangt, und bleibt verbunden (übersteuerbar). Geländer laufen an freien Seiten automatisch mit, an Wänden ein Handlauf.
