# 0007 2D-Darstellung der Pläne auf dem Bildschirm mit lyon

Status: entschieden für M1, in M0 noch nicht genutzt · 22.10.2026

## Kontext

Die Planansicht auf dem Bildschirm braucht Linien mit exakten Breiten in mm bei jedem Zoom, dazu Haarlinien, Füllungen und Schraffuren.

## Entscheidung

**lyon** (MIT/Apache-2.0) zerlegt Linien und Flächen auf dem Prozessor in Dreiecke, nur wenn sich die Geometrie ändert. Die Linienbreite setzt der Shader in jedem Bild aus Stiftbreite, Maßstab und Zoom. So bleiben Breiten exakt, ohne neu zu zerlegen. Für die vielen geraden Linien kommt voraussichtlich ein eigener Shader für Liniensegmente dazu.

## Alternativen

- **vello** (Apache/MIT): rechnet auf der Grafikkarte, der Renderer gilt laut eigener Angabe als experimentell, ist an eine bestimmte wgpu-Version gebunden und baut die Szene in jedem Bild neu auf.

## Folgen

PDF, SVG und PNG entstehen nicht über lyon, sondern direkt aus den Planobjekten (ADR 0010). Bildschirm und Ausdruck teilen sich die Planobjekte, nicht den Zeichenweg.
