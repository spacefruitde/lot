# 0032 Unterlagen

Status: umgesetzt in M4 · 23.10.2026

## Kontext

Alte Bestandspläne liegen als PDF, oft als Scan, manchmal als DXF oder DWG vor (6.05, 6.03). Man zeichnet darauf nach.

## Entscheidung

- **Unterlage** (`Underlay`): Datei, Seite, Lage der linken unteren Ecke, Meter je Einheit der Vorlage (PDF-Punkt, Bildpunkt oder Zeichnungseinheit), Drehung, Deckkraft. Sie gehört zu einem Geschoss und erscheint nur in dessen Grundriss in der App, nicht in gedruckten Plänen.
- **Dateien im Projekt**: Die Datei selbst liegt in der Projektdatei (Tabelle `attachments`), bei Testprojekten im Projektordner. So bleibt ein Projekt eine Datei.
- **Anzeige**: PDF und Bilder als Textur unter dem Plan, blass; PDF-Seiten rendert auf dem Mac PDFKit mit 150 dpi (ADR 0016), PNG liest der Kern selbst. DXF und DWG als dünne graue Linien; an ihren Endpunkten und Ecken rastet der Zeiger ein.
- **Kalibrieren** (Taste K): zwei Punkte einer bekannten Strecke klicken, das wahre Maß tippen; der erste Punkt bleibt. **Ausrichten**: zwei Punkte einer Linie, die waagerecht oder senkrecht liegen soll. Vor dem Kalibrieren gilt bei PDF 1:100, bei Bildern 150 dpi in 1:100, bei DXF und DWG die Einheit der Datei.

## Alternativen

- **Vektoren aus PDF lesen** (lopdf): erlaubt Fangen an Linien in Vektor-PDF, hilft aber bei Scans nicht. Später, wenn Bens Bestandspläne oft Vektor-PDF sind.
- **Datei nur verknüpfen**: kleiner, aber Projekte gehen beim Weitergeben kaputt, und die App darf ohne Erlaubnis nicht in `~/Documents` lesen.

## Folgen

- Unter Linux und in Planausgaben gibt es nur PNG-Unterlagen; PDF braucht das Betriebssystem.
