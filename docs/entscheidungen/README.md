# Entscheidungen (ADR)

Kurze Festlegungen mit Kontext, Entscheidung, Alternativen und Folgen. Bens Entscheidungen stehen in seinen Worten.

| Nr. | Thema | Status |
|---|---|---|
| [0001](0001-werkzeugkette.md) | Werkzeugkette und Aufbau des Workspace | umgesetzt (rustup, Ben) |
| [0002](0002-einheiten-und-koordinaten.md) | Einheiten und Koordinaten | umgesetzt |
| [0003](0003-swift-anbindung-uniffi.md) | Swift-Anbindung mit UniFFI | umgesetzt |
| [0004](0004-3d-booleans-manifold-rust.md) | 3D-Booleans mit manifold-rust | umgesetzt |
| [0005](0005-2d-polygone-clipper2.md) | 2D-Polygone mit clipper2-rust | umgesetzt |
| [0006](0006-renderer-wgpu.md) | Renderer mit wgpu im NSView der App | umgesetzt |
| [0007](0007-2d-tessellierung-lyon.md) | Planansicht auf dem Bildschirm mit lyon | umgesetzt |
| [0008](0008-projektdatei-sqlite.md) | Projektdatei als SQLite | umgesetzt |
| [0009](0009-testprojekte-ron.md) | Testprojekte als RON-Textdateien | umgesetzt |
| [0010](0010-planausgabe-pdf-svg-png.md) | Planausgabe als PDF, SVG und PNG | umgesetzt |
| [0011](0011-planschrift-osifont.md) | Planschrift osifont | umgesetzt, Auslieferung offen |
| [0012](0012-dxf-und-dwg.md) | DXF und DWG | entschieden (acadrust, Ben), umgesetzt in M4 |
| [0013](0013-ifc-ifclite.md) | IFC mit IFClite | Import umgesetzt in M4 |
| [0014](0014-brep-rueckfallebene.md) | B-Rep als Rückfallebene | kein B-Rep in V1 |
| [0015](0015-tests-golden-und-snapshots.md) | Golden-Tests und Snapshot-Tests | umgesetzt |
| [0016](0016-pdf-als-unterlage.md) | PDF als Unterlage | umgesetzt in M4 (Mac) |
| [0017](0017-wandanschluesse-nach-schichtrang.md) | Wandanschlüsse nach Schichtrang | umgesetzt, Ben prüft |
| [0018](0018-buerostandard-als-daten.md) | Bürostandard als Daten | umgesetzt, Vorschlag |
| [0019](0019-bearbeiten-im-kern.md) | Werkzeuge, Fangen und Befehle im Kern | umgesetzt |
| [0020](0020-gebogene-waende.md) | Gebogene Wände | umgesetzt, Ben prüft |
| [0021](0021-raeume-und-flaechen.md) | Räume und Flächen | umgesetzt, Vorschlag |
| [0022](0022-tueren-und-fenster.md) | Türen und Fenster | umgesetzt, Vorschlag |
| [0023](0023-massketten.md) | Assoziative Maßketten | umgesetzt, Vorschlag |
| [0024](0024-decken-und-bodenplatten.md) | Decken und Bodenplatten | umgesetzt, Vorschlag |
| [0025](0025-daecher.md) | Dächer | umgesetzt, Vorschlag |
| [0026](0026-schnitte-und-ansichten.md) | Schnitte und Ansichten | umgesetzt, Vorschlag |
| [0027](0027-planblaetter.md) | Planblätter und Plankopf | umgesetzt, Vorschlag |
| [0028](0028-treppen-und-gelaender.md) | Treppen und Geländer | umgesetzt, Vorschlag |
| [0029](0029-gauben-und-dachfenster.md) | Gauben und Dachfenster | umgesetzt, Vorschlag |
| [0030](0030-zeichnen-in-2d.md) | 2D-Zeichnen | umgesetzt |
| [0031](0031-bestand-abbruch-neubau.md) | Bestand, Abbruch und Neubau | umgesetzt, Vorschlag |
| [0032](0032-unterlagen.md) | Unterlagen (PDF, Bild, DXF, DWG) | umgesetzt |
| [0033](0033-dxf-dwg-austausch.md) | DXF und DWG im Austausch | umgesetzt, Vorschlag |
| [0034](0034-gelaende.md) | Gelände | umgesetzt, Vorschlag |
| [0035](0035-varianten.md) | Varianten | umgesetzt |
| [0036](0036-ifc-referenz.md) | IFC als Referenz | umgesetzt |
| [0037](0037-landesrecht-und-abstandsflaechen.md) | Landesrecht als Daten, Abstandsflächen | umgesetzt, Vorschlag |
| [0038](0038-pruefbericht.md) | Prüfbericht und Hinweise im Modell | umgesetzt, Vorschlag |
| [0039](0039-bauantragsplaene.md) | Bauantragspläne, Lageplan, Tür- und Fensterliste | umgesetzt, Vorschlag |
| [0040](0040-buerostandard.md) | Bürostandard je Büro | umgesetzt |
| [0041](0041-praesentationsplaene.md) | Präsentationspläne | umgesetzt, Vorschlag |
| [0042](0042-meilensteine-anhalten-bedienung-vergleichen.md) | Meilensteine anhalten, erst die Bedienung vergleichen | entschieden |
| [0043](0043-auswaehlen.md) | Auswählen | umgesetzt, Teile offen |
| [0044](0044-bearbeiten.md) | Bearbeiten | umgesetzt, Teile offen |
| [0045](0045-wandlogik.md) | Wandlogik: Bezugslinie, Anschlüsse, Höhen | umgesetzt, Teile offen |
| [0046](0046-zeichnen-und-fangen.md) | Zeichnen mit Zahleneingabe und Fangen | umgesetzt, Teile offen |
| [0047](0047-oeffnungen.md) | Öffnungen: Rohbauöffnung zuerst, zwei Brüstungshöhen | umgesetzt, Teile offen, Ben testet |
| [0048](0048-typen-und-eigenschaften.md) | Typen und Eigenschaften | umgesetzt, Teile offen, Ben testet |
| [0049](0049-navigation.md) | Navigation: Seitenleiste, Geschosse, Schnitte und Blätter | umgesetzt, Teile offen, Ben testet |
| [0050](0050-sichtbarkeit.md) | Sichtbarkeit und Ansichtsvorlagen | umgesetzt, Teile offen, Ben testet |
| [0051](0051-decken-daecher-treppen.md) | Decken, Dächer und Treppen in der App | umgesetzt bis auf Gauben, Ben testet |
