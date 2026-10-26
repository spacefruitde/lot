# Navigation: Vergleich und Vorschlag

Quellen und Einzelheiten: `recherche-navigation.md`. Was Lot heute kann: `lot-heute.md`.

## Kurzvergleich

| Thema | Revit | Archicad | Vectorworks | Allplan | Lot heute |
|---|---|---|---|---|---|
| Browser | Baum mit Tabs, Ordnungsschemata | 4 Mappen | Palette mit 6 Tabs | modaler Dialog | keiner |
| Suche | ja, live | ja | Name, Tags | unbelegt | Befehlspalette ⌘K, nur Befehle |
| Geschoss wechseln | Browser | Tab-Menü, hoch/runter | ⌘Pfeil | Teilbildstatus | Menü „Geschoss“ |
| Geschoss darunter | Unterlage, halbton | Trace, folgt mit | Ebenen grau | passives Teilbild | nein |
| Geschoss anlegen | nur in Schnitt oder Ansicht | Dialog | Stories | Bauwerksstruktur | „Geschoss hinzufügen“ |
| Schnitt anlegen | Werkzeug, Knick, Tiefe | Werkzeug, abgesetzt, Reichweite | aus Ebene oder Clip Cube | Polygonzug | nur Projektdatei |
| Marke zur Ansicht | Doppelklick | verknüpfte Marke | unbelegt | unbelegt | nein |
| Ansicht auf mehreren Plänen | nein, Duplikat | Kopie | ja | unbelegt | ja (in der Datei) |
| Nebeneinander | Tabs, Kacheln | Tabs, nur ein Grundriss | Ansichtsbereiche | unbelegt | eine Ansicht |
| Verlauf | Strg+Tab | F2 bis F7, Zoom zurück | 50 Ansichten | unbelegt | nein |
| 3D-Zuschnitt | Schnittbox | Filter, Schnittebenen | Clip Cube | unbelegt | nein |
| Mac | nein | nativ | nativ | nein | nativ |

**Was Nutzer am meisten ärgert:** Revit erlaubt eine Ansicht nur auf einem Plan [65] und kann Ansichten über Pläne kaum ausrichten [66]; Archicads Projektmappe gegen Ansichtsmappe verwirrt, Bildschirmänderungen laufen still auseinander [70]; Archicad hat nur ein Grundrissfenster [23, 71]; Allplan braucht Dialoge für jeden Wechsel [74]; Archicads Trackpad-Zoom ruckelt [72].

## Vorschlag für Lot

1. **Seitenleiste „Projekt“** links, Mac-typisch wie im Finder oder in Xcode: Geschosse, Grundrisse, Schnitte und Ansichten, Lageplan, Planblätter, Listen (Flächen, Türen und Fenster, Prüfbericht). Ein- und ausblenden mit ⌃⌘S wie in macOS üblich, mit Suche oben.
2. **Ein Sprungbefehl für alles:** ⌘K findet neben Befehlen auch Geschosse, Schnitte, Blätter und Räume („Bad 1.OG“). Zurück und Vorwärts mit ⌘[ und ⌘], wie in Xcode und Safari.
3. **Geschosse als Leiste:** EG, 1.OG, DG mit Höhen direkt editierbar, ⌘↑ und ⌘↓ wechseln (wie Vectorworks). Das Geschoss darunter ist im Grundriss blass zu sehen und läuft beim Wechsel mit (wie Archicad), je Ansicht abschaltbar.
4. **Schnitte und Ansichten im Grundriss zeichnen:** Werkzeug Schnitt (zwei Klicks, Blickrichtung mit Leertaste, Tiefe als Griff), Knick mit einem Klick auf die Linie. Doppelklick auf die Schnittmarke öffnet den Schnitt, ⌘[ führt an dieselbe Stelle im Grundriss zurück.
5. **Planblätter in der App:** Ansichten aus der Seitenleiste aufs Blatt ziehen; eine Ansicht darf auf mehreren Blättern liegen, eine Platzierung ist die „Heimat“, auf die Schnittmarken verweisen (löst Revits Streit); gleiche Lage über Blätter per gemeinsamem Anker.
6. **Nebeneinander mit gekoppelter Auswahl:** Grundriss und Schnitt oder zwei Geschosse nebeneinander (Teilen wie in macOS); die Auswahl ist überall dieselbe. Eine Ansicht heißt „Ansicht“; weicht der Bildschirm von der gespeicherten ab, sagt Lot es („geändert: sichern oder verwerfen“).
7. **3D aus dem Grundriss:** Eine Taste öffnet 3D mit Schnittbox um die Auswahl oder das Geschoss; Trackpad wie heute (zwei Finger drehen, ⇧ verschieben, spreizen).
