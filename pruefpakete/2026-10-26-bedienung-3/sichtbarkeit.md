# Sichtbarkeit: Vergleich und Vorschlag

Quellen und Einzelheiten: `recherche-sichtbarkeit.md`. Was Lot heute kann: `lot-heute.md`.

## Kurzvergleich

| Thema | Revit | Archicad | Vectorworks | Allplan | Lot heute |
|---|---|---|---|---|---|
| Ordnung | Kategorien | Ebenen, Kombinationen | Klassen, Konstruktionsebenen | Teilbilder, Layer | Bauteilart, Bürostandard |
| gespeichert | je Ansicht oder Vorlage | je Ansicht, sechs benannte Sätze | Zeichenfläche, Ansicht, Viewport | Layerstatus, Druckset, Planelement | Bürostandard je Maßstab |
| Filter nach Eigenschaft | ja, auch ausblenden | ja, ohne Ausblenden | Datenvisualisierung | ein/aus in einer Palette | nein |
| Detailgrad je Ansicht | grob, mittel, fein | Modelldarstellung | niedrig, mittel, hoch | Maßstab, Zeichnungstyp | aus dem Maßstab |
| Vorlagen | Ansichtsvorlage, verknüpft | Sätze je Ansicht | Viewport-Stil | Favorit, Druckset | nein |
| temporär | ausblenden, isolieren | Ebenen der Auswahl | Objekt ausblenden | Layer isolieren | nein |
| Bauzustand | Phasenfilter | Renovierungsfilter | Phasen (neu 2026) | Umbaukategorie | nur als Ausgabe |
| Geschoss darunter | Unterlage, Stil projektweit | Trace-Referenz | graue Ebenen | passive Teilbilder | nein |
| nur Bildschirm | Referenzebenen, Zuschnitt | Bildschirmfarben | geisterhaft Ausgeblendete | Hilfskonstruktion, Raster | Hinweise, Fangpunkte |

**Was Nutzer am meisten ärgert:** „Warum sehe ich das Bauteil nicht?“, in allen vier Foren; Revit hat dafür eine Checkliste mit rund 20 Prüfstellen [R3, F1, G4, W1, C1]; Vorlagen wirken nur ganz oder gar nicht [F2, F4]; Archicad kann per Regel nicht ausblenden, Nutzer legen doppelte Ebenen an [G1, G2]; neue Klassen sind in alten Ansichten unsichtbar [W1 bis W3].

**Was Lot schon anders macht:** Die Darstellung kommt aus dem Bürostandard und dem Maßstab, ohne Einstellung je Ansicht. Das vermeidet viel Verwirrung, ist aber zu starr, sobald eine Ansicht etwas anderes zeigen soll.

## Vorschlag für Lot

1. **Eine sichtbare Kette statt versteckter Rangfolge:** Bürostandard, dann Ansichtsvorlage, dann Ansicht, dann Bauteil. Der Inspektor zeigt bei jeder Darstellungseinstellung ihre Herkunft („aus Vorlage Grundriss 1:100“) und bietet Zurücksetzen an, wie bei Typen (Paket 2).
2. **„Warum sehe ich das nicht?“ als Funktion.** Klick auf einen Raum, eine Liste oder einen Befund: Lot nennt die Gründe (Bauzustand ausgeblendet, über der Schnitthöhe, anderes Geschoss, Regel X) und behebt jeden mit einem Klick.
3. **Vorlagen aus Bausteinen:** Eine Ansicht stapelt Bausteine (Maßstab mit Detailgrad, Bauzustand, Thema wie Brandschutz oder Flächen); jeder steuert nur seine Felder. Abweichen ist eine bewusste Aktion mit Schloss, nicht grau.
4. **Regeln dürfen ausblenden und sagen es:** Regeln nach Eigenschaft (Bauzustand, Aufbau, Feuerwiderstand) färben, grauen oder blenden aus; ein Zähler zeigt „12 Bauteile durch Regel ‚Abbruch ausblenden‘“.
5. **Bauzustand und Geschoss darunter im Kopf jeder Ansicht:** Umbauplan, Bestand, Neuzustand als Schalter (heute nur als Ausgabe); das Geschoss darunter oder darüber als Unterlage mit eigenem Stil je Ansicht.
6. **Temporär auffällig, Druck ehrlich:** Ausblenden und Isolieren auf einer Taste (H wie Snaptrude, I), Rahmen und Zähler zeigen den Zustand, nie im Druck. Ein Schalter „Druckbild“ zeigt genau, was gedruckt wird, ohne Hinweise und Fangpunkte.
