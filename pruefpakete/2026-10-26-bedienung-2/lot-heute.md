# Lot heute: Zeichnen und Fangen, Öffnungen, Typen und Eigenschaften

Stand nach M5 (Branch `m5-bauantrag`), aus dem Code gelesen. Grundlage für den Vergleich in diesem Paket.

## Zeichnen und Fangen

| Was | Heute in Lot |
|---|---|
| Fangpunkte | Wandenden (Bezugslinie), Achsschnittpunkte, Punkte auf Wänden und Achsen, Endpunkte von Linien in DXF- und DWG-Unterlagen; sonst frei auf 1 cm gerundet |
| nicht | Mitte, Schnittpunkt zweier Wände, lotrecht, Wandflächen und Ecken der Außenkante, Ecken von Decken und Räumen |
| Winkel | Richtung rastet in 15°-Schritten, mit ⇧ nur 90° |
| Spur- und Hilfslinien | nein |
| Zahleneingabe | beim Wandzeichnen Länge tippen und ⏎, beim Bogen der Stich; keine Koordinaten, kein Winkel, kein Abstand von einem Bezugspunkt |
| Werkzeuge | Wand als Kette, Bogenwand, Achse (2 Klicks), Stütze (1 Klick), Deckendurchbruch (2 Ecken), 2D-Linie, Text, Raum (1 Klick); kein Rechteck, Polygon oder „aus Linien“ |
| Anzeige | Fangpunkt und Länge am Zeiger, Hinweis in der Statuszeile |
| Bauteile ohne Werkzeug | Decken (nur „Decke aus Wänden“), Dächer (nur „Satteldach über dem Geschoss“), Treppen, Geländer, Gauben, Schnitte, Planblätter, Gelände, Grundstück, Aufzug: nur in der Projektdatei |

## Öffnungen

| Was | Heute in Lot |
|---|---|
| Setzen | Tür- oder Fensterwerkzeug, Klick auf die Wand; die Öffnung sitzt mittig am Klickpunkt (auf 1 cm), Vorschau beim Überfahren |
| Maße | feste Voreinstellung: Tür 1,01 x 2,135, Fenster 1,26 x 1,385 mit Brüstung 0,885 (Rohbaumaße); danach im Inspektor änderbar (Breite, Höhe, Brüstung, Abstand vom Wandanfang) |
| Aufschlag | Seite aus dem Klick (welche Wandseite), Anschlag aus der Wandhälfte; umdrehen mit Befehlen |
| Bezug | Abstand immer vom Wandanfang; keine temporären Maße zu Wandende, Achse oder Nachbaröffnung |
| Typen | keine: jede Öffnung trägt ihre Maße selbst; Durchgang ohne Füllung nur in der Projektdatei |
| Darstellung | je Maßstab (ADR 0022): 1:200 vereinfacht, 1:100 Rahmen und Aufschlag, 1:50 mit Türblatt und Glas; Rohbaumaße in Maßketten und an Türen |
| Bewegen | wandert mit der Wand; nicht in eine andere Wand umsetzbar |
| Liste | Tür- und Fensterliste mit Positionen und Typen nach Rohbaumaß (M5) |

## Typen und Eigenschaften

| Was | Heute in Lot |
|---|---|
| Typen | Wandaufbau und Deckenaufbau (auch für Dächer), nur in der Projektdatei; Türen, Fenster, Stützen, Treppen haben keine Typen |
| in der App | Wandaufbau für die Auswahl wählen (Menü „Wandaufbau“), nicht anlegen, ändern oder duplizieren |
| Inspektor | Abschnitt je gewähltem Element mit Parametern (Länge, Stich, Höhe, Versatz, Öffnungsmaße, Geschosshöhe, Stützenmaße, Traufhöhe, Neigung), Raum (Nummer, Name, Nutzung, Wohnfläche), Text, Bauzustand; Flächen je Geschoss ohne Auswahl |
| Mehrfachauswahl | jedes Element ein eigener Abschnitt, keine gemeinsamen Werte |
| eigene Eigenschaften | nein; keine Klassifikation, keine IFC-Klassen in der App |
| Bürostandard | Materialien, Schraffuren, Stifte, Layernamen als Datei (ADR 0040), keine Typenbibliothek |
