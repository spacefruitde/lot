# 0023 Assoziative Maßketten

Status: umgesetzt in M2 (Vorschlag), Ben prüft · 23.10.2026

## Kontext

M2 verlangt assoziative Bemaßung (3.05): Maßketten, die nach einer Änderung von selbst stimmen. Die Abnahme von M2 ist ein EFH-Grundriss mit Maßketten.

## Entscheidung

- **Eine Maßkette speichert Bezüge, keine Maße**: Wandende (an L-Ecken die gemeinsame Ecke, außen oder innen), Laibung einer Öffnung, Wandfläche, Achse oder fester Punkt, dazu Lage und Richtung der Maßlinie. Die Punkte werden bei jedem Zeichnen neu aufgelöst (`lot-geom/src/dimensions.rs`); fehlt ein Element, fällt sein Bezug weg.
- **Darstellung** (`lot-plan/src/dimensions.rs`), Vorschlag nach DIN 1356-1 und DIN 406-11: Maßlinie und kurze Maßhilfslinien dünn, Maßbegrenzung als Schrägstrich unter 45°, Maßzahl über der Linie, lesbar von unten oder von rechts. Unter 1 m in Zentimetern, sonst in Metern, Millimeter hochgestellt (36⁵, 1,13⁵). Bei Öffnungen steht die Höhe unter der Maßlinie.
- **Befehl Bemaßen (M)** an gewählten Wänden: außen in 1,00 m Abstand Wandenden und Laibungen, in 1,70 m das Gesamtmaß.
- Dafür kann Text in allen Ausgaben gedreht werden (PDF, SVG, PNG, Bildschirm).

## Alternativen

- **Maße als feste Zahlen** (2D-Bemaßung wie in CAD): veraltet bei jeder Änderung.
- **Automatische Außenbemaßung ohne gespeicherte Kette**: bequem, aber Ben kann Lage und Inhalt nicht steuern. Kann als Befehl dazukommen, der Ketten anlegt.

## Folgen

- Bogenwände lassen sich noch nicht bemaßen (Radius und Bogenlänge folgen).
- Innenmaße und Höhenkoten fehlen noch; die Bezüge `WallFace` und `Grid` sind dafür schon da.
