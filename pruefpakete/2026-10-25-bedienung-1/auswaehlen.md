# Auswählen: Vergleich und Vorschlag

Quellen und Einzelheiten: `recherche-auswaehlen.md`. Was Lot heute kann: `lot-heute.md`.

## Kurzvergleich

| Thema | Revit | Archicad | Vectorworks | Allplan | Lot heute |
|---|---|---|---|---|---|
| Überfahren | hervorgehoben, Name im Tooltip | Info-Tag | eigene Farben je Zustand | orange | nichts |
| Überlappende | Tab | Tab | J öffnet Liste | Schalter | nein |
| Hinzufügen, entfernen | Strg, ⇧ | ⇧ | ⇧ schaltet um | Strg schaltet um | ⇧ schaltet um |
| Rahmen | links nach rechts innen, rechts nach links kreuzend | einstellbar, umgekehrt zu Revit | innen, Alt kreuzt | wie Revit | nein |
| Lasso, Polygon | nein (angenommen 10/2026) | Polygon | beides | Zaun | nein |
| Wandkette | Tab | nein | Befehl | nein | nein |
| Alle gleichen | SA, Filter nach Kategorie | ⌘A je Werkzeug, Suchen und Auswählen | Ähnliche auswählen | Filter | nein |
| Teile | nach Fläche; Schichten nur als „Teile“ | Pet-Palette | Scheitelpunkt | Schichtmodus | Griffe an Wandenden |
| Gesperrt, Unterlagen | fünf Schalter | wählbar, nicht änderbar | wählbar, ohne Griffe | Referenz nicht änderbar | Varianten nicht wählbar |

**Was Nutzer am meisten ärgert:** Ziehen verschiebt versehentlich, weil ein Klick auf ein Bauteil statt eines Rahmens beginnt (Archicad über Decken [AC1], Revit warnt selbst [R11]); kein Lasso in Revit [RI1]; nur eine vorherige Auswahl [RI2]; Filter nur nach Kategorie, nicht nach Typ [RI3]; bei Unterlagen wird immer das Bild gewählt [VF1].

## Vorschlag für Lot

1. **Ziehen verschiebt nur, was gewählt ist.** Ziehen im Leeren oder auf einem nicht gewählten Bauteil zieht einen Rahmen auf; verschoben wird nur die Auswahl oder über Griffe. Auf dem Trackpad sind Klick und kleines Ziehen schwer zu trennen, deshalb ist das hier wichtiger als in Windows-Programmen.
2. **Rahmen wie Revit, dazu Lasso:** links nach rechts wählt, was ganz innen liegt, rechts nach links auch Angeschnittenes (so auch Allplan, Qonic, Snaptrude, Rhino). Mit ⌥ wird der Rahmen zum Lasso.
3. **⇧ fügt hinzu oder nimmt heraus**, mit Klick und mit Rahmen (Mac-üblich; Revit braucht dafür zwei Tasten).
4. **Vorhervorhebung immer**, mit kurzem Namen am Zeiger („Wand · KS 17,5 · 4,20 m“).
5. **Tab für alles Mehrdeutige, mit Zähler:** Tab blättert durch die Kandidaten unter dem Zeiger, die Wandkette ist einer davon (wie Revit). Am Zeiger steht etwa „2 von 4: Wandkette (3 Wände)“.
6. **Gleiche auswählen** mit einem Kürzel (SA wie Revit): gleicher Typ im Geschoss, ein zweites Mal im ganzen Projekt. Der Inspektor zeigt danach die Anzahl je Art und Typ als Schalter zum Herausfiltern (die Lücke bei Revit [RI3]).
7. **Teile per Doppelklick:** Doppelklick auf ein gewähltes Bauteil wählt ein Teil darin (Schicht, Öffnung, Wandende, Anschlussknoten), Esc geht zurück (wie Figma und Qonic). Das ersetzt Revits „Teile erstellen“.
8. **Klare Sperren:** Unterlagen und Referenzmodelle sind nie wählbar, aber fangbar (wie Rhino). Fixierte Bauteile sind wählbar zum Lesen, lassen sich aber nicht verschieben, beim Überfahren zeigt ein Schloss den Grund.
9. **Auswahlverlauf:** ⌘← holt die vorige Auswahl zurück, mehrmals hintereinander. Benannte Auswahlen kommen später.
10. **Eine Auswahl für alle Ansichten:** Was im Grundriss gewählt ist, ist auch im 3D, im Schnitt und in der Ansicht gewählt und umgekehrt (Auswahl in 3D und Schnitt folgt mit dem Thema Navigation).
