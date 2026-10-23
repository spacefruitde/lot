# 0007 2D-Darstellung der Pläne auf dem Bildschirm mit lyon

Status: umgesetzt in M1 · 23.10.2026

## Kontext

Die Planansicht auf dem Bildschirm braucht Linien mit exakten Breiten in mm bei jedem Zoom, dazu Haarlinien, Füllungen und Schraffuren.

## Entscheidung

**lyon** (MIT/Apache-2.0) zerlegt Linien und Flächen auf dem Prozessor in Dreiecke, nur wenn sich die Geometrie ändert. Die Linienbreite setzt der Shader in jedem Bild aus Stiftbreite, Maßstab und Zoom. So bleiben Breiten exakt, ohne neu zu zerlegen. Für die vielen geraden Linien kommt voraussichtlich ein eigener Shader für Liniensegmente dazu.

## Alternativen

- **vello** (Apache/MIT): rechnet auf der Grafikkarte, der Renderer gilt laut eigener Angabe als experimentell, ist an eine bestimmte wgpu-Version gebunden und baut die Szene in jedem Bild neu auf.

## Umsetzung in M1

Die Planansicht zerlegt dieselben Planobjekte wie das PDF, in Blattmillimetern. Zerlegt wird neu, wenn sich der Zoom um mehr als 5 % ändert; so bleibt jede Linie mindestens ein Pixel breit. Die Breiten wachsen mit dem Zoom wie in einem PDF-Betrachter. Der Ansatz mit Breiten im Shader bleibt die Option, wenn große Pläne beim Zoomen ruckeln.

## Folgen

PDF, SVG und PNG entstehen nicht über lyon, sondern direkt aus den Planobjekten (ADR 0010). Bildschirm und Ausdruck teilen sich die Planobjekte, nicht den Zeichenweg.
