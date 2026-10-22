# 0009 Testprojekte als RON-Textdateien

Status: umgesetzt in M0 · 22.10.2026

## Kontext

Testprojekte sollen lesbare Textdateien sein, die Claude schreiben kann und die sich im Git-Diff lesen lassen (PLAN.md, Abschnitt 4).

## Entscheidung

**RON** (Rusty Object Notation, Crate `ron` 0.12, MIT/Apache). Je Projekt ein Ordner `testprojekte/<name>/` mit `projekt.ron`. Erlaubt Kommentare und schreibt Bauteile lesbar, etwa `Wall(start: (x: 0.0, y: 0.0), ...)`. Testprojekte durchlaufen dieselben Prüfregeln wie das Modell in der App.

## Alternativen

- JSON: verbreiteter, aber ohne Kommentare und mit mehr Klammern.
- TOML: für tief verschachtelte Bauteile unhandlich.

## Folgen

Erstes Testprojekt: `testprojekte/m0-wand`.
