# Recherche: Typen und Eigenschaften

Anhang zum Paket. Stand Oktober 2026, etwa 70 Seitenaufrufe. Kürzel in eckigen Klammern verweisen auf die Quellen am Ende. Mehrere Revit-Belege stammen aus den Hilfen 2022 bis 2025. Seiten der Graphisoft Community lieferten 403, die Aussagen dort stützen sich auf Suchauszüge.

**Kurzfazit:** Nur Revit hat ein strenges Modell mit Typ und Exemplar; ein Typwert lässt sich nicht am Exemplar überschreiben, deshalb entstehen viele Typen. Archicad arbeitet mit Einstellungen am einzelnen Element, Favoriten sind nur Vorlagen. Vectorworks legt je Parameter fest, ob er „nach Stil“ oder „nach Exemplar“ gilt. Allplan führt verknüpfte Stile ein, bisher nur für Wände, und markiert Abweichungen mit `*`. Figma zeigt am besten, wie man Überschreibungen sichtbar macht und zurücksetzt.

## Revit

- **Typ und Exemplar:** Typwerte wirken auf alle Exemplare, Exemplarwerte auf die Auswahl [R1]; Typwerte im Dialog „Typ bearbeiten“ [R1]; eingebaute Parameter sind fest Typ oder Exemplar, kein Überschreiben am Exemplar [K4].
- **Anlegen und ändern:** Duplizieren und Umbenennen im Dialog oder Projektbrowser [R4], Vorschau im Dialog [R4b]; Typ tauschen über die Typauswahl, nur in der Kategorie, oder „Typ anpassen“ [R3]; „Nicht verwendete bereinigen“ [R6].
- **Bibliothek:** „Projektstandards übertragen“ nach Kategorien, bei Duplikaten überschreiben oder nur neue, Abhängigkeiten werden mitkopiert [R5].
- **Palette:** gemeinsame Parameter bei Mehrfachauswahl, abweichende als `<varies>` [R2]; Suche nur nach Familien- und Typnamen [K1]; Formeln im Familientypen-Dialog [R8]; 2026/2027 wandern Optionen in die Palette [R10].
- **Eigene Eigenschaften:** Projektparameter und gemeinsam genutzte Parameter aus einer Datei, nötig für Beschriftungen und kategorieübergreifende Listen [R7]; 2027 IFC-Parametermapping, regelbasierte Nummerierung, „Extended Properties“ [R10].
- **Pläne:** Wandbeschriftung mit Typkennzeichen, Türbeschriftung mit Kennzeichen [R9].

## Archicad 29

- **Typ und Exemplar:** jedes Element trägt seine Einstellungen; Favoriten speichern alles außer der Länge und sind nicht assoziativ [A1, AK1]; verknüpft sind die Attribute (Mehrschichtaufbauten, Profile, Baustoffe); Dialogtitel zeigt „Standard“ oder „ausgewählt/bearbeitbar“ [A7]; Verknüpfungssymbol zeigt, ob ein Eigenschaftswert dem Standard folgt [A10].
- **Anlegen und ändern:** Favoriten-Palette mit Vorschau, Ordnern, Suche [A2], mehrere gleichzeitig umdefinieren [A3]; Favorit auf die Auswahl anwenden mit Übertragungs-Sets [A12]; Purge und „Löschen und ersetzen“ [A6].
- **Bibliothek:** Favoriten aus PRF- oder Projektdateien [A4]; Attributmanager mit zwei Listen, Abgleich nach Name oder Index, fehlende Klassifizierungen verlieren Werte [A5].
- **Eigenschaften:** Verfügbarkeit nach Klassifizierung, mehrere Systeme parallel [A9]; Ausdrücke mit Parametern, Einheiten, Funktionen [A8]; keine DIN 276 mitgeliefert [AK3].
- **Listen:** Interaktive Liste editierbar, wirkt sofort im Modell [A11].

## Vectorworks 2026

- **Stile:** Parameter „nach Stil“, „nach Exemplar“ oder „nach Katalog“, gemischt in einem Stil [V1]; „Stilparameter ausblenden“ [V2]; „In stillos umwandeln“ [V2b].
- **Wandstile:** anlegen aus Voreinstellungen, aus einer Wand oder im Ressourcenmanager [V3]; an der Wand nur Textur und Komponentenhöhen änderbar [V4]; Stiländerung wirkt auf alle [V4b]; „Ersetzen“ mit Auswahl, was übernommen wird [V4]; Löschen und Ersetzen [V5].
- **Bibliothek:** Ressourcenmanager mit Vectorworks-, Arbeitsgruppen-, Benutzerbibliotheken und Favoriten [V7]; Import kopiert, Referenzieren hält die Verknüpfung [V7b].
- **Palette:** Objekt-Info mit Reitern Form, Daten, Render; abweichende Werte als leeres Feld, Einzelmodus blättert durch die Auswahl [V6]. **Daten:** Datensätze, Data Manager, IFC-Psets [V8]; Data Tag [V9].

## Allplan 2026

- **Palette:** „Gemeinsame Eigenschaften“, abweichende als `*variiert*`; „Parameter übernehmen“ von einem angeklickten Element [P1].
- **Favorit und Stil:** Favoriten sind gespeicherte Einstellungen ohne Verbindung [P3]; Stile nur für Wände, weitere „in Vorbereitung“ [P1, P2]; Abweichungen mit `*` vor dem Stilnamen [P3]; `<aus>` löst den Stil [P2].
- **Bibliothek:** Stilpfade Standard, Büro, Privat, Projekt [P2]. **Eigene Eigenschaften:** Attributsätze [P1], Attributmanager [P5], Formelattribute [P6]; DIN 276 im Modul IBD 2017 [P9]. **Listen:** Informations-Palette mit Gruppen und Summen [P7].

## Neuere Werkzeuge und Figma

- **Qonic:** Bibliotheken für Codes, Materialien, Eigenschaften, Orte, Typen, je Projekt oder Modell; Änderung wirkt in allen Modellen [N1]; Typ mit Name, IFC-Klasse, Subtyp [N2].
- **Snaptrude:** Wandtyp per Dropdown, Dicke und Höhe je Wand änderbar [N4], Team-Bibliothek und Import von Revit-Wandtypen [N4b]. **Arcol:** Kennzahlen live rechts [N5]. **Motif:** kein Typsystem dokumentiert [N6].
- **Figma:** Überschreibungen erlaubt, Struktur nicht; ein Menü listet nur die geänderten Eigenschaften, Zurücksetzen je Eigenschaft oder alles, „Push changes to main component“ [F1]; Varianten als benannte Eigenschaften [F2]; Bibliotheksupdates mit Vorher-Nachher-Vergleich [F3, F4].

## Was Nutzer nervt (belegt)

| Kritik | Programm | Beleg |
|---|---|---|
| keine Suche nach Parametern, Wunsch nach angehefteten Parametern | Revit | [K1], [K6] |
| Projektstandards übertragen nimmt alle Typen einer Kategorie | Revit | [K2] |
| Klassifikation am Typ erzwingt doppelte Typen | Revit | [K3] |
| Typ oder Exemplar bei eingebauten Parametern fest | Revit | [K4] |
| Optionsleiste entfällt, mehr Scrollen | Revit 2027 | [K5] |
| Favoriten aktualisieren platzierte Elemente nicht | Archicad | [AK1], [AK2] |
| keine DIN 276, keine Klassifizierung je Schicht | Archicad | [AK3], [AK4] |
| Änderung am Exemplar sperrt Komponenten gegen spätere Stiländerungen | Vectorworks | [VK1], [VK2], [VK3] |
| Stile nur für Wände | Allplan | [P1], [PK1] |

## Quellen

Revit: R1 https://help.autodesk.com/cloudhelp/2025/ENU/RevitLT-GetStarted/files/GUID-1B5B5C2E-072F-4349-9C70-D88204F9145D.htm · R2 https://help.autodesk.com/view/RVT/2022/ENU/?guid=GUID-A1F618B8-8C65-4D4B-B2E8-E24A190900B6 · R3 https://help.autodesk.com/cloudhelp/2023/ENU/Revit-GetStarted/files/GUID-6B966275-DDC7-4B01-A87F-8040FAF952FE.htm · R4 https://help.autodesk.com/cloudhelp/2025/ENU/Revit-GetStarted/files/GUID-2E3AA034-9A51-4A5C-B2A6-6ED83487BEBB.htm · R4b https://help.autodesk.com/cloudhelp/2016/ENU/Revit-GetStarted/files/GUID-119039A6-7720-43ED-975F-2B00009CE010.htm · R5 https://help.autodesk.com/cloudhelp/2026/ENU/RevitLT-Customize/files/GUID-2A49FE80-1B07-4C95-A0FE-42CE2001F3C2.htm · R6 https://help.autodesk.com/cloudhelp/2022/ENU/Revit-Customize/files/GUID-4EBBDE76-E019-486E-BFBF-EE326CAAE2C4.htm · R7 https://help.autodesk.com/cloudhelp/2025/ENU/RevitLT-Model/files/GUID-AEBA08ED-BDF1-4E59-825A-BF9E4A871CF5.htm · R8 https://help.autodesk.com/view/RVT/2023/ENU/?guid=GUID-7C0C2F22-8E85-4576-BD84-D710AC6D4B6F · R9 https://help.autodesk.com/cloudhelp/2025/ENU/Revit-GetStarted/files/GUID-1383D1BB-0FBA-40FB-9DCB-26E404835454.htm · R10 https://help.autodesk.com/cloudhelp/2027/ENU/Revit-WhatsNew/files/GUID-C81929D7-02CB-4BF7-A637-9B98EC9EB38B.htm

Revit Ideas und Forum: K1 https://forums.autodesk.com/t5/revit-ideas/properties-palette-search/idi-p/13276454 · K2 https://forums.autodesk.com/t5/revit-architecture-forum/transfer-certain-project-standards/td-p/11367227 und https://forums.autodesk.com/t5/revit-ideas/transfer-project-standards-better-control/idi-p/6664377 · K3 https://forums.autodesk.com/t5/revit-ideas/revit-assembly-code-type-vs-instance/idi-p/13928702 · K4 https://forums.autodesk.com/t5/revit-ideas/allow-built-in-parameters-to-change-between-instance-and-type/idi-p/9687131 · K5 https://forums.autodesk.com/t5/revit-ideas/restore-the-contextual-options-bar/idi-p/14273338 · K6 https://forums.autodesk.com/t5/revit-ideas/hide-family-parameters/idi-p/13805669

Archicad: A1 https://help.graphisoft.com/AC/29/GER/_AC29_Help/020_Configuration/020_Configuration-56.htm · A2 https://help.graphisoft.com/AC/29/INT/_AC29_Help/020_Configuration/020_Configuration-63.htm · A3 https://help.graphisoft.com/AC/29/INT/_AC29_Help/020_Configuration/020_Configuration-61.htm · A4 https://help.graphisoft.com/AC/29/INT/_AC29_Help/020_Configuration/020_Configuration-64.htm · A5 https://help.graphisoft.com/AC/29/INT/_AC29_Help/025_Attributes/025_Attributes-73.htm · A6 https://help.graphisoft.com/AC/29/INT/_AC29_Help/025_Attributes/025_Attributes-76.htm und https://help.graphisoft.com/AC/29/INT/_AC29_Help/025_Attributes/025_Attributes-69.htm · A7 https://help.graphisoft.com/AC/29/INT/_AC29_Help/150_UserInterfaceToolSettings/150_UserInterfaceToolSettings-2.htm · A8 https://help.graphisoft.com/AC/28/INT/_AC28_Help/045_PropertiesClassifications/045_PropertiesClassifications-24.htm · A9 https://help.graphisoft.com/AC/26/GER/_AC26_Help/045_PropertiesClassifications/045_PropertiesClassifications-1.htm · A10 https://help.graphisoft.com/AC/20/INT/AC20Help/03_1_Elements_Virtual_Building/03_1_Elements_Virtual_Building-234.htm · A11 https://help.graphisoft.com/AC/29/INT/_AC29_Help/055_InteractiveSchedule/055_InteractiveSchedule-26.htm · A12 https://help.graphisoft.com/AC/26/INT/_AC26_Help/020_Configuration/020_Configuration-53.htm · AK1 https://community.graphisoft.com/t5/Project-data-BIM/Save-and-Apply-Favorite-Settings/ta-p/303527 und https://community.graphisoft.com/t5/Project-data-BIM/Managing-Favourites/td-p/227565 · AK2 https://community.graphisoft.com/t5/Bibliotheken-und-Objekte/Favoriten-autom-aktualisieren-AC-25/m-p/559639 und https://community.graphisoft.com/t5/Alte-Wunschliste/Favoriten-als-Eigenschaft/m-p/482905 · AK3 https://community.graphisoft.com/t5/Projektdaten-BIM/DIN-276-Klassifizierung/m-p/550800 · AK4 https://community.graphisoft.com/t5/Alte-Wunschliste/Klassifizierung-bis-auf-Schichtebene/td-p/556372

Vectorworks: V1 https://app-help.vectorworks.net/2025/eng/VW2025_Guide/Symbols/Concept_Plug-in_object_styles.htm · V2 https://app-help.vectorworks.net/2026/eng/VW2026_Guide/EventDesign1/LED_wall_settings.htm · V2b https://app-help.vectorworks.net/2026/eng/VW2026_Guide/DoorWindow/Window_settings.htm · V3 https://app-help.vectorworks.net/2026/eng/VW2026_Guide/Walls/Creating_wall_styles.htm · V4 https://app-help.vectorworks.net/2025/eng/VW2025_Guide/Walls/Replacing_wall_styles_from_the_Object_Info_palette.htm · V4b https://app-help.vectorworks.net/2025/eng/VW2025_Guide/Walls/Editing_wall_styles.htm · V5 https://app-help.vectorworks.net/2026/eng/VW2026_Guide/Walls/Deleting_and_or_replacing_wall_styles_from_the_Resource_Manager.htm · V6 https://app-help.vectorworks.net/2026/eng/VW2026_Guide/Objects_edit1/The_Object_Info_palette.htm · V7 https://app-help.vectorworks.net/2026/eng/VW2026_Guide/ResourceManager/Resource%20Manager.htm · V7b https://app-help.vectorworks.net/2026/eng/VW2026_Guide/Workgroup/Referencing_resources.htm · V8 https://app-help.vectorworks.net/2026/eng/VW2026_Guide/DataManager/Using_the_Data_Manager.htm und https://app-help.vectorworks.net/2026/eng/VW2026_Guide/IFC/Viewing_and_editing_IFC_data.htm · V9 https://app-help.vectorworks.net/2026/eng/VW2026_Guide/Annotation2/Data_tag_settings.htm · VK1 https://forum.vectorworks.net/topic/101837-why-vectorworks-does-not-update-changes-if-i-make-a-change-in-wall-component-offset/ · VK2 https://forum.vectorworks.net/topic/75489-wall-heights/ · VK3 https://forum.vectorworks.net/index.php?/topic/86798-wall-components-and-wall-styles-eliminate-undesired-behavior/

Allplan: P1 https://help.allplan.com/Allplan/2026-1/1031/Allplan/46881.htm · P2 https://help.allplan.com/Allplan/2026-1/1031/Allplan/297107.htm · P3 https://help.allplan.com/Allplan/2026-1/1031/Allplan/295440.htm · P5 https://help.allplan.com/Allplan/2026-0/1031/Allplan/298882.htm · P6 https://help.allplan.com/Allplan/2021-0/1031/New/295387.htm · P7 https://help.allplan.com/Allplan/2026-1/1031/Allplan/423770.htm · P8 https://www.allplan.com/products/allplan-2026-features/modeling-management-collaboration/ · P9 https://help.allplan.com/Allplan/2017-1/1031/Documents/IBD_Handbuch.pdf · PK1 https://campus.allplan.com/de/forum/themen/topic/topics/cad-allgemein-1/cds-allplan-add-ons-fuer-allplan-2026-inkl-dark-mode.html

Neuere Werkzeuge und Figma: N1 https://help.qonic.com/documentation/qonic-dashboard/libraries · N2 https://help.qonic.com/documentation/qonic-dashboard/libraries/type-libraries.md · N3 https://aecmag.com/bim/qonic-edges-towards-bim-authoring · N4 https://help.snaptrude.com/en/articles/9091066-walls · N4b https://help.snaptrude.com/en/articles/10854844-central-library-creation-pt-3-wall-types-revit-import · N5 https://aecmag.com/bim/arcol-unleashed-bim-2-0/ · N6 https://www.businesswire.com/news/home/20250326740027/en · F1 https://help.figma.com/hc/en-us/articles/360039150733-Apply-changes-to-instances · F2 https://help.figma.com/hc/en-us/articles/5579474826519-Explore-component-properties · F3 https://help.figma.com/hc/en-us/articles/360039234193-Review-and-accept-library-updates · F4 https://help.figma.com/hc/en-us/articles/360025508373-Publish-styles-and-components

Lücken: Archicad (Anzeige bei Mehrfachauswahl, Wirkung von Attributänderungen, Beschriftungen), Vectorworks (Suche, Formeln, Bereinigen, Listen), Allplan (Bereinigen, Beschriftung, Forenkritik), Arcol und Motif ohne Doku zu Typen, DIN 276 bei Revit und Vectorworks.
