# Typen und Eigenschaften: Vergleich und Vorschlag

Quellen und Einzelheiten: `recherche-typen.md`. Was Lot heute kann: `lot-heute.md`.

## Kurzvergleich

| Thema | Revit | Archicad | Vectorworks | Allplan | Lot heute |
|---|---|---|---|---|---|
| Konzept | Familientyp, verknüpft | Favorit ist Vorlage; Aufbauten verknüpft | Stil, verknüpft | Stil (nur Wände), Favorit | Wand- und Deckenaufbau verknüpft |
| am einzelnen Bauteil überschreiben | nein, fest Typ oder Exemplar | jeder Wert | je Parameter nach Stil oder Exemplar | ja, `*` am Stilnamen | nein |
| Editor | modaler Dialog mit Vorschau | Dialog, Favoriten-Palette | Voreinstellungsdialog | Dialog | keiner, nur Projektdatei |
| Typ tauschen | Typauswahl, gleiche Kategorie | Favorit anwenden, Übertragungs-Set | Ersetzen mit Optionen | Stil zuweisen | Wandaufbau wählen |
| Bereinigen | ja | Purge, Löschen und ersetzen | unbelegt | unbelegt | nein |
| Übertragen | ganze Kategorie | Attributmanager nach Name oder Index | Import oder Referenz | Stilpfade Büro, Projekt | nein |
| Mehrfachauswahl | `<varies>` | unbelegt | leeres Feld | `*variiert*` | je Element ein Abschnitt |
| Suche in Eigenschaften | nein | nur Favoriten | unbelegt | unbelegt | nein |
| eigene Felder | Projekt- und gemeinsame Parameter | Eigenschaften nach Klassifizierung | Datensätze | Attribute | nein |

**Was Nutzer am meisten ärgert:** Revit erzwingt viele Typen, weil sich nichts am Exemplar überschreiben lässt, auch Klassifikationen nicht [K3, K4]; „Projektstandards übertragen“ nimmt nur ganze Kategorien [K2]; keine Suche nach Parametern [K1]; Archicads Favoriten aktualisieren platzierte Bauteile nicht [AK1, AK2]; Vectorworks sperrt Komponenten gegen spätere Stiländerungen, sobald man am Exemplar etwas ändert [VK1 bis VK3]; Archicad liefert keine DIN 276 [AK3].

## Vorschlag für Lot

1. **Typ und Bauteil in einem Inspektor, Herkunft je Feld sichtbar.** Jedes Feld zeigt, ob der Wert vom Typ kommt oder an diesem Bauteil überschrieben ist; Zurücksetzen je Feld, und „in den Typ übernehmen“. Typwerte ändert man direkt im Inspektor, mit dem Hinweis „wirkt auf 14 Wände“, ohne modalen Dialog. Vorbild Figma, besser als Revits Dialog und Allplans `*`, das nur sagt, dass etwas abweicht, nicht was.
2. **Der Typ legt fest, was am Bauteil überschreibbar ist.** Beim Wandaufbau sind die Schichten fest, Höhe und Bezugslinie gehören zur Wand; bei Türen sind Rohbaumaß und Art am Typ, Brüstung und Anschlag am Bauteil. So entstehen keine Typen für jede Kleinigkeit, und niemand ändert aus Versehen einen Aufbau.
3. **Inspektor mit Suche, Anheften und Tastatur.** Felder suchen, wichtige anheften, mit Tab durch die Felder. Bei Mehrfachauswahl „3 Werte: 11,5 / 17,5 / 24“ statt eines leeren Felds; ein Klick auf einen Wert wählt nur diese Bauteile.
4. **Typen-Übersicht als Tabelle.** Alle Typen mit Nutzungszahl, Vorschau, Umbenennen, Duplizieren, Tauschen und Filter „unbenutzt“; Löschen nur mit Ersatztyp. Typen eines Projekts gehören zum Projekt.
5. **Büro-Bibliothek mit Einzelauswahl.** Typen kommen einzeln aus der Bibliothek des Büros (neben dem Bürostandard, ADR 0040), mit Vorher-Nachher-Vergleich bei Konflikten und allen Abhängigkeiten (Materialien, Eigenschaften). Später auf Wunsch verknüpft, mit Hinweis auf Updates.
6. **Deutsche Klassifikation am Typ.** Kennzeichen, IFC-Klasse und später DIN-276-Kostengruppe sitzen am Typ und lassen sich am Bauteil überschreiben; Positionsnummern nach Regeln (wie in der Tür- und Fensterliste aus M5).
