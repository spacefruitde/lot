# 0018 Bürostandard als Daten

Status: umgesetzt in M1, Inhalte sind ein Vorschlag · 23.10.2026

## Kontext

Darstellungsregeln sollen Daten sein, keine Programmlogik (PLAN.md, Grundentscheidung 10), und mit guten deutschen Voreinstellungen kommen.

## Entscheidung

`core/crates/lot-plan/src/buerostandard.ron` enthält:

- **Materialien** mit Namen, 3D-Farbe und Schnittmuster (Linien, Kreuzschraffur, Vollfläche, Dämmstoff weich oder hart, leer)
- **Maßstabsbereiche** mit Detailgrad und Stiften: bis 1:50 alle Schichten mit Materialmuster, bis 1:100 nur die Tragschicht gemustert, darüber gefüllte Wände

Wandaufbauten gehören zum Projekt und verweisen auf Materialien über einen Schlüssel („kalksandstein“). Unbekannte Materialien erscheinen leer und grau.

Der Bürostandard ist in das Programm eingebaut. Ein eigener Bürostandard je Büro (2.08) kommt in M5 als Datei im Projekt oder im Benutzerordner.

## Alternativen

- Regeln im Code: schneller zu schreiben, aber für Ben nicht prüfbar und für andere Büros nicht anpassbar.
- Bürostandard in der Projektdatei: kommt in M5; bis dahin reicht ein gemeinsamer Standard.

## Folgen

Ben prüft Schraffuren und Linienbreiten im Prüfpaket M1. Änderungen sind Datenänderungen; die Golden-Tests zeigen danach jede Abweichung.
