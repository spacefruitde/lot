# Prüfpaket M1: Wände, die stimmen

23.10.2026. Etwa 15 Minuten, davon 5 am Mac für die App.

## Was drin ist

- `wandknoten-50.pdf`: Katalog mit 30 Wandknoten, 1:50, davon drei mit Bogenwänden (28 bis 30), fünf Blätter A4 quer. **In Originalgröße drucken (100 %).**
- `wandknoten-50-1.png` bis `-5.png`: dieselben Blätter zum Ansehen auf dem iPhone
- `wandknoten-100.pdf`: der Katalog in 1:100, zum Vergleich der Detailstufen
- `eg-100.pdf`, `eg-50.pdf`, `eg-200.pdf`: Grundriss EG des Testprojekts EFH in drei Maßstäben, dazu `eg-50.png`, `eg-100.png` und `eg-200.png`
- `kg-100.pdf`, `dg-100.pdf` und `dg-100.png`: Keller und Dachgeschoss
- `efh-3d.png`: das ganze Haus in 3D, gerechnet vom Renderer der App
- `eg-planansicht.png`: der Grundriss so, wie ihn die App auf dem Bildschirm zeigt
- Entscheidungen: ADR 0017 (Wandanschlüsse nach Schichtrang), 0018 (Bürostandard als Daten), 0019 (Werkzeuge und Befehle im Kern), 0020 (Gebogene Wände) in `docs/entscheidungen/`

## App starten

Ein fertiger Stand liegt in `~/Applications/Lot.app`. Oder neu bauen:

```
cd app-mac && xcodebuild -project Lot.xcodeproj -scheme Lot -derivedDataPath build/DerivedData build && open build/DerivedData/Build/Products/Debug/Lot.app
```

Die App startet mit einem leeren Projekt im Grundriss 1:100. Das EFH öffnest du über Ablage, Testprojekt öffnen (⇧⌘O), Ordner `testprojekte/efh`.

| Taste | Was |
|---|---|
| W | Wand zeichnen: Klick, Klick, Klick, Esc beendet die Kette. Eine Länge tippen und Return setzt den nächsten Punkt genau. |
| B | Bogenwand: Anfang klicken, Ende klicken, dann den Stich mit dem Zeiger ziehen und klicken (oder tippen und Return). Den Stich jeder Wand kannst du rechts ändern. |
| V | Auswählen. Ziehen verschiebt, angeschlossene Wände ziehen mit. Endpunkte lassen sich einzeln ziehen. |
| A | Achse zeichnen. Senkrechte Achsen bekommen Zahlen, waagrechte Buchstaben. |
| S | Stütze setzen, 30 × 30 cm Stahlbeton |
| F | Gewählte Wände umdrehen (innen und außen tauschen) |
| ⌘K | Befehlspalette mit allen Befehlen |
| ⌘1, ⌘2 | Grundriss, 3D |
| ⌘0 | Alles zeigen |
| ⌥ oder rechte Maustaste ziehen, zwei Finger | Verschieben, spreizen zoomt zum Zeiger |

Gefangen werden Wandenden, Achsschnitte, Wandflächen und Achsen, dazu 15°-Winkel und ein Raster von 1 cm. Wandaufbau, Geschoss und Maßstab stehen oben in der Leiste.

## Prüffragen

Antwort jeweils Ja oder Nein und ein Satz. Wo ein Vorschlag steht, reicht „Vorschlag ok“.

1. **Abnahme:** Wandknoten 1 bis 30 auf `wandknoten-50.pdf`: Würdest du jeden so von Hand zeichnen? Bitte die Nummern nennen, die nicht stimmen. Spitze Winkel (17, 18) laufen voll aus und werden erst ab zwei Wanddicken über dem Knotenpunkt gekappt.
2. Der Rang der Schichten entscheidet, welche Schicht durchläuft: Tragend vor Dämmung vor Bekleidung vor Putz, bei gleichem Rang die dickere Wand. Passt diese Reihenfolge? Vorschlag: ja, so lassen.
3. Knoten 6 (WDVS-Außenwand trifft als L auf eine Innenwand): Die Dämmung läuft bis zur Außenkante der Innenwand und endet dort stumpf, die Innenwand bleibt ohne Dämmung. Vorschlag: so lassen, eine gedämmte Ecke wäre ein eigener Wandaufbau.
4. Knoten 12: Die Trockenbauwand stößt durch den Putz bis ans Mauerwerk. Vorschlag: so lassen, so wird es auch gebaut.
5. In 1:50 sieht man keinen Putz: Die Schnittlinie mit 0,70 mm reicht 0,35 mm nach innen, 1,5 cm Putz sind dort nur 0,3 mm. Vorschlag: Schnittlinie in 1:50 auf 0,50 mm und Putz mit einer feinen Linie 0,13 abgrenzen.
6. Knoten 23 und 24 (freie Wandenden): Im Modell läuft der Putz nicht um das Wandende herum, sobald er sichtbar ist (Frage 5), fehlt er an der Stirnseite. Vorschlag: in 1:50 um das Ende herumführen, in 1:100 und 1:200 nicht.
7. Schraffuren (1:50): Mauerwerk 45° im Abstand 1,25 mm, Stahlbeton gekreuzt, harte Dämmung (WDVS) als Zickzack, weiche Dämmung (Mineralwolle, etwa im Trockenbau) als Schlangenlinie, Putz und Gipskarton leer. In 1:100 nur das Tragende schraffiert, die übrigen Schichten als Linie, in 1:200 schwarz gefüllt ohne Schichten (wie in PLAN.md vorgeschlagen). Passt das? Vorschlag: ja, als Grundeinstellung des Bürostandards.
8. Bogenwände (Knoten 28 bis 30): Du gibst Anfang, Ende und den Stich (Pfeilhöhe) an, höchstens ein Halbkreis. Intern sind es gerade Stücke, höchstens 2 mm vom Bogen entfernt, Laibungen zeigen zum Mittelpunkt. Die Schraffur bleibt bei 45° und dreht nicht mit dem Bogen. Vorschlag: so lassen, Radius später zusätzlich als Anzeige rechts.
9. Schnitthöhe 1,00 m über OKFF. Im DG (`dg-100.pdf`) liegt der Kniestock darunter und erscheint deshalb geschnitten, die Dachflächen kommen erst mit M3. Vorschlag: 1,00 m als Grundeinstellung, je Geschoss änderbar.
10. **Abnahme:** In der App das EG des EFH in wenigen Minuten nachzeichnen: Außenwände, Innenwände, zwei Achsen, eine Stütze. Geht das ohne Anleitung, und fühlen sich Fangen und Verschieben richtig an?

## Was ich geprüft habe und was nicht

- Geometrie: 22 Tests, davon Zufallstests mit 200 Gruppen gerader Wände, 100 Gruppen mit Bögen und 300 einzelnen Bögen. Keine Schicht überlappt eine andere, keine Lücke entsteht, jede Schicht einer Bogenwand ist ein Stück ohne Fuge. Ausnahme sind Splitter unter 2 µm Breite aus der Rundung.
- Gerade Wände haben sich durch die Bögen nicht verändert: Die Ausgaben von EFH und Katalogblatt 1 bis 4 sind bit-gleich zum Stand davor.
- Katalog und EFH: Grundrisse 1:50, 1:100, 1:200 als PDF, SVG und PNG, dazu die 3D-Ansicht und die Planansicht des Bildschirms. Angesehen habe ich jeden Knoten selbst, die fachliche Abnahme bleibt bei dir (Frage 1).
- App per Bildschirmsteuerung: Wände per Klick gezeichnet, T-Stoß fängt an der Wandfläche, Auswahl mit Rahmen und Eigenschaften rechts, 3D zeigt automatisch alles, Befehlspalette per Klick, Achse mit Kreis und Nummer, Stütze gesetzt und in den Eigenschaften geändert. Bogenwand mit drei Klicks gezeichnet, gerade Wand per T-Stoß an den Bogen gesetzt, beides im 3D geprüft.
- Nicht prüfen konnte ich das Ziehen mit der Maus (die Steuerung schickt kein Ziehen an ein Fenster im Hintergrund) und das Tippen in die Befehlspalette. Beides ist im Kern getestet (Verschieben mit Anschlüssen, Endpunkt ziehen), in der App aber nur von Hand prüfbar (Frage 10).
- CI auf GitHub: Kern unter Linux und macOS, Mac-App. Ergebnis steht im Pull Request.
- Alle 93 Tests im Kern grün, clippy ohne Warnung.

## Messwerte

| Was | Gemessen | Ziel aus PLAN.md |
|---|---|---|
| Grundriss EG neu rechnen nach einer Änderung (Anschlüsse, Schraffuren, Dreiecke) | 4 bis 5 ms | unter 50 ms |
| Nur die Schichtflächen der Anschlüsse, EG | 1 ms | |
| 3D des ganzen EFH | 17 ms (vorher 91 ms) | |
| Wandlänge ändern, danach Grundriss und 3D neu | 21 ms | unter 50 ms |

Für das 3D fasse ich gleiche Höhenabschnitte einer Wand zusammen und rechne auf allen Prozessorkernen. Die App rechnet 3D ohnehin nur, wenn die 3D-Ansicht offen ist. Messen: `cd core && cargo test -p lot-cli --test messung -- --ignored --nocapture`.

## Bekannte Lücken

- Dämmstoffmuster auf Bögen: Die Zickzacklinie hat ihre Periode auf der Bezugslinie, außen wird sie etwas länger, innen etwas kürzer.
- Fenster und Türen sind im Grundriss nur Lücken mit Brüstungslinie. Symbole, Anschläge und Maßketten kommen mit M2.
- Der Bürostandard ist eine Datei im Code (`core/crates/lot-plan/src/buerostandard.ron`). Eigene Standards je Büro kommen mit M5.
- Referenzen für die Golden-Tests habe ich nicht angelegt. Sobald du die Knoten freigibst, lege ich sie aus genau diesen Ausgaben an.

## Hinweis

Das Repository `spacefruitde/lot` ist auf GitHub öffentlich. PLAN.md geht von privat aus. Umstellen kannst du das unter Settings, General, Danger Zone, Change visibility.
