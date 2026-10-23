# 0033 DXF und DWG im Austausch

Status: umgesetzt in M4 (Vorschlag), Ben prüft · 23.10.2026

## Kontext

ADR 0012 entschied acadrust für DXF und DWG. Abnahme M4: Ein Fachplaner öffnet den DWG-Export ohne Rückfragen.

## Entscheidung

- **Was**: Grundrisse, Schnitte und Ansichten, Planblätter; aus derselben Planpipeline wie PDF, damit alle Ausgaben gleich aussehen.
- **Einheiten**: Grundrisse und Schnitte im Modellbereich in Metern (INSUNITS 6), Koordinaten wie im Modell; Schnitte mit x entlang der Schnittlinie und y als Höhe. Planblätter in Millimetern (INSUNITS 4).
- **Layer** im Grundriss nach Inhalt: `A_Sichtkanten`, `A_Bauteile_Schnitt`, `A_Tueren_Fenster`, `A_Deckendurchbrueche`, `A_Treppen`, `A_Dach`, `A_Schnittlinien`, `A_Achsen`, `A_Bemassung`, `A_Raumstempel`, `A_2D` (Vorschlag, ohne Umlaute).
- **Darstellung**: Linienstärken als Linienstärke der Elemente, Farben RGB (Schwarz als Farbe 7), Gestricheltes mit eigenen Linientypen, Flächen als HATCH (gefüllt oder als Linienschar), Texte als TEXT. Format AutoCAD 2013 (AC1027), DXF und DWG.
- **Prüfung**: Die DXF-Dateien liest ezdxf ohne Fehler und zeichnet sie richtig; die DWG-Dateien liest acadrust wieder.
- **Lesen**: Linien, Polylinien, Kreise, Bögen und Texte als Unterlage (ADR 0032).

## Alternativen

- **1 Einheit = 1 mm**: üblich im Maschinenbau und bei manchen Fachplanern; die Einheit steht in der Datei, umstellbar wird sie mit dem Bürostandard.
- **Bemaßung als DIMENSION-Objekte**: Fachplaner könnten Maße ändern; Lot schreibt Maßketten bisher als Linien und Texte.

## Folgen

- Ob AutoCAD die Dateien ohne Rückfrage öffnet, ist noch nicht an einer echten Installation geprüft (Prüfpaket M4).
- Golden-Tests mit echten DXF und DWG aus Bens Büro stehen weiter aus (ADR 0012).
