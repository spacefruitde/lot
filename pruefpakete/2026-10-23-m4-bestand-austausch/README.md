# Prüfpaket M4: Bestand und Austausch

23.10.2026. Etwa 20 Minuten, davon 5 am Mac. M4 baut auf M3 auf und liegt im Branch `m4-bestand-austausch`. Was in M1 bis M3 offen ist, gilt hier unverändert.

## Was drin ist

Neues Testprojekt `umbau`: ein Altbau von 1962 (Ziegel 36,5, Satteldach), nach seinem alten Bestandsplan modelliert. Umbau: Wand zwischen Wohnzimmer und Küche und Kamin abbrechen, Küchenfenster Ost zumauern, Anbau im Osten mit Durchbruch von der Küche.

- **`umbau-eg-umbau.pdf`**: Umbauplan EG, Bestand grau, Abbruch gelb, Neubau rot, 1:100 A4. **In Originalgröße drucken.** Dazu `umbau-eg-umbau-50.pdf` in 1:50.
- `umbau-eg-bestand.pdf`: Bestandsplan vor dem Umbau, schwarz-weiß; `umbau-eg-neu.pdf`: Neuzustand nach dem Umbau
- `umbau-eg-planansicht.png`: so zeigt die App das EG, mit dem alten Bestandsplan blass als Unterlage und dem Tragwerksmodell (IFC) hellgrau
- `umbau-schnitt-a.pdf` (durch den Anbau), `umbau-ansicht-sued.png`, `umbau-ansicht-ost.png`, `umbau-3d.png` (Neuzustand), `umbau-flaechen.pdf` (Neuzustand)
- **`umbau-eg-umbau.dwg`** und `.dxf`: der Umbauplan für Fachplaner, in Metern mit Layern; ebenso `umbau-schnitt-a.dwg`, `efh-eg-100.dwg`, `efh-planblaetter-1.dwg` (Planblatt in mm). `umbau-eg-umbau-dxf-gelesen-mit-ezdxf.png` zeigt, wie ein fremdes Programm (ezdxf) die DXF zeichnet.
- `efh-schnitt-a.png`, `efh-ansicht-sued.png`: das EFH mit Gelände statt der Linie auf -0,30 m; `efh-eg-100.png` und `efh-eg-variante-offen.png`: zwei Varianten der Küche
- Zum Ausprobieren in der App: `bestand-eg.pdf` (alter Bestandsplan) und `tragwerk-anbau.ifc` (Tragwerk des Anbaus, ein kleines, selbst geschriebenes IFC4)
- Entscheidungen: ADR 0031 (Bestand, Abbruch, Neubau), 0032 (Unterlagen), 0033 (DXF und DWG), 0034 (Gelände), 0035 (Varianten), 0036 (IFC als Referenz)

## App

Neu gebaut liegt sie in `~/Applications/Lot.app`.

| Wo | Was |
|---|---|
| Ablage, Unterlage laden | PDF, Bild, DXF oder DWG unter das aktuelle Geschoss legen; danach ist Kalibrieren gewählt |
| K | Unterlage kalibrieren: Anfang und Ende einer bekannten Strecke klicken, wahres Maß tippen, ⏎ |
| Werkzeug, Unterlage ausrichten | zwei Punkte einer Linie, die waagerecht liegen soll |
| Ablage, Referenzmodell laden | IFC eines Fachplaners, grau im Grundriss und im 3D |
| Inspektor, Bauzustand | Bestand, Abbruch, Neubau für die gewählten Bauteile |
| ⌘K, Varianten | neue Variantengruppe, Auswahl in die aktive Variante, Variante wechseln |

## Prüffragen

Antwort jeweils Ja oder Nein und ein Satz. Wo ein Vorschlag steht, reicht „Vorschlag ok“.

1. **Abnahme:** `umbau-eg-umbau.pdf`: Bestand grau, Abbruch gelb, Neubau rot, neue Öffnung in Bestandswand gelb (Ausbruch), zugemauertes Fenster rot, im Umbauplan keine Materialschraffur. Stimmt das mit der Bauvorlagenverordnung deines Landes überein (welches Land)? Vorschlag: Farbtöne wie hier, je Land im Bürostandard änderbar.
2. Planarten: Umbauplan (alles farbig), Bestandsplan (vor dem Umbau, schwarz-weiß, alte Raumnamen), Neuzustand (Bestand grau, Neubau rot, Ausbrüche offen). Brauchst du alle drei? Vorschlag: ja, im Bauantrag Umbauplan und Bestandsplan.
3. **Abnahme am Mac:** Neues Projekt, Ablage, Unterlage laden, `bestand-eg.pdf`; mit K über eine Außenwand kalibrieren (10,24 m) und zwei Wände nachzeichnen. Geht das ohne Anleitung? Vor dem Kalibrieren nimmt Lot bei PDF 1:100 an. Vorschlag: so lassen.
4. **Abnahme DWG:** Öffne `umbau-eg-umbau.dwg` in AutoCAD oder schick ihn einem Fachplaner. Öffnet er sich ohne Rückfragen? 1 Einheit ist 1 m, Linienstärken und Farben stehen an den Elementen, Format AutoCAD 2013. Vorschlag: Meter. Oder lieber Millimeter?
5. Layer im Grundriss: `A_Sichtkanten`, `A_Bauteile_Schnitt`, `A_Tueren_Fenster`, `A_Deckendurchbrueche`, `A_Treppen`, `A_Dach`, `A_Schnittlinien`, `A_Achsen`, `A_Bemassung`, `A_Raumstempel`, `A_2D`. Vorschlag: so lassen, eigene Layernamen kommen mit dem Bürostandard (M5). Gibt es in deinem Büro schon eine Layerliste, die Fachplaner erwarten?
6. Maßketten gehen als Linien und Texte ins DWG, nicht als Bemaßungsobjekte; Schraffuren als Schraffur, Texte in der Standardschrift. Reicht das für Fachplaner? Vorschlag: ja für M4.
7. Gelände (`efh-schnitt-a.png`, `efh-ansicht-sued.png`): Höhenpunkte, dazwischen ein Dreiecksnetz, im Schnitt Erdreich schraffiert (Linien unter 135°), in Ansichten die Geländelinie vor dem Gebäude. Höhen relativ zu ±0,00. Vorschlag: so lassen; m ü. NHN kommt mit dem Bauantrag in M5.
8. Varianten (`efh-eg-100.png` und `efh-eg-variante-offen.png`): Küche abgetrennt oder offen mit Kochinsel. Bauteile in keiner Variante gehören zu allen, Pläne zeigen die aktive Variante. Vorschlag: so lassen; ein Flächenvergleich der Varianten folgt bei Bedarf.
9. IFC-Referenz (`umbau-eg-planansicht.png`, `umbau-3d.png`): Bauteile des Fachplaners grau im Grundriss in der Schnittebene, im Schnitt und im 3D, nicht bearbeitbar; in gedruckten Grundrissen nur auf Wunsch. Vorschlag: so lassen.
10. Echte Testdaten: Kannst du mir einen alten Bestandsplan (PDF, gern ein Scan), ein DWG eines Fachplaners und ein IFC (Tragwerk oder TGA) aus deinem Büro geben? Sie bleiben nur lokal, nicht im Repository (ADR 0012, 0013).

## Was ich geprüft habe und was nicht

- Kern: 178 Tests grün, clippy ohne Warnung. Neu getestet: Bauzustände (Farben im Grundriss, Bestandsplan ohne Farben und Anbau, Neuzustand ohne Abbruch, Ausbruch und zugemauerte Öffnung), Unterlagen (Lage, Kalibrieren, Ausrichten, Darstellung als Textur, PNG lesen, Dateien in der Projektdatei), DXF und DWG (Layer, Einheiten, Linientypen, Schraffuren, Texte; Schreiben und Wiederlesen, auch DWG), Fangen an Linien von DXF-Unterlagen, Gelände (Triangulation, Höhen, Erdreich ohne Baugrube), Varianten (ausblenden, wechseln, nicht anklickbar), IFC (Bauteile in Metern mit Öffnungen, im Grundriss geschnitten).
- Abnahmetest Umbau (`cargo test -p lot-cli --test umbau`): alle drei Farben im Umbauplan, Bestand und Neuzustand richtig, Wohnen, Kochen 38,37 m² (21,40 + 15,73 + 1,24 für die abgebrochene Wand), Unterlage und IFC-Referenz im Projekt.
- Fremd geprüft: Alle DXF-Dateien liest ezdxf 1.4 ohne Fehler, die Zeichnung (Bild im Paket) sieht aus wie das PDF.
- **Nicht geprüft:** das Öffnen der DWG in AutoCAD (AutoCAD ist installiert, aber ich habe es über Nacht nicht gestartet) und die App per Bildschirmsteuerung (Vollbild-Space wie in M3). PDF-Unterlagen rendert in der App PDFKit; das ist gebaut, aber nicht von Hand ausprobiert. Fragen 3 und 4 sind deshalb die ersten echten Tests.
- Neue Bibliotheken: acadrust 0.6.6, ifc-lite-core und ifc-lite-geometry 20.4 (alle MPL-2.0, nur als Bibliothek, fest gepinnt), in `THIRD_PARTY.md`. Keine GPL.

## Messwerte

| Was | Gemessen | Ziel aus PLAN.md |
|---|---|---|
| Wandlänge ändern am EFH, mit Gelände, Grundriss, 3D und Flächen | 39 ms | unter 50 ms |

## Bekannte Lücken

- Der Bestandsplan im Testprojekt stammt aus Lot selbst und ist deshalb sauberer als ein echter Scan; mit Bens Plänen (Frage 10) prüfe ich Kalibrieren und Ausrichten an echten Vorlagen.
- Unterlagen aus PDF kann der Kern nicht selbst rendern (nur die App auf dem Mac); Fangen an Linien gibt es bei DXF und DWG, bei PDF noch nicht.
- Mehrere Umbauphasen nacheinander, geplantes Gelände (Aufschüttung, Abgrabung) und NHN-Höhen fehlen.
- Werkzeuge für Gelände und Referenzmodelle (verschieben, drehen) fehlen in der App; beides geht bisher über die Projektdatei.
- DWG-Bemaßung als Bemaßungsobjekte, eigene Layernamen und Schriften folgen mit dem Bürostandard.
