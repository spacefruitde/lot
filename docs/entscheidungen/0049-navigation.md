# 0049 Navigation: Projekt-Seitenleiste, Geschosse, Schnitte und Blätter in der App

Status: entschieden von Ben · 26.10.2026; umgesetzt am 28.10.2026 bis auf die offenen Punkte unter „Folgen“, die Seitenleiste testet Ben in der App

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
- Umgesetzt (28.10.2026): Ansichten der App (`navigation::ViewRef`), Seitenleiste mit Suche, ⌘K auch für Ansichten und Räume, Zurück und Vorwärts mit Kamera, Geschossleiste mit OKFF und Höhe (die Geschosse darüber rücken mit), ⌘↑ und ⌘↓, Geschoss darunter blass und je Grundriss abschaltbar, Werkzeuge Schnitt (SC) und Ansicht (AN), Doppelklick auf die Schnittmarke, Planblätter anlegen, Ansichten aus der Seitenleiste aufs Blatt ziehen, verschieben, Maßstab, wegnehmen, Heimat (`SectionView::home`), zwei Ansichten nebeneinander mit gemeinsamer Auswahl, Auswahl im Schnitt.
- Offen: Knick in der Schnittlinie; Blicktiefe als Griff im Plan (heute im Inspektor); Plannummer der Heimat an der Schnittmarke (Frage im Prüfpaket); gleiche Lage über Blätter per Anker; die Abschaltung des blassen Geschosses gilt je Sitzung, gespeichert wird sie mit den Ansichten aus ADR 0050; 3D mit Schnittbox kommt mit Paket 4.
