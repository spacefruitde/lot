# Auswählen und Bearbeiten: zum Ausprobieren in der App

26.10.2026. Etwa 20 Minuten **am Mac**, in der App. Die ersten Bauschritte nach deinen Antworten (ADR 0043, 0044, 0046). Branch `bedienung-auswaehlen`.

## So probierst du es aus

App bauen und starten wie in `CLAUDE.md` (Xcode ⌘R), dann Ablage, Testprojekt öffnen (⇧⌘O), `testprojekte/efh` wählen. Oder im leeren Projekt mit **WA** ein paar Wände zeichnen.

**Auswählen**
- Klick wählt. **Ziehen im Leeren oder auf einem nicht gewählten Bauteil zieht einen Rahmen**: nach rechts nur, was ganz innen liegt (durchgezogen, blau), nach links auch Angeschnittenes (gestrichelt, grün). Mit **⌥** wird es ein Lasso, mit **⇧** fügst du hinzu oder nimmst heraus.
- Über einem Bauteil steht am Zeiger, was ein Klick wählt („Wand · Außenwand WDVS 36 · 6,00 m“). **⇥** blättert durch alles darunter, auch die Wandkette („2 von 2: Wandkette (4 Wände)“).
- **SA** wählt gleiche Bauteile im Geschoss, ein zweites **SA** im ganzen Projekt. Der Inspektor zeigt die Gruppen mit Anzahl, „nur diese“ und „ohne“.
- **⌘←** holt die vorige Auswahl zurück. Unterlagen lassen sich nicht anklicken; „Unterlage wählen“ steht im Menü Werkzeug.

**Bearbeiten**
- Gewähltes ziehen verschiebt, **mit ⌥ entsteht eine Kopie** samt Fenstern und Türen. **Pfeiltasten** schieben 1 cm, mit ⇧ 10 cm. **Leertaste** dreht Wände um und Türen durch alle vier Lagen.
- **Temporäre Maße:** Eine Wand, Stütze oder Öffnung wählen, die lichten Abstände zur nächsten Wandfläche erscheinen. **Maßzahl anklicken, neuen Wert tippen, ⏎**: das Bauteil rückt, Anschlüsse ziehen mit.
- **Kürzel wie Revit:** MV, CO, RO, MM, AR, OF, AL, TR, SL, dazu WA, DR, WN, RM, GR, CL, DI, LI, TX. **⏎** ohne laufende Aktion wiederholt den letzten Befehl.
- **Zahlen tippen ohne Feld:** beim Zeichnen und in MV, CO, AR, RO, OF einfach tippen: `3,65`, `365cm`, `2*1,25+0,115`, `<30` (Winkel zuerst, dann Länge), `3,65<30`, `@1,20;0,80`; bei Reihe `4x` (vier Stück im Abstand) oder `4/` (vier Stück bis zum Punkt).
- **Mehrfachauswahl im Inspektor:** gleiche Werte stehen da, sonst „verschieden“ mit Spanne; eine Zahl setzt alle, `+0,10` ändert alle relativ, ein Undo-Schritt.
- **⌘C, ⌘V:** an derselben Stelle ins aktuelle Geschoss einfügen, etwa Wände vom EG ins OG.

**Zeichnen und Fangen** (ADR 0046, seit 27.10.2026)
- Am Zeiger steht, was gefangen wird („Ende Wand“, „Mitte“, „Schnittpunkt“, „Ecke Wandfläche“, „lotrecht“, „Wandfläche“, „Ecke Decke“). **⇥** blättert durch alle Fangpunkte unter dem Zeiger („⇥ 2/5“), der Klick nimmt den gewählten.
- **SE, SM, SI, SP, SN, SO** gelten für einen Klick (nur Endpunkte, Mitten, Schnittpunkte, lotrecht, auf Linien, Fangen aus).
- **Q** setzt am Fangpunkt einen Bezugspunkt (bis zu drei, nummeriert). Von dort laufen Spurlinien waagerecht und senkrecht; am Zeiger steht „Spurlinie 1 · 1,20 m“. **Länge tippen und ⏎** setzt den Punkt auf der Spurlinie, `@0,50;0` setzt ihn relativ zum Bezugspunkt. Zwei Bezugspunkte: Der Schnitt ihrer Spurlinien ist fangbar. **Esc** löscht sie.
- **ZR**, dann Klick auf eine Wand, Achse oder Linie: Die Zeichenrichtung folgt ihr. Rechte Winkel (⇧), Winkelraster, `@x;y`, Pfeiltasten und **WR** (Wände als Rechteck) richten sich danach. ZR und Klick ins Leere setzt auf 0° zurück.

## Fragen

1. **Rahmen statt Verschieben:** Fühlt es sich richtig an, dass Ziehen auf ein nicht gewähltes Bauteil einen Rahmen aufzieht und erst Gewähltes sich verschieben lässt?
2. **Text am Zeiger und ⇥:** hilfreich oder störend? Soll er erst nach kurzem Verweilen erscheinen?
3. **Temporäre Maße:** Lot misst lichte Weite zwischen Wandflächen (Rohbau). Revit misst ab Werk Mitte zu Mitte. Vorschlag: lichte Weite ab Werk, ein Klick auf das Maßende wechselt später zu Achse oder Kern. Ok?
4. **Spiegeln als Kopie** (wie Revit), mit ⇧ ohne Kopie. Ok?
5. **Reihe zählt das Original mit** (wie Revit: `4x` ergibt vier Stück). Ok?
6. **Versetzen (OF)** braucht erst den getippten Abstand, dann den Klick auf die Wand, die Kopie entsteht zur Seite des Zeigers. Fehlt dir das grafische Versetzen mit zwei Klicks?
7. **Einfügen ⌘V** setzt an dieselbe Stelle ins aktuelle Geschoss. Brauchst du zusätzlich Revits „an ausgewählten Ebenen ausrichten“ für mehrere Geschosse auf einmal?
8. **Fangen:** Der nächste Punkt gewinnt, egal welcher Art. Bei gleichem Ort gilt Ende vor Mitte vor Schnittpunkt. Passt das, oder soll ein Wandende immer vor allem anderen kommen (wie AutoCAD)?
9. **Bezugspunkte:** Reichen waagerechte und senkrechte Spurlinien, oder brauchst du auch 45°?
10. **Was fehlt dir beim Auswählen, Bearbeiten und Fangen am meisten**, wenn du es ausprobiert hast?

## Noch nicht gebaut (kommt in den nächsten Schritten)

- Doppelklick wählt ein Teil (Schicht, Öffnung, Anschlussknoten); kommt mit der Wandlogik (ADR 0045).
- Temporäre Maße bei Mehrfachauswahl mit EQ, Bezug per Klick wechseln.
- Beim Ziehen tippen setzt den Abstand; „Wiederholen“ mit demselben Versatz.
- Fixieren, Verbinden fluchtender Wände, Auswahl in 3D und im Schnitt.
- Achtelmeter-Fang (DIN 4172), Wände aus Linien einer Unterlage, Fang-Einstellungen im Projekt.

## Was ich nicht selbst prüfen konnte

Ich habe die App nur kurz selbst bedient: **WA** und Klicks zeichnen Wände wie gewünscht. Danach lag dein Bildschirm in einer Vollbild-App, und im Hintergrund kommen weder Ziehen (Rahmen, Lasso, ⌥-Kopie) noch neue Bilder der Planansicht an. Alles oben ist im Kern mit Tests abgesichert (66 Tests im Bearbeiten-Teil), Rahmen, Lasso, Vorschau und temporäre Maße in der App aber nur gebaut, nicht von mir ausprobiert.
