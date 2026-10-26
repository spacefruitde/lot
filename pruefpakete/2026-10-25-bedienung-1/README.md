# Paket Bedienung 1: Auswählen, Bearbeiten, Wände

25.10.2026. Etwa 20 Minuten, nur lesen und antworten, kein Mac nötig. Grundlage ist deine Entscheidung vom 25.10.2026 (ADR 0042): Meilensteine anhalten, je Thema vergleichen, wie etablierte BIM-Programme es lösen, und dann entscheiden, wie Lot es macht, mindestens gleich gut oder besser.

## Was drin ist

- **`auswaehlen.md`**, **`bearbeiten.md`**, **`waende.md`**: je Thema ein Kurzvergleich (Revit, Archicad, Vectorworks, Allplan, Lot heute), was Nutzer dort am meisten ärgert, und mein Vorschlag für Lot. Je eine Seite, die lohnt sich zu lesen.
- `lot-heute.md`: was Lot bei diesen Themen heute kann, aus dem Code gelesen. Ehrlich: wenig.
- `recherche-auswaehlen.md`, `recherche-bearbeiten.md`, `recherche-waende.md`: die Recherche mit Quellen (Herstellerhilfen 2026/2027, Revit Ideas, Archicad Community, Vectorworks Forum, Allplan Campus, dazu Qonic, Snaptrude, Arcol, Motif, SketchUp, Rhino, Figma). Nur zum Nachschlagen.

Revit und Archicad habe ich nicht selbst bedient; alles stammt aus den Hilfen und Foren. Wo dein Alltag etwas anderes sagt, gilt dein Alltag.

## Leitlinie hinter den Vorschlägen

1. Wo Revit gut ist, nimmt Lot es so, damit dein Muskelgedächtnis passt (Kürzel, Rahmenrichtung, Tab, temporäre Maße).
2. Wo Nutzer belegt leiden, macht Lot es anders (versehentliches Verschieben, Wände springen, Anschlüsse reißen ab, kein Lasso, kein „verschieden“).
3. Wo der Mac eine Konvention hat, gilt sie (⇧ zum Hinzufügen, ⌥ zum Kopieren beim Ziehen, ⌘Z).
4. Lots Stärke bleibt: Wandanschlüsse ohne Einstellungen nach Schichtrang. Daran scheitern die anderen am häufigsten.

## Fragen

Antwort jeweils „Vorschlag ok“, „wie Revit“, „wie Archicad“ oder ein Satz, was anders sein soll.

**Auswählen** (`auswaehlen.md`)

1. **Ziehen und Rahmen:** Ziehen auf ein nicht gewähltes Bauteil zieht einen Rahmen auf und verschiebt nichts; verschoben wird nur die Auswahl. Rahmen links nach rechts wählt, was ganz innen liegt, rechts nach links auch Angeschnittenes (wie Revit); mit ⌥ Lasso; ⇧ fügt hinzu oder nimmt heraus. Vorschlag ok?
2. **Tab und Kette:** Tab blättert durch alles unter dem Zeiger, die Wandkette ist ein Kandidat, am Zeiger steht „2 von 4: Wandkette (3 Wände)“. Vorschlag ok?
3. **Gleiche, Teile, Sperren:** SA wählt den gleichen Typ (zweimal: im ganzen Projekt), der Inspektor filtert nach Art und Typ; Doppelklick wählt ein Teil (Schicht, Öffnung, Anschlussknoten); Unterlagen nie wählbar, aber fangbar. Fehlt dir etwas aus Revit, das du täglich nutzt?

**Bearbeiten** (`bearbeiten.md`)

4. **Kürzel:** Revits Zwei-Buchstaben-Kürzel als Voreinstellung (MV, CO, RO, MM, AR, OF, AL, TR, SL, WA, DR, WN, RM, DI) statt eigener Ein-Tasten-Kürzel? Vorschlag: ja, wegen deines Muskelgedächtnisses. Welche nutzt du am meisten?
5. **Zahleneingabe:** In jeder laufenden Aktion einfach tippen, ohne Klick ins Feld; Einheiten, Rechnen („2,40+0,365“), relativ und absolut, „3x“ für Reihen; korrigierbar bis zur nächsten Aktion. Vorschlag ok?
6. **Temporäre Maße:** wie Revit (anklicken, Wert tippen, Bauteil rückt), aber auch bei Mehrfachauswahl und mit EQ; keine Schlösser und Zwangsbedingungen in V1. Vorschlag ok, oder brauchst du Schlösser?
7. **Am Element:** ⌥ beim Ziehen kopiert, Pfeiltasten schieben 1 cm (⇧ 10 cm), Anschlüsse bleiben beim Verschieben und Strecken erhalten, Mehrfachauswahl zeigt „verschieden“ mit Spanne und nimmt „+10 cm“ für alle. Vorschlag ok?

**Wände** (`waende.md`)

8. **Bezugslinie:** sechs Lagen (Mitte, Außenfläche, Innenfläche, Kernmitte, Kern außen, Kern innen); ein Wechsel verschiebt nie etwas; Umdrehen tauscht innen und außen, die Wand bleibt liegen. Vorschlag ok?
9. **Anschlüsse:** bleiben automatisch nach Schichtrang; jeder Stoß ist ein auswählbarer Knoten mit Erklärung („Mauerwerk schneidet Dämmung“) und Ausnahmen (welche Wand durchläuft, nicht verbinden), für beliebig viele Wände; verschiedene Bauzustände verbinden sich nicht. Brauchst du zusätzlich Revits Anschlusstypen (Gehrung, stumpf, rechtwinklig)?
10. **Höhe:** Oberkante an Geschoss mit Versatz, an die Decke darüber, ans Dach oder fest. Später je Schicht. Wie führst du im 1:50 die Schichten am Deckenanschluss (etwa Mauerwerk bis Unterkante Rohdecke, Putz bis Unterkante Decke, WDVS durchlaufend)? Vorschlag: so als Voreinstellung je Schichtfunktion.

## Was mir am meisten hilft

Deine Revit-Liste (PLAN.md, offene Frage 1): was dich in Revit täglich nervt und was Revit richtig gut kann, dazu drei bis fünf Dinge, die dir beim Ausprobieren von Lot am meisten gefehlt haben. Stichworte reichen, gern als Sprachnotiz.

## Wie es weitergeht

Deine Antworten werden ADRs. Dann baue ich Auswählen, Bearbeiten und Wände nach diesen Entscheidungen und teste die App selbst per Bildschirmsteuerung. Die nächsten Themen für den Vergleich: Zeichnen mit Zahleneingabe und Fangen, Öffnungen, Typen und Eigenschaften, Navigation, Sichtbarkeit, Decken, Dächer und Treppen in der App.
