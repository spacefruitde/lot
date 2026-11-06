# Typen und Eigenschaften: zum Ausprobieren in der App

28.10.2026. Etwa 20 Minuten **am Mac**, in der App. Umsetzung von ADR 0048. Branch `bedienung-typen`, baut auf `bedienung-waende` auf.

## So probierst du es aus

App bauen und starten wie in `CLAUDE.md` (Xcode ⌘R), Testprojekt `testprojekte/efh` öffnen (⇧⌘O) oder im leeren Projekt mit **WR** ein paar Wände ziehen.

**Typ und Bauteil in einem Inspektor** (eine Wand wählen)
- Oben der **Typ**: Auswahl zum Tauschen, Name, Kennzeichen, IFC-Klasse, Brandschutz, Schallschutz und die Schichten. Die Überschrift sagt, worauf eine Änderung wirkt („Wandaufbau · wirkt auf 14 Wände“). Werte tippen und ⏎, ohne Dialog.
- Das **Schloss** neben einem Typwert legt ihn fest: Keine Wand darf mehr abweichen, bestehende Abweichungen gehen auf den Typwert zurück.
- Darunter **Werte dieses Bauteils**: grau vom Typ, orange abweichend. Ein eigener Wert macht ihn abweichend, ⟲ setzt zurück, der Pfeil nach oben übernimmt den Wert in den Typ (dann gilt er für alle).

**Tür- und Fenstertypen** (Fenster wählen)
- Ohne Typ: Typ aus der Liste wählen oder „Typ aus dieser Öffnung anlegen“ (etwa „Fenster 1,26 × 1,385“, Kennzeichen F1, mit Rohbaumaß).
- Der Typ gibt das **Rohbaumaß** vor. Änderst du die Breite am Typ, folgen alle Fenster des Typs. Änderst du sie an einem Fenster, bestimmt dieses Fenster sie danach selbst (orange), außer der Typ hat sie festgelegt.
- Die **Tür- und Fensterliste** übernimmt das Kennzeichen des Typs.

**Typen-Übersicht** (⇧⌘T, Menü Werkzeug oder Knopf „Typen“)
- Alle Typen mit Schichtvorschau, Kennzeichen und Nutzung, Schalter „Nur unbenutzte“. Im Menü je Typ: kopieren, in die Bibliothek des Büros, tauschen gegen, löschen. Löschen geht nur mit Ersatz, außer der Typ ist unbenutzt.
- Reiter **Bibliothek des Büros**: Typen einzeln holen. Gibt es im Projekt einen Typ gleichen Namens, der abweicht, zeigt Lot die Unterschiede („Schicht 2: Dämmung 16 cm → 18 cm“) und bietet „Ersetzen, wirkt auf 14 Wände“ oder „Als neuen Typ“.
- Die Bibliothek liegt neben dem Bürostandard in `~/Library/Application Support/Lot/bibliothek.ron` (Menü Prüfen, „Bibliothek des Büros bearbeiten …“). Ohne eigene gilt eine eingebaute als Vorschlag: `core/crates/lot-edit/src/bibliothek.ron`.

## Fragen

1. **Typwerte direkt ändern:** ohne Rückfrage, die Überschrift sagt „wirkt auf 14 Wände“, ⌘Z nimmt es zurück. Ok, oder ab einer Anzahl nachfragen?
2. **Grau und orange** für „vom Typ“ und „weicht ab“: klar genug?
3. **Schloss:** Festlegen setzt alle Abweichungen ohne Rückfrage zurück. Ok?
4. **Rohbaumaß bei Türen und Fenstern:** am Typ, an einer Öffnung abweichend möglich, solange der Typ es nicht festlegt. Streng wie Revit wäre: nur am Typ. Vorschlag: so wie jetzt. Ok?
5. **Tür- und Fensterliste:** Öffnungen mit Typ behalten dessen Kennzeichen, die übrigen nummeriert Lot nach Größe mit den nächsten freien Kennzeichen. Ok?
6. **Eingebaute Bibliothek** (Vorschlag): 9 Wandaufbauten, 3 Decken- und Dachaufbauten, 5 Tür- und Fenstertypen nach DIN 18100. Was fehlt, was stimmt nicht? Am schnellsten: deine Typen im Projekt anlegen und „In die Bibliothek des Büros“ legen.
7. **Bibliothek verknüpfen:** Heute holt man Typen einmalig. Soll Lot sich merken, woher ein Typ kommt, und später melden, wenn er sich in der Bibliothek geändert hat?
8. **IFC-Klasse** ist freier Text (IfcWall, IfcWall.PARTITIONING). Vorschlag: eine Auswahl je Art, wenn der IFC-Export kommt. Ok?
9. **Brandschutz und Schallschutz** sind freier Text (F 90-A, REI 90, R'w = 53 dB). Reicht das für LP 1 bis 4, oder brauchst du eine Auswahl nach DIN 4102 und DIN EN 13501?
10. **Löschen:** nur mit Ersatz, Tür- und Fenstertypen auch ohne (die Öffnungen haben dann keinen Typ). Ok?

## Noch nicht gebaut

- Suche, Anheften und Tastatur im Inspektor (Vorschlag 3 aus Paket 2, nicht in ADR 0048).
- Mehrfachauswahl zeigt Typwerte nur, wenn alle denselben Typ haben.
- DIN-276-Kostengruppe (nach Version 1).
- Verknüpfte Bibliothek mit Update-Hinweis (Frage 7).

## Was ich nicht selbst prüfen konnte

Das Lot-Fenster lag wieder hinter deiner Vollbild-App. Typ-Logik, Bibliothek und Liste sind im Kern mit Tests abgesichert, die App baut; Inspektor, Typen-Übersicht und Bibliothek habe ich nicht selbst bedient.
