# Recherche: Gauben

Anhang zu `gauben.md` (ADR 0051, Folgen). Etwa 58 Seitenaufrufe. Hilfen: Revit 2026, Archicad 28, Vectorworks 2026, Allplan 2026; die Gaubendachhaut in Allplan nur aus der Hilfe 2020 [36]. Die Foren von Autodesk, Graphisoft, Vectorworks und Allplan sperren direkte Abrufe (403); alle Forenaussagen ([5] bis [13], [18], [22] bis [29], [31] bis [33], [38] bis [41]) stammen deshalb aus Suchauszügen. Ebenso [3], [4], [14] bis [16]. Nummern in eckigen Klammern verweisen auf die Quellen am Ende.

## Begriffe

- Grundformen: Fledermaus-, Schlepp-, Giebel- (Satteldach-), Walm- und Dreiecksgaube; die Namen schwanken je Region [1].

## Revit 2026

- **Kein Gaubenbauteil.** Erst Wände und ein eigenes Dach bauen, dann das Gaubendach mit „Dach verbinden“ an das Hauptdach hängen, ausdrücklich nicht mit „Geometrie verbinden“ [2].
- **Ausschnitt von Hand:** Werkzeug „Gaubenöffnung“, Hauptdach wählen, dann die Ränder picken: verbundenes Dach oder seine Unterseite, Seitenflächen von Wänden, Deckenunterseite, Dachkante oder Modelllinie auf dem Dach [2].
- **Kleine Gaube ohne Wangen** über Neigungspfeile in der Dachskizze [2].
- **Wangen:** Fuß der Wände an das Hauptdach fixieren [3]; eine Giebelwand der Gaube geht oft nur, wenn man sie teilt: unten ans Hauptdach, oben ans Gaubendach [9].
- **Formen:** Schlepp, Sattel, Walm, Flach von Hand aus Dächern. Fledermaus- und Tonnengaube („brow“, „eyebrow“) über Extrusionsdach, Hilfswände und „Dach verbinden“, laut Anleitung „nicht einfach“ [4]; ein gebogenes Extrusionsdach ließ sich gar nicht verbinden [5].
- **Bestand:** Gaubenöffnung und die übrigen Öffnungswerkzeuge sind Geometrie-Modifikatoren ohne Phase. Eine neue Gaube schneidet das Bestandsdach schon im Bestand; Ausweg ist ein Abzugskörper in der Phase Neubau [10].

**Was Nutzer ärgert:**
- „Dach kann nicht verbunden werden“: tritt auf, wenn ein Streifen der Gaube unter dem Hauptdach liegt; kleine Änderungen an Höhe oder Öffnung schalten die Verbindung an und aus; ein Moderator rät, die Datei an den Support zu schicken [6, 7, 8].
- Wand und Dach: Teilen der Wand half, nach dem Verschieben der Wände kam der Fehler zurück [11].
- Idee „Revit - Impove Dormer“: bessere Gauben gewünscht; Ausweg: Wände vom Dach lösen und das Dach mit einer Abzugskörper-Familie schneiden, andere bauen eigene Gaubenfamilien [12].
- Gauben in Gruppen funktionieren nicht richtig [13].

## Archicad 28

- **Kein Gaubenwerkzeug.** Zwei Wege:
  - **Bibliotheksobjekte** „Dormer Shed“ (Schleppgaube, Neigung automatisch, Rinne, Zahl der Fenster) [14], „Dormer Pitched“ (Satteldach) [15], „Dormer Ox-Eye“ (Ochsenauge) [16]. Mit dem Dachflächenfenster-Werkzeug gesetzt, sitzen sie im Dach, lesen die Neigung und schneiden das Loch [17, 18]; beim Verschieben zwischen Dachflächen passen sie sich an [17].
  - **Von Hand** aus Dächern, Wänden, Fenstern: Bauteile an Dach oder Schale trimmen ist assoziativ, „a change to any element will immediately affect all the connected elements“; aneinander getrimmte Dächer wirken wie ein Körper [19]; man wählt, welcher Teil bleibt [20]. Seit Version 28 Öffnungen in jedem Dach, im Grundriss projiziert [21].

**Was Nutzer ärgert:**
- Dormer Shed: Neigung fest bei 35° (Archicad 25 bis 29), mit dem Dachfenster-Werkzeug 5° unter der Dachneigung; ein Moderator nutzt die Gaubenobjekte seit über 25 Jahren nicht und baut von Hand oder mit Objekten von Drittanbietern [18].
- Objekt „more trouble than it's worth“, Länge des Schleppdachs nicht einstellbar [22, 23]; das Objekt schneidet nur ein Rechteck aus [24].
- Standardgauben setzen die Unterkante auf das Dach, das stört bei bodentiefen Fenstern; das Gaubendach passt man per Strg-Klick auf die Kante ans Hauptdach an; viele bauen die Gaube selbst [25, 26].
- Grundriss: Gaubenwangen erscheinen voll; Nutzer teilen die Wange von Hand dort, wo die Dachfläche 1,00 m schneidet [27]; Wunsch nach richtiger Darstellung von Wänden unter Dachflächen [28].
- Raumfläche unter der Gaube stimmt nicht, wenn der Raum ans Dach angepasst wird; Ausweg Solid-Element-Operation [29].

## Vectorworks 2026

- **Gaube als Dachelement** im Dachobjekt oder in einer Dachfläche: ein Fenstersymbol aus dem Ressourcen-Manager aktivieren, in der Draufsicht ins Dach klicken, „Gaube bearbeiten“, Stil wählen [30].
- **Stile:** Trapez, Satteldach (Gable), Schlepp (Shed), Walm (Hip), Fledermaus (Bat) [30].
- **Parameter je Stil:** Breite, Höhe der Front oder Tiefe im Grundriss (eins ergibt das andere), Abstand von der Dachecke und von der Gebäudelinie, Höhenversatz, Neigung, Überstand (Sattel, Schlepp, Walm), Neigung der Walmfläche vorn, obere und untere Breite, Neigung der Frontkanten links und rechts (auch asymmetrisch); Fledermaus mit unterer Höhe und Kontrollpunkt der Kurven; Fenster mittig oder mit Abstand von oben [30].
- **Aufbau:** Wand- und Dachdicke nur für die Gaube, oder „Use Style For Dormer“ übernimmt den Aufbau des Hauptdachs [30].
- **Anschluss:** „A hole is automatically created in the roof where the dormer walls exist“ [30].
- **Bearbeiten:** Dach wählen, Griff der Gaube anklicken, Dialog [30]. Dachfenster gleich, mit Lochschnitt senkrecht, schräg oder rechtwinklig [30].

**Was Nutzer ärgert:**
- Das Symbol muss aus dem Ressourcen-Manager kommen; Steuerung „rather confusing“, Dialog ohne Live-Vorschau, Bezugspunkte unklar; Ausweg: Gaube von Hand aus Wänden und Dächern, oder Fensterobjekt mit „Use Symbol Geometry“, damit das Fenster in Listen erscheint [31].
- Gauben verschwinden beim Drucken und im PDF aus schattierten Ansichtsbereichen [32].
- Vectorworks 2026: ein Symbol im Dach wird immer Gaube, nie Dachfenster [33].

## Allplan 2026

- **Gaube in Dachebene:** in den Dachebenenkörper klicken, Gaubenumriss als Polygon, Höhen einstellen, Dachprofil wählen (Satteldach, Pultdach, Mansarddach, Tonnendach), die Traufseiten des Gaubenkörpers anklicken; „Durch Punkt“ setzt eine Höhe. Danach wird der Dachebenenkörper neu berechnet und ebenenbezogene Bauteile passen sich an [34].
- **Dachhaut** „in einem Zug auch über Dachlandschaften mit Gauben“; First-, Grat-, Kehl-, Ortgang- und Trauflänge in Beschriftung, Mengen und Listen auswertbar, auch für AVA [35]. Für die Gaube Eingabeart „Gaube“, Umriss, Überstand je Punkt [36]. Soll die Dachhaut der Gaube einen Überstand haben, muss der Gaubenkörper ganz im Hauptdachkörper liegen [34].
- **Add-in „Allplan Dach“** (Holzbau): Satteldach-, Walmdach-, Schlepp-, Trapez- und Spitzgaube; beim Absetzen entstehen Auswechslungen, Pfosten und Brüstungshölzer; eine neue Dachneigung wirkt auch auf Gauben [37].

**Was Nutzer ärgert:**
- Gaubenwände an die falsche Dachebene gebunden, nichts passt sich an [38].
- Gaubenwange stanzt den Raum aus (Option Unterkante) [39].
- Dachhaut der Gaube schwebt, Übergänge an Kehlen „nicht ganz vermeidbar“; Ausweg: erst Hauptdach, dann Gauben [40].
- Dachlandschaft muss nach jeder neuen Gaube neu erzeugt werden [41].

## Holzbau-CAD als Maßstab (Dietrich's)

- Typ aus einer Bildliste, dann Vorlage einer gespeicherten Gaube, dann Dialog mit den Werten [42].
- Formen: Giebel-, Spitz-, Dreiecks-, Walm-, Schlepp-, Trapez-, Rundbogengaube [43].
- Werte sind alternativ: zwei oder drei eingeben, den Rest rechnet das Programm; Brüstung mit Oberkante, Überstand über der Deckung und Versatz zur Vorderkante; Lage in der Draufsicht, wahlweise nur parallel zur Traufe verschiebbar [43].

## Lot heute (aus dem Code)

- **Modell** (`core/crates/lot-model/src/roof.rs`): `Dormer` als Schleppgaube in der Fläche einer Traufe: Mitte, Breite, Abstand der Vorderwand vom Umriss, Traufhöhe vorn, Neigung, ein Überstand für vorn und seitlich, Wandaufbau, wahlweise ein mittiges Fenster (`DormerWindow`). Wo das Gaubendach das Hauptdach trifft, wird abgeleitet. Neigung 0° ergibt eine Flachdachgaube ohne Attika.
- **Geometrie** (`lot-geom/src/dormers.rs`): Dachschichten mit dem Aufbau des Hauptdachs, Wangen und Vorderwand schichtweise vom Hauptdach bis unter das Gaubendach, Fenster mit Rahmen und Glas; das Hauptdach ist zwischen den Wangen ausgeschnitten; Wände „bis Dach“ reichen unter das Gaubendach.
- **Pläne und Nachweise:** Der Grundriss schneidet die Gaube wie jedes Bauteil, der Umriss ist gestrichelt; der Schnitt zeigt die Schichten; die Wohnfläche zählt die lichte Höhe unter der Gaube; die Abstandsflächen rechnen die Gaube mit (`TopProbe`).
- **Fehlt:** weitere Formen; Werkzeug und Griffe in der App; Bauzustand (Gaube und Dachfenster haben keinen); Gauben- und Dachfenster in der Tür- und Fensterliste; eigener Dachaufbau der Gaube; getrennte Überstände; Vorderwand in der Flucht der Außenwand (ADR 0029).

## Quellen

Begriffe: 1 https://www.baunetzwissen.de/glossar/g/gauben-49031

Revit: 2 https://help.autodesk.com/cloudhelp/2026/ENU/Revit-ArchDesign/files/GUID-E715D7B4-5C53-4DD9-8F03-FB35981A797E.htm · 3 https://emea.support.arkance.world/hc/fr/articles/36470348442514-Revit-Dormer-Variation-1 · 4 https://ukcommunity.arkance.world/hc/en-us/articles/21565451620754-Creating-a-Brow-Dormer-in-Revit-2017 · 5 https://forums.augi.com/archive/index.php/t-104150.html · 6 https://forums.autodesk.com/t5/revit-architecture-forum/gable-roof-quot-cannot-join-roof-quot-plague/td-p/10791319 · 7 https://forums.autodesk.com/t5/revit-architecture-forum/can-t-attach-dormers-to-main-roof/td-p/11345557 · 8 https://forums.autodesk.com/t5/revit-architecture-forum/roof-unable-to-join-misses-target/td-p/8852804 · 9 https://forums.autodesk.com/t5/revit-architecture-forum/gable-wall-will-not-attach-to-dormer-roof/td-p/8090411 · 10 https://forums.autodesk.com/t5/revit-architecture-forum/demolish-existing-roof-with-new-dormer-in-new-construction/td-p/10619217 · 11 https://forums.autodesk.com/t5/revit-architecture-forum/roof-wall-join-one-side-roof-misses/m-p/7688786 · 12 https://forums.autodesk.com/t5/revit-ideas/revit-impove-dormer/idi-p/10990809 · 13 https://forums.autodesk.com/t5/revit-ideas/revit-bug-when-using-dormers-in-groups/idi-p/12844438

Archicad: 14 https://beta.bimcomponents.com/GSM/Details/16412 · 15 https://bimcomponents.com/GSM/Details/16411 · 16 https://bimcomponents.com/GSM/Details/16410 · 17 https://help.graphisoft.com/AC/28/INT/_AC28_Help/040_ElementsVB/040_ElementsVB-318.htm · 18 https://community.graphisoft.com/t5/Libraries-objects/Lifting-the-limitations-of-Dormer-Shed-library/m-p/703464 · 19 https://help.graphisoft.com/AC/28/INT/_AC28_Help/040_ElementsVB/040_ElementsVB-90.htm · 20 https://help.graphisoft.com/AC/28/INT/_AC28_Help/040_ElementsVB/040_ElementsVB-95.htm · 21 https://help.graphisoft.com/AC/28/INT/_AC28_Help/005_NewFeatures/005_NewFeatures-4.htm · 22 https://community.graphisoft.com/t5/Modeling/Dormer-Shed-Roof/td-p/17036 · 23 https://community.graphisoft.com/t5/Libraries-objects/shed-dormer-object/td-p/271879 · 24 https://community.graphisoft.com/t5/Libraries-objects/Skylights-making-the-hole-in-the-roof/td-p/5030 · 25 https://community.graphisoft.com/t5/Modellierung/Dachgauben-erstellen/m-p/565614 · 26 https://community.graphisoft.com/t5/Modellierung/Dachgauben/td-p/486998 · 27 https://community.graphisoft.com/t5/Modellierung/Darstellung-Gaubenseitenw%C3%A4nde/td-p/489961 · 28 https://community.graphisoft.com/t5/Wunschliste/Korrekte-Grundrissdarstellung-von-W%C3%A4nden-unter-Dachfl%C3%A4chen/idi-p/603902 · 29 https://community.graphisoft.com/t5/Modellierung/Raumfl%C3%A4che-unter-Dachgaube-stimmt-nicht/td-p/497809

Vectorworks: 30 https://app-help.vectorworks.net/2026/eng/VW2026_Guide/Roofs/Adding_roof_elements_to_roof_objects_and_roof_faces.htm · 31 https://forum.vectorworks.net/topic/71200-roof-dormers/ · 32 https://forum.vectorworks.net/index.php?/topic/115929-roof-dormers-dont-print-in-top-view-pdf-or-printing-to-plotter/ · 33 https://forum.vectorworks.net/topic/111707-cant-insert-skylight/

Allplan: 34 https://help.allplan.com/Allplan/2026-0/1031/Allplan/6495.htm · 35 https://help.allplan.com/Allplan/2026-0/1031/Allplan/6496.htm · 36 https://help.allplan.com/Allplan/2020-1/1033/Allplan/9858.htm · 37 https://weto-nord.de/pdf/produkte/allplan-dach_2024.pdf · 38 https://campus.allplan.com/de/forum/themen/topic/topics/cad-architektur/anzeigefehler-bei-gaubenwaende-3d-cad.html · 39 https://campus.allplan.com/de/forum/themen/topic/topics/cad-architektur/gaubenwange-stanzt-raum-aus.html · 40 https://campus.allplan.com/de/forum/themen/topic/topics/cad-allgemein/dachhaeute-der-gaubendaecher.html · 41 https://connect.allplan.com/de/forum/themen/topic/topics/cad-architektur/dachebene-es-laesst-sich-nur-eine-gaube-einfuegen.html

Holzbau: 42 https://docs.dietrichs.com/konstruktionsprogramm/modellbereiche/gaube/neue-gaube · 43 https://docs.dietrichs.com/docs-de-en/design-program/modellbereiche/gaube/gaubenausmittlung

Lücken: Revit Grundriss und Mengen der Gaube nicht eigens belegt (Teile bleiben Wände und Dächer); Archicad Walm-, Fledermaus- und Tonnengaube nur von Hand, nicht belegt; Vectorworks Darstellung in Grundriss und Schnitt nicht in der Hilfe; Allplan Walm- und Spitzgaube nur im Add-in, Fledermausgaube und Ochsenauge nirgends belegt; Flachdachgaube in keiner Hilfe eigens genannt.
