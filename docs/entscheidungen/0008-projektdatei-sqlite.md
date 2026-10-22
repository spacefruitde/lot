# 0008 Projektdatei als SQLite

Status: umgesetzt in M0 · 22.10.2026

## Kontext

PLAN.md legt fest: Die Projektdatei `.lot` ist eine SQLite-Datei mit Elementen und Operationsprotokoll.

## Entscheidung

- **rusqlite 0.40** (MIT) mit eingebautem SQLite 3.53 (gemeinfrei). Braucht nur einen C-Compiler, baut auf Mac und Linux.
- Formatversion 1, gekennzeichnet über `application_id` („LOT1“) und `user_version`. Fremde Datenbanken und neuere Formate lehnt Lot mit einer klaren Meldung ab.
- Tabellen: `meta` (Name, Programmversion), `elements` (eine Zeile je Element, Parameter als JSON), `operations` (Protokoll in Reihenfolge, als JSON).
- Gespeichert wird zuerst in eine Nachbardatei, die dann umbenannt wird. Ein Absturz beim Speichern lässt die alte Datei unversehrt.
- Beim Laden gelten dieselben Prüfregeln wie für jede Operation.

## Alternativen

- Eigenes Binärformat oder ZIP mit JSON: kein Vorteil gegenüber SQLite, aber kein laufendes Speichern einzelner Änderungen.

## Folgen

In M0 speichert die App auf Befehl (⌘S). Laufendes Speichern jeder Operation folgt, sobald die App mit Dokumenten arbeitet. Das Schema ist dafür schon ausgelegt.
