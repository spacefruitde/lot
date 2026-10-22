# 0002 Einheiten und Koordinaten

Status: umgesetzt in M0 · 22.10.2026

## Kontext

Modell, Geometrie, Pläne und Dateien müssen dieselben Einheiten verwenden. IFC rechnet in SI-Einheiten, Pläne in Millimetern auf dem Papier.

## Entscheidung

- **Modell:** Längen in Metern als `f64`. Rechtshändiges System, x und y im Grundriss, z nach oben (wie IFC).
- **Blatt:** Millimeter, Ursprung links unten, y nach oben. Umrechnung über den Maßstab: Papier-mm = Modell-m × 1000 / Maßstabszahl.
- **Grafikkarte:** `f32`, erst beim Hochladen umgerechnet.
- **Schnitte im Grundriss** werden auf 1 µm gerundet, damit Ausgaben zwischen Läufen gleich bleiben (Golden-Tests).

## Alternativen

- Millimeter als Ganzzahl im Modell: exakt, aber unbequem für Neigungen, Bögen und Booleans, die ohnehin in Gleitkomma rechnen.
- Millimeter als `f64`: möglich, aber IFC und die meisten Bibliotheken erwarten Meter.

## Folgen

Für 2D-Booleans mit Clipper2 rechnen wir auf einem Ganzzahlraster (ADR 0005). Die Umrechnung dafür liegt an einer Stelle in `lot-geom`.
