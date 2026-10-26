# 0048 Typen und Eigenschaften

Status: entschieden von Ben · 26.10.2026; die Typenverwaltung testet Ben in der App

## Kontext

Vergleich in `pruefpakete/2026-10-26-bedienung-2/typen.md` (ADR 0042). Wandaufbauten sind seit M1 Typen im Projekt; Tür- und Fenstertypen fehlen.

## Entscheidung

Ben am 26.10.2026: „18 ok“, zur Typenverwaltung „19 muss ich testen check ich sonst nicht“, „20 ok“.

- **Typ und Bauteil in einem Inspektor, Herkunft je Feld sichtbar,** mit Zurücksetzen und „in den Typ übernehmen“. Typwerte ändert man direkt mit dem Hinweis „wirkt auf 14 Wände“.
- **Der Typ legt fest, was am Bauteil überschreibbar ist.**
- **Typen-Übersicht als Tabelle** mit Nutzungszahl, Löschen nur mit Ersatztyp; Typen einzeln aus der Büro-Bibliothek, mit Vorher-Nachher-Vergleich. Ben probiert es in der App aus.
- **Eigenschaften:** Kennzeichen und IFC-Klasse jetzt, Brandschutz und Schallschutz mit den Typen, DIN 276 nach Version 1.

## Alternativen

- **Streng wie Revit** (Typwerte nur im Dialog): sicher, aber langsam.

## Folgen

- Tür- und Fenstertypen kommen mit den Öffnungen (ADR 0047).
