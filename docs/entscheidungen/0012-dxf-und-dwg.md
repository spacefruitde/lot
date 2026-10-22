# 0012 DXF und DWG

Status: **Vorschlag, Entscheidung durch Ben offen** · 22.10.2026

## Kontext

PLAN.md sieht vor: DXF lesen und schreiben wir direkt, DWG über den kostenlosen ODA File Converter als Zwischenschritt. In M0 sollten die Nutzungsbedingungen des Converters geprüft werden. Gebraucht wird das erst in M4.

## Ergebnis der Prüfung

**ODA File Converter:**
- Die FAQ der Open Design Alliance erlaubt die kostenlosen Werkzeuge für Nicht-Mitglieder nur nichtkommerziell.
- Das Community User Agreement (5.9.2025, Abschnitt 3.1.1) erlaubt die Nutzung zum vorgesehenen Zweck, ausdrücklich auch zum Konvertieren. Es verbietet Weitergabe als Teil einer kommerziellen Anwendung und Service-Bureau-Betrieb. Die Rechte sind persönlich und gehen nicht auf den Arbeitgeber über.
- Bewertung: Täglicher Einsatz im Büro ist eine Grauzone, eher unzulässig. Mit Lot ausliefern ist klar verboten. Den Lizenztext im Installer haben wir nicht gesehen.

**dxf-Crate** (0.6, MIT): liest R10 bis R2018. Schraffuren (HATCH) und Ansichtsfenster werden beim Lesen still verworfen. In vier echten DXF-Plänen fehlten dadurch 3 bis 2960 Schraffuren je Datei.

**acadrust** (0.6.6, MPL-2.0, reines Rust): liest und schreibt DXF **und DWG** von R12 bis 2018+, 48 Elementtypen mit Schraffuren. Der DWG-Teil ist aus ACadSharp (MIT) portiert. Im Test wurden vier DXF und zwei echte DWG fehlerfrei gelesen, DWG in 27 bis 60 ms. Sehr jung (seit Januar 2026), häufige Versionen.

## Vorschlag

DXF und DWG über **acadrust**, als Bibliothek (MPL-2.0 erlaubt das), fest gepinnt und hinter einer eigenen Schnittstelle in `lot-io`. Der ODA File Converter entfällt. Abgesichert mit Golden-Tests an echten Plänen aus Bens Büro. Rückfallebene für das Produkt bleibt eine ODA-Mitgliedschaft.

## Alternativen

- Plan wie bisher (dxf-Crate plus ODA File Converter): Lizenz für den Büroeinsatz unklar, Schraffuren fehlen beim Lesen.
- ODA-Mitgliedschaft mit Drawings SDK schon jetzt: rechtlich sauber, kostet und bringt C++ in den Build.
- LibreDWG: GPL-3.0, ausgeschlossen.

## Folgen

Wenn Ben zustimmt: PLAN.md, Abschnitt 4, Punkt 11 anpassen, die Prüfung echter DXF/DWG-Pläne in M4 einplanen.
