# 0020 Gebogene Wände

Status: umgesetzt in M1, Ben prüft an den Knoten 28 bis 30 · 23.10.2026

## Kontext

PLAN.md, Abschnitt 2: „Gebogene Wände sind möglich, intern werden sie facettiert.“ Der Wandknoten-Katalog verlangt den Fall „gebogene Wand an gerader Wand“ (Abschnitt 7).

## Entscheidung

- **Modell:** Eine Wand bekommt den Parameter `sagitta`, den Stich (Pfeilhöhe): Abstand der Bogenmitte von der Sehne zwischen Anfang und Ende, links der Zeichenrichtung positiv. 0 heißt gerade. Höchstens ein Halbkreis, und der Innenradius muss größer als 1 cm bleiben. Gespeichert wird nur der Stich, Mittelpunkt und Radius sind abgeleitet (CLAUDE.md, Regel 6).
- **Länge** meint bei Bogenwänden die Bogenlänge. Öffnungen sitzen entlang des Bogens, ihre Laibungen zeigen zum Mittelpunkt. Ändert man die Länge, bleibt der Öffnungswinkel, Sehne und Stich wachsen mit.
- **Anschlüsse:** Für die Knoten wird eine Bogenwand in gerade Facetten zerlegt, höchstens 2 mm vom Bogen entfernt (gemessen an der äußeren Fläche), höchstens 64 Stück. Die Facetten stoßen untereinander immer auf Gehrung, für alle anderen Wände sind sie gewöhnliche Wände. So gelten alle Regeln aus ADR 0017 unverändert. Die Schichtflächen entstehen in einem Stück über alle Facettenecken, damit an den Stößen keine Fugen bleiben. Freie Enden sind radial.
- **Zeichnen:** Werkzeug Bogenwand (Taste B): Anfang klicken, Ende klicken, dann den Stich mit dem Zeiger ziehen oder tippen. Im Inspektor lässt sich der Stich jeder Wand ändern, auch von 0 auf einen Bogen.

## Alternativen

- **Echte Kreisbögen in der 2D-Geometrie:** Clipper2 rechnet nur mit Polygonen, und 3D mit manifold-rust ebenso. Exakte Bögen hätten einen zweiten Geometriekern bedeutet, ohne sichtbaren Gewinn: 2 mm sind in 1:50 vier hundertstel Millimeter.
- **Radius statt Stich als Parameter:** Bei festem Radius lassen sich die Enden nicht beliebig weit auseinanderziehen. Der Stich ist immer gültig, Radius und Öffnungswinkel kann der Inspektor später als Anzeige dazu nehmen.
- **Bogen über drei beliebige Punkte:** üblich in CAD, aber der dritte Klick als Stich ist für Wände direkter. Kommt bei Bedarf als zweite Eingabeart dazu.

## Folgen

- Das Dämmstoffmuster läuft entlang des Bogens; seine Periode gilt auf der Bezugslinie und wird nach außen etwas länger, nach innen etwas kürzer.
- Wände, die mit Wänden verschoben werden, folgen an Bogenwänden ohne Richtungsbezug mit (an geraden Wänden bleiben sie in ihrer Richtung).
- DXF und IFC (M4) geben Bögen als echte Bögen aus, nicht als Facetten.
