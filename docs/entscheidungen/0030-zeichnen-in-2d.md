# 0030 2D-Zeichnen

Status: umgesetzt in M3 · 23.10.2026

## Kontext

M3 verlangt 2D-Zeichnen (3.09): Linien, Flächen und Texte, die nicht zum Modell gehören, etwa Terrassen, Nordpfeile, Hinweise oder Ergänzungen in Schnitten.

## Entscheidung

- **2D-Element** (`Annotation`, IfcAnnotation) gehört zu genau einer Ansicht: Grundriss eines Geschosses (Koordinaten in m wie das Modell), Schnitt oder Ansicht (x entlang der Schnittlinie, y als Höhe über ±0,00, beides in m) oder Planblatt (mm auf dem Papier).
- Formen: Linienzug, Fläche mit wahlweiser Schraffur eines Materials aus dem Bürostandard, Kreis, Text. Strichstärke in mm auf dem Papier, wahlweise gestrichelt; Schrifthöhe in mm auf dem Papier.
- **App**: Werkzeuge Linie (L) mit Fangen, rechten Winkeln und getippter Länge, und Text (X). Texte werden nach dem Setzen gewählt und im Inspektor geändert. 2D-Elemente lassen sich wählen, verschieben und löschen wie Bauteile.

## Alternativen

- **2D-Elemente im Modellraum ohne Ansicht**: erschienen in jedem Grundriss und jedem Maßstab, das will man fast nie.

## Folgen

- Bogen, Bemaßung in Schnitten und Schraffuren mit eigenem Muster folgen später. In Schnitten und auf Planblättern legt man 2D-Elemente bisher nur in der Projektdatei an.
