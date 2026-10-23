# 0034 Gelände

Status: umgesetzt in M4 (Vorschlag), Ben prüft · 23.10.2026

## Kontext

Bis M3 stand in Schnitten und Ansichten eine feste Geländelinie auf -0,30 m. M4 verlangt Gelände (1.12).

## Entscheidung

- **Gelände** (`Terrain`): Höhenpunkte (x, y, Höhe über ±0,00), dazwischen ein Dreiecksnetz nach Delaunay. Außerhalb der Punkte gilt die Höhe am Rand.
- **Erdreich** als Körper: unter der Oberfläche bis 1 m unter den tiefsten Punkt, ohne die Baugrube der Geschosse, die unter das Gelände reichen.
- **Schnitt**: Das Erdreich ist geschnitten und schraffiert (Material `erdreich`, Linien unter 135°, Vorschlag). **Ansicht**: Geländelinie vor dem Gebäude; was darunter liegt, ist nicht zu sehen. **3D**: Erdreich grün.
- Ohne Gelände bleibt die Linie auf -0,30 m.

## Alternativen

- **Geländeschnitte als Polylinien je Ansicht**: einfacher, aber jede Ansicht müsste man pflegen.
- **NHN-Höhen**: Bauantragspläne brauchen m ü. NHN; das kommt mit dem Bauantrag (M5) als Bezug von ±0,00 in den Projektangaben.

## Folgen

- Geplantes Gelände (Aufschüttung, Abgrabung) und Böschungen fehlen noch.
