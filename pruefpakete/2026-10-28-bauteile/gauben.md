# Gauben: Vergleich und Vorschlag

Anlass: ADR 0051, Gauben sind „in revit sehr schlecht umgesetzt“. Quellen und Einzelheiten: `recherche-gauben.md`.

## Kurzvergleich

| Thema | Revit | Archicad | Vectorworks | Allplan | Lot heute |
|---|---|---|---|---|---|
| Gaube als Bauteil | nein: Wände, Dach, Öffnung | Objekt oder von Hand | ja, Dachelement | Körper in der Dachebene | ja |
| Schlepp | von Hand | Objekt | ja | Pultprofil | ja |
| Satteldach | von Hand | Objekt | ja | Satteldachprofil | nein |
| Walm | von Hand | von Hand | ja | nur Add-in | nein |
| Flach | von Hand | von Hand | unbelegt | unbelegt | Schlepp mit 0° |
| Spitz, Trapez | von Hand | von Hand | Trapez | nur Add-in | nein |
| Fledermaus | mühsam | von Hand | ja | nein | nein |
| Tonne | mühsam | von Hand | nein | Tonnenprofil | nein |
| Ochsenauge | von Hand | Objekt | nein | nein | nein |
| Ausschnitt Hauptdach | Ränder picken | Objekt: Rechteck | automatisch | Ebene neu gerechnet | automatisch |
| Wangen am Dach | fixieren, oft Fehler | trimmen, assoziativ | Teil der Gaube | an Ebenen gebunden | Teil der Gaube |
| Gaubenfenster | in eigenen Wänden | im Objekt | Symbol, nicht in Listen | in eigenen Wänden | eins, mittig, nicht in Liste |
| Bestand, Neubau | Öffnung ohne Phase | unbelegt | unbelegt | unbelegt | Dach ja, Gaube nein |
| Setzen | Teil für Teil | Klick ins Dach | Symbol ins Dach, Dialog | Polygon in Dachebene | nur in der Datei |
| Mengen | je Wand, je Dach | Raumfläche oft falsch | unbelegt | Kehl-, Ortganglänge | Wohnfläche, Abstandsfläche |

**Was Nutzer am meisten ärgert:** In Revit besteht die Gaube aus vier Teilen; „Dach kann nicht verbunden werden“ kommt und geht bei kleinen Änderungen, selbst der Moderator weiß keinen Grund [6 bis 8]; Wangen haften oft erst nach Teilen am Dach, nach dem Verschieben kommt der Fehler zurück [9, 11]; die Gaubenöffnung hat keine Phase, eine neue Gaube schneidet schon das Bestandsdach [10]. Archicads Gaubenobjekte sind zu starr (Neigung bis 35°, Loch nur als Rechteck, Unterkante immer auf dem Dach), Profis bauen von Hand [18, 22 bis 26]; Wangen muss man im Grundriss von Hand teilen [27, 28], die Raumfläche unter der Gaube stimmt oft nicht [29]. Vectorworks setzt Gauben über ein Fenstersymbol und einen Dialog ohne Vorschau, „rather confusing“, das Fenster fehlt in Listen [31]. In Allplan hängen Wangen an der falschen Ebene, Kehlen der Dachhaut werden unsauber, die Dachlandschaft muss neu erzeugt werden [38 bis 41].

**Wo Lot schon vorn ist:** Die Gaube ist ein Bauteil, Ausschnitt und Wangen rechnet Lot (wie Vectorworks, anders als Revit). Wohnfläche und Abstandsflächen rechnen die Gaube mit, der Grundriss schneidet die Wangen an der Schnitthöhe, ohne Teilen von Hand. Es fehlen Formen, Werkzeug und Bauzustand.

## Vorschlag für Lot

1. **Eine Gaube, die Form ist eine Eigenschaft.** Schlepp (heute), Flach, Satteldach, Walm, Spitz, Trapez, Tonne, Fledermaus, Ochsenauge. Die Form wechselt im Inspektor, Lage und Breite bleiben. Das sind mehr Formen als bei jedem der vier (Vectorworks hat fünf), das Holzbau-CAD Dietrich's hat sieben [43].
2. **Den Anschluss rechnet Lot, nie von Hand.** Ausschnitt im Hauptdach entlang der echten Kehlen (nicht nur als Rechteck), Wangen vom Hauptdach bis unter das Gaubendach, Vorderwand mit Brüstung über der Deckung, Überstand vorn und seitlich getrennt. Gaubendach mit dem Aufbau des Hauptdachs oder einem eigenen (Flachdachgaube mit Gefälle und Attika nach ADR 0051).
3. **Nie still scheitern.** Ragt die Gaube über den First, in den Ortgang oder ist sie steiler als das Dach, bleibt sie stehen und Lot sagt am Bauteil, woran es liegt. Kein „Dach kann nicht verbunden werden“.
4. **Setzen mit einem Klick auf die Dachfläche**, in Grundriss, Schnitt oder 3D: Form aus einer Bildliste, Werte aus einem Gaubentyp im Bürostandard (ADR 0048, wie die Vorlagen in Dietrich's [42]). Lage über temporäre Maße zu Ortgang, Traufe und Nachbargaube; Verschieben bleibt in der Dachfläche. Griffe für Breite, Vorderwand, Traufhöhe und Neigung. Zwei von drei Höhenwerten eingeben (Traufhöhe vorn, Neigung, Anschlusshöhe am Hauptdach), den dritten rechnet Lot und zeigt ihn.
5. **Gaubenfenster sind echte Fenster.** Die Vorderwand nimmt normale Öffnungen mit Tür- und Fenstertyp auf, auch mehrere und bodentief; sie stehen in der Tür- und Fensterliste, Dachfenster ebenso. Vorhandene `DormerWindow` wandelt Lot beim Laden um.
6. **Bestand und Umbau.** Gaube und Dachfenster bekommen einen Bauzustand. Eine neue Gaube im Bestandsdach zeigt den Ausschnitt im Umbauplan gelb als Ausbruch; eine abgebrochene Bestandsgaube schließt das Dach im Neuzustand. Revit kann das nur über Abzugskörper [10].
7. **Pläne und Mengen ohne Nacharbeit.** Grundriss wie heute (geschnitten bis Schnitthöhe, darüber gestrichelt); Schnitt und Ansicht mit Kehlen und Ortgang. Mengen je Gaube: Dachfläche, Wangenfläche, Kehl-, Ortgang- und Trauflänge (wie Allplans Dachhaut [35]).
8. **Nicht in Version 1:** Sparren, Auswechslungen, Lattung (Holzbau, nach LP 4).

**Im Modell:** `Dormer` bekommt eine Form (`DormerShape`, Glossar „Gaubenform“) mit den Werten je Form, `front_overhang` und `side_overhang`, `roof_type` (leer heißt wie das Hauptdach) und `phase`. Öffnungen dürfen eine Gaube als Wirt haben. Gespeichert bleiben Traufhöhe und Neigung; die Anschlusslinie bleibt abgeleitet.

## Fragen an Ben

1. **Reihenfolge der Formen.** Vorschlag: zuerst Schlepp, Flach, Satteldach, Walm und Spitz, danach Trapez, Tonne, Fledermaus und Ochsenauge, ohne Änderung am Modell.
2. **Gaube in der Flucht der Außenwand** (Traufe unterbrochen, auch Zwerchhaus): gehört das zur Gaube? Vorschlag: ja, als Lage „in der Außenwand“; die Außenwand darunter läuft bis unter das Gaubendach durch. Ein Zwerchgiebel mit eigenem First quer zum Hauptfirst kommt später als Dachform.
3. **Welche Höhe führt?** Vorschlag: Traufhöhe vorn und Neigung wie heute; die Anschlusshöhe am Hauptdach zeigt Lot an, und tippt man sie, ändert sich die Neigung. Bei Satteldachgauben wahlweise die Firsthöhe statt der Neigung.
4. **Umbauplan.** Vorschlag: neue Gaube im Bestandsdach zeigt den Ausschnitt gelb als Ausbruch und die Gaube rot, wie eine neue Öffnung in einer Bestandswand (ADR 0031). Passt das?
5. **Festsetzungen zu Gauben** (etwa Summe der Gaubenbreiten, Abstand vom Ortgang, Abstand unter dem First, Dachneigung): soll Lot sie prüfen? Vorschlag: Werte je Projekt eintragbar, Lot meldet Abweichungen im Prüfbericht, sperrt aber nichts.
