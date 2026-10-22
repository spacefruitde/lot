# 0010 Planausgabe als PDF, SVG und PNG

Status: umgesetzt in M0 · 22.10.2026

## Kontext

Pläne müssen maßstäblich und mit exakten Linienbreiten gedruckt werden, Schraffuren und Schrift eingeschlossen. Für Golden-Tests braucht es ein textbasiertes Format, für das iPhone ein Bild.

## Entscheidung

- Aus einer Ansicht entstehen Planobjekte in `lot-plan` (Linie mit Stiftbreite in mm, Fläche, Text). Alle Ausgaben lesen dieselben Planobjekte.
- **PDF mit krilla 0.8** (MIT/Apache-2.0, aus dem Typst-Umfeld). 1 mm sind 72/25,4 pt. Die Schrift wird als Teilmenge eingebettet. Ein Test prüft die Linienbreite im PDF auf sieben Stellen genau.
- **Schraffuren sind einzelne Linien**, die `lot-plan` selbst berechnet, auf einem festen Raster durch den Blattursprung. Damit hat jede Schraffurlinie die exakte Stiftbreite und sieht in jedem Betrachter und im späteren DXF gleich aus.
- **SVG** schreiben wir selbst, mit drei Nachkommastellen. Es dient den Golden-Tests.
- **PNG** rastert tiny-skia (BSD-3-Clause) aus denselben Planobjekten, mit 200 dpi.

## Alternativen

- **pdf-writer** direkt: Grundlage von krilla, aber ohne Teilmengen von Schriften. Das müssten wir selbst bauen.
- Schraffur als PDF-Muster (Pattern): kleiner, aber je nach Betrachter mit Nahtstellen und ohne Kontrolle über die Lage.

## Folgen

PDF-Linienbreiten stimmen nur beim Druck in Originalgröße. „An Seite anpassen“ skaliert sie mit. Das steht auf jedem Prüfblatt.
