# 0040 Bürostandard je Büro

Status: umgesetzt in M5 · 24.10.2026

## Kontext

PLAN.md, Grundentscheidung 10: Darstellungsregeln sind Daten im Bürostandard (2.08), mit guten deutschen Voreinstellungen. Bisher gab es nur den eingebauten Standard. M4 hat eigene Layernamen auf den Bürostandard verschoben.

## Entscheidung

- **Eingebauter Standard plus Abweichungen**: Ein Büro legt `buerostandard.ron` an und schreibt nur hinein, was anders ist. Materialien und Layernamen ergänzen einzelne Einträge, andere Abschnitte (Maßstäbe, Farben für Bauzustände, Büro, Planblätter, Präsentation) ersetzen den Abschnitt. Unbekannte Felder sind ein Fehler, damit Tippfehler auffallen.
- **Neue Abschnitte**: Büro (Name, Anschrift, Kontakt für den Plankopf), Layernamen für DXF und DWG, Unterschriftsfelder im Plankopf, Farben für Präsentationspläne.
- **Ort**: In der App `~/Library/Application Support/Lot/buerostandard.ron`, gelesen beim Start; das Menü Prüfen, Bürostandard bearbeiten legt die Datei mit Vorlage an und öffnet sie. `lot-cli` liest die Datei aus dem Projektordner.

## Alternativen

- **Einstellungsfenster in der App**: bequemer, aber viel Oberfläche für Dinge, die ein Büro einmal einstellt. Kommt, wenn andere Büros Lot nutzen.
- **Bürostandard in der Projektdatei**: Pläne sähen überall gleich aus, aber jedes Projekt hätte eine Kopie, die veraltet. Später als Option beim Weitergeben.

## Folgen

- Änderungen gelten nach einem Neustart der App.
- Schriften und eigene Schriftfelder (Logo) fehlen noch.
