# Planerstellung und Planköpfe: Vergleich und Vorschlag

Quellen und Einzelheiten: `recherche-planerstellung.md`. Was Lot heute kann: `lot-heute.md`.

## Kurzvergleich

| Thema | Revit | Archicad | Vectorworks | Allplan | Lot heute |
|---|---|---|---|---|---|
| Plankopf | Familie mit gemeinsamen Parametern | 2D mit Autotext auf dem Master | Planrahmen-Objekt als Stil | Legende aus Plan- und Projektattributen | fest im Kern |
| kennt Maßstab, Format, Inhalt | nein, nur über Parameter | nein, nur über Zeichnungstitel | Maßstab nein | ja, über Attribute | ja |
| Format wechseln | Familie tauschen | Master tauschen | Rahmen ändern | Rahmen und Kopf passen sich an | Feld in der Datei |
| gleiche Lage über Blätter | erst seit 2026 (Anker) | über den Master | zentriert verteilen | unbelegt | Koordinate je Ansicht |
| Revisionen | je Plan anhaken | aus Modelländerungen, Sätze sperren | Title Block Manager | Indextabelle | ein Textfeld |
| Ausgabe | PDF-Satz mit Namensregel | Publisher-Sätze | „Veröffentlichen“ | Stapeldruck, langsam | Kommandozeile |
| Faltmarken DIN 824 | unbelegt | Büros zeichnen sie selbst | unbelegt | unbelegt | nein |
| Bauantrag-Plankopf mit Unterschriften | nicht gefunden | nicht gefunden | nicht gefunden | nicht gefunden | ja (M5) |

**Was Nutzer am meisten ärgert:** In Revit hakt man eine Revision bei 50 Plänen 50 Mal an [44], unfertige Revisionen stehen schon im Plankopf [45], eigene Felder zeigen „?“ und das Format erscheint nicht von selbst [47]; Archicads Master kennt die platzierte Zeichnung nicht, Autotexte fehlen im DWG-Export seit über zehn Jahren [51, 52]; Vectorworks hat keine zentrale Stelle für Revisionen [56]; DIN-Rahmen falten falsch oder Faltmarken fehlen [53]; Allplan braucht 15 Minuten für 12 PDFs [61].

**Wo Lot schon vorn ist:** Der Plankopf aus M5 liest Projektangaben, Grundstück, Entwurfsverfasser, Maßstab und Format selbst und hat die Unterschriftszeile für den Bauantrag; ein Grundriss darf auf mehreren Blättern liegen.

## Vorschlag für Lot

1. **Planblatt in der App:** Ansichten aus der Seitenleiste aufs Blatt ziehen (ADR 0049); Geschosspläne liegen ab Werk auf allen Blättern deckungsgleich, Lage über einen gemeinsamen Anker. Maßstabsleiste, Nordpfeil und Zeichnungstitel kommen mit der Ansicht.
2. **Plankopf als Baustein im Bürostandard, direkt auf dem Blatt bearbeitet:** Felder lesen Maßstab, Format, Planinhalt, Index und Planstand vom Blatt, nie „?“. Ein Büro-Plankopf mit Logo und eigenen Feldern; daneben die Bauantrag-Variante mit Unterschriftsfeldern für Bauherr und Entwurfsverfasser. Ändern wirkt auf alle Blätter.
3. **Formate und Faltung automatisch:** DIN-Formate und freie Formate (Länge frei, Höhe aus der DIN-Reihe), Rahmen, Heftrand und Faltmarken nach DIN 824 wachsen mit dem Format mit.
4. **„Planstand ausgeben“ in einem Schritt:** Index vergeben, Blätter einfrieren, PDF-Satz mit Dateinamen nach Büroschema schreiben, Planliste und Indexverlauf mitführen. Ändert sich das Modell danach, markiert Lot die betroffenen Blätter („Plan 102: geändert seit Index b“) und schlägt den nächsten Index vor.
5. **Ausgabe schnell und im Hintergrund:** PDF, DWG mit einem Layout je Blatt, Planliste als PDF und CSV; der Satz läuft, während man weiterarbeitet.
