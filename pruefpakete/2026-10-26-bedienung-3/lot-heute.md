# Lot heute: Navigation, Sichtbarkeit, Decken, Dächer und Treppen

Stand nach M5 (Branch `m5-bauantrag`), aus dem Code gelesen. Grundlage für den Vergleich in diesem Paket.

## Navigation

| Was | Heute in Lot |
|---|---|
| Projektbrowser | keiner; Projektname und Geschoss stehen in der Titelzeile |
| Ansichten in der App | Grundriss und 3D (⌘1, ⌘2); Schnitte, Ansichten, Lageplan und Planblätter gibt es nur als Ausgabe der Kommandozeile |
| Geschosse | wechseln und anlegen über das Menü „Geschoss“ in der Werkzeugleiste oder die Befehlspalette; Höhen im Inspektor ohne Auswahl |
| Maßstab | Menü in der Werkzeugleiste; der Grundriss zeigt die Darstellung je Maßstab |
| Schnitte und Ansichten anlegen | nur in der Projektdatei (Linie, Blickrichtung, Tiefe); im Grundriss erscheinen die Schnittlinien |
| Planblätter | nur in der Projektdatei; Ausgabe als PDF, SVG, PNG, DXF, DWG über die Kommandozeile |
| Fenster, Tabs | ein Fenster, eine Ansicht; kein Zurück |
| 3D | zwei Finger drehen, mit ⇧ verschieben, spreizen zoomt; keine Schnittbox, keine Geschosse ausblenden |
| Sprung zu einem Ort | nur aus dem Prüfbericht (M5) |

## Sichtbarkeit

| Was | Heute in Lot |
|---|---|
| je Ansicht | nichts einstellbar; die Darstellung folgt dem Bürostandard (Detailgrad, Stifte, Schraffuren je Maßstab) |
| Bauzustände | Umbauplan, Bestandsplan, Neuzustand nur als Ausgabe; im Grundriss der App immer alles farbig |
| Varianten | aktive Variante gilt überall (Befehl „Variante wechseln“) |
| Unterlagen, Referenzmodelle | sichtbar mit Deckkraft je Unterlage; Referenzmodelle immer hellgrau |
| Geschoss darunter | nein |
| ausblenden, isolieren | nein |
| Hinweise aus dem Prüfbericht | ein- und ausblendbar (⇧⌘H) |

## Decken, Dächer, Treppen in der App

| Was | Heute in Lot |
|---|---|
| Decke | Befehl „Decke aus Wänden“: Oberkante auf OKFF, folgt der Außenkante der Tragschicht darunter; Durchbruch als Rechteck mit zwei Klicks; Umriss nicht bearbeitbar |
| Dach | Befehl „Satteldach über dem Geschoss“; Traufhöhe und Neigung im Inspektor; Walm-, Pult-, Flachdach, Gauben und Dachfenster nur in der Projektdatei |
| Treppe | nur in der Projektdatei; Prüfung nach DIN 18065 läuft (Prüfbericht) |
| Geländer | nur in der Projektdatei |
| Aufzug, Gelände, Grundstück | nur in der Projektdatei |
