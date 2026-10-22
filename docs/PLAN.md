# Lot: Plan für Version 1

Arbeitstitel „Lot“, nach dem Lot der Maurer; auf Englisch heißt „lot“ auch Grundstück. Der Name kann sich noch ändern.

Stand 21.10.2026. Grundlage ist Bens Priorisierung im Artefakt „BIM-Werkzeug Funktionsumfang“ (https://claude.ai/artifact/RvTLKor9isErPdeBmrpAZ8). Die Positionsnummern (OZ) unten entsprechen denen im Artefakt.

## 1. Kurzfassung

- **Was:** ein natives BIM-Werkzeug für kleine Architekturbüros. Mac zuerst, Windows später möglich.
- **Version 1:** LP 1-4, vom Entwurf bis zum Bauantrag. Alle Bauaufgaben sind dabei: Einfamilienhaus, Mehrfamilienhaus, Umbau und Sanierung, Gewerbe und Büro, öffentliche Bauten, Innenausbau.
- **Für wen zuerst:** Ben und sein Büro. Danach als Produkt für andere kleine Büros.
- **Wer baut:** Ben entscheidet und prüft fachlich. Claude schreibt den Code.
- **Die Engstelle:** Ben hat weniger als 5 Stunden pro Woche. Code entsteht schnell, Prüfzeit nicht. Deshalb ist der ganze Ablauf darauf ausgelegt, dass Ben in kurzer Zeit viel prüfen kann (Abschnitt 5).
- **Leitsatz:** Wenige Werkzeuge, gute Voreinstellungen, und Pläne, die beim ersten Ausdruck stimmen.

## 2. Produktthese

### Was an Revit stört (Hypothesen, Ben ersetzt sie durch seine Liste)

1. Gute Pläne brauchen zu viele Einstellungen: Ansichtsvorlagen, Objektstile, Sichtbarkeit, Filter.
2. Familien zu bauen und anzupassen ist eine eigene Disziplin.
3. Deutsche Planungspraxis ist nicht eingebaut. DIN 277, WoFlV, Abstandsflächen, Bauantrag und die Farben für Bestand, Abbruch und Neubau gehen nur über Umwege.
4. Revit ist langsam: Die Neuberechnung läuft auf einem Prozessorkern, große Modelle werden zäh.
5. Bedient wird über Dialoge und Eigenschaftenpaletten statt direkt am Bauteil.
6. Es gibt Revit nicht für den Mac.

### Was Lot anders macht

- Deutsche Konventionen sind die Voreinstellung. Man muss sie nicht erst als Vorlage bauen.
- Ein Grundriss sieht ohne Einstellungen aus wie von Hand gezeichnet, mit Linienstärken, Schraffuren und dem passenden Detailgrad je Maßstab.
- Bauteile fasst man direkt an und zieht sie. Zahlen tippt man nur, wenn man will.
- Wandanschlüsse stimmen automatisch, auch bei unterschiedlichen Aufbauten.
- Prüfungen laufen während der Arbeit mit statt am Ende in Excel: Abstandsflächen, Flächen, Barrierefreiheit, Rettungswege, KI-Prüfung.
- Die Neuberechnung läuft parallel und nur für das, was sich geändert hat.

### Bens Liste

_Noch offen. Ben trägt hier ein, was ihn an Revit täglich aufregt, möglichst mit Situationen aus echten Projekten. Diese Liste wiegt schwerer als die Hypothesen oben._

## 3. Umfang

### Version 1: Kern (49 Positionen)

| Bereich | Positionen |
|---|---|
| Modellieren | 1.01 Wände mit Schichtaufbau, 1.02 saubere Wandanschlüsse, 1.03 Decken und Bodenplatten, 1.04 Dächer, 1.05 Gauben und Dachfenster, 1.06 Türen und Fenster, 1.07 Treppen, 1.08 Geländer und Brüstungen, 1.09 Stützen und Unterzüge, 1.10 Pfosten-Riegel-Fassaden, 1.11 Schlitze und Durchbrüche, 1.12 Gelände, 1.14 Möblierung und Ausstattung, 1.15 eigene Bauteile bauen |
| Projektstruktur | 2.01 Geschosse und Höhenbezüge, 2.02 Achsraster, 2.03 Räume automatisch erkennen, 2.04 Varianten, 2.05 Bestand/Abbruch/Neubau, 2.08 Bürostandard |
| Pläne | 3.01 Grundrisse, 3.02 Schnitte und Ansichten, 3.03 Darstellung je Maßstab, 3.04 Linien und Schraffuren nach DIN 1356, 3.05 assoziative Bemaßung, 3.06 Raumstempel, 3.07 Planlayout und Plankopf, 3.08 PDF-Export, 3.09 2D-Zeichnen, 3.11 Präsentationspläne, 3.13 flüssige 3D-Ansicht |
| Mengen | 4.01 Flächen nach DIN 277, 4.02 Wohnflächen nach WoFlV, 4.03 BRI und Kennwerte, 4.05 Tür- und Fensterlisten |
| Genehmigung | 5.01 Abstandsflächen, 5.02 Bauantragspläne, 5.03 Barrierefreiheit (DIN 18040), 5.05 Rettungswege |
| Austausch | 6.02 IFC-Import, 6.03 DWG/DXF-Import, 6.04 DWG-Export, 6.05 PDF als Unterlage |
| Zusammenarbeit | 7.05 offline arbeiten |
| Bedienung | 8.01 echte Mac-App, 8.03 direkt am Bauteil arbeiten, 8.04 Befehlspalette und Kürzel, 8.05 in einem Tag produktiv, 8.07 KI-Prüfung |

### Später (19 Positionen)

1.13 Volumenstudien, 2.06 Bestandsaufnahme, 2.07 Versionsverlauf, 3.10 Details, 4.04 Raumbuch, 4.06 Kosten nach DIN 276, 4.07 AVA-Export, 5.04 U-Werte, 5.06 Kollisionsprüfung, 6.01 IFC-Export, 6.07 BCF, 6.08 SketchUp/Rhino-Import, 7.01 Mehrbenutzer, 7.02 Kommentare, 7.03 Link für Bauherren, 7.04 Cloud-Sync, 8.02 iPad, 8.06 KI-Assistent, 8.08 Skripte

### Irgendwann

3.12 fotorealistische Visualisierung, 6.06 Revit-Dateien öffnen

### Bewusst nicht in Version 1

- Ausführungsplanung (LP 5) und Details 1:20 bis 1:1
- Tragwerk und TGA modellieren. Diese Modelle kommen nur als IFC-Referenz ins Projekt.
- Freiformen (NURBS). Gebogene Wände sind möglich, intern werden sie facettiert.
- Cloud und gleichzeitiges Arbeiten mehrerer Personen

### Was wir für „später“ schon jetzt vorbereiten

Einige spätere Positionen brauchen von Anfang an die richtige Architektur. Sonst wird der Umbau später teuer.

- **IFC-Export (6.01):** Die Bauteilklassen folgen von Anfang an IFC (IfcWall, IfcSlab, IfcSpace, IfcDoor ...). Der Export ist dann nur noch eine Abbildung.
- **Versionsverlauf (2.07) und Mehrbenutzer (7.01):** Jede Änderung ist von Anfang an eine Operation in einem Protokoll.
- **Windows und iPad (8.02):** Der Kern ist plattformunabhängig, ebenso der Renderer (wgpu).
- **KI-Assistent (8.06):** Jede Funktion ist als Befehl mit Parametern definiert, so wie sie die Befehlspalette (8.04) braucht. Dieselben Befehle kann später die KI aufrufen.

## 4. Architektur

### Überblick

```
app-mac (Swift, SwiftUI/AppKit)
  Fenster, Paletten, Befehlspalette, Trackpad-Gesten, Dateien
  Metal-Ebene, in die lot-render zeichnet
        |
        |  UniFFI (erzeugte Swift-Anbindung)
        v
core (Rust, plattformunabhängig, läuft auch headless unter Linux)
  lot-model   Elemente, Typen, Parameter, Bezüge, Operationen, Undo
  lot-geom    Geometrie, Booleans, Wandanschlüsse, Verschneidungen
  lot-plan    Schnitt, Projektion, Symbolik, Stiftsets, Bemaßung
  lot-rules   DIN 277, WoFlV, BRI, Abstandsflächen, DIN 18040, Rettungswege
  lot-io      Projektdatei (SQLite), PDF, DXF, IFC-Import, PDF-Unterlage
  lot-render  wgpu-Renderer für 3D und 2D-Pläne
  lot-ffi     Schnittstelle für Swift
  lot-cli     Testprojekte rechnen, Pläne als PDF/SVG/PNG ausgeben
```

### Grundentscheidungen

1. **Kern in Rust.** Er ist schnell, sicher bei Parallelität und läuft auf Mac, Windows, iPad und Linux. Weil er unter Linux läuft, können Cloud-Sessions vom iPhone aus am Kern arbeiten und Pläne zum Prüfen erzeugen.
2. **Mac-App in Swift mit SwiftUI und AppKit,** damit sie sich wie eine echte Mac-App anfühlt (8.01). Der Kern wird über UniFFI angebunden.
3. **Darstellung mit wgpu.** Auf dem Mac läuft wgpu über Metal, auf Windows später über DirectX 12 oder Vulkan. Gezeichnet wird in eine Metal-Ebene, die die Swift-App bereitstellt. 3D-Ansicht und 2D-Planansicht laufen über denselben Renderer.
4. **Geometrie aus ebenen Flächen.** Architektur besteht zum größten Teil aus ebenen Flächen. Für robuste 3D-Booleans (Öffnungen, Verschneidungen) nutzen wir Manifold. Für 2D-Polygone (Wandanschlüsse, Schnittflächen, Flächenberechnung) Clipper2 oder i_overlay. In V1 gibt es keinen vollen NURBS-Kernel. Open CASCADE bleibt Rückfallebene, falls sich zeigt, dass wir echte Freiformflächen (B-Rep) brauchen.
5. **Parameter und Bezüge sind die Wahrheit, Geometrie wird abgeleitet.** Ein Abhängigkeitsgraph berechnet nach einer Änderung nur das Betroffene neu, parallel über alle Kerne.
6. **Jede Änderung ist eine Operation.** Daraus ergeben sich Undo/Redo, später Versionsverlauf und Mehrbenutzer, ohne dass wir umbauen müssen.
7. **Projektdatei ist eine SQLite-Datei** (`.lot`) mit Elementen, Typen, Ansichten, Planblättern und dem Operationsprotokoll. Gespeichert wird laufend, ein Absturz verliert nichts, alles funktioniert offline (7.05).
8. **Bauteilklassen nach IFC.** Der IFC-Import (6.02) läuft in V1 voraussichtlich über IFClite, eine Rust-Bibliothek (MPL-2.0), die IFC auch schreiben kann. Damit wird der spätere IFC-Export (6.01) deutlich billiger. Alternativen sind IfcOpenShell und web-ifc. Importierte Modelle sind Referenzen: sichtbar, schneidbar, aber nicht bearbeitbar.
9. **Eine Planpipeline, drei Ausgaben.** Aus einer Ansicht werden Planobjekte (Abschnitt „Planpipeline“), und diese Planobjekte gehen auf den Bildschirm, ins PDF und ins DXF. Dadurch sieht der Plan überall gleich aus.
10. **Darstellungsregeln sind Daten:** Stiftsets, Schraffur je Material, Detailgrad je Maßstab, Farben für Bestand, Abbruch und Neubau. Sie liegen im Bürostandard (2.08) und kommen mit guten deutschen Voreinstellungen.
11. **DWG:**
    - In V1 lesen und schreiben wir DXF direkt.
    - DWG läuft über den kostenlosen ODA File Converter als Zwischenschritt. Ob dessen Nutzungsbedingungen für Bens Büro passen, prüfen wir in M0. (Ergebnis M0: unklar bis unzulässig, Vorschlag acadrust, siehe ADR 0012.)
    - Für das spätere Produkt kommt eine ODA-Mitgliedschaft mit dem Drawings SDK infrage.
12. **PDF als Unterlage (6.05):** Auf dem Mac rendert PDFKit die Seite. Kalibriert wird über zwei Punkte mit bekanntem Abstand, Vektorlinien werden nach Möglichkeit als Fangpunkte genutzt.

### Bibliotheken (geprüft in M0, Details in `docs/entscheidungen/`)

| Aufgabe | Wahl nach M0 | Lizenz | Ergebnis der Prüfung | ADR |
|---|---|---|---|---|
| 3D-Booleans | manifold-rust 0.18 (reine Rust-Portierung), fest gepinnt | Apache-2.0 | Robust auch bei deckungsgleichen Flächen, 12 s Build ohne C++. manifold-csg bleibt Rückfallebene | 0004 |
| B-Rep als Rückfallebene | keine in Version 1 | | truck seit 2024 ohne Release, scheitert an deckungsgleichen Flächen | 0014 |
| 2D-Polygone | clipper2-rust 1.2 auf Ganzzahlraster (1 µm) | BSL-1.0 | Robust bei fast parallelen Kanten. i_overlay als Alternative | 0005 |
| Renderer | wgpu 30 | MIT/Apache-2.0 | Läuft im CAMetalLayer eines NSView, Zeichnen auf dem Main-Thread | 0006 |
| 2D-Tessellierung | lyon | MIT/Apache-2.0 | Breiten im Shader, exakt bei jedem Zoom. vello noch experimentell | 0007 |
| Swift-Anbindung | UniFFI 0.32 | MPL-2.0 | 6 bis 12 ns je Aufruf mit Zahlen. Große Daten nur als Bytes | 0003 |
| Projektdatei | SQLite 3.53 über rusqlite 0.40 | gemeinfrei / MIT | | 0008 |
| PDF-Ausgabe | krilla 0.8 | MIT/Apache-2.0 | Exakte Linienbreiten, Schrift als Teilmenge eingebettet. Schraffuren als einzelne Linien | 0010 |
| PDF-Unterlage | PDFKit (Mac), später pdfium für Windows | Apple / BSD-3 | für M4 | 0016 |
| DXF und DWG | **Vorschlag:** acadrust statt dxf-Crate und ODA File Converter | MPL-2.0 | dxf-Crate verwirft Schraffuren. ODA-Converter für den Büroeinsatz lizenzrechtlich unklar | 0012 |
| IFC lesen und schreiben | IFClite (`ifc-lite-core`) | MPL-2.0 | Baut nativ ohne WASM. Bei M4 neu bewerten (häufige Hauptversionen) | 0013 |
| Planschrift nach ISO 3098 | osifont, Datei `osifont-lgpl3fe.ttf` | LGPL-3.0 mit Font-Ausnahme | Einbetten in PDFs erlaubt. Zur Laufzeit geladen. Keine deutschen Anführungszeichen | 0011 |
| Snapshot-Tests | eigene Golden-Tests, insta ab M1 | Apache-2.0 | | 0015 |

### Open Source als Starthilfe

Ein fertiges Open-Source-BIM, das wir als Grundlage übernehmen könnten, gibt es nicht:

- **FreeCAD BIM** (LGPL) ist in C++/Python mit Qt gebaut, fühlt sich nicht wie eine Mac-App an und hat Schwächen bei Plänen und Listen.
- **Bonsai** (GPL) läuft nur innerhalb von Blender und lässt sich wegen der GPL nicht in ein geschlossenes Produkt einbauen.

Die Starthilfe kommt deshalb aus zwei Richtungen: Bausteine, die wir einbauen, und Projekte, aus denen wir lernen.

**Einbauen** (Lizenz passt auch für ein späteres Produkt): alle Bibliotheken aus der Tabelle oben. Den größten Sprung bringen:

- **IFClite:** IFC lesen und schreiben mit Geometrie, in Rust
- **Manifold:** robuste Booleans
- **wgpu:** Darstellung auf Mac und später Windows

**Lernen und abschauen:**

| Projekt | Lizenz | Was wir daraus lernen |
|---|---|---|
| Hypar Elements (C#) | MIT | Aufbau eines Bauteilmodells (Wall, Floor, Opening, Grid, Profile) und dessen Abbildung auf IFC, DXF und SVG. Wegen MIT dürfen wir Teile übernehmen und portieren, mit Lizenzhinweis. |
| FreeCAD BIM | LGPL-2.1+ | Wände, Dächer, Treppen, Schnittebenen, native IFC-Arbeitsweise. Nur Konzepte übernehmen, keinen Code. |
| Bonsai / IfcOpenShell | GPL-3.0 / LGPL-3.0 | Planerzeugung aus dem Modell (SVG mit Schraffuren und Beschriftung), sehr weit entwickelt. Nur Konzepte übernehmen, keinen Code. |
| Straight Skeleton (Literatur, z. B. Held und Palfrader; Crate `straight_skeleton`) | | Dachgenerierung aus dem Grundriss (Walm, Mansard). Für unterschiedliche Neigungen je Dachfläche brauchen wir die gewichtete Variante, voraussichtlich als eigene Umsetzung. |
| Speckle Revit-Connector | Apache-2.0 | Möglicher Weg für „Revit-Dateien öffnen“ (6.06), ohne Autodesks Dateiformat selbst lesen zu müssen. |

**Lizenzregel:** Code übernehmen wir nur aus Projekten mit freizügiger Lizenz (MIT, Apache, BSD, BSL, Zlib). MPL-2.0 und LGPL nutzen wir nur als Bibliothek. GPL- und AGPL-Projekte lesen wir, um Konzepte zu verstehen; Code daraus kopieren oder übersetzen wir nicht.

### Wandanschlüsse (das Herzstück)

- Jede Schicht eines Wandaufbaus hat eine Funktion mit Priorität: Tragschicht, Dämmung, Installationsebene, Putz oder Bekleidung, Luftschicht.
- Die Wandachsen bilden einen Graphen. An jedem Knoten werden die Schichten paarweise nach Priorität verschnitten. Beispiel: Die Tragschicht einer Innenwand bindet in die Tragschicht der Außenwand ein, die Dämmung der Außenwand läuft durch.
- Sonderfälle bekommen eigene Regeln: spitze Winkel, Wand an Stütze, unterschiedliche Höhen und Wandenden.
- Ben kann jeden Knoten manuell übersteuern. Das soll aber die Ausnahme bleiben.
- Abgesichert wird das durch einen Knotenkatalog als Testprojekt (Abschnitt 7) und durch Eigenschaftstests mit zufälligen Wandkonstellationen. Diese prüfen, dass es keine Überlappungen gibt, alle Polygone geschlossen sind und jede Schicht durchgängig ist.

### Planpipeline

```
Ansicht (Schnittebene, Blickbereich, Maßstab, Phase, Variante)
  -> je Element: Schnittfläche + Projektion (sichtbar/verdeckt) + Symbolik je Maßstab
  -> Planobjekte: Linie (Stift, Linienart), Fläche (Schraffur, Füllung), Text, Bemaßung
  -> Bildschirm (lot-render), PDF, DXF
```

Darstellung je Maßstab, als Vorschlag. Ben prüft das in M1.

| Maßstab | Wände | Öffnungen |
|---|---|---|
| 1:200 | gefüllte Fläche, keine Schichten | Öffnung ohne Rahmen, Aufschlag vereinfacht |
| 1:100 | Tragschicht schraffiert, weitere Schichten als Linie | Rahmen vereinfacht, Aufschlag |
| 1:50 | alle Schichten mit Materialschraffur | Rahmen, Laibung, Fensterbank, Anschlag |

- Linienbreiten nach DIN ISO 128 / DIN 1356-1: 0,13 / 0,18 / 0,25 / 0,35 / 0,5 / 0,7 / 1,0 mm. Die Stiftsets je Maßstab legen fest, welche Breite für Schnittkante, Ansichtskante und verdeckte Kante gilt.
- Farben im Bauantrag: Bestand grau/schwarz, Abbruch gelb, Neubau rot (2.05). Die genaue Ausprägung richtet sich nach der Bauvorlagenverordnung des Bundeslands.

### Regeln und Prüfungen

- `lot-rules` enthält die Prüfungen 4.01-4.03 und 5.01-5.05. Jede Regel nennt ihre Quelle mit Norm, Paragraf und Fassung.
- Landesrecht liegt als Daten je Bundesland vor. Wir starten mit dem Bundesland von Bens Büro.
- Ergebnisse erscheinen als Hinweise direkt im Modell, zum Beispiel eine rot markierte Abstandsfläche, und als Prüfbericht.

### KI-Prüfung (8.07)

- **Stufe 1, ohne KI:** die Regeln aus `lot-rules` sowie Plausibilitätsprüfungen, etwa Räume ohne Tür, Treppen außerhalb des Steigungsverhältnisses oder Fenster ohne Brüstungshöhe.
- **Stufe 2, mit KI:** Das Modell wird als kompakte, strukturierte Beschreibung (JSON) zusammen mit Planbildern an die Claude API geschickt. Die KI sucht nach fehlenden Maßen, Widersprüchen zwischen Plänen und Auffälligkeiten.
- Die Prüfung läuft nur auf Klick. Vorher sieht man, was gesendet wird. Der API-Schlüssel liegt im macOS-Schlüsselbund.
- Für die Anbindung nutzt die Session das `claude-api`-Skill und die aktuelle Doku. Das Modell wird erst beim Bauen gewählt.

### Zielwerte für die Leistung

- Die 3D-Ansicht bleibt flüssig (mindestens 60 Bilder pro Sekunde, auf ProMotion-Displays 120) bei einem Mehrfamilienhaus mit 4 Geschossen.
- Nach einer Wandänderung sind Modell und sichtbare Ansichten in unter 50 ms aktualisiert.
- Ein Projekt öffnet in unter 2 Sekunden.

### Repository

```
lot/
  CLAUDE.md
  docs/PLAN.md, docs/glossar.md, docs/entscheidungen/
  core/            Rust-Workspace (Crates siehe Überblick)
  app-mac/         Xcode-Projekt
  testprojekte/    je Projekt: Modell als Textdatei + referenz/ (freigegebene Pläne)
  pruefpakete/     Lieferungen zum Prüfen
```

Testprojekte sind lesbare Textdateien (zum Beispiel RON oder JSON) und keine Binärdateien. Claude kann sie schreiben und ändern, Unterschiede lassen sich im Git-Diff lesen.

## 5. Arbeitsweise

### Bens 5 Stunden pro Woche

- etwa 1 Stunde: Entscheidungen treffen und Fragen beantworten
- etwa 3 Stunden: Prüfpakete durchgehen und die App ausprobieren
- etwa 1 Stunde: Puffer

### Prüfpakete

Jede Lieferung endet mit einem Ordner `pruefpakete/JJJJ-MM-TT-thema/`. Er enthält:

- PDFs der betroffenen Pläne, maßstabsgetreu, damit Ben sie ausdrucken und nachmessen kann
- PNGs zum schnellen Ansehen auf dem iPhone
- eine `README.md` mit höchstens 10 Prüffragen. Jede Frage ist konkret und lässt sich mit Ja oder Nein und einem Satz beantworten. Beispiel: „Knoten 7: Bindet die Innenwand so ein, wie du es zeichnen würdest?“

### Freigaben

- Sagt Ben zu einer Ausgabe „freigegeben“, wird sie zur Referenz in `testprojekte/*/referenz/`.
- Golden-Tests vergleichen jede künftige Ausgabe damit, als SVG-Text-Diff und als Bildvergleich. Jede Abweichung fällt sofort auf.
- So wird jede Prüfstunde von Ben zu einem dauerhaften Schutz vor Rückschritten.

### Cloud oder lokal

| Wo | Wofür |
|---|---|
| Cloud-Session, auch vom iPhone | alles in `core/`, Testprojekte, Prüfpakete, Regeln, Dateiformate |
| Lokale Session auf dem Mac | `app-mac/`, alles mit Xcode, Bedienung und Gefühl ausprobieren, Leistung messen |

### Entscheidungen und Zentrale

- Entscheidungen mit Tragweite kommen als ADR nach `docs/entscheidungen/`.
- Aufgaben und offene Entscheidungen für Ben gehen in die Zentrale, falls der Connector verfügbar ist.

## 6. Meilensteine

Wir messen den Aufwand in Bens Prüfrunden und nicht in Entwicklerwochen. Nach M0 und M1 haben wir echte Messwerte und schätzen neu.

Jeder Meilenstein liefert etwas, das Ben in der App benutzen oder als Plan prüfen kann. Die App wächst von Anfang an mit, damit sich früh zeigt, wie sich die Bedienung anfühlt.

### M0: Technik bestätigen

- **Ziel:** Alle Grundentscheidungen aus Abschnitt 4 sind an echtem Code bestätigt oder begründet geändert.
- **Inhalt:**
  - Rust-Workspace mit der Swift-Anbindung über UniFFI
  - SwiftUI-Fenster mit wgpu-3D-Ansicht und Trackpad-Navigation (Orbit, Pan, Zoom)
  - Eine Wand mit Fensteröffnung über Manifold
  - Speichern und Laden als SQLite-Datei
  - `lot-cli` gibt ein PDF mit exakten Linienbreiten aus
  - Die Lizenz des ODA File Converters ist geprüft
- **Abnahme durch Ben:**
  - Die App startet, die Wand mit Öffnung lässt sich mit dem Trackpad flüssig drehen.
  - Das ausgedruckte PDF zeigt bei 1:100 ein 5-m-Rechteck als 5 cm, und die Linien 0,25 und 0,5 mm sind klar unterscheidbar.
- **Ergebnis:** ADRs für jede Bibliothekswahl.

### M1: Wände, die stimmen

- **Ziel:** Der Lackmustest. Wände und Anschlüsse sind besser als in Revit und Archicad.
- **Inhalt:**
  - Geschosse und Höhenbezüge (2.01), Achsraster (2.02)
  - Wände mit Schichtaufbau (1.01), saubere Anschlüsse (1.02), Stützen (1.09)
  - Grundriss aus dem Modell (3.01) in 1:100 und 1:50 mit Darstellung je Maßstab (3.03) nach DIN 1356 (3.04), als PDF (3.08)
  - App: Wände zeichnen und direkt verschieben (8.03), Befehlspalette in einer Grundversion (8.04), Undo
- **Abnahme durch Ben:**
  - Der Wandknoten-Katalog als PDF: Jeder Knoten sieht aus, als hätte Ben ihn von Hand gezeichnet.
  - In der App ist ein EG-Grundriss in Wänden in wenigen Minuten gezeichnet.

### M2: Öffnungen, Decken, Räume und Flächen

- **Inhalt:**
  - Türen und Fenster (1.06), Decken und Bodenplatten (1.03), Schlitze und Durchbrüche (1.11)
  - Räume automatisch erkennen (2.03), Raumstempel (3.06), assoziative Bemaßung (3.05)
  - DIN 277 (4.01), WoFlV (4.02), BRI und Kennwerte (4.03)
  - App: Türen und Fenster setzen und entlang der Wand ziehen, Bemaßung setzen
- **Abnahme durch Ben:**
  - Der EFH-Erdgeschossgrundriss ist komplett, mit Raumstempeln und Maßketten.
  - Die Flächenberechnung stimmt mit Bens Handrechnung überein.

### M3: Schnitte, Ansichten, Dächer, Treppen und Planblätter

- **Inhalt:**
  - Schnitte und Ansichten (3.02), Dächer (1.04), Gauben und Dachfenster (1.05), Treppen mit Prüfung des Steigungsverhältnisses (1.07), Geländer (1.08)
  - 2D-Zeichnen (3.09), Planlayout und Plankopf (3.07)
- **Abnahme durch Ben:** Das EFH ist komplett: Grundrisse aller Geschosse, 2 Schnitte, 4 Ansichten, auf Planblättern mit Plankopf als PDF.

**Probeprojekt 1:** Ben entwirft ein echtes Einfamilienhaus in Lot, ein neues oder ein früheres Projekt von ihm. Was dabei fehlt oder stört, bestimmt die Reihenfolge der nächsten Meilensteine mit.

### M4: Bestand und Austausch

- **Inhalt:**
  - PDF als Unterlage (6.05), DWG/DXF-Import (6.03) und -Export (6.04), IFC-Import als Referenz (6.02)
  - Bestand/Abbruch/Neubau (2.05), Gelände (1.12), Varianten (2.04)
- **Abnahme durch Ben:**
  - Ein Altbau-Umbau ist auf Basis alter Bestandspläne als PDF modelliert, Abbruch und Neubau sind korrekt eingefärbt.
  - Ein Fachplaner öffnet den DWG-Export ohne Rückfragen.

### M5: Bauantrag

- **Inhalt:**
  - Abstandsflächen (5.01), Bauantragspläne nach Bauvorlagenverordnung (5.02), Barrierefreiheit nach DIN 18040 (5.03), Rettungswege (5.05)
  - Tür- und Fensterlisten (4.05), Bürostandard (2.08), Präsentationspläne (3.11)
- **Abnahme durch Ben:** Für das MFH-Testprojekt entstehen vollständige Bauantragsunterlagen. Die Abstandsflächen sind gegen eine Handberechnung geprüft.

### M6: Bauteile, Ausstattung und Fassade

- **Inhalt:**
  - Eigene Bauteile bauen (1.15), Möblierung und Ausstattung (1.14), Pfosten-Riegel-Fassaden (1.10), Unterzüge (1.09)
  - Eigene Bauteile werden als Daten beschrieben (Profile, Extrusionen, Parameter, 2D-Symbol je Maßstab). Ein einfacher Editor erzeugt diese Beschreibung. Alternativ beschreibt Ben das Bauteil in Worten, und Claude schreibt die Definition. Das ist ein klarer Unterschied zu Revit-Familien.
- **Abnahme durch Ben:** Ben baut ohne Hilfe ein eigenes Bauteil, zum Beispiel eine Einbauküche mit Parametern, in unter 30 Minuten.

### M7: KI-Prüfung und Feinschliff

- **Inhalt:**
  - KI-Prüfung (8.07)
  - Leistung bei großen Projekten (3.13)
  - Einstieg in einem Tag (8.05): Beispielprojekt, Hinweise beim ersten Benutzen, sinnvolle Voreinstellungen
- **Abnahme durch Ben:** Eine Kollegin oder ein Kollege aus dem Büro ist nach einem Tag produktiv.

**Version 1 ist erreicht,** wenn Ben einen echten Bauantrag komplett mit Lot erstellt und eingereicht hat, ohne Revit.

## 7. Testprojekte

| Projekt | Zweck | ab |
|---|---|---|
| `wandknoten` | Katalog von etwa 25 Anschlussfällen | M1 |
| `efh` | Einfamilienhaus, KG + EG + DG, Satteldach mit Gauben | M1 |
| `mfh` | 4 Geschosse, Treppenhaus mit Aufzug, 8 Wohnungen | M3 |
| `umbau` | Altbau nach PDF-Bestandsplan, mit Abbruch und Neubau | M4 |
| `probeprojekt` | ein echtes Projekt von Ben, anonymisiert | nach M3 |
| `ifc-referenz` | Fachplanermodelle aus den IFC-Beispieldateien von buildingSMART (Sample-Test-Files) | M4 |

Die Lizenz der buildingSMART-Dateien ist noch nicht geklärt. Bis dahin liegen sie nur lokal und nicht im Repository.

Die Fälle im `wandknoten`-Katalog:

- L-Ecke mit gleichem Aufbau
- L-Ecke mit Außenwand (WDVS) an Innenwand
- T-Stoß einer Innenwand an eine Außenwand
- Kreuzung
- spitze und stumpfe Winkel
- unterschiedliche Dicken
- Wand an Stütze
- freies Wandende
- gebogene Wand an gerader Wand
- unterschiedliche Wandhöhen
- Trennwand an zweischaliges Mauerwerk

## 8. Nach Version 1

- Beta mit 3-5 befreundeten kleinen Büros, ein Preismodell finden
- IFC-Export (6.01) und Zertifizierung bei buildingSMART, Mehrbenutzer (7.01), Cloud-Sync (7.04)
- Windows-App mit demselben Kern, iPad-App (8.02)
- ODA-Mitgliedschaft für natives DWG
- Revit-Brücke (6.06), voraussichtlich über den Speckle-Connector
- KI-Assistent (8.06) auf Basis der Befehle

## 9. Offene Fragen an Ben

1. Deine Revit-Liste (Abschnitt 2): Was regt dich täglich auf, in welchen Situationen?
2. Bundesland deines Büros, für Abstandsflächen und Bauvorlagenverordnung
3. Ein PDF eines eigenen, eingereichten Bauantrags als Referenz für Darstellung, Plankopf und Schrift
4. ArchiPedia: Was soll Lot davon nutzen? Wissen für die KI-Prüfung, Herstellerdaten für Bauteile, etwas anderes?
5. Name: Bleibt es bei „Lot“?

## 10. Start auf dem Mac

### Vorbereitung

1. Xcode in aktueller Version, Rust über `rustup`, Git, Claude Code
2. Ein neues privates Repository anlegen, `CLAUDE.md` und `docs/PLAN.md` hineinlegen
3. Falls relevant, das ArchiPedia-Repository daneben klonen, damit die Session es lesen kann

### Erster Auftrag an die lokale Session

> Lies CLAUDE.md und docs/PLAN.md. Wir starten mit Meilenstein M0.
> 1. Prüfe die Bibliotheken aus Abschnitt 4 gegen ihren aktuellen Stand: Lizenz, Pflege, Rust-Anbindung. Schlag mir Änderungen vor, wo nötig.
> 2. Lege den Rust-Workspace, das Xcode-Projekt und die Befehle in CLAUDE.md an.
> 3. Baue M0 bis zu den Abnahmekriterien und halte jede Bibliothekswahl als ADR fest.
> 4. Schließe mit einem Prüfpaket ab.
> Wenn du an einer Stelle eine Entscheidung von mir brauchst, sammle sie und frag mich gebündelt.
