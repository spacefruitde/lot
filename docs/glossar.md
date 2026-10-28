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
| Lage der Bezugslinie | `WallAlignment` | | Mitte, Außenfläche, Innenfläche, Kernmitte, Kern außen, Kern innen; eine Lesart, ein Wechsel verschiebt die Wand nicht (ADR 0045) |
| Kern | `WallType::core` | | von der ersten bis zur letzten Tragschicht |
| Umdrehen | `walls::flip_ops` | | innen und außen tauschen, die Wand bleibt liegen |
| Oberkante | `Top` | | feste Höhe oder bis zum nächsten Geschoss |
| Unterkante über OKFF | `base_offset` | | |
| OKFF | `Storey::elevation` | | Oberkante Fertigfußboden über ±0,00 |
| Geschosshöhe | `Storey::height` | | OKFF bis OKFF darüber |
| Schichtrang | `LayerFunction::priority` | | 1 = Tragschicht, bestimmt das Ineinandergreifen an Anschlüssen |
| Kniestock | | | niedrige Außenwand im Dachgeschoss, `Top::Height` |
| Anschluss, Wandknoten | `Junction` | | auswählbar, mit Erklärung, welche Schicht welche schneidet (ADR 0045) |
| Ausnahme am Anschluss | `JoinRule` | | durchlaufen (`Through`), nicht verbinden (`Butt`) |
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
| Bestand, Abbruch, Neubau | `Phase::{Existing, Demolished, New}` | | Bauzustand; Farben nach Bauvorlagenverordnung (ADR 0031) |
| Umbauplan, Bestandsplan, Neuzustand | `PhaseFilter::{All, Existing, Planned}` | | welcher Bauzustand gezeigt wird |
| Ausbruch | | | neue Öffnung in einer Bestandswand, im Umbauplan gelb |
| Unterlage | `Underlay` | | PDF, Bild, DXF oder DWG unter einem Grundriss (ADR 0032) |
| Kalibrieren | `CalibrateUnderlay` | | Unterlage über eine Strecke mit bekanntem Maß skalieren |
| Datei im Projekt | `Attachments` | | etwa Unterlagen und Referenzmodelle, in der Projektdatei gespeichert |
| Gelände | `Terrain` | IfcSite | Höhenpunkte und Dreiecksnetz (ADR 0034) |
| Höhenpunkt | `TerrainPoint` | | |
| Erdreich | `erdreich` (Material) | | Körper unter dem Gelände, im Schnitt schraffiert |
| Variantengruppe | `VariantSet` | | eine Frage mit mehreren Varianten (ADR 0035) |
| Referenzmodell | `ReferenceModel` | | IFC eines Fachplaners, sichtbar, nicht bearbeitbar (ADR 0036) |
| Layer (DXF, DWG) | `floor_plan_layers` | | Gruppen im CAD-Austausch, etwa `A_Bauteile_Schnitt` |
| Abstandsfläche | `SetbackArea` | | nach Landesbauordnung, Tiefe T vor jeder Außenwand (ADR 0037) |
| Variante | `Variant` | | eine Antwort in einer Variantengruppe |
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
| Wandhöhe H | `SetbackSample::height` | | Gelände bis Oberkante der Wand oder Dachhaut, je Messpunkt |
| Tiefe T | `SetbackSample::depth` | | Tiefe der Abstandsfläche, etwa 0,4 H, mindestens 3 m |
| Landesrecht | `StateRules` | | Werte je Bundesland als Daten in `lot-rules/landesrecht/` |
| Gebäudeklasse | `BuildingClassResult` | | nach MBO § 2 Abs. 3; nicht zu verwechseln mit der Gebäudeart nach DIN 18065 (`BuildingClass`) |
| oberirdisches Geschoss | `BuildingClassResult::above_ground` | | Deckenoberkante im Mittel mehr als 1,40 m über Gelände |
| Vollgeschoss | `land_use` | | für die Geschossfläche, Definition nach Landesrecht |
| GRZ, GFZ | `land_use` | | Grundflächenzahl und Geschossflächenzahl nach §§ 19, 20 BauNVO |
| Grundstück, Flurstück | `Plot` | | Grenzen; Bezeichnung nach Kataster in `ProjectInfo::parcel` |
| Nachbargrundstück | `Plot::neighbour` | | nur zur Darstellung im Lageplan |
| öffentliche Fläche | `PublicEdge` | | Straße, Grün oder Wasser vor einer Grenze, mit Breite |
| Höhe über NHN | `ProjectInfo::datum` | | Höhe von ±0,00 in m über Normalhöhennull |
| Bauvorhaben | `ProjectInfo::description` | | etwa „Neubau eines Mehrfamilienhauses“ |
| Lageplan | `site_plan` | | mit Grundstück, Gebäude und Abstandsflächen (ADR 0039) |
| Aufzug | `Elevator` | IfcTransportElement | Fahrkorb mit Tür, hält in den genannten Geschossen |
| Fahrkorb | `Elevator::cabin` | | nutzbare Grundfläche steht im Bauantrag |
| Nutzungseinheit, Wohnung | `Space::unit` | | etwa „W 01“, leer bei gemeinsamen Räumen |
| Treppenraum | `StoreyRooms::stair_rooms` | | Raum, in dem eine notwendige Treppe beginnt oder ankommt |
| Rettungsweg | `escape` | | erster über den Treppenraum, zweiter über Fenster und Feuerwehr |
| Anleiterhöhe | | | Oberkante Brüstung über Gelände, über 8 m nur mit Hubrettungsfahrzeug |
| Bewegungsfläche | `square_fits` | | freies Quadrat, etwa 1,20 x 1,20 m nach DIN 18040-2 |
| lichte Durchgangsbreite | `AccessibleNorm::clear_door` | | Rohbaumaß minus Zarge und Türblatt |
| Prüfbericht | `CheckReport` | | alle Prüfungen mit Quelle (ADR 0038) |
| Befund | `Finding` | | erfüllt, Hinweis oder nicht erfüllt, mit Quelle und Ort |
| Hinweis im Modell | `draw_findings` | | Befund mit Ort als Kreis im Grundriss |
| Tür- und Fensterliste | `opening_schedule` | | Positionen und Typen mit Rohbaumaßen |
| Position (Liste) | `OpeningRow::mark` | | etwa „T 0.03“ |
| Typ (Liste) | `OpeningType` | | gleiche Art und Rohbaumaße |
| Bürostandard eines Büros | `OfficeOverrides` | | `buerostandard.ron` mit den Abweichungen (ADR 0040) |
| Präsentationsplan | `presentation_plan` | | Wände dunkel, Räume farbig, ohne Maße (ADR 0041) |
| Maßstabsleiste | `scale_bar` | | |
| Nordpfeil | `north_arrow` | | Richtung aus `ProjectInfo::north` |
