# 0043 Auswählen

Status: entschieden von Ben · 26.10.2026

## Kontext

Vergleich in `pruefpakete/2026-10-25-bedienung-1/auswaehlen.md` (ADR 0042). Lot kannte bisher nur Klick und ⇧-Klick; Ziehen auf einem Bauteil verschob es sofort, auch wenn es nicht gewählt war.

## Entscheidung

Ben am 26.10.2026 zu Frage 1 bis 3: „1 ok“, „2 ok“, „3 ok“.

- **Ziehen verschiebt nur, was gewählt ist.** Ziehen im Leeren oder auf einem nicht gewählten Bauteil zieht einen Rahmen auf.
- **Rahmen wie Revit:** von links nach rechts wählt, was ganz innen liegt, von rechts nach links auch Angeschnittenes. Mit ⌥ wird der Rahmen zum Lasso.
- **⇧ fügt hinzu oder nimmt heraus**, mit Klick und mit Rahmen.
- **Vorhervorhebung immer**, mit kurzem Namen am Zeiger („Wand · KS 17,5 · 4,20 m“).
- **Tab blättert durch alles unter dem Zeiger**, die Wandkette ist ein Kandidat. Am Zeiger steht „2 von 4: Wandkette (3 Wände)“.
- **SA wählt Gleiche:** gleicher Typ im Geschoss, ein zweites SA im ganzen Projekt. Der Inspektor zeigt danach die Anzahl je Art und Typ zum Herausfiltern.
- **Doppelklick wählt ein Teil** im gewählten Bauteil (Schicht, Öffnung, Wandende, Anschlussknoten), Esc geht zurück.
- **Unterlagen und Referenzmodelle sind nie wählbar, aber fangbar.** Fixierte Bauteile sind wählbar, lassen sich aber nicht verschieben; ein Schloss am Zeiger nennt den Grund.
- **Auswahlverlauf:** ⌘← holt die vorige Auswahl zurück.
- **Eine Auswahl für alle Ansichten.**

## Alternativen

- **Wie Archicad** (Rahmen nur mit eigenem Werkzeug): ein Modus mehr.
- **Ziehen verschiebt sofort** (wie bisher in Lot): auf dem Trackpad verschiebt man so aus Versehen.

## Folgen

- Auswahl, Vorhervorhebung und Tab-Kandidaten rechnet der Kern (`lot-edit`), die App zeichnet nur.
