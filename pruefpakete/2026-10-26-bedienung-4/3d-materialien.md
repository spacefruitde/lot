# 3D, Perspektiven und Materialien: Vergleich und Vorschlag

Quellen und Einzelheiten: `recherche-3d-materialien.md`. Was Lot heute kann: `lot-heute.md`.

## Kurzvergleich

| Thema | Revit | Archicad | Vectorworks | Allplan | Synaps | Lot heute |
|---|---|---|---|---|---|---|
| Mac | nein | nativ | nativ | nein | im Browser | nativ |
| Schnittbox | ja | 3D-Schnitt mit Laschen | ja | unbelegt | unbelegt | nein |
| Stile | Verdeckt bis Realistisch, je Ansicht | 3D-Stile, wirken auf alle Ansichten, Weißmodell | Render-Stile | Draht bis RTRender, kein Weißmodell | Stil für das KI-Bild | ein Stil |
| Kamera | Augen- und Zielhöhe, keine Brennweite | Augpunkt, Zielpunkt, Sichtkegel | Renderworks-Kamera | Brennweite in mm | im Grundriss setzen, Augenhöhe, keine Brennweite gefunden | keine |
| 2-Punkt-Perspektive | nur per Trick | Schalter | bleibt beim Zielen | unbelegt | unbelegt | nein |
| Rendern | Realistisch, sonst Twinmotion oder Enscape (nur Windows) | CineRender, Redshift, Enscape live | Renderworks, Redshift (eigene Lizenz) | Redshift, Twinmotion, Lumion | KI-Bild in Konzeptqualität, Export OBJ, FBX laut Website | 3D-Bild als PNG |
| Material | ein Material, vier Teile | Baustoff und Oberfläche getrennt | ein Material mit Füllung, Textur, Kennwerten | Materialkatalog und Oberfläche getrennt | Füllung und Strich je Wandschicht | Farbe und Schnittmuster |
| Verschneidung nach Material | nein | Priorität 0 bis 999 | nein | Priorität im Katalog | unbelegt | nach Schichtrang |

**Was Nutzer am meisten ärgert:** Revits Orbit ist in der Perspektive unberechenbar („Panning and Orbiting: I HATE IT!“ [21]), es gibt keine Brennweite [16]; Rendern ist langsam (dieselben Vectorworks-Bilder brauchen in 2026 über 16 statt gut 1 Minute [62], CineRender gilt als überholt [42]); duplizierte Revit-Materialien teilen ihre Darstellung mit dem Original [19]; Archicads Textur folgt der Füllung nicht mehr [47]; Twinmotion setzt geänderte Materialien zurück [45].

**Wo Lot schon vorn ist:** Lot ist eine echte Mac-App mit Echtzeit-3D über die Grafikkarte; Trackpad-Gesten funktionieren schon heute. Schnittmuster und 3D-Farbe kommen bereits aus einem Material im Bürostandard.

**Synaps** (ergänzt am 6.11.2026, nur aus Website und Doku): Bilder rechnet eine KI aus einer Kamera, die man im Grundriss setzt, dazu Sonnenstudien; das Bild bleibt vom Modell getrennt, Änderungen per Prompt (Boden, Wandfarbe) wirken nur aufs Bild [97, 98]. Echtzeit-3D, Brennweite, 2-Punkt und Schnittbox beschreibt es nicht. Für Lot: Kamera im Grundriss setzen steht schon in Punkt 3; ein KI-Bild aus einer Lot-Kamera wäre später ein weiteres Ziel neben dem Export aus Punkt 5, kein eigener Renderer.

## Vorschlag für Lot

1. **Ein Material, eine Wahrheit:** Ein Material im Bürostandard trägt Schnittschraffur je Maßstab, Ansichtsschraffur, Farbe und Textur in 3D, Kennwerte (λ, Rohdichte, Baustoffklasse) und seinen Rang für Anschlüsse. Duplizieren kopiert immer alles; eine Vorschau zeigt Schnitt, Ansicht und 3D nebeneinander. Schraffur und Textur haben denselben Ursprung und Maßstab.
2. **3D zum Arbeiten:** Schnittbox um Auswahl oder Geschoss mit einer Taste (ADR 0049), Geschosse ein- und ausblenden, Orbit um den Punkt unter dem Zeiger, Auswahl und Bearbeiten auch in 3D.
3. **Kamera wie beim Fotografen:** Perspektiven sind Ansichten im Projekt mit Augenhöhe (Vorschlag 1,60 m), Brennweite in mm (24, 35, 50), 2-Punkt-Perspektive ab Werk an und Seitenverhältnis des Bildfelds. Sie aktualisieren sich live, auch auf dem Planblatt. Kamera setzen im Grundriss: Standpunkt klicken, Blickrichtung ziehen.
4. **Ein Stilumschalter:** Linie (verdeckte Kanten, druckbar als Vektor), Weißmodell, Material, dazu Sonne und Schatten nach Ort, Datum und Uhrzeit. Echtzeit über Metal; im Stillstand rechnet Lot das Bild schöner nach (weiche Schatten, Umgebungsverdeckung).
5. **Fotorealistisch über Export:** Für Wettbewerbsbilder gibt Lot das Modell mit Materialien als USDZ oder glTF aus (Twinmotion, D5, Blender, Keynote), statt einen eigenen Renderer zu bauen.
