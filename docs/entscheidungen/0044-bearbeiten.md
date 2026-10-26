# 0044 Bearbeiten

Status: entschieden von Ben · 26.10.2026

## Kontext

Vergleich in `pruefpakete/2026-10-25-bedienung-1/bearbeiten.md` (ADR 0042). Lot konnte bisher Ziehen, Wandenden und Undo; Kopieren, Drehen, Spiegeln, Versetzen, Stutzen, Teilen und temporäre Maße fehlten.

## Entscheidung

Ben am 26.10.2026 zu Frage 4 bis 7: „4 ok“, „5 ok“, „6 ok“, „7 ok“.

- **Kürzel aus Revit als Voreinstellung**, zwei Buchstaben: MV verschieben, CO kopieren, RO drehen, MM spiegeln, AR Reihe, OF versetzen, AL ausrichten, TR stutzen, SL teilen, dazu WA Wand, DR Tür, WN Fenster, RM Raum, DI Bemaßung. Die Befehlspalette (⌘K) findet alles über den Namen.
- **Eine Zahleneingabe für alles:** in jeder laufenden Aktion einfach tippen, ohne ins Feld zu klicken. Einheiten („50cm“, „1,26“), Rechnen („2,40+0,365“), relativ und absolut, „3x“ und „3/“ für Reihen. Der Wert bleibt korrigierbar, bis die nächste Aktion beginnt.
- **Temporäre Maße** zum nächsten Bezug, auch bei Mehrfachauswahl, mit EQ zum Mittig-Setzen. Keine Schlösser und Zwangsbedingungen in Version 1.
- **Direkt am Element:** ⌥ beim Ziehen kopiert, Pfeiltasten schieben 1 cm, mit ⇧ 10 cm. Anschlüsse bleiben beim Verschieben und Strecken erhalten, kein Bauteil bekommt dabei eine neue Kennung.
- **Ehrliche Mehrfachauswahl:** „verschieden“ mit Spanne, „+10 cm“ gilt für alle.
- **Verlässliches Undo:** Ansicht und Auswahl kommen nie in den Verlauf. „Wiederholen“ (Enter ohne laufende Aktion) wiederholt den letzten Befehl.
- **Befehle:** Kopieren, Drehen, Spiegeln, Reihe, Versetzen, Ausrichten, Stutzen und Dehnen, Teilen, Verbinden, Kopieren und Einfügen, auch an dieselbe Stelle in einem anderen Geschoss.

## Alternativen

- **Eigene Ein-Tasten-Kürzel** (wie Archicad und SketchUp): schneller zu tippen, aber gegen Bens Muskelgedächtnis aus 15 Jahren Revit.
- **Zwangsbedingungen wie Revit** (Schlösser, gleiche Abstände): mächtig, aber Quelle vieler Fehlermeldungen; kann später kommen.

## Folgen

- Jede Bearbeitung bleibt eine Operation (Regel 7); Kopieren, Drehen und Spiegeln werden neue Operationen in `lot-edit`.
- Die Zahleneingabe ist ein eigener Parser im Kern, damit die Schreibweise überall gleich ist (siehe ADR 0046).
