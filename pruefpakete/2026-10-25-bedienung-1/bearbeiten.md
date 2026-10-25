# Bearbeiten: Vergleich und Vorschlag

Quellen und Einzelheiten: `recherche-bearbeiten.md`. Was Lot heute kann: `lot-heute.md`.

## Kurzvergleich

| Thema | Revit | Archicad | Vectorworks | Allplan | Lot heute |
|---|---|---|---|---|---|
| Aufruf | Zwei-Buchstaben-Kürzel (MV, CO, RO, MM, AL, TR, SL, OF, AR) | Cmd-Kürzel, Pet-Palette am Griff | Werkzeuge, Kürzel | Kontextmenü, Griffe | nur Ziehen und Löschen |
| Wand 50 cm schieben | 2 Klicks plus Zahl, oder temporäres Maß anklicken | 2 Klicks plus Tracker | 2 Klicks plus Datenzeile | 2 Klicks plus Delta | Ziehen mit Fangen, keine Zahl |
| Zahleneingabe | Zahl tippen und Enter | Tracker, rechnen mit 20+, 3* | Datenzeile am Zeiger, volle Ausdrücke mit Einheiten | Delta-Punkt | nur Wandlänge beim Zeichnen |
| Maße steuern das Modell | ja, temporäre Maße, Schloss, EQ | nein | 2D-Zwangsbedingungen | nein | nein |
| Mehrfachauswahl ändern | gemeinsame Parameter, keine temporären Maße | ein Element maßgeblich, kein „verschieden“ | leer heißt verschieden, einzeln blättern | gemeinsame Werte | einige Befehle für alle |
| Wiederholen | Enter | Cmd+B | R | Menü | nein |

**Was Nutzer am meisten ärgert:** Anschlüsse reißen beim Strecken ab [AP3], Wände verbinden sich ungewollt [RP4]; temporäre Maße fehlen bei Mehrfachauswahl und haben kein EQ [R8, RP2]; Schlösser machen mehr Probleme als sie lösen [RP6]; „verschieden“ fehlt bei Mehrfachauswahl [AP4]; der Versatz muss jedes Mal neu getippt werden [AP2]; Ansichtsänderungen landen im Undo [VP1]; die Undo-Liste wird gelöscht [A10].

## Vorschlag für Lot

1. **Kürzel aus Revit als Voreinstellung**, zwei Buchstaben: MV verschieben, CO kopieren, RO drehen, MM spiegeln, AR Reihe, OF versetzen, AL ausrichten, TR stutzen, SL teilen, dazu WA Wand, DR Tür, WN Fenster, RM Raum, DI Bemaßung. Dein Muskelgedächtnis zählt mehr als neue Ein-Tasten-Kürzel. Die Befehlspalette (⌘K) findet alles über den Namen.
2. **Eine Zahleneingabe für alles:** In jeder laufenden Aktion einfach tippen, ohne ins Feld zu klicken. Einheiten („50cm“, „1,26“), Rechnen („2,40+0,365“, „3,6/2“), relativ und absolut, „3x“ und „3/“ für Reihen, Tab wechselt Länge und Winkel. Der Wert bleibt korrigierbar, bis die nächste Aktion beginnt. Jedes Programm hat nur einen Teil davon (SketchUp, Archicad, Vectorworks).
3. **Temporäre Maße wie Revit, ohne dessen Lücken:** Nach der Auswahl stehen Maße zum nächsten Bezug (Wandfläche, Achse, Kern; Bezug per Klick wechseln). Maßzahl anklicken, Wert tippen, das Bauteil rückt. Auch bei Mehrfachauswahl, mit EQ zum Mittig-Setzen. Keine Schlösser und Zwangsbedingungen in V1.
4. **Direkt am Element, Mac-typisch:** Ziehen verschiebt, was gewählt ist (siehe `auswaehlen.md`, Punkt 1), mit ⌥ entsteht eine Kopie, beim Ziehen tippen setzt den Abstand. Pfeiltasten schieben die Auswahl um 1 cm, mit ⇧ um 10 cm. Griffe an Enden und Kanten statt einer Palette.
5. **Anschlüsse bleiben erhalten.** Verschieben und Strecken halten L, T und Kreuzungen (das kann Lot beim Verschieben schon); bewusst trennen mit einer Taste beim Ziehen. Kein Bauteil bekommt dabei eine neue Kennung.
6. **Ehrliche Mehrfachauswahl:** Unterschiedliche Werte zeigen „verschieden“ mit Spanne (etwa „2,40 bis 2,75“); relative Eingabe „+10 cm“ gilt für alle.
7. **Verlässliches Undo und Wiederholen:** Ansicht und Auswahl kommen nie in den Verlauf, der Verlauf wird nie gelöscht. „Wiederholen“ (⌘Y oder Enter ohne laufende Aktion) wiederholt den letzten Befehl mit demselben Abstand, etwa denselben Versatz.
8. **Befehle:** Kopieren, Drehen, Spiegeln, Reihe, Versetzen, Ausrichten, Stutzen und Dehnen, Teilen, Verbinden, dazu Kopieren und Einfügen über die Zwischenablage, auch in ein anderes Geschoss an dieselbe Stelle.
