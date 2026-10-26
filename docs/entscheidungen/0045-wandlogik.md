# 0045 Wandlogik: Bezugslinie, Anschlüsse, Höhen

Status: entschieden von Ben · 26.10.2026

## Kontext

Vergleich in `pruefpakete/2026-10-25-bedienung-1/waende.md` (ADR 0042). Wandanschlüsse nach Schichtrang hat Lot seit M1 (ADR 0017); offen waren Bezugslinie, Umdrehen, Eingriffe am Knoten und Höhen.

## Entscheidung

Ben am 26.10.2026: „8 ok“, zu den Anschlusstypen aus Revit „9 nein brauch ich nicht“, zur Höhe am Deckenanschluss „10 ich verstehe die frage nicht. schichten nach tatsächlichem bauablauf immer..“

- **Bezugslinie ist eine Lesart, keine Geometrie.** Sechs Lagen: Mitte, Außenfläche, Innenfläche, Kernmitte, Kern außen, Kern innen. Ein Wechsel verschiebt nie etwas. Ändert sich der Aufbau, bleibt die Bezugslinie stehen. **Umdrehen** tauscht innen und außen, die Wand bleibt liegen.
- **Anschlüsse automatisch, erklärbar am Knoten.** Jeder Stoß ist ein auswählbarer Knoten mit Erklärung („Mauerwerk schneidet Dämmung“) und Ausnahmen (welche Wand durchläuft, nicht verbinden). Keine Anschlusstypen wie Gehrung oder stumpf. Wände verschiedener Bauzustände verbinden sich nicht.
- **Höhen:** Oberkante an Geschoss mit Versatz, an die Decke darüber, ans Dach oder fest.
- **Schichten enden nach Bauablauf.** Ben gibt das Prinzip vor; die Regeln daraus sind mein Vorschlag, Ben prüft sie im Schnitt 1:50:
  - Tragschicht bis Unterkante Rohdecke, die Rohdecke liegt auf der Tragschicht auf.
  - Innenputz von Oberkante Rohdecke bis Unterkante der Rohdecke darüber.
  - Außendämmung und Außenputz laufen über die Deckenstirn durch.
  - Estrich und Trittschalldämmung stoßen mit Randstreifen an den Innenputz.
- **Wandaufbau-Editor in der App;** der Aufbau gehört zum Typ, Bezugslinie und Höhen zur Wand.

## Alternativen

- **Anschlusstypen wie Revit** (Gehrung, stumpf, rechtwinklig): von Ben verworfen.
- **Höhe je Schicht von Hand** (wie Allplan): mehr Eingaben für dasselbe Ergebnis.

## Folgen

- Die Schichtregeln am Deckenanschluss kommen als Prüffrage mit Schnitt 1:50 ins nächste Prüfpaket.
