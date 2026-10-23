# 0031 Bestand, Abbruch und Neubau

Status: umgesetzt in M4 (Vorschlag), Ben prüft · 23.10.2026

## Kontext

M4 verlangt Bestand/Abbruch/Neubau (2.05). Abnahme: Ein Altbau-Umbau ist nach alten Bestandsplänen modelliert, Abbruch und Neubau sind korrekt eingefärbt.

## Entscheidung

- **Bauzustand** (`Phase`): Bestand, Abbruch, Neubau, als Feld an Wänden, Öffnungen, Stützen, Decken, Durchbrüchen, Treppen, Geländern, Dächern und Räumen. Ohne Angabe gilt Neubau; Projekte ohne Bestand sehen aus wie bisher.
- **Farben** im Bürostandard (`phases`): Bestand grau, Abbruch gelb, Neubau rot (Vorschlag; die Bauvorlagenverordnungen der Länder verlangen diese Farben, die Töne legt das Büro fest).
- **Umbauplan**: Sobald es Bestand oder Abbruch gibt, sind geschnittene Bauteile im Grundriss und im Schnitt nach Bauzustand gefüllt statt mit Materialschraffur. Eine neue Öffnung in einer Bestandswand durchbricht die Wand nicht, ihre Fläche ist gelb (Ausbruch); eine zugemauerte Öffnung ist rot und ohne Symbol. Öffnungen in abgebrochenen Wänden zeigen kein Symbol. Räume zeigen den Neuzustand mit den Flächen ohne abgebrochene Wände. Was keinen Bauzustand hat (Erdreich, Referenzmodelle), bleibt wie gewohnt.
- **Bauzustände als Ansicht** (`PhaseFilter`): Umbauplan (alles), Bestand (vor dem Umbau, schwarz-weiß, mit den Räumen von früher) und Neuzustand (Bestand und Neubau). Das 3D zeigt den Neuzustand.
- In der App setzt der Inspektor den Bauzustand der Auswahl, ebenso der Befehl `SetPhase`.

## Alternativen

- **Phasen wie in Revit** (Phase erstellt, Phase abgebrochen, beliebig viele Phasen): mächtiger, aber für LP 1 bis 4 eines Umbaus reichen drei Zustände.
- **Bauzustand als eigene Liste neben den Elementen**: hält die Elemente schlank, aber jedes Werkzeug müsste die Liste mitpflegen.

## Folgen

- Bauantragspläne sind farbig; ob das Büro Schraffur und Farbe zugleich will, ist eine Prüffrage.
- Mehrere Umbauphasen nacheinander gehen noch nicht.
