# Öffnungen: Vergleich und Vorschlag

Quellen und Einzelheiten: `recherche-oeffnungen.md`. Was Lot heute kann: `lot-heute.md`.

## Kurzvergleich

| Thema | Revit | Archicad | Vectorworks | Allplan | Lot heute |
|---|---|---|---|---|---|
| Setzen | 1 Klick, Seite per Zeiger | 2 Klicks, Richtung beim zweiten | 1 Klick, Versatzmodus | Klick, Abstand in der Dialogzeile, Breite | 1 Klick, mittig am Zeiger |
| Abstandsbezug | Achse der Querwand, umstellbar | Abstandsführung, misst am Rahmen | Bezugspunkt im Versatzmodus | nächster Bezugspunkt, per Klick wechseln | Wandanfang, nur im Inspektor |
| Maß | Nennmaß, Rohbau nur per Familie | Wandöffnung, Einheit, Durchgang, Flügel | Flügel, Einheit, Rohbau | Öffnung ist Rohbau | Rohbaumaß |
| Brüstung | über Ebene | Wand oder Geschoss, Fußbodenaufbau nur Anzeige | Brüstung oder Sturz | Ebenentechnik | über Wandunterkante (OKFF) |
| DIN links/rechts | fehlt, nicht auswertbar | unbelegt | Anschlagsparameter | unbelegt | ja, mit Aufschlagseite |
| Öffnung ohne Füllung | unbelegt | leere Öffnung, Nische | unbelegt | Öffnung zuerst, Füllung danach | Durchgang, nur in der Datei |
| Umsetzen in andere Wand | „Neuen Host wählen“ | nur gleiche Ebene | unbelegt | unbelegt | nein |
| Typen | Familien | GDL-Objekte | Plug-ins mit Stilen | Makros, SmartParts | keine |

**Was Nutzer am meisten ärgert:** Aufschlag in Listen nicht auswertbar (Revit [10, 11], Archicad [27]); Abstand wird am Rahmen statt an der Rohbauöffnung gemessen [23]; Schnitt und Grundriss zeigen verschiedene Detailgrade [24]; Fenster in Wänden schwer zu wählen [25, 36]; Umsteiger vermissen „erst Öffnung, dann Füllung“ [29].

**Wo Lot schon gut ist:** Rohbaumaß als Grundlage, DIN links/rechts als echte Eigenschaft, Darstellung je Maßstab ohne Familienbau, Öffnungen über der Schnitthöhe gestrichelt, Tür- und Fensterliste mit Anschlag.

## Vorschlag für Lot

1. **Die Öffnung ist das Bauteil, die Füllung ist optional.** Wandöffnung mit Rohbaumaß zuerst (Entwurf, LP 1 bis 2), Tür oder Fenster füllen sie später, ohne dass sich die Öffnung ändert (wie Allplan). Dasselbe Bauteil kann Durchgang ohne Füllung oder Nische mit Tiefe sein. Schlitze folgen später.
2. **Setzen mit Abstand und Bezug:** Beim Überfahren zeigt ein temporäres Maß den Abstand zum nächsten Bezug, immer an der Rohbauöffnung gemessen. Tab wechselt den Bezug: Wandende (Rohbau), Achse der Querwand, Kante der Nachbaröffnung. Eine Zahl tippen setzt den Abstand. Ein Fang „mittig zwischen“ setzt die Öffnung in die Mitte zwischen zwei Bezügen.
3. **Seite beim Setzen und danach:** Die Zeigerseite bestimmt den Aufschlag (wie heute); die Leertaste schaltet beim Setzen und bei gewählter Tür durch die vier Lagen (Seite und DIN links/rechts), wie Revit, aber mit allen vier Zuständen.
4. **Maße wie in der Werkplanung:** gespeichert wird das Rohbaumaß; lichte Durchgangsbreite und Baurichtmaß nach DIN 18100 stehen daneben (aus dem Typ, siehe `typen.md`). Die Brüstung zeigt Lot über OKFF und über Rohfußboden, aus dem Fußbodenaufbau der Decke.
5. **Umsetzen per Ziehen** in jede andere Wand, Positionsnummer, Rohbaumaß und Brüstung bleiben; wird die Wand geteilt, bleibt die Öffnung auf ihrer Seite, wird die Wand gelöscht, fragt Lot nach.
6. **Typen für Türen und Fenster** (siehe `typen.md`): Art (Drehflügel, zweiflügelig, Fenstertür, Hebeschiebetür), Rohbaumaß, Rahmen, Anschlag in mehrschaligem Mauerwerk, Fensterbank, Rollladenkasten. Wer will, setzt eine Öffnung frei mit Maßen ohne Typ.
