# 0046 Zeichnen mit Zahleneingabe und Fangen

Status: entschieden von Ben · 26.10.2026; umgesetzt am 27.10.2026 bis auf die Punkte unter „Folgen“

## Kontext

Vergleich in `pruefpakete/2026-10-26-bedienung-2/zeichnen.md` (ADR 0042). Lot fängt heute Wandenden und Unterlagen, Zahlen nur über den Inspektor.

## Entscheidung

Ben am 26.10.2026: „11 ok“, zu den Fangkürzeln „12 die kürzel kenn ich nicht aber klingt gut - kannst du kurz erklären?“, zu Bezugspunkten und Richtung „13 klingt gut - gerne auch nochmal genauer erklären“, zum Skizzenmodus „14 haben wir gar keinen skizzenmodus für nichts oder was meinst du?“

- **Ein Eingabefeld am Zeiger:** „3,65“ Länge, „<30“ Winkel, „3,65<30“ beides, „@1,20;0,80“ relativ, „2*1,25+0,115“ gerechnet, „365cm“ mit Einheit. Der Winkel darf zuerst kommen, Tab springt zwischen Länge und Winkel.
- **Vorhersagbarer Fang:** Der Punkt unter dem Zeiger gewinnt, Art und Name stehen am Zeiger, Tab blättert. Fangkürzel wie Revit (SE, SM, SI, SP, SN, SO) gelten für einen Klick; sie sind ein Notausgang, nicht der Normalfall (Ben kannte sie nach 15 Jahren Revit nicht). Neu: Mitte, Schnittpunkt zweier Wände, lotrecht, Wandflächen. Die Fangeinstellungen gehören zum Projekt.
- **Bezugspunkte mit Q,** höchstens drei, nummeriert; Spurlinien waagerecht, senkrecht und im Winkelraster, ihr Schnitt ist fangbar; Esc löscht sie.
- **Zeichenrichtung von einer Kante** für schiefwinklige Grundrisse und Altbauten.
- **Kein Skizzenmodus.** Bauteile entstehen sofort (Kette, Rechteck, Polygon, Klick in eine Fläche, aus Linien einer Unterlage) und werden danach direkt am Bauteil bearbeitet.
- **Achtelmeter-Fang** (DIN 4172) als Schalter, ab Werk aus.
- **Nachfrage Ben zu 14:** „naja aber wie macht man dann zb ausschnitte oder entscheidet bis wo genau der fußboden geht in einer öffnung zb?“ Mein Vorschlag:
  - Löcher in Decken (Treppenloch, Schacht, Luftraum) sind eigene Bauteile „Durchbruch“: Rechteck, Polygon oder Klick in eine umschlossene Fläche, dann wie jedes Bauteil mit Griffen und temporären Maßen bearbeitet. Treppen schneiden ihr Loch selbst.
  - Kerben am Rand: Decke wählen, Kante oder Ecke ziehen, Doppelklick auf eine Kante setzt einen Punkt; ein Rechteck über den Rand zieht mit ⌥ ab und fügt mit ⇧ an.
  - Fußboden in der Öffnung als Eigenschaft der Öffnung: bis unter das Türblatt, bis Wandmitte oder durchlaufend von einem der beiden Räume; im Grundriss als Linie des Belagswechsels, ein Klick schaltet um. Voreinstellung „Wechsel unter dem Türblatt“ ist eine Frage fürs nächste Prüfpaket.

## Alternativen

- **Revits Skizzenmodus:** sammelt Fehler bis zum Haken und braucht zum Ändern einen eigenen Modus.
- **Verweil-Spuren wie AutoCAD:** Bezugspunkte entstehen unabsichtlich.

## Folgen

- Die Schreibweise der Zahleneingabe ist ein Parser im Kern (`lot-edit`), mit Tests je Form.
- Umgesetzt am 27.10.2026 (Branch `bedienung-auswaehlen`): Zahleneingabe, Fangpunkte mit Namen und Tab, SE bis SO, Bezugspunkte mit Spurlinien, Zeichenrichtung (ZR), Wandrechteck (WR). Offen: Achtelmeter-Fang, Wände aus Linien einer Unterlage, Polygon, Fang-Einstellungen im Projekt, Klick in eine Fläche (kommt mit den Decken).
