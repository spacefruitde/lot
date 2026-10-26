# Paket Bedienung 4: eigene Bauteile, Planerstellung, 3D und Materialien

26.10.2026. Etwa 20 Minuten, nur lesen und antworten, kein Mac nötig. Viertes Paket nach deiner Liste vom 26.10.2026: „eigene bauteile bauen - wir bauen viel im bestand und müssen häufig individuelle situationen darstellen“ und „planerstellung / planköpfe / 3d / perspektiven / materialien etc“.

## Was drin ist

- **`bauteile-bauen.md`**, **`planerstellung.md`**, **`3d-materialien.md`**: je Thema ein Kurzvergleich (Revit, Archicad, Vectorworks, Allplan, Lot heute), was Nutzer dort am meisten ärgert, und mein Vorschlag für Lot. Je eine Seite.
- `lot-heute.md`: was Lot bei diesen Themen heute kann.
- `recherche-bauteile-bauen.md`, `recherche-planerstellung.md`, `recherche-3d-materialien.md`: die Recherche mit Quellen. Die Foren sperrten diesmal das direkte Lesen; Forenkritik stammt aus Suchauszügen und ist mit Stern markiert.

Wo dein Alltag etwas anderes sagt, gilt dein Alltag.

## Fragen

Antwort jeweils „Vorschlag ok“, „wie Revit“ oder ein Satz, was anders sein soll.

**Eigene Bauteile und Bestand** (`bauteile-bauen.md`)

1. **Häufigste Bestandssituationen:** Welche kommen bei dir am meisten vor? Vorschlag für die Reihenfolge: Wand mit wechselnder Dicke und schiefen Fluchten, Kamin, Unterzug und Vorsprung, Nische und Schlitz, Holzbalkendecke, Kappendecke und Gewölbe, Fachwerk, Bestandstreppe.
2. **Wand aus dem Aufmaß und Körper an Ort und Stelle:** Bestandswand mit eigenen Punkten je Seite statt Stückelung; jeder gezeichnete Körper ist sofort ein Bauteil mit Art, Material und Bauzustand, Abzugskörper schneiden Wände und Decken und erscheinen im Grundriss. Vorschlag ok?
3. **Eigene Bauteile als Typ:** Körper „als Bauteil sichern“, Parameter durch benannte Maße im Modell, oder in Worten beschreiben und Claude schreibt die Definition. Welches Bauteil würdest du als erstes bauen (etwa Einbauküche, Sprossenfenster, Kamin)?
4. **Bauzustand je Teilbereich und Schicht:** Abbruchbereich als Rechteck in einer Bestandswand, einzelne Schicht abbrechen, ohne die Wand zu teilen. Vorschlag ok?

**Planerstellung und Planköpfe** (`planerstellung.md`)

5. **Plankopf als Baustein im Bürostandard:** liest Maßstab, Format, Inhalt, Index vom Blatt; Büro-Plankopf mit Logo plus Bauantrag-Variante mit Unterschriftsfeldern; direkt auf dem Blatt bearbeitet, wirkt auf alle Blätter. Vorschlag ok? Wenn du magst, leg deinen Büro-Plankopf (PDF reicht) lokal in `~/Documents/Lot/buero/`, nicht ins Repository; dann baue ich ihn nach.
6. **Planstand ausgeben:** Index vergeben, einfrieren, PDF-Satz nach Büroschema, Planliste; Lot markiert danach geänderte Blätter. Wie sehen deine Plannummern und Indizes aus? Vorschlag: „102“ mit Index „a, b, c“ und Dateiname „Projekt_102_Grundriss-EG_b.pdf“.
7. **Formate und Faltung:** DIN- und freie Formate, Rahmen, Heftrand und Faltmarken nach DIN 824 automatisch, Geschosspläne deckungsgleich. Faltest du mit Heftrand für den Ordner? Vorschlag: ja, ab Werk mit Heftrand.

**3D, Perspektiven und Materialien** (`3d-materialien.md`)

8. **Ein Material, eine Wahrheit:** Schnittschraffur, Ansichtsschraffur, 3D-Farbe und Textur, Kennwerte und Rang für Anschlüsse in einem Material, Duplizieren kopiert alles. Vorschlag ok?
9. **Kamera wie beim Fotografen:** Perspektiven als Ansichten im Projekt, Augenhöhe 1,60 m, Brennweite in mm, 2-Punkt ab Werk an, live auch auf dem Planblatt. Vorschlag ok?
10. **Bilder:** Was brauchst du in LP 1 bis 4 wirklich? Vorschlag: Linie, Weißmodell, Material sowie Sonne und Schatten in Lot; fotorealistische Wettbewerbsbilder über Export (USDZ, glTF) nach Twinmotion, D5 oder Blender, kein eigener Renderer.

## Wie es weitergeht

Deine Antworten werden ADRs. Eigene Bauteile und Planerstellung gehören zu M6 und zum Thema Navigation mit Blättern; ich baue sie nach Auswählen und Bearbeiten in die Reihenfolge ein, die du festlegst.
