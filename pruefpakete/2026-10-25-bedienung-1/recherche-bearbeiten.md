# Recherche: Bauteile bearbeiten

Anhang zum Paket. Stand Oktober 2026, geprüft an Revit 2026, Archicad 29/30, Vectorworks 2026 und Allplan 2026; wo es nur ältere Hilfeseiten gab, steht das Jahr dabei. Kürzel in eckigen Klammern verweisen auf die Quellen am Ende. „Unbelegt“ heißt: keine Quelle gefunden. Klickzahlen sind aus den Hilfeschritten abgeleitet, nicht gemessen.

## Revit

- **Grundbefehle:** MV, CO, RO, AR, OF, AL, TR, SL; Spiegeln mit DM [R3]. Werkzeugoptionen bleiben gemerkt (Versetzen mit Kopie, Spiegeln ohne Kopie, Reihe ohne Gruppe) [R4]; seit 2026 im Kontextbereich des Menübands, nur Drittquelle [R14].
- **Verschieben:** wählen, MV, Startpunkt, Ziel oder Zahl und Enter: 3 Klicks oder 2 Klicks plus Zahl [R1]. „Lösen“ gibt dem Element eine neue ID, Kennzeichnung und Bemaßungen können verloren gehen [R1].
- **Ausrichten:** AL, Referenz, Element, optional Schloss; mit Strg mehrere; „Prefer“ wählt Wandfläche, Achse oder Kern [R10]. **Stutzen/Dehnen:** zur Ecke, einzeln, mehrfach [R11].
- **Griffe:** Ziehpunkte, Formgriffe, Spiegelpfeile, Drehgriffe, 3D-Steuerungen; Tab schaltet bei mehrfach belegten Griffen um [R8].
- **Zahlen:** beim Skizzieren Zahl tippen und Enter („zuhörende“ Maße), nicht bei Rechteck und Spline [R5]. Temporäre Maße anklicken, Wert eingeben, das Element verschiebt sich [R7]; die Bezugslinie des Maßes lässt sich umsetzen [R6]. Rechnen in Feldern unbelegt.
- **Steuernde Maße:** Schloss und EQ; die Hilfe warnt vor Überbestimmung [R9].
- **Mehrfachauswahl:** Eigenschaften zeigen gemeinsame Exemplarparameter mit Filter nach Kategorie; Typ bearbeiten nur bei gleichem Typ [R12]. Temporäre Maße verschwinden bei Mehrfachauswahl [R8].
- **Wiederholen und Undo:** Enter, RC oder Kontextmenü mit den letzten 5 Befehlen [R2]; Undo-Liste, eine neue Aktion löscht Redo [R13].
- **3D, Schnitt:** Formgriffe in Ansicht und 3D [R8].

## Archicad

- **Aufruf:** Menü, Kontextmenü, Leiste „Elemente ziehen“, Pet-Palette [A1]. Ziehen Cmd+D, mit Wahl als Kopie, mit Strg+Alt mehrere [A2]. Ab 28: Drehen Cmd+R, Spiegeln F, Schieben mit Pfeiltasten [A3]; F funktionierte anfangs nicht [AC2].
- **Vervielfältigen** (Ziehen, Drehen, Anheben, Matrix) [A4], **alle Kanten versetzen** mit Vorschau [A5], **Anpassen** (Stutzen/Dehnen) mit Cmd-Klick [A6], **Schere** auch in 3D [A7], Ausrichten und Verteilen [A12]. Wiederholen Cmd+B, nur Community [AC1].
- **Pet-Palette:** erscheint am Knoten, an der Kante oder Fläche eines gewählten Elements, Inhalt je Kontext, nicht anpassbar [A8].
- **Tracker:** direkt tippen, X3 oder Y-4, relativ zum Bearbeitungsursprung, Operatoren 20+, 3*, „/“ wechselt Polar und XY [A9].
- **Maße** steuern das Modell nicht [AP5]. **Mehrfachauswahl** zeigt die Werte eines Elements, „verschieden“ fehlt [AP4].
- **Undo-Liste** wird gelöscht beim Anlegen oder Löschen von Geschossen, Ändern der Geschosshöhe, Hotlinks, Teamwork [A10].
- **3D:** Bearbeitungsebene, Ziehen bleibt darauf [A11, A2].

## Vectorworks

- **Kürzel 2025:** Teilen L, Verbinden Wahl+L, Stutzen Wahl+Shift+L, Verschieben nach Punkten Shift+M, Versatz Shift+- [V7]. Duplizieren Cmd+D [V12]; „Verschieben nach Punkten“ mit Modi Verschieben, Duplizieren, Verteilen, Referenzpunkt [V5]; 2026 Anzahl der Duplikate in der Datenzeile [V6]. Wiederholen R [V11].
- **Umformen** mit Modi für Punkte und Kanten [V10]; bei Wänden Endpunkte, in 3D „Wand bearbeiten“ [V9]. „Verbundene Wände“ hält T und L, nicht Y und Bögen [V14].
- **Datenzeile** schwebt am Zeiger: tippen, Tab wechselt das Feld, zweimal Enter [V1, V2]. Rechnen mit Einheiten und Klammern („4"+3 cm“) [V3]. Bezugspunkt als temporärer Ursprung, Smart Points mit T [V15], Vektor-Sperren [V16].
- **Zwangsbedingungen** für 2D, Wände und Symbole [V13]; Alltagstauglichkeit unbelegt.
- **Mehrfachauswahl:** verschiedene Werte als leeres Feld, „Einzeln bearbeiten“ blättert durch die Auswahl [V4].

## Allplan

- Rechtsklick ins Leere: Verschieben, Drehen, Kopieren, Löschen; auf ein Element: dessen Werkzeuge [P5]. Wiederholen-Menü mit 30 Werkzeugen nur für 2016 belegt [P10]. Abstand paralleler Linien ändern [P7].
- **Direkte Objektmodifikation:** Zentral-, Punkt- und Geometriegriffe, Kontextleiste mit einstellbarer Verzögerung [P1, P2]; Verschieben über den Zentralgriff mit Delta [P2], Kopieren mit Strg und Anzahl [P3], mehrere Griffe gemeinsam ziehen [P4].
- **Delta-Punkt:** relative Eingabe zum gefangenen Punkt [P6]. Assoziative Maße folgen, der normale Verschiebebefehl bricht die Assoziativität [P8]. Mehrfachauswahl zeigt gemeinsame Werte (Hilfe 2015) [P9].

## Neuere Werkzeuge und Vorbilder

- **Arcol:** M, Referenzpunkt, Ziel, Distanzfeld an der Hilfslinie, Wahl kopiert [N1]; Q dreht [N2]; Raster über temporäre Maße [N3].
- **Snaptrude:** M, Strg+M für Kopie und Reihe, Q, P (Push/Pull), Pfeiltasten sperren Achsen [N4]; „Edit Length“ zeigt mit Pfeil die Richtung [N5].
- **Motif:** Wände verbinden sich automatisch; Klicken und Halten listet überlappende Objekte; Bearbeiten auch per Text an einen KI-Agenten [N6].
- **Qonic:** keine Doku gefunden; laut App-Store bleiben verbundene Elemente beim Verschieben verbunden [N7], nicht geprüft.
- **SketchUp:** tippen ohne Klick, Wert korrigierbar bis zur nächsten Aktion, [x,y,z] absolut, <x,y,z> relativ, Reihen mit „3x“ oder „3/“, Pfeiltasten sperren die Richtung [N8, N9].
- **Rhino 8:** Griff anklicken und Wert tippen oder beim Ziehen tippen, Alt kopiert [N10]. **Figma:** Mehrfachbearbeitung über Rahmen, gleichartige auswählen [N11].

## Was Nutzer nervt (belegt)

- **Revit:** temporäre Maße stören an Griffen [RP1]; EQ fehlt bei temporären Maßen [RP2]; eigene temporäre Maße fehlen [RP3]; Wände verbinden sich ungewollt, besonders über Bauphasen [RP4], springen nach „Verbindung nicht zulassen“ zurück [RP7]; keine XYZ-Eingabe [RP5]; Schlösser machen mehr Probleme als sie lösen [RP6].
- **Archicad:** Klicks in der Pet-Palette greifen nicht oder öffnen das falsche Werkzeug [AP1]; Versatz muss jedes Mal neu getippt werden [AP2]; Versatz-Symbol erscheint nur unter Bedingungen [AP7]; Anschlüsse reißen beim Strecken ab [AP3]; „verschieden“ fehlt [AP4]; keine bearbeitbaren temporären Maße [AP5].
- **Allplan, Archicad:** „Punkte modifizieren“ (mehrere Elemente auf einmal strecken) vermissen Umsteiger, andere nennen es gefährlich [AP6]; ungewollt mitgewählte Nachbardecken [PP1].
- **Vectorworks:** Ansichtsänderungen landen im Undo [VP1, VP2], Schwenken beim Rückgängigmachen löscht Redo [VP4], Stutzen in 3D schwach [VP3].

## Quellen

Revit: R1 https://help.autodesk.com/cloudhelp/2026/ENU/Revit-Model/files/GUID-399D3557-93A0-42B7-AF77-E28931823335.htm · R2 https://help.autodesk.com/cloudhelp/2026/ENU/Revit-Model/files/GUID-B0C50D1C-6A4C-4F9A-A418-0AEE07BC940B.htm · R3 https://www.autodesk.com/shortcuts/revit und https://revit.downloads.autodesk.com/download/2024RVT_RTM/Docs/InProd/RVTKeyboardShortcuts.pdf · R4 https://help.autodesk.com/cloudhelp/2026/ENU/Revit-Customize/files/GUID-C08A9D8C-4E9F-4E1D-A515-26F33509ABA1.htm · R5 https://help.autodesk.com/cloudhelp/2026/ENU/Revit-DocumentPresent/files/GUID-84FFE118-6EF7-4747-8DF7-AE675818645E.htm · R6 https://help.autodesk.com/cloudhelp/2026/ENU/Revit-DocumentPresent/files/GUID-74FFB00B-FB29-44B7-8FEA-613DBE010AD4.htm · R7 https://help.autodesk.com/cloudhelp/2019/ENU/RevitLT-GetStarted/files/GUID-CC0FF240-4F0E-4393-AA50-17FA1E4A58DF.htm · R8 https://help.autodesk.com/cloudhelp/2026/ENU/Revit-GetStarted/files/GUID-EECE4EA1-DDBB-49A4-88DA-733DE0DE0B9E.htm · R9 https://help.autodesk.com/cloudhelp/2026/ENU/Revit-Model/files/GUID-08891DA1-73C1-45D3-9DE2-DAB016763AB6.htm · R10 https://help.autodesk.com/cloudhelp/2026/ENU/RevitLT-Model/files/GUID-9C867721-1970-4ECC-90F1-3C112B9EC2E6.htm · R11 https://help.autodesk.com/cloudhelp/2020/ENU/RevitLT-Model/files/GUID-D9AB5E1D-2D76-4D92-8000-24B78AAF09CC.htm · R12 https://help.autodesk.com/cloudhelp/2023/ENU/Revit-GetStarted/files/GUID-A764EA7A-FE26-469B-857C-F3A70812FC34.htm · R13 https://help.autodesk.com/cloudhelp/2016/ENU/Revit-Model/files/GUID-B534535A-5BB6-49E0-B49C-9453FB6B80CC.htm · R14 https://ukcommunity.arkance.world/hc/en-us/articles/27208220231314-Revit-2026-Changes-to-the-User-Interface-UI

Revit Ideas und Forum: RP1 https://forums.autodesk.com/t5/revit-ideas/turn-off-temporary-dimensions/idc-p/10496105 · RP2 https://forums.autodesk.com/t5/revit-ideas/eq-function-to-work-in-temporary-dimensions/idi-p/8148787 · RP3 https://forums.autodesk.com/t5/revit-ideas/create-temporary-dimensions/idi-p/9555355 · RP4 https://forums.autodesk.com/t5/revit-ideas/do-not-automatically-join-walls-in-different-phases/idc-p/7857978 · RP5 https://forums.autodesk.com/t5/revit-ideas/creating-a-point-in-space-with-direct-input-of-x-y-z-coordinates/idc-p/13305040 · RP6 https://forums.autodesk.com/t5/revit-architecture-forum/the-fifth-dimension/td-p/7841690 · RP7 https://forums.autodesk.com/t5/revit-architecture-forum/wall-join-problem-with-phasing-and-design-options/m-p/9570717

Archicad: A1 https://help.graphisoft.com/AC/29/INT/_AC29_Help/030_Interaction/030_Interaction-69.htm · A2 https://help.graphisoft.com/AC/30/INT/Topics/030_Interaction/030_Interaction-71.html · A3 https://help.graphisoft.com/AC/28/INT/_AC28_Help/005_NewFeatures/005_NewFeatures-22.htm · A4 https://help.graphisoft.com/AC/30/INT/Topics/030_Interaction/030_Interaction-96.html · A5 https://help.graphisoft.com/AC/30/INT/Topics/030_Interaction/030_Interaction-75.html · A6 https://help.graphisoft.com/AC/30/INT/Topics/030_Interaction/030_Interaction-86.html · A7 https://help.graphisoft.com/AC/30/INT/Topics/030_Interaction/030_Interaction-89.html · A8 https://help.graphisoft.com/AC/30/INT/Topics/020_Configuration/020_Configuration-27.html · A9 https://help.graphisoft.com/AC/29/INT/_AC29_Help/030_Interaction/030_Interaction-40.htm · A10 https://help.graphisoft.com/AC/30/INT/Topics/030_Interaction/030_Interaction-67.html · A11 https://help.graphisoft.com/AC/29/INT/_AC29_Help/030_Interaction/030_Interaction-50.htm · A12 https://help.graphisoft.com/AC/30/INT/Topics/030_Interaction/030_Interaction-66.html

Archicad Community: AC1 https://community.graphisoft.com/t5/Modeling/Repeat-last-command/td-p/118882 · AC2 https://community.graphisoft.com/t5/Modeling/Pet-Palette-Short-Cut/td-p/628856 · AP1 https://community.graphisoft.com/t5/Modeling/Bad-Pet-Pallet-Bad-No-biscuit/m-p/148683 · AP2 https://community.graphisoft.com/t5/Documentation/Is-there-a-way-to-set-Offset-that-you-frequently-use/td-p/605579 · AP3 https://community.graphisoft.com/t5/Wishlist/Maintain-Wall-Connections-when-Editing-Wall-Geometry/idc-p/716134 · AP4 https://community.graphisoft.com/t5/Wishlist/Display-quot-varies-quot-for-a-property-when-multiple-selected/idi-p/675250 · AP5 https://community.graphisoft.com/t5/Documentation/Temporary-dimensions/td-p/194278/page/2 · AP6 https://community.graphisoft.com/t5/Modellierung/Gummibandfunktion-wie-bei-allplan/td-p/490731 und https://community.graphisoft.com/t5/Alte-Wunschliste/2-wichtige-Funktionen-fehlen/m-p/481263 · AP7 https://community.graphisoft.com/t5/Installation-update/Offset-problem-in-BIM-Curriculum/td-p/180304

Vectorworks: V1 https://app-help.vectorworks.net/2026/eng/VW2026_Guide/Basic1/Drawing_with_the_Data_bar.htm · V2 https://app-help.vectorworks.net/2026/eng/VW2026_Guide/Basic1/Data_bar_options.htm · V3 https://app-help.vectorworks.net/2026/eng/VW2026_Guide/Start/Additional_key_functionality.htm · V4 https://app-help.vectorworks.net/2026/eng/VW2026_Guide/Objects_edit1/The_Object_Info_palette.htm · V5 https://app-help.vectorworks.net/2026/eng/VW2026_Guide/Objects_edit1/Moving_objects_by_clicking_with_the_Move_by_Points_tool.htm · V6 https://www.vectorworks.net/2026 · V7 https://app-help.vectorworks.net/2025/eng/assets/docs/VWKeyboardShortcuts.pdf · V8 https://app-help.vectorworks.net/2024/eng/VW2024_Guide/Objects_edit2/Combining_and_connecting_objects.htm · V9 https://app-help.vectorworks.net/2026/eng/VW2026_Guide/Walls_edit/Reshaping_walls.htm · V10 https://app-help.vectorworks.net/2020/eng/VW2020_Guide/Objects_edit2/2D_reshape_modes.htm · V11 https://app-help.vectorworks.net/2024/eng/VW2024_Guide/Basic1/Repeating_commands.htm · V12 https://app-help.vectorworks.net/2026/eng/VW2026_Guide/Objects_edit1/Duplicating_objects.htm · V13 https://app-help.vectorworks.net/2025/eng/VW2025_Guide/Basic3/Concept_Parametric_constraints.htm · V14 https://app-help.vectorworks.net/2016/eng/VW2016_Guide/Walls_edit/Moving_Connected_Walls.htm · V15 https://app-help.vectorworks.net/2025/eng/VW2025_Guide/Basic2/Snapping_to_a_smart_point.htm · V16 https://app-help.vectorworks.net/2026/eng/VW2026_Guide/Basic2/Creating_vector_locks.htm

Vectorworks Forum: VP1 https://forum.vectorworks.net/topic/43617-exclude-views-from-undo-history/ · VP2 https://forum.vectorworks.net/topic/62988-viewcamera-changes-are-undoed-why/ · VP3 https://forum.vectorworks.net/topic/108419-trim-tool-in-3d/ · VP4 https://forum.vectorworks.net/topic/126901-no-more-redo-history/

Allplan: P1 https://help.allplan.com/Allplan/2026-0/1033/Allplan/80483.htm · P2 https://help.allplan.com/Allplan/2026-0/1033/Allplan/5615.htm · P3 https://help.allplan.com/Allplan/2026-1/1033/Allplan/79539.htm · P4 https://help.allplan.com/Allplan/2026-1/1033/Allplan/5617.htm · P5 https://help.allplan.com/Allplan/2026-1/1033/Allplan/7562.htm · P6 https://help.allplan.com/Allplan/2026-1/1033/Allplan/10637.htm · P7 https://help.allplan.com/Allplan/2026-1/1033/Allplan/5923.htm · P8 https://help.allplan.com/Allplan/2026-1/1033/Allplan/6039.htm · P9 https://help.allplan.com/Allplan/2015-0/1033/Allplan/78018.htm · P10 https://help.allplan.com/PLANBAR/2016-1/1033/Allplan/7622.htm · PP1 https://campus.allplan.com/de/forum/themen/topic/topics/cad-architektur/punkmodifikation-deckenteile/page/2.html

Neuere Werkzeuge: N1 https://intercom.help/arcol/en/articles/10447851-move-copy-tool · N2 https://intercom.help/arcol/en/articles/10447913-rotate-tool · N3 https://intercom.help/arcol/en/articles/14809293-structural-grids · N4 https://help.snaptrude.com/en/articles/11639988-keyboard-shortcuts · N5 https://help.snaptrude.com/en/articles/11059545-push-pull-edit-tool · N6 https://help.motif.io/articles/3357326473-model-walls-floors-ceilings · N7 https://apps.apple.com/app/id6476260802 · N8 https://help.sketchup.com/en/using-measurements-box · N9 https://help.sketchup.com/en/sketchup/introducing-drawing-basics-and-concepts · N10 https://docs.mcneel.com/rhino/8/help/en-us/commands/gumball.htm · N11 https://help.figma.com/hc/en-us/articles/21635177948567

Nicht abrufbar (403) waren die Kürzelseite von Autodesk, mehrere Seiten der Archicad Community und die Website von Qonic; deren Aussagen stammen aus Suchtreffern.
