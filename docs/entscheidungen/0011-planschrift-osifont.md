# 0011 Planschrift osifont

Status: umgesetzt in M0, Auslieferung im Produkt offen · 22.10.2026

## Kontext

Pläne brauchen eine Normschrift nach DIN EN ISO 3098. PLAN.md nennt osifont und fragt, ob wir sie mit der App ausliefern dürfen.

## Entscheidung

- Datei `osifont-lgpl3fe.ttf` aus github.com/hikikomori82/osifont, unverändert in `core/assets/fonts/`.
- Lizenz dieser Datei: **LGPL-3.0 mit GPL-Font-Ausnahme**. Laut Namenstabelle der Schrift macht das Einbetten in ein Dokument dieses nicht lizenzpflichtig. Teilmengen in PDFs sind damit erlaubt. Die Schrift erlaubt Einbetten ohne Einschränkung (fsType 0).
- Die Schrift wird **zur Laufzeit als eigene Datei geladen**, nicht in das Programm einkompiliert. Das entspricht der Regel „LGPL nur dynamisch“ (CLAUDE.md, Punkt 12).
- osifont enthält keine deutschen Anführungszeichen („“). Ein Test prüft, dass jeder Text auf dem Blatt nur vorhandene Zeichen nutzt.

## Alternativen

- `osifont.ttf` (GPL-3 mit Font-Ausnahme): gleiche Zeichen, strengere Lizenz.
- OpenGost Type A/B (laut openSUSE OFL-1.1, Lizenz beim Ursprung nicht geprüft): russische Normschrift im verwandten Stil.
- Isonorm (kommerziell), ISOCPEUR (Autodesk): nicht frei.

## Folgen

Für das spätere Produkt nach üblicher Auslegung zulässig: die Datei unverändert und getrennt im App-Bundle, mit Lizenztext und Link zum Quelltext. Das ist keine Rechtsberatung. Vor dem Produktstart prüfen.
