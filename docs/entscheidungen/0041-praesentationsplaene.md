# 0041 Präsentationspläne

Status: umgesetzt in M5 (Vorschlag), Ben prüft · 24.10.2026

## Kontext

3.11 verlangt Präsentationspläne: Pläne für Bauherren und Wettbewerbe, ohne Werkplaninhalt.

## Entscheidung

- **Präsentationsgrundriss** als eigene Ansicht (`ViewContent::Presentation`): Wände als dunkle Fläche ohne Schraffur, Räume farbig nach Nutzung (DIN 277), Raumname und Fläche groß, die Schrift passt sich der Raumbreite an, keine Maße und Achsen, Nordpfeil und Maßstabsleiste.
- **Farben im Bürostandard** (Abschnitt `presentation`).

## Alternativen

- **Schalter am normalen Grundriss**: weniger Code, aber jede Darstellungsregel bräuchte eine Ausnahme.
- **Export nach Illustrator oder Affinity**: bleibt möglich über PDF und SVG.

## Folgen

- Möbel, Schatten, Bäume und farbige Ansichten fehlen noch; die 3D-Ansicht als Bild gibt es (M2).
