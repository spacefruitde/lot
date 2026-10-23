# Prüfpaket M3: Schnitte, Ansichten, Dächer, Treppen und Planblätter

23.10.2026. Etwa 15 Minuten, davon 3 am Mac. M3 baut auf M2 auf und liegt im Branch `m3-schnitte-daecher`. Was in M1 und M2 offen ist, gilt hier unverändert.

## Was drin ist

- **`planblaetter.pdf`**: das EFH auf drei Blättern A2 mit Plankopf (101 Grundrisse KG und EG, 102 Grundriss DG und Schnitte, 103 Ansichten). **In Originalgröße drucken (100 %)**, sonst stimmt 1:100 nicht. Fürs iPhone `planblaetter-1.png` bis `-3.png`.
- `schnitt-a.pdf` (durch Treppe und Gaube), `schnitt-b.pdf`, `ansicht-sued.pdf` und die übrigen Ansichten als PNG, je A4 1:100
- `eg-100.pdf`, `eg-50.pdf`, `dg-100.pdf`, `kg-100.png`: Grundrisse mit Treppen, Geländern, Gaube und Dachfenstern
- `flaechen.pdf` und `flaechen.csv`: Wohnfläche im DG jetzt nach lichter Höhe unter Dach und Gaube
- `efh-3d.png`: das Haus mit Dach, Schleppgaube und Dachfenster; `eg-planansicht.png`: das EG, wie es die App zeigt
- Entscheidungen: ADR 0025 (Dächer), 0026 (Schnitte und Ansichten), 0027 (Planblätter), 0028 (Treppen und Geländer), 0029 (Gauben und Dachfenster), 0030 (2D-Zeichnen) in `docs/entscheidungen/`

## App

Neu gebaut liegt sie in `~/Applications/Lot.app`. Neu sind zwei Werkzeuge, auch im Menü Werkzeug:

| Taste | Was |
|---|---|
| L | Linie: Punkte nacheinander klicken, Länge tippen und ⏎, ⇧ rechtwinklig, Doppelklick oder Esc beendet |
| X | Text: Stelle klicken; der Text ist dann gewählt, Inhalt und Schrifthöhe stehen im Inspektor |

Dächer, Treppen, Gauben, Schnitte und Planblätter legst du bisher in der Projektdatei an (`testprojekte/efh/projekt.ron`); Werkzeuge dafür folgen. „Satteldach über dem Geschoss“ im Menü Werkzeug gibt es seit Beginn von M3.

## Prüffragen

Antwort jeweils Ja oder Nein und ein Satz. Wo ein Vorschlag steht, reicht „Vorschlag ok“.

1. **Abnahme:** `planblaetter.pdf`: Ist das EFH damit komplett, also Grundrisse aller Geschosse, zwei Schnitte, vier Ansichten, Plankopf? Was fehlt dir zuerst?
2. Schnitte (`schnitt-a.pdf`): Geschnittenes wie im Grundriss schraffiert, Kanten dahinter dünn, Gelände vorläufig als kräftige Linie 30 cm unter ±0,00, links Höhenkoten für OKFF je Geschoss, Traufe und First. Vorschlag: so lassen und als Nächstes Höhenkoten für Rohdecken (OKRD) und Brüstungen ergänzen.
3. Schnittlinie im Grundriss: nur die kräftigen Enden mit Pfeil in Blickrichtung und Buchstabe, keine durchgehende Linie. Vorschlag: so lassen.
4. Dach: Die Traufhöhe gilt für die Dachhaut an der Außenkante der Wand (hier +4,13), Überstände waagerecht. Der Dachaufbau ist ein Paket von 29 cm (Ziegel mit Lattung, Dämmung zwischen den Sparren, Gipskarton) ohne einzelne Sparren. Vorschlag: so lassen, Sparren als eigenes Bauteil erst bei Bedarf.
5. Treppe im Grundriss (`eg-50.pdf`, `dg-100.pdf`): bis zur Schnittebene durchgezogen, schräge Schnittlinie quer über den Lauf, darüber gestrichelt; im DG die ganze Treppe durch das Treppenloch. Lauflinie mit Kreis am Antritt und Pfeil am Austritt, über das Halbpodest im Bogen; vor dem Antritt „16 Stg. 17,2/27,0“. Vorschlag: so lassen. Oder den Teil über der Schnittebene ganz weglassen?
6. Treppenprüfung nach DIN 18065:2020-08, Tabelle 1: Wohngebäude mit höchstens zwei Wohnungen, notwendige Treppe: Steigung 14 bis 20 cm, Auftritt 23 bis 37 cm, Laufbreite ab 80 cm; sonstige Gebäude: 14 bis 19, 26 bis 37, ab 100 cm; nicht notwendige Treppen: bis 21, ab 21, ab 50 cm. Dazu Schrittmaß 59 bis 65 cm und lichte Durchgangshöhe ab 2,00 m. Stimmen die Werte mit deiner Fassung überein? Vorschlag: Laufbreite vorerst ohne Abzug für Handläufe.
7. Geländer: Oberkante 0,90 m über Stufenvorderkante oder Fußboden, senkrechte Stäbe mit 12 cm Achsabstand (10 cm lichte Weite), im Grundriss eine dünne Linie 5 cm innerhalb der Laufkante. Vorschlag: so lassen; 1,10 m kommt mit den Absturzhöhen über 12 m im MFH.
8. Gaube (`efh-3d.png`, `schnitt-a.pdf`, `ansicht-sued.pdf`): Schleppgaube, Vorderwand 20 cm hinter der Außenkante auf dem Dach, Gaubendach 12°, im Grundriss gestrichelt; die Fläche darunter zählt mit ihrer lichten Höhe zur Wohnfläche (Schlafen +1,04 m²). Welche Gaube brauchst du als Nächstes: Satteldachgaube, Gaube in der Fassadenflucht mit unterbrochener Traufe oder Zwerchgiebel?
9. Plankopf (`planblaetter-1.png`): Schriftfeld 180 mm unten rechts mit Bauvorhaben; Bauherr, Bauort, Entwurfsverfasser; Planinhalt; Maßstab, Format, Datum, Index, Plannummer. Heftrand 20 mm. Vorschlag: so lassen, bis dein eigener Plankopf mit dem Bürostandard in M5 kommt. Fehlt dir etwas sofort (Unterschrift, Änderungsliste, Logo)?
10. **Abnahme:** In der App im EG mit L eine Linie und mit X einen Text setzen, den Text im Inspektor ändern und verschieben. Geht das ohne Anleitung?

## Was ich geprüft habe und was nicht

- Kern: 153 Tests grün, clippy ohne Warnung. Neu getestet: Läufe, Podeste und Lauflinie, Steigung aus der Geschosshöhe; Treppenkörper (Höhe, Ausdehnung, Volumen); Geländer an einem Lauf; Treppe im Grundriss (Schnittlinie an der richtigen Stelle, im Geschoss darüber ohne gestrichelte Teile); DIN-18065-Prüfung (passende Treppe, zu steile, strengere Werte für sonstige Gebäude, zu wenig Kopfhöhe unter einer Decke); Gaube (Anschluss an die Dachfläche, Wände auf der Dachunterseite, Ausschnitt im Dach, lichte Höhe); Planblätter (Rand, Schriftfeld, Überdeckung); 2D-Elemente und die Werkzeuge Linie und Text.
- Abnahmetest am EFH (`cargo test -p lot-cli --test efh`): alle drei Geschosse, zwei Schnitte und vier Ansichten liegen überdeckungsfrei auf den Blättern; beide Treppen erfüllen DIN 18065.
- Von Hand nachgerechnet: Wohnfläche Schlafen 28,10 m² (27,06 m² ohne Gaube plus 1,04 m² unter der Gaube); EG-Treppe 16 Steigungen zu 17,19 cm, 2s + a = 61,4 cm; KG-Treppe 14 zu 17,86 cm, 62,7 cm; kleinste Durchgangshöhe im KG 2,27 m unter der EG-Treppe.
- **Nicht geprüft:** die App per Bildschirmsteuerung. Dein Bildschirm war im Vollbild-Space, dorthin lässt sich das App-Fenster nicht holen. Die App ist gebaut, Linie und Text sind im Kern getestet. Frage 10 ist deshalb der erste echte Test.

## Messwerte

| Was | Gemessen | Ziel aus PLAN.md |
|---|---|---|
| Wandlänge ändern, danach Grundriss, 3D mit Dach, Gaube und Treppen, Flächen | 38 ms | unter 50 ms |

## Bekannte Lücken

- Gelände kommt in M4; bis dahin steht in Schnitten und Ansichten „Gelände vorläufig -0,30 m“.
- Dächer nur über konvexen Umrissen, nur Schleppgauben, keine gewendelten Treppen. Treppenlöcher legt man selbst als Deckendurchbruch an.
- Werkzeuge in der App fehlen für Dächer, Treppen, Gauben, Schnitte und Planblätter; 2D-Elemente in Schnitten und auf Blättern gibt es nur über die Projektdatei.
- Durchbrüche mindern die Netto-Raumfläche weiterhin nicht (Frage 7 aus M2 ist offen).
- Testdaten geändert: Die Hauswirtschaft im EG ist jetzt der Treppenraum (0.05 „Treppe“) mit Durchgang zur Diele; das EG-Fenster Ost entfällt, das Fenster Nord ist nach links gerückt. Schnitt A-A liegt jetzt bei x = 9,10 m durch Treppe und Gaube. Im Keller fehlen weiterhin Türen.
