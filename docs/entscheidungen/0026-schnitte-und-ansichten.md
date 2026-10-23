# 0026 Schnitte und Ansichten

Status: umgesetzt in M3 (Vorschlag), Ben prüft · 23.10.2026

## Kontext

M3 verlangt Schnitte und Ansichten (3.02). Sie sollen aus demselben Modell entstehen wie Grundrisse und 3D und sich bei jeder Änderung mit ändern.

## Entscheidung

- **Schnitt oder Ansicht** (`SectionView`): Linie im Grundriss, Blickrichtung (links oder rechts der Linie), Blicktiefe, Name und Kennbuchstabe. Ansichten sind Schnitte ohne geschnittene Flächen, mit der Linie außerhalb des Gebäudes.
- **Geschnittene Flächen** aus den 3D-Körpern je Schicht: Jedes Dreiecksnetz wird an der Schnittebene geschnitten, die Strecken zu Ringen verbunden. Die Darstellung (Schraffur, Schnittkante, Dämmstoffmuster) ist dieselbe wie im Grundriss.
- **Sichtbare Kanten dahinter**: Kanten, an denen Flächen um mehr als 20° abknicken, über alle Körper gemeinsam (bündig weiterlaufende Flächen ergeben keine Linie). Verdeckt wird über einen Tiefenpuffer mit 2 cm Raster.
- **Gelände** vorläufig als waagerechte Linie 30 cm unter ±0,00. Unter dem Gelände ist in Ansichten nichts zu sehen, im Schnitt nur innerhalb des geschnittenen Gebäudes.
- **Höhenkoten** links: OKFF je Geschoss, Traufe und First, Dreieck auf der Höhe, Wert mit Vorzeichen (+2,75, ±0,00, -2,50).
- **Schnittlinie im Grundriss**: nur die kurzen kräftigen Enden mit Pfeil in Blickrichtung und Kennbuchstabe (Vorschlag).

## Alternativen

- **Exakte Verdeckung analytisch** (Kanten gegen alle Flächen schneiden): genauer, aber langsam und anfällig bei vielen kleinen Flächen. Das Raster reicht für 1:50 bis 1:200; feiner geht es, wenn die Prüfung es verlangt.
- **Schnitte aus dem Grundriss ableiten**: geht nur für senkrechte Bauteile; Dächer, Treppen und Gauben brauchen ohnehin das 3D.

## Folgen

- Gelände kommt in M4 (1.12); bis dahin steht im Plan „Gelände vorläufig -0,30 m“.
- Treppen, Geländer, Gauben und Fenster erscheinen ohne eigene Darstellungsregeln, so wie sie im 3D gebaut sind.
