# Lot heute: Auswählen, Bearbeiten, Wände

Stand nach M5 (Branch `m5-bauantrag`), aus dem Code gelesen. Grundlage für den Vergleich in diesem Paket.

## Auswählen

| Was | Heute in Lot |
|---|---|
| Klick | wählt das Element unter dem Zeiger (Öffnung vor Wand vor Raum); Klick ins Leere hebt die Auswahl auf |
| ⇧-Klick | fügt hinzu oder nimmt heraus |
| ⌘A | alles im Geschoss |
| Vorhervorhebung beim Überfahren | nein (nur Fangpunkte beim Zeichnen) |
| Überlappende durchblättern | nein |
| Rahmen, Lasso | nein |
| verbundene Wände als Kette | nein |
| alle gleichen, Filter | nein |
| in 3D, Schnitt, Ansicht | nein, nur im Grundriss |
| gesperrte Elemente | nein; Bauteile anderer Varianten sind nicht anklickbar |
| nach der Auswahl | Inspektor rechts mit Parametern; Griffe an Wandenden |

## Bearbeiten

| Was | Heute in Lot |
|---|---|
| Verschieben | Auswahl ziehen, mit Fangen; anschließende Wände ziehen an Ecken mit, an T-Stößen werden sie verlängert |
| Wandende | Griff ziehen |
| Zahlen | Wandlänge beim Zeichnen tippen und ⏎; Parameter im Inspektor (Länge, Höhe, Versatz, Öffnungsmaße) |
| Kopieren, Drehen, Spiegeln, Reihe | nein |
| Versetzen, Ausrichten, Stutzen, Dehnen, Teilen, Verbinden | nein |
| temporäre Maße | nein; beim Zeichnen steht die Länge am Zeiger |
| Mehrfachauswahl ändern | Wandaufbau, Bezugslinie, Bauzustand, Umdrehen für alle gewählten |
| Zwischenablage | nein |
| Undo | jede Änderung ein Schritt, auch Verschieben mit mitgezogenen Wänden |
| Befehle | Befehlspalette ⌘K, Kürzel W, B, A, S, T, F, R, D, L, X, K, M, ⇧F |

## Wände

| Was | Heute in Lot |
|---|---|
| Bezugslinie | Mitte, Außenkante, Innenkante des ganzen Aufbaus; keine Kernkanten. Ändern verschiebt die Wand (die Linie bleibt, der Körper springt) |
| Zeichnen | Kette mit Klicks, Länge tippen, Winkel rasten in 15°, mit ⇧ in 90°; Bogenwand mit Stich; kein Rechteck, nicht aus Linien oder Unterlage |
| Anschlüsse | automatisch nach Schichtrang (ADR 0017): Tragschicht vor Dämmung vor Luftschicht vor Installation vor Putz; L, T, Kreuzung, Y, Stützen; keine Übersteuerung je Knoten |
| Aufbau | Wandaufbauten nur in der Projektdatei; in der App wählbar, nicht bearbeitbar |
| Höhe | Oberkante feste Höhe, am nächsten Geschoss mit Versatz oder unter dem Dach; Unterkante Versatz über OKFF; nicht an Decken gebunden, keine Wand über mehrere Geschosse |
| Öffnungen | sitzen in der Wand mit Abstand vom Wandanfang, wandern beim Verschieben mit |
| Teilen, Verbinden, Umdrehen | Umdrehen ja (⇧F), Teilen und Verbinden nein |
