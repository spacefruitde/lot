# Wände und Öffnungen: zum Ausprobieren in der App

28.10.2026. Etwa 20 Minuten, davon 5 auf dem iPhone (Schnitt und Fragen) und 15 **am Mac** in der App. Umsetzung von ADR 0045 (Wandlogik) und ADR 0047 (Öffnungen). Branch `bedienung-waende`, baut auf `bedienung-auswaehlen` auf.

## Erst auf dem iPhone: Schnitt 1:50

`efh-schnitt-a-50.png` (und als PDF) zeigt das EFH im Schnitt A-A mit den neuen Schichtregeln nach Bauablauf. Ausschnitte: `detail-aussenwand-decke.png` und `detail-innenwand-decke.png`.

- Die Rohdecke liegt auf der Tragschicht auf, die Wand darunter endet an der Unterkante der Rohdecke.
- Die Wand darüber steht auf der Rohdecke, auch ihr Innenputz beginnt dort.
- Estrich und Trittschalldämmung stoßen an den Innenputz, der Deckenputz endet an den Wänden darunter.
- Außendämmung und Außenputz laufen vor der Deckenstirn durch.
- In Türöffnungen läuft der Estrich durch.

## Dann am Mac

App bauen und starten wie in `CLAUDE.md` (Xcode ⌘R), Ablage, Testprojekt öffnen (⇧⌘O), `testprojekte/efh`. Oder im leeren Projekt mit **WR** ein Rechteck aus Wänden ziehen und mit **WA** eine Innenwand hineinzeichnen.

**Wände** (Wand wählen, Inspektor rechts)
- **Bezugslinie:** sechs Lagen (Mitte, Außenfläche, Innenfläche, Kernmitte, Kern außen, Kern innen). Ein Wechsel verschiebt die Wand nicht, nur die Linie; Ecken bleiben verbunden. Am besten erst alle Wände wählen (Rahmen), dann wechseln.
- **Umdrehen** (Knopf oder Leertaste): innen und außen tauschen, die Wand bleibt liegen, die Dämmung wechselt die Seite, Türen schlagen weiter zur selben Seite auf.
- **Oberkante:** Geschoss darüber, Decke darüber, Dach oder feste Höhe. „Decke darüber“ folgt der Decke, die über dem Kern der Wand liegt, auch wenn sie tiefer gesetzt ist.
- **Wandaufbau:** Schichten mit Material, Dicke in cm und Funktion, hinzufügen, entfernen, verschieben. Wirkt auf alle Wände mit diesem Aufbau; die Bezugslinien bleiben stehen.

**Anschlüsse**
- **Doppelklick auf eine Ecke oder einen T-Stoß** wählt den Anschluss: ein blauer Kreis im Plan, im Inspektor die Wände mit ihrer Rolle (auf Gehrung, läuft durch, stößt an) und warum es so aussieht, etwa „Kalksandstein schneidet Putz“.
- **Ausnahme je Wand:** keine, läuft durch, nicht verbinden. Esc hebt die Auswahl auf.
- Eine Wand im Bauzustand Neubau stößt an Bestand und Abbruch nur stumpf an. Zum Probieren eine Wand auf Bestand stellen.

**Öffnungen** (DR für Tür, WN für Fenster)
- Beim Setzen stehen die **Maße zu den Nachbarn** davor und dahinter (Wandfläche, Nachbaröffnung, Wandende). **Abstand tippen und ⏎** setzt die Öffnung in diesem Abstand vom blau hervorgehobenen Anschlag, **⇥ wechselt** den Anschlag.
- Nahe der Mitte zwischen zwei Anschlägen rastet sie **mittig** ein, am Zeiger steht „mittig“.
- Bei Türen schaltet die **Leertaste** vor dem Klick durch die vier Lagen von Aufschlag und Anschlag.
- Im Inspektor: **Füllung** Durchgang, Nische, Tür, Fenster, ohne dass sich die Öffnung ändert. Bei der Nische Seite und Tiefe.
- **Zwei Brüstungshöhen:** „BRH über OKFF (fertig)“ ist führend; darunter die **Rohbaubrüstung über Rohdecke**, ebenfalls tippbar, dazu Fußbodenaufbau und **unterer Anschluss**. Der Anschluss gehört zum Fenstertyp; hat das Fenster noch keinen, legt Lot beim Tippen einen an (F1 · Fenster).
- Wird der Fußbodenaufbau dicker, bleibt die BRH, die Rohbaubrüstung wandert, und oben im Inspektor erscheint ein Hinweis. In der App lässt sich der Deckenaufbau noch nicht ändern (kommt mit den Typen), getestet ist das im Kern.

## Fragen

1. **Schnitt 1:50:** Stimmen die Schichten am Deckenanschluss so (Bild)? Den Randstreifen am Estrich zeichnet Lot noch nicht.
2. **Bauzustände:** Neubau verbindet sich nicht mit Bestand oder Abbruch, Bestand und Abbruch verbinden sich miteinander (sie waren ein Gebäude). Passt das?
3. **„Nicht verbinden“** heißt bei Lot: Die Wand stößt stumpf an die Fläche der anderen, die läuft durch, keine Schicht greift ein. Revit lässt solche Wände sich überlappen. Vorschlag: so wie jetzt. Ok?
4. **Erklärung am Anschluss:** Lot nennt Materialien („Kalksandstein schneidet Putz“). Lieber Funktionen („Tragschicht schneidet Innenputz“) oder beides?
5. **Umdrehen bei Bezugslinie Außenfläche:** Die Linie bleibt liegen und heißt danach Innenfläche. Alternative: Sie bleibt „Außenfläche“ und springt auf die neue Außenseite. Vorschlag: so wie jetzt, weil sich nichts bewegt. Ok?
6. **Unterer Anschluss ab Werk:** Lot setzt ihn erst, wenn du ihn tippst (sonst 0). Welcher Wert gilt in deinem Büro für Fenster? Vorschlag: 3 cm, bei Türen 0.
7. **Fenster ab Werk:** Die BRH ist jetzt das fertige Maß. Lot setzt bisher 0,885 m (aus dem Rohbaumaß). Vorschlag: 0,90 m fertig. Ok?
8. **Türen:** Die Rohbauöffnung reicht bis auf die Rohdecke, der Estrich läuft unter der Tür durch, bis der Belagswechsel aus ADR 0046 kommt. Ok so lange?
9. **Nische ab Werk:** 10 cm tief, höchstens die halbe Wand, auf der linken Wandseite. Ok?
10. **Fußbodenaufbau für die Rohbaubrüstung:** Lot nimmt die Decke auf der Raumseite, bei Innenwänden den dickeren Aufbau beider Seiten. Passt das, oder soll es der Raum sein, zu dem die Tür aufschlägt?

## Noch nicht gebaut

- Durchlaufen an Knoten aus drei und mehr Wänden (geht an Ecken und T-Stößen).
- Randstreifen am Estrich; Belagswechsel in Öffnungen (ADR 0046).
- Beschriftung im Plan je Planart: Werkplan mit Rohbaubrüstung, Bauantrag mit BRH (kommt mit der Planerstellung, Paket 4).
- Typen-Übersicht, Fenstertypen und Deckenaufbau bearbeiten in der App (ADR 0048, nächster Schritt).

## Was ich nicht selbst prüfen konnte

Wie schon beim letzten Mal lag dein Bildschirm in einer Vollbild-App, das Lot-Fenster ließ sich nicht herüberholen. Alles oben ist im Kern mit Tests abgesichert (unter anderem Bezugslinie ohne Verschieben, Umdrehen, Anschlussregeln, Bauablauf im 3D, Oberkante an der Decke, Brüstungen, Setzen mit Abstand, Nische), und die App baut. Inspektor, Doppelklick auf Anschlüsse und das Setzen mit Maßen habe ich in der App aber nicht selbst bedient.
