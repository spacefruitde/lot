# Prüfpaket M5: Bauantrag

24.10.2026. Etwa 25 Minuten, davon 5 am Mac. M5 baut auf M4 auf und liegt im Branch `m5-bauantrag`. Was in M1 bis M4 offen ist, gilt hier unverändert.

## Was drin ist

Neues Testprojekt `mfh`: Mehrfamilienhaus mit vier Geschossen, Treppenhaus mit Aufzug und acht Wohnungen (Zweispänner, Flachdach mit Attika, Gelände nach Süden fallend). Alle Prüfungen laufen nach der Musterbauordnung, bis ich dein Bundesland kenne (Frage 1).

- **`bauantrag.pdf`**: die Bauantragspläne, 9 Blätter. BA-01 Lageplan mit Abstandsflächen 1:250, BA-02 bis BA-05 Grundrisse 1:100, BA-06 Schnitte, BA-07 und BA-08 Ansichten, P-01 Präsentationsplan. **Ein Blatt in Originalgröße drucken (A3).**
- **`handrechnung.md`** und **`lageplan.pdf`**: Abstandsflächen von Hand gerechnet, dieselben Zahlen im Lageplan und im Test
- `lageplan-probe-rot.png`: dasselbe Haus mit der Ostgrenze 4,50 m vor der Wand, die Abstandsfläche ragt 5,04 m² darüber (rot)
- **`pruefbericht.pdf`**: alle Prüfungen mit Quelle: Gebäudeklasse, Abstandsflächen, GRZ und GFZ, Barrierefreiheit (DIN 18040-2), Rettungswege, Treppen. Ergebnis für das MFH: 43 erfüllt, 6 Hinweise, 0 nicht erfüllt. `berechnungen.pdf` ist der Auszug für den Bauantrag.
- `planansicht-hinweise.png`: so zeigt die App Hinweise im Grundriss (3.OG: Anleitern nur mit Hubrettungsfahrzeug)
- `tuer-fensterliste.pdf` und `.csv`, `flaechen.pdf` (mit Wohnfläche je Wohnung)
- `grundriss-eg.png` (Blatt BA-02), `grundriss-eg-50.pdf`, `schnitt-a.png` (mit Wandhöhe H, lichten Raumhöhen, Steigungsverhältnis), `ansicht-sued.png`, `praesentation.png`, `mfh-3d.png`
- `buerostandard-beispiel.ron`: wie ein Büro seinen Bürostandard anpasst (Büro im Plankopf, eigene Layernamen)
- Entscheidungen: ADR 0037 (Landesrecht, Abstandsflächen), 0038 (Prüfbericht), 0039 (Bauantragspläne, Lageplan, Türliste), 0040 (Bürostandard), 0041 (Präsentationspläne)

Beim Bauen hat der Prüfbericht drei echte Fehler in meinem eigenen Entwurf gefunden (Flurnische 0,83 m, kein Fenster im Treppenhaus EG, falsch gesetzte Wohnungstür). Die sind im Testprojekt behoben.

## App

Neu gebaut liegt sie in `~/Applications/Lot.app`.

| Wo | Was |
|---|---|
| Menü Prüfen, Prüfbericht (⇧⌘P) oder Knopf mit Siegel | Liste aller Befunde; Klick auf einen Befund mit Zielkreuz zeigt den Ort im Grundriss |
| Menü Prüfen, Hinweise im Grundriss (⇧⌘H) | farbige Kreise an den Orten der Hinweise |
| Menü Prüfen, Bürostandard bearbeiten | legt `~/Library/Application Support/Lot/buerostandard.ron` mit Vorlage an und öffnet sie; gilt nach Neustart |

## Prüffragen

Antwort jeweils Ja oder Nein und ein Satz. Wo ein Vorschlag steht, reicht „Vorschlag ok“.

1. **Bundesland**: In welchem Land baut dein Büro meistens? Dann lege ich dessen Datei an (Tiefe der Abstandsflächen, Mindestmaß, Dächer, kleine Gebäude, Vollgeschoss, Bauvorlagen). Bis dahin: MBO mit 0,4 H, mindestens 3 m, Dächer über 45° zu einem Drittel, über 70° ganz, kleine Wohngebäude 3 m. Stimmen diese MBO-Werte so?
2. **Abnahme Handrechnung** (`handrechnung.md`, `lageplan.pdf`): Stimmen H und T je Seite? Lot misst H an jedem Punkt der Wand vom Gelände direkt davor, bei geneigtem Gelände wird die Fläche ein Trapez. Vorschlag: so lassen; „im Mittel je Wand“ wird ein Schalter je Land, falls dein Land es verlangt.
3. **Lageplan** (`lageplan.pdf`, `lageplan-probe-rot.png`): Reicht das als Abstandsflächenplan zum Bauantrag (Neubau rot schraffiert, Abstandsflächen schraffiert mit H und T, Abstände zu den Grenzen, Höhen über NHN, Nordpfeil, rot außerhalb)? Vorschlag: so lassen, der amtliche Lageplan des Vermessers kommt als Unterlage darunter.
4. **Abnahme Bauantragspläne** (`bauantrag.pdf`): Sind die Unterlagen für einen Bauantrag in deinem Land vollständig, und passt der Plankopf mit Unterschriftsfeldern? Was fehlt dir, etwa eine Dachaufsicht oder Angaben zur Entwässerung?
5. **Türen im Grundriss**: Türen ohne Maßkette tragen ihr Rohbaumaß „1,01/2,13⁵“. Die lichte Breite rechnet Lot als Rohbaumaß minus 11 cm (1,01 ergibt 0,90), bei Fenstern minus 12 cm. Vorschlag: so lassen, bis Türen und Fenster Typen mit eigenen lichten Maßen haben (V1).
6. **Barrierefreiheit** (`pruefbericht.pdf`, Abschnitt Barrierefreiheit): Werte nach meinem Stand der DIN 18040-2: Türen 0,90 m im Gebäude, 0,80 m in der Wohnung, Flure und Bewegungsflächen 1,20 m, Fahrkorb 1,10 x 1,40 m, davor 1,50 x 1,50 m, Handläufe beidseitig, Badtür schlägt nach außen auf. Stimmen Werte und Abschnittsnummern mit deiner Norm?
7. **Rettungswege**: Weg als Luftlinie im Raum von Tür zu Tür, höchstens 35 m; Fenster für den zweiten Rettungsweg im Lichten 0,90 x 1,20 m, Brüstung bis 1,20 m; über 8 m Anleiterhöhe nur ein Hinweis auf das Hubrettungsfahrzeug; Treppenraum mit Fenster in jedem Geschoss. Vorschlag: so lassen.
8. **Tür- und Fensterliste** (`tuer-fensterliste.pdf`): Positionen T 0.01 und F 0.01 je Geschoss, Typen T1, F1 nach Rohbaumaß. Passt das, und welche Spalten fehlen dir (Brand-, Schall-, Einbruchschutz kommen mit Typen)?
9. **Präsentationsplan** (`praesentation.png`): Wände dunkel, Räume farbig nach Nutzung, Name und Fläche, keine Maße, Maßstabsleiste und Nordpfeil. Stimmt die Richtung? Vorschlag: als Nächstes Möbel und Terrassen.
10. **Am Mac**: Öffne das MFH (⇧⌘O, `testprojekte/mfh`), Menü Prüfen, Prüfbericht, klick auf einen Hinweis im 3.OG. Springt die Ansicht hin? Dann Bürostandard bearbeiten, dein Büro eintragen, Lot neu starten: Steht es im Plankopf? Wenn Fachplaner eine Layerliste erwarten (Frage 5 aus M4), kann sie jetzt dort hinein.

## Was ich geprüft habe und was nicht

- Kern: alle Tests grün, clippy ohne Warnung. Neu getestet: Abstandsflächen (Mindesttiefe, Verletzung an der Grenze, öffentliche Fläche bis zur Mitte), Gebäudeklasse, Himmelsrichtungen, Landesrecht laden, Bürostandard mit Abweichungen, Quadrat in Fläche und schmale Stellen, Raumverbund, Aufzug, Prüfbericht in der App-Schnittstelle.
- Abnahmetest MFH (`cargo test -p lot-cli --test mfh`): Gebäudeklasse 4, Abstandsflächen je Seite gegen die Handrechnung (H, T, Fläche, Abstand zur Grenze), 5,04 m² rot in der Probe, Prüfbericht ohne Fehler, Planblätter vollständig ohne Überdeckungen.
- Nebenbei behoben: Flachdächer bekamen aus einer Boolean-Operation Reste bis 188 m unter das Gelände (betraf Schnitte mit weit reichendem Gelände); Ansichten zeigten Linien an Geschossgrenzen, wo eine Fenstertür die Wandkante teilt.
- **Nicht geprüft**: die App per Bildschirmsteuerung (Vollbild-Space wie in M3 und M4); Prüfbericht und Hinweise laufen über dieselbe Schnittstelle, die der Test aufruft. Die DWG mit den neuen Layernamen habe ich nicht in AutoCAD geöffnet.
- Keine neuen Bibliotheken.

## Messwerte

| Was | Gemessen | Ziel aus PLAN.md |
|---|---|---|
| Wandlänge ändern am EFH | 36 ms | unter 50 ms |
| MFH ganz rechnen mit allen 20 Ausgaben (Release) | 8,8 s | kein Ziel |

## Bekannte Lücken

- Landesrecht nur als Musterbauordnung; Abstandsflächen ohne Überdeckung eigener Flächen, Vorbauten, Dachaufbauten und Gebäude an der Grenze.
- Barrierefreiheit ohne Schwellen, Rampen, Bedienelemente und Ausstattung; nur „barrierefrei nutzbar“, noch nicht „R“.
- Rettungswege ohne Rauchableitung oben im Treppenraum und ohne Aufstellflächen der Feuerwehr.
- Kein Bebauungsplan im Modell (Baugrenzen, zulässige GRZ und GFZ); kein Stellplatznachweis.
- Präsentationspläne ohne Möbel, Schatten und farbige Ansichten.
- Bürostandard nur als Datei, ohne Einstellungsfenster; gilt nach Neustart.
