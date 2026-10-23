# 0027 Planblätter und Plankopf

Status: umgesetzt in M3 (Vorschlag), Ben prüft · 23.10.2026

## Kontext

M3 verlangt Planlayout und Plankopf (3.07). Die Abnahme ist das komplette EFH auf Planblättern mit Plankopf als PDF.

## Entscheidung

- **Planblatt** (`PlanSheet`): Nummer, Planinhalt, Format A4 bis A0 quer, Datum, Index und eine Liste platzierter Ansichten. Jede Ansicht (Grundriss eines Geschosses oder Schnitt/Ansicht) hat einen Maßstab und die Lage ihrer Mitte auf dem Blatt in mm.
- **Projektangaben** (`ProjectInfo`) einmal je Projekt: Bauherr, Bauort, Entwurfsverfasser, Gebäudeart.
- **Blattrand** nach DIN EN ISO 5457: Heftrand links 20 mm, sonst 10 mm (Vorschlag).
- **Schriftfeld** unten rechts, 180 mm breit, in Anlehnung an DIN EN ISO 7200: Bauvorhaben; Bauherr, Bauort, Entwurfsverfasser; Planinhalt; Maßstab, Format, Datum, Index, Plannummer. Haben alle Ansichten denselben Maßstab, steht er im Feld, sonst „wie angegeben“.
- **Titel** unter jeder Ansicht: Name unterstrichen, darunter „M 1:100“.
- **Prüfung** `layout_problems`: Ansichten über dem Rand, auf dem Schriftfeld oder übereinander werden gemeldet; das EFH ist dagegen getestet.

## Alternativen

- **Automatische Anordnung**: bequem, aber Planlayout ist Gestaltung. Zuerst feste Lagen, später Hilfen zum Ausrichten.
- **Plankopf als Vorlage aus dem Büro** (SVG oder DWG): kommt mit dem Bürostandard in M5.

## Folgen

- Blätter werden bisher nur in der Projektdatei beschrieben; ein Blatt-Editor in der App folgt.
