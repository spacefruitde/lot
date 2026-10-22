# 0016 PDF als Unterlage

Status: vorgemerkt für M4 · 22.10.2026

## Kontext

Alte Bestandspläne als PDF sollen als maßstäbliche Unterlage dienen (6.05).

## Entscheidung

Wie geplant: auf dem Mac PDFKit (Teil von macOS), später auf Windows PDFium über `pdfium-render` (MIT/Apache, PDFium selbst BSD-3 und Apache-2.0). Fertige PDFium-Bibliotheken gibt es bei bblanchon/pdfium-binaries.

## Folgen

Keine in M0. Vor Windows die Lizenzhinweise der Bestandteile im PDFium-Binary prüfen (FreeType, libjpeg-turbo, zlib, ICU).
