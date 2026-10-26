# 0049 Navigation: Projekt-Seitenleiste, Geschosse, Schnitte und Blätter in der App

Status: entschieden von Ben · 26.10.2026; die Seitenleiste testet Ben in der App

## Kontext

Vergleich in `pruefpakete/2026-10-26-bedienung-3/navigation.md` (ADR 0042). Die App zeigt nur Grundriss und 3D; Schnitte, Ansichten, Lageplan und Blätter gibt es nur als Ausgabe der Kommandozeile.

## Entscheidung

Ben am 26.10.2026: „21 muss ich testen - versteh ich noch nicht“, „22 ok“, „23 ok“.

- **Seitenleiste „Projekt“** links mit Suche: Geschosse, Grundrisse, Schnitte und Ansichten, Lageplan, Planblätter, Listen. ⌘K springt auch zu Geschossen, Schnitten, Blättern und Räumen; ⌘[ und ⌘] gehen zurück und vor. Ben probiert es in der App aus.
- **Geschosse als Leiste** mit editierbaren Höhen, ⌘↑ und ⌘↓ wechseln; das Geschoss darunter blass, je Ansicht abschaltbar.
- **Schnitte im Grundriss zeichnen**, Doppelklick auf die Marke öffnet den Schnitt.
- **Planblätter in der App;** eine Ansicht darf auf mehreren Blättern liegen, eine Platzierung ist die „Heimat“ für die Schnittmarke.
- **Nebeneinander mit gemeinsamer Auswahl.**

## Alternativen

- **Eine Ansicht je Blatt wie Revit:** zwingt zu Duplikaten.

## Folgen

- Schnitte und Blätter rechnet der Kern schon (ADR 0026, 0027); neu ist die Ansicht in der App.
