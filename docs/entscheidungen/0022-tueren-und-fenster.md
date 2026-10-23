# 0022 Türen und Fenster

Status: umgesetzt in M2 (Vorschlag), Ben prüft · 23.10.2026

## Kontext

M2 verlangt Türen und Fenster (1.06), die sich in der App setzen und entlang der Wand ziehen lassen, und eine Darstellung je Maßstab (PLAN.md, Abschnitt 4: 1:200 vereinfacht, 1:100 Rahmen vereinfacht und Aufschlag, 1:50 mit Rahmen, Laibung, Anschlag).

## Entscheidung

- **Eine Öffnung trägt eine Art**: nichts (Durchbruch), Tür oder Fenster. Maße bleiben Rohbaumaße der Öffnung. Eine Tür hat die Aufschlagseite (links oder rechts der Wand in Zeichenrichtung) und den Anschlag DIN links oder DIN rechts nach DIN 107:1974-04.
- **Darstellung** (`lot-plan/src/floor_plan.rs`), Vorschlag nach DIN 1356-1:
  - Tür: Türblatt rechtwinklig an den Bändern, Aufschlag als Viertelkreis; in 1:50 das Türblatt mit 4 cm Dicke.
  - Fenster: 1:200 eine Linie, 1:100 Rahmen (7 cm), 1:50 Rahmen und Glaslinie. Der Rahmen sitzt an der Außenkante der Tragschicht, wenn außen gedämmt ist, sonst in der Mitte der Tragschicht.
  - Öffnungen ganz über der Schnittebene (Oberlichter, hohe WC-Fenster) gestrichelt.
- **Werkzeuge**: Tür (T) und Fenster (F) setzen sich mit einem Klick auf eine Wand mittig zur Klickstelle, in üblichen Rohbaumaßen nach DIN 18100 (Tür 1,01 × 2,135 m, Fenster 1,26 × 1,385 m mit Brüstung 0,885 m). Die Tür schlägt zur geklickten Seite auf, die Bänder sitzen am näheren Wandende. Gewählte Öffnungen gleiten beim Ziehen entlang ihrer Wand. „Wand umdrehen“ liegt dafür jetzt auf ⇧F.
- **Regel**: Öffnungen in derselben Wand dürfen sich nicht überschneiden.

## Alternativen

- Tür- und Fenstertypen mit Rahmenprofilen, Flügeln und Sprossen (Revit-Familien): für LP 1 bis 4 zu viel; kommt als Typ mit Parametern, wenn die Darstellung abgenommen ist.

## Folgen

- Laibungsdämmung, Fensterbänke, Zargen und Anschläge im Detail fehlen noch (Prüfpaket M2).
- Zweiflügelige Türen, Schiebetüren und Fenstertüren mit eigenem Symbol folgen nach Bens Rückmeldung.
