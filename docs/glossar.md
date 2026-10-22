# Glossar: Deutsch und Code

Bezeichner im Code sind englisch. Diese Liste legt fest, welcher Begriff wofür steht. Wo es eine IFC-Entsprechung gibt, steht sie dabei. Neue Begriffe ergänzen, bevor sie im Code auftauchen.

| Deutsch | Im Code | IFC | Anmerkung |
|---|---|---|---|
| Wand | `Wall` | IfcWall | |
| Wandaufbau | `WallType` | IfcWallType | Folge von Schichten |
| Schicht | `Layer` | IfcMaterialLayer | |
| Schichtfunktion | `LayerFunction` | | Tragschicht, Dämmung, Installationsebene, Bekleidung, Luftschicht |
| Tragschicht | `LayerFunction::Structure` | | höchste Priorität bei Anschlüssen |
| Bezugslinie | `Wall::start`, `Wall::end` | | Linie, an der eine Wand gezeichnet wird |
| Lage der Bezugslinie | `WallAlignment` | | Mitte, Außenkante, Innenkante des Aufbaus |
| Oberkante | `Top` | | feste Höhe oder bis zum nächsten Geschoss |
| Unterkante über OKFF | `base_offset` | | |
| OKFF | `Storey::elevation` | | Oberkante Fertigfußboden über ±0,00 |
| Geschosshöhe | `Storey::height` | | OKFF bis OKFF darüber |
| Schichtrang | `LayerFunction::priority` | | 1 = Tragschicht, bestimmt das Ineinandergreifen an Anschlüssen |
| Kniestock | | | niedrige Außenwand im Dachgeschoss, `Top::Height` |
| Anschluss, Wandknoten | `Join` | | |
| Geschoss | `Storey` | IfcBuildingStorey | |
| Achsraster | `Grid` | IfcGrid | |
| Decke, Bodenplatte | `Slab` | IfcSlab | Bodenplatte als `SlabKind::Base` |
| Stütze | `Column` | IfcColumn | |
| Unterzug | `Beam` | IfcBeam | |
| Öffnung | `Opening` | IfcOpeningElement | |
| Tür | `Door` | IfcDoor | |
| Fenster | `Window` | IfcWindow | |
| Laibung | `Reveal` | | |
| Brüstung, Brüstungshöhe | `sill_height` | | Höhe über Fertigfußboden |
| Sturz, Sturzhöhe | `head_height` | | |
| Anschlag | `Rebate` | | |
| Türaufschlag | `Swing` | | |
| Raum | `Space` | IfcSpace | |
| Raumstempel | `SpaceTag` | | |
| Dach | `Roof` | IfcRoof | |
| Traufe | `Eave` | | |
| Ortgang | `Verge` | | |
| First | `Ridge` | | |
| Gaube | `Dormer` | | |
| Treppe | `Stair` | IfcStair | |
| Geländer | `Railing` | IfcRailing | |
| Höhenkote | `LevelMark` | | |
| Maßkette | `DimensionChain` | | |
| Stiftset | `PenSet` | | Linienbreiten je Maßstab |
| Stift | `Pen` | | eine Linienbreite in mm auf dem Papier |
| Linienart | `LineStyle` | | durchgezogen, gestrichelt |
| Planobjekt | `PlanObject` | | Linie, Fläche oder Text auf dem Blatt; daraus entstehen Bildschirm, PDF, SVG, DXF |
| Operation | `Operation` | | eine Änderung am Modell, umkehrbar; Grundlage für Undo |
| Projekt, geöffnet | `Document` | | Projekt mit Operationsprotokoll und Undo/Redo |
| Abstand der Öffnung vom Wandanfang | `Opening::offset` | | entlang der Wandachse bis zur Kante der Öffnung |
| Schraffur | `Hatch` | | |
| Planblatt | `Sheet` | | |
| Plankopf | `TitleBlock` | | |
| Bestand, Abbruch, Neubau | `Phase::{Existing, Demolished, New}` | | Farbkonvention nach Bauvorlagenverordnung |
| Abstandsfläche | `SetbackArea` | | nach Landesbauordnung |
| Variante | `Variant` | | |
| Bürostandard | `OfficeStandard` | | Materialien, Schraffuren, Stiftsets je Maßstab |
| Detailgrad | `Detail` | | grob (1:200), mittel (1:100), fein (1:50) |
| Schnittkante | `Pens::cut` | | Umriss geschnittener Bauteile |
| Werkzeug | `Tool` | | Auswählen, Wand, Achse, Stütze |
| Fangen, Fangpunkt | `snap`, `Snap` | | Einrasten an Wandenden, Wänden, Achsen, Winkeln |
| Befehl | `Command` | | jede Funktion mit Kennung, für Befehlspalette und KI |
| Planansicht | `ViewMode::Plan` | | Grundriss auf dem Bildschirm, wie gedruckt |
