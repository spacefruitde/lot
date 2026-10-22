# Glossar: Deutsch und Code

Bezeichner im Code sind englisch. Diese Liste legt fest, welcher Begriff wofür steht. Wo es eine IFC-Entsprechung gibt, steht sie dabei. Neue Begriffe ergänzen, bevor sie im Code auftauchen.

| Deutsch | Im Code | IFC | Anmerkung |
|---|---|---|---|
| Wand | `Wall` | IfcWall | |
| Wandaufbau | `WallType` | IfcWallType | Folge von Schichten |
| Schicht | `Layer` | IfcMaterialLayer | |
| Schichtfunktion | `LayerFunction` | | Tragschicht, Dämmung, Installationsebene, Bekleidung, Luftschicht |
| Tragschicht | `LayerFunction::Structure` | | höchste Priorität bei Anschlüssen |
| Wandachse | `WallAxis` | | Bezugslinie der Wand |
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
| Schraffur | `Hatch` | | |
| Planblatt | `Sheet` | | |
| Plankopf | `TitleBlock` | | |
| Bestand, Abbruch, Neubau | `Phase::{Existing, Demolished, New}` | | Farbkonvention nach Bauvorlagenverordnung |
| Abstandsfläche | `SetbackArea` | | nach Landesbauordnung |
| Variante | `Variant` | | |
| Bürostandard | `OfficeTemplate` | | |
