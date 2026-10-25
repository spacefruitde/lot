# Recherche: Öffnungen, Türen und Fenster

Anhang zum Paket. 59 Seiten aufgerufen. Revit-Hilfen 2026 waren nicht abrufbar, die Revit-Angaben stammen aus den Hilfen 2020, 2023 und 2024; Archicad aus der Hilfe 29, Vectorworks und Allplan aus den Hilfen 2025. Nummern in eckigen Klammern verweisen auf die Quellen am Ende. „Unbelegt“ heißt: keine Quelle gefunden.

## Revit

- **Setzen:** Vorschau beim Überfahren, ein Klick setzt, die Öffnung schneidet sich selbst [1]. Temporäre Maße von der Türachse zu den Achsen der nächsten Querwände [1, 2], umstellbar auf Oberfläche, Kern, Öffnung [3]. Mittig setzen, mehrere auf einmal: unbelegt.
- **Maße:** Breite und Höhe sind Typparameter [4], Rohbaumaße nur, wenn die Familie sie definiert [5]; Brüstung und Sturz sind Exemplarparameter über der Ebene [4].
- **Aufschlag:** Zeigerposition bestimmt die Seite, Leertaste wechselt links und rechts [1]. Kein Parameter für DIN links/rechts; Revit Ideas fordert „is flipped“ [10], Workarounds im Forum [11].
- **Darstellung:** Detailgrade je Maßstab, Familie zeigt je Detailgrad andere Geometrie [7]; „vorgeschnitten in Grundrissen“ [8]; Fenster werden auf der Schnitthöhe der Familie geschnitten, nicht des Projekts [9].
- **Typen:** ladbare Familien, eigene im Familieneditor [4, 7]. **Bewegen:** „Neuen Host wählen“ [6]. **Kennzeichnung:** Beschriftung beim Setzen [1], Aufschlag in Listen nicht auswertbar [10, 11].

## Archicad 29

- **Setzen:** nur in Wände; Tab wechselt an Kreuzungen die Wand; Sonnensymbol zeigt die Laibungsseite; Ankerpunkt Mitte, links, rechts; zwei Klicks (Position, dann Öffnungsrichtung) [12]; mehrere über Multiplizieren [18].
- **Maße:** Nennmaß als Wandöffnung, Einheit, Durchgang oder Flügel [21]; Brüstung und Sturz auf Wandunterkante, Wandoberkante oder Geschoss [14]; „Fußbodenaufbau“ ändert nur die Anzeige im Maßmarker [15].
- **Aufschlag:** „Spiegeln“ kehrt die Richtung um [12]; DIN links/rechts unbelegt.
- **Darstellung:** Symbolisch, Alles Überkopf, Projiziert [19]; Schnitt passt nicht zum Detailgrad des Grundrisses [24].
- **Typen:** GDL-Bibliothek [13]; Laibung am Wandkern verankerbar [16]; leere Öffnung als Durchbruch [17], Nischen mit dem Öffnungswerkzeug [22].
- **Bewegen:** entlang der Wand ziehen; in eine andere Wand nur bei gleicher Referenzebene [18].

## Vectorworks 2025

- **Setzen:** Standard- und Versatzeinfügung, Vorschau mit Anschlag und Innenseite [30]; im Versatzmodus Abstand zu einem Bezugspunkt [33]; Lage in der Wanddicke wählbar [31].
- **Maße:** Bezug Flügelmaß, Einheit oder Rohbauöffnung (metrisch Standard) [31, 32]; Brüstung oder Sturz, Feld für Mauerwerksmaß [32].
- **Darstellung:** Detailstufen [31, 32, 34]; Fenster über einer Höhe im Grundriss gestrichelt [32].
- **Typen:** Plug-in-Objekte mit Stilen [30]; Wandabschluss für mehrschalige Wände [31, 32]. **Kennzeichnung:** Präfix, Bezeichnung, Suffix, Datenfelder [31, 32].

## Allplan 2025

- **Setzen:** Klick nahe der ersten Öffnungsseite; die Dialogzeile zeigt den Abstand zum nächsten Bezugspunkt, Enter übernimmt oder man tippt; anderer Bezug per Klick; weitere Öffnungen ohne neue Eingaben, Favoriten [38].
- **Maße:** Öffnungsbreite, Brüstungshöhe, Öffnungshöhe [39]; Höhenbezug über die Ebenentechnik [39]; die Öffnung ist das Rohbaumaß, SmartParts und Makros füllen sie [42].
- **Aufschlag:** Aufschlagsart mit Symbol und Öffnungswinkel [39].
- **Typen:** Öffnung nimmt Bibliothekselemente auf, Kennung „Fenstertür“ [39]; Anschlagtypen nach Zahl der Wandschalen, Öffnungsmodellierer mit Falz, Rollladenkasten, Sturz [40]; Außen- und Innenlaibung [41].

## Neuere Werkzeuge

- **Snaptrude:** Abstand zur nächsten Wandkante beim Setzen, nicht in eine andere Wand kopierbar, Abzug in Mengen [46]; Orientierung über vier Quadranten, zweiter Klick bestätigt [47].
- **Arcol:** Türen bleiben beim Bearbeiten der Wand stehen, Seite per Hover [48, nur Suchtreffer]. **Qonic:** Nettoflächen mit Öffnungsabzug [49]. **Motif:** verschobene Wand nimmt Abhängiges mit [50].

## Was Nutzer nervt (belegt)

- **Revit:** Aufschlag in Listen nicht auswertbar, Spiegeln bricht Workarounds [10, 11]; Schnitthöhe der Familie statt des Projekts [9].
- **Archicad:** Abstandsführung misst am Rahmen statt an der Öffnung, 15 mm Differenz [23]; Schnitt passt nicht zum Detailgrad [24]; Fenster in 1:100 nicht so detailliert wie gewünscht [28]; Fenster in Wänden mühsam zu wählen [25]; zu viele Einstellungen [26]; gespiegelte Türen in der Liste gleich [27]; Umsteiger vermissen „erst Öffnung, dann Füllung“ [29].
- **Vectorworks:** Öffnungen schneiden 2026 im Grundriss nicht (Bug) [35]; Türen in Wänden nicht wählbar [36]; kein Loch ohne Einfügemodus [37].
- **Allplan:** Öffnung landet in der falschen Wand [43]; Fenstermodellierer liest nur den äußeren Anschlag [44]; Bibliothek „very primitive“ [45].

## Quellen

[1] https://help.autodesk.com/cloudhelp/2020/ENU/Revit-Model/files/GUID-7E952007-8A5A-4693-8E0F-A847F72F7617.htm · [2] https://help.autodesk.com/cloudhelp/2020/ENU/Revit-Model/files/GUID-3B7BAC64-6120-4372-A29E-95C4A1D853F5.htm · [3] https://www.autodesk.com/learn/ondemand/module/layout-basic-architectural-elements-in-revit/unit/62eg5yXtt2MEBBs2g3hTBo · [4] https://help.autodesk.com/cloudhelp/2023/ENU/Revit-ArchDesign/files/GUID-08C6541F-4974-489E-B941-F365628B2884.htm · [5] https://static.au-uw2-stg.autodesk.com/presentation_10688_AS10688_20Revit_20Master_20Class_20Building_20Terrific_20Real-World_20Door_20Families_20PPTX.pdf · [6] https://help.autodesk.com/cloudhelp/2014/ENU/Revit/files/GUID-9958015A-2449-4EA0-804C-E561C3D24D6B.htm · [7] https://help.autodesk.com/cloudhelp/2024/ENU/Revit-Customize/files/GUID-0BB861D5-27CB-4276-93AE-F2E949B0D466.htm · [8] https://help.autodesk.com/cloudhelp/2020/ENU/Revit-Model/files/GUID-8B72F46F-3341-474E-861D-5538E2D7C128.htm · [9] https://forums.autodesk.com/t5/revit-architecture-forum/window-display-problem-when-cut-in-floor-plans/m-p/9087849 · [10] https://forums.autodesk.com/t5/revit-ideas/adding-quot-is-flipped-quot-parameter-to-families/idi-p/12792365 · [11] https://forums.autodesk.com/t5/revit-architecture-forum/door-swing-direction-in-schedule/td-p/7595717

[12] https://help.graphisoft.com/AC/29/INT/_AC29_Help/040_ElementsVB/040_ElementsVB-306.htm · [13] https://help.graphisoft.com/AC/29/INT/_AC29_Help/040_ElementsVB/040_ElementsVB-305.htm · [14] https://help.graphisoft.com/AC/29/INT/_AC29_Help/040_ElementsVB/040_ElementsVB-310.htm · [15] https://help.graphisoft.com/AC/29/INT/_AC29_Help/040_ElementsVB/040_ElementsVB-311.htm · [16] https://help.graphisoft.com/AC/29/INT/_AC29_Help/040_ElementsVB/040_ElementsVB-312.htm · [17] https://help.graphisoft.com/AC/29/INT/_AC29_Help/040_ElementsVB/040_ElementsVB-313.htm · [18] https://help.graphisoft.com/AC/29/INT/_AC29_Help/040_ElementsVB/040_ElementsVB-314.htm · [19] https://help.graphisoft.com/AC/29/INT/_AC29_Help/040_ElementsVB/040_ElementsVB-308.htm · [20] https://help.graphisoft.com/AC/29/INT/_AC29_Help/060_ElementParameters/060_ElementParameters-12.htm · [21] https://help.graphisoft.com/AC/22/INT/_AC22_Help/141_DoorWindowCustomSettings/141_DoorWindowCustomSettings-3.htm · [22] https://help.graphisoft.com/AC/29/INT/_AC29_Help/040_ElementsVB/040_ElementsVB-258.htm · [23] https://community.graphisoft.com/t5/Wishlist/Improved-Distance-Guides/idc-p/650767 · [24] https://community.graphisoft.com/t5/Wishlist/Door-window-MVO-section-still-doesn-t-correspond-to-plan-detail/idi-p/714377 · [25] https://community.graphisoft.com/t5/Libraries-objects/Selecting-windows-and-doors-in-a-wall/td-p/75184 · [26] https://community.graphisoft.com/t5/Libraries-objects/Problems-ganging-windows-and-other-settings/m-p/192612 · [27] https://community.graphisoft.com/t5/Collaboration-with-other/Door-window-handing-in-interactive-schedules/m-p/12965 · [28] https://community.graphisoft.com/t5/Bibliotheken-und-Objekte/T%C3%BCren-und-Fenster-im-Ma%C3%9Fstab-1-50-detaillierter-darstellen/m-p/558848 · [29] https://community.graphisoft.com/t5/Bibliotheken-und-Objekte/Bibliothekselemente-zB-Fenster-immer-mit-fixen-Ma%C3%9Fen/td-p/552041

[30] https://app-help.vectorworks.net/2025/eng/VW2025_Guide/Doors/Inserting_doors.htm · [31] https://app-help.vectorworks.net/2025/eng/VW2025_Guide/Doors/Door_settings.htm · [32] https://app-help.vectorworks.net/2025/eng/VW2025_Guide/Windows/Window_settings.htm · [33] https://app-help.vectorworks.net/2023/eng/VW2023_Guide/Symbols/Offset_symbol_insertion_mode.htm · [34] https://app-help.vectorworks.net/2019/eng/VW2019_Guide/Symbols/Customizing_Detail_Levels_for_2D_and_3D_Components_of_Symbol.htm · [35] https://forum.vectorworks.net/topic/128267-vw-2026-doors-windows-inserted-in-walls-not-opening-in-plan-view/ · [36] https://forum.vectorworks.net/topic/134012-cant-select-door-or-window-objects-set-in-walls/ · [37] https://forum.vectorworks.net/topic/102505-doors-and-windows-not-creating-holes-in-walls/

[38] https://help.allplan.com/Allplan/2025-0/1031/Allplan/10270.htm · [39] https://help.allplan.com/Allplan/2025-0/1031/Allplan/10075.htm · [40] https://help.allplan.com/Allplan/2025-0/1031/Allplan/10276.htm · [41] https://help.allplan.com/Allplan/2018-1/1031/Allplan/10097.htm · [42] https://help.allplan.com/Allplan/2024-1/0000/SmartPartsScriptReference/81184.htm · [43] https://campus.allplan.com/cz/forum/topics/topic/topics/cad-architektur/oeffnung-kommt-in-falsche-wand.html · [44] https://campus.allplan.com/de/forum/themen/topic/topics/cad-architektur/fenstertueranschlaege.html · [45] https://campus.allplan.com/es/foro/topics/topic/topics/cad-architecture/the-object-library-of-allplan-is-very-primitive.html

[46] https://help.snaptrude.com/en/articles/9040869-placing-and-editing-doors-and-windows-in-a-project · [47] https://snaptrude.com/product-releases/v2-1-0 · [48] https://arcol.io/product-updates/extrusions · [49] https://help.qonic.com/documentation/qonic-application/analysing/quantify · [50] https://motif.io/design/bim

Lücken: Verhalten beim Löschen oder Teilen der Wand, mittig setzen, Baurichtmaß nach DIN 18100 und Rollladen waren nirgends belegt; Qonic und Motif fast ohne Doku; einige Forenbelege nur über Titel oder Suchauszüge, [25] bis [27] und [44] sind älter.
