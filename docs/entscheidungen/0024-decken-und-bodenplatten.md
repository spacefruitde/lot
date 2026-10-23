# 0024 Decken und Bodenplatten

Status: umgesetzt in M2 (Vorschlag), Ben prüft · 23.10.2026

## Kontext

M2 verlangt Decken und Bodenplatten (1.03). Ohne sie fehlen im 3D die Böden, und der Brutto-Rauminhalt nach DIN 277 hat keine untere Grenze.

## Entscheidung

- **Deckenaufbau** (`SlabType`) wie der Wandaufbau als Folge von Schichten, hier von oben nach unten (Estrich, Trittschalldämmung, Rohdecke, Putz).
- **Decke** (`Slab`): Umriss als Polygon, Oberkante auf OKFF des Geschosses plus Versatz, Art Geschossdecke oder Bodenplatte. Der Umriss ist ein gespeicherter Parameter.
- **Befehl „Decke aus Wänden“**: Die Decke eines Geschosses liegt auf den Wänden des Geschosses darunter; ihr Umriss folgt der Außenkante der Tragschicht dieser Wände, sodass die Außendämmung vor der Deckenstirn durchläuft. Im untersten Geschoss entsteht eine Bodenplatte unter den eigenen Wänden.
- **3D**: Wände und Stützen enden unter den Decken, die auf ihnen liegen (die Decke wird aus ihnen ausgeschnitten).
- **Brutto-Rauminhalt**: im untersten Geschoss zusätzlich die Dicke der Bodenplatte.
- Vorlage und EFH bekommen zwei Deckenaufbauten: Geschossdecke StB 20 (29,5 cm) und Bodenplatte StB 25 (41 cm).

## Alternativen

- **Umriss an die Wände binden** (ändert sich mit ihnen): richtig für später, braucht aber Bezüge auf Wandflächen wie bei den Maßketten. Bis dahin legt der Befehl die Decke neu an.
- **Wände bis Unterkante Rohdecke modellieren**: mehr Eingaben je Wand; das Ausschneiden im 3D erreicht dasselbe Bild ohne Eingabe.

## Folgen

- Im Grundriss erscheinen Decken noch nicht (Deckenränder und Aussparungen gestrichelt folgen mit Bens Rückmeldung), im Schnitt ab M3.
- Durchbrüche in Decken (1.11) und Treppenlöcher fehlen noch.
