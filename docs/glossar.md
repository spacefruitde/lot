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
| Deckenaufbau | `SlabType` | IfcSlabType | Schichten von oben nach unten |
| Estrich | `estrich` (Material) | | |
| Stütze | `Column` | IfcColumn | |
| Unterzug | `Beam` | IfcBeam | |
| Öffnung | `Opening` | IfcOpeningElement | |
| Tür | `Door` | IfcDoor | |
| Fenster | `Window` | IfcWindow | |
| Laibung | `Reveal` | | |
| Brüstung, Brüstungshöhe | `sill_height` | | Höhe über Fertigfußboden |
| Sturz, Sturzhöhe | `head_height` | | |
| Anschlag (Fenster, Tür) | `Rebate` | | Falz, an dem der Flügel anschlägt |
| Anschlag DIN links, DIN rechts | `Handing` | | nach DIN 107: Bänder links oder rechts, gesehen von der Seite, zu der die Tür aufschlägt |
| Türaufschlag | `Door::opens_to`, `FlipDoorSwing` | | Seite der Wand, zu der die Tür aufschlägt |
| Raum | `Space` | IfcSpace | gespeichert nur als Punkt, Fläche aus den Wänden |
| Raumstempel | `draw_space_tags` | | Nummer, Name, Netto-Raumfläche |
| Dach | `Roof` | IfcRoof | Umriss mit Traufen und Ortgängen, Dachfläche als untere Hülle der Traufebenen (ADR 0025) |
| Dachaufbau | `Roof::roof_type` | | ein `SlabType`, Schichten von oben nach unten |
| Dachfläche | `RoofFace` | | Fläche einer Traufe: a entlang der Traufe, d waagerecht nach innen |
| Traufhöhe | `Roof::eave_height` | | Dachhaut an der Außenkante über OKFF |
| Traufe | `RoofEdge::Eave` | | mit Neigung und Überstand |
| Ortgang | `RoofEdge::Verge` | | Giebelseite, nur Überstand |
| First | | | ergibt sich, wo Dachflächen sich treffen |
| Gaube | `Dormer` | | vorerst Schleppgaube (ADR 0029) |
| Dachfenster | `RoofWindow` | IfcWindow | in der Dachfläche einer Traufe |
| Treppe | `Stair` | IfcStair | gerade Läufe mit Podesten (ADR 0028) |
| Lauf, Treppenlauf | `StairFlight` | IfcStairFlight | |
| Podest | `Landing` | | Zwischen-, Viertel- oder Halbpodest |
| Lauflinie | `StairLayout::walking_line` | | |
| Antritt, Austritt | `Stair::start`, `StairLayout::end` | | erste Stufe, Ankunft oben |
| Steigung (s) | `StairLayout::rise` | | Geschosshöhe durch Zahl der Steigungen |
| Auftritt (a) | `Stair::tread` | | |
| Schrittmaß | `StairCheck::step` | | 2s + a nach DIN 18065 |
| Laufbreite | `Stair::width` | | |
| Laufplatte | `Stair::thickness` | | |
| lichte Treppendurchgangshöhe | `StairCheck::headroom` | | |
| Geländer | `Railing` | IfcRailing | frei oder an einem Treppenlauf |
| Höhenkote | `draw_level_marks` | | in Schnitten und Ansichten |
| Schnitt, Ansicht | `SectionView`, `SectionKind` | | Linie, Blickrichtung, Blicktiefe (ADR 0026) |
| Blickrichtung | `SectionView::look` | | links oder rechts der Schnittlinie |
| Blicktiefe | `SectionView::depth` | | |
| Maßkette | `DimensionChain` | IfcAnnotation | speichert Bezüge (`DimRef`), nicht Maße |
| Stiftset | `PenSet` | | Linienbreiten je Maßstab |
| Stift | `Pen` | | eine Linienbreite in mm auf dem Papier |
| Linienart | `LineStyle` | | durchgezogen, gestrichelt |
| Planobjekt | `PlanObject` | | Linie, Fläche oder Text auf dem Blatt; daraus entstehen Bildschirm, PDF, SVG, DXF |
| Operation | `Operation` | | eine Änderung am Modell, umkehrbar; Grundlage für Undo |
| Projekt, geöffnet | `Document` | | Projekt mit Operationsprotokoll und Undo/Redo |
| Abstand der Öffnung vom Wandanfang | `Opening::offset` | | entlang der Wandachse bis zur Kante der Öffnung |
| Schraffur | `Hatch` | | |
| Planblatt | `PlanSheet` | | im Modell; die gezeichnete Seite heißt `Sheet` (ADR 0027) |
| Plankopf, Schriftfeld | `title_block` | | |
| Projektangaben | `ProjectInfo` | | Bauherr, Bauort, Entwurfsverfasser, Gebäudeart |
| Gebäudeart | `BuildingClass` | | nach DIN 18065 Tabelle 1 |
| 2D-Element | `Annotation` | IfcAnnotation | Linie, Fläche, Kreis oder Text in einer Ansicht (ADR 0030) |
| Bestand, Abbruch, Neubau | `Phase::{Existing, Demolished, New}` | | Farbkonvention nach Bauvorlagenverordnung |
| Abstandsfläche | `SetbackArea` | | nach Landesbauordnung |
| Variante | `Variant` | | |
| Bürostandard | `OfficeStandard` | | Materialien, Schraffuren, Stiftsets je Maßstab |
| Detailgrad | `Detail` | | grob (1:200), mittel (1:100), fein (1:50) |
| Schnittkante | `Pens::cut` | | Umriss geschnittener Bauteile |
| Werkzeug | `Tool` | | Auswählen, Wand, Bogenwand, Achse, Stütze |
| Fangen, Fangpunkt | `snap`, `Snap` | | Einrasten an Wandenden, Wänden, Achsen, Winkeln |
| Befehl | `Command` | | jede Funktion mit Kennung, für Befehlspalette und KI |
| Planansicht | `ViewMode::Plan` | | Grundriss auf dem Bildschirm, wie gedruckt |
| Bogenwand | `Wall` mit `sagitta` ≠ 0 | | gebogene Wand, intern facettiert (ADR 0020) |
| Stich (Pfeilhöhe) | `sagitta` | | Abstand der Bogenmitte von der Sehne, links positiv |
| Sehne | `chord` | | Gerade zwischen Anfang und Ende einer Bogenwand |
| Facette | | | gerades Stück einer Bogenwand für die Anschlüsse, höchstens 2 mm vom Bogen |
| Nutzungsgruppe | `SpaceUsage` | | NUF 1 bis 7, TF, VF nach DIN 277 |
| Netto-Raumfläche (NRF) | `SpaceArea::net` | | DIN 277, an den fertigen Oberflächen |
| Brutto-Grundfläche (BGF) | `StoreyAreas::gross` | | DIN 277, innerhalb der Außenkanten |
| Konstruktions-Grundfläche (KGF) | `StoreyAreas::construction` | | BGF minus NRF |
| Brutto-Rauminhalt (BRI) | `StoreyAreas::volume` | | vorläufig BGF mal Geschosshöhe |
| Wohnfläche | `SpaceArea::living` | | nach WoFlV, nur Räume mit `living_space` |
| Maßhilfslinie | | | kurze Linie vom Bauteil zur Maßlinie |
| Maßbegrenzung | | | Schrägstrich unter 45° |
| Rohbaumaß | | | Maß der Öffnung im Rohbau, nach DIN 18100 |
