# Lot heute: eigene Bauteile, Planerstellung, 3D und Materialien

Stand 26.10.2026 (Branch `bedienung-auswaehlen`), aus dem Code gelesen. Grundlage für den Vergleich in diesem Paket.

## Eigene Bauteile und Bestand

| Was | Heute in Lot |
|---|---|
| Bauteilarten | fest: Wand (Schichten, gerade oder gebogen), Decke, Bodenplatte, Durchbruch, Dach, Gaube, Dachfenster, Stütze (Rechteck, Kreis), Treppe, Geländer, Aufzug, Öffnung mit Tür oder Fenster |
| eigene Bauteile | keine; geplant für M6 (1.15) als Daten mit Profilen, Extrusionen, Parametern und 2D-Symbol je Maßstab |
| freie Körper, Abzugskörper | keine |
| Bestandswände | nur gerade oder gebogene Wände mit gleicher Dicke; keine wechselnde Dicke, keine schiefen Wände |
| Bauzustand | Bestand, Abbruch, Neubau je ganzem Bauteil |
| Aufmaß als Grundlage | Unterlagen aus PDF, Bild, DXF, DWG (fangbar), IFC als graue Referenz |
| 2D | Linien, Flächen mit Schraffur, Kreise, Texte im Grundriss |

## Planerstellung und Planköpfe

| Was | Heute in Lot |
|---|---|
| Planblätter | im Kern: Nummer, Titel, Format, Datum, Index, Ansichten mit Maßstab und Lage; anlegen nur in der Projektdatei |
| Plankopf | fest gebaut: Bauvorhaben, Bauort und Grundstück, Entwurfsverfasser aus dem Bürostandard, Planinhalt, Maßstab, Format, Datum, Index, Zeile für Unterschriften |
| Ausgabe | PDF, SVG, PNG, DXF, DWG über die Kommandozeile |
| in der App | kein Blatt, keine Planliste |
| Planindex, Revisionen | ein Textfeld „Index“ je Blatt, kein Verlauf |
| Faltmarken, freie Formate | nein |

## 3D und Perspektiven

| Was | Heute in Lot |
|---|---|
| 3D-Ansicht | Echtzeit über die Grafikkarte, Flächen mit Kanten, Farbe aus dem Material; zwei Finger drehen, mit ⇧ verschieben, spreizen zoomt |
| Umbau | 3D zeigt den Zustand nach dem Umbau |
| Kamera, Perspektive als Ansicht | nein; die 3D-Ansicht ist perspektivisch mit fester Brennweite |
| Schnittbox, Geschosse ausblenden | nein |
| Schatten, Sonne, Stile | nein |
| Bild | 3D-Bild als PNG über die Kommandozeile |

## Materialien

| Was | Heute in Lot |
|---|---|
| Material | im Bürostandard: Name, Farbe in 3D, Darstellung im Schnitt (leer, Vollfläche, Schraffur) |
| an Bauteilen | Schichten von Wänden, Decken und Dächern verweisen auf einen Materialnamen |
| Anschlüsse | nach Rang der Schicht (ADR 0017), nicht nach Material |
| Kennwerte (λ, Rohdichte, Brandverhalten) | nein |
| Texturen, Ansichtsschraffur | nein |
