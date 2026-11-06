# Navigation: zum Ausprobieren in der App

28.10.2026. Etwa 20 Minuten, davon 5 auf dem iPhone (Bilder und Fragen) und 15 **am Mac** in der App. Umsetzung von ADR 0049. Branch `bedienung-navigation`, baut auf `bedienung-typen` auf.

## Erst auf dem iPhone

- `seitenleiste-efh.jpg`: die neue Seitenleiste „Projekt“ mit dem EFH, aus der App. Oben die Geschossleiste mit OKFF und Höhe, darunter Grundrisse, Schnitte und Ansichten, Lageplan, Planblätter mit ihren Ansichten.
- `grundriss-dg-eg-blass.png`: Grundriss DG, darunter das EG blass (nur geschnittene Bauteile, Türen, Fenster und Treppen, hellgrau und dünn, ohne Texte). In der App liegt es so unter jedem Grundriss und folgt beim Geschosswechsel.
- `schnitt-a-auswahl.png`: Schnitt A-A, blau sind die Wände des EG, die im Grundriss gewählt sind. Die Auswahl ist in beiden Ansichten dieselbe.
- `planblatt-102.png`: Plan 102, wie die App ihn zeigt; die gewählte Ansicht (Schnitt A-A) ist blau umrandet.

## Dann am Mac

App bauen und starten wie in `CLAUDE.md` (Xcode ⌘R), dann Ablage, Testprojekt öffnen (⇧⌘O), `testprojekte/efh`.

**Seitenleiste** (links; ein- und ausblenden mit ⌃⌘S)
- **Klick auf einen Eintrag** zeigt ihn: Grundriss, 3D, Schnitt, Ansicht, Lageplan, Planblatt oder eine Liste (Flächen, Türen und Fenster, Prüfbericht).
- **Suche oben** findet Ansichten und Räume, etwa „bad dg“.
- **Rechtsklick** auf einen Eintrag: „Rechts daneben öffnen“, bei Schnitten „Löschen“, bei Blättern „Blatt löschen“, bei Ansichten auf einem Blatt „Vom Blatt nehmen“ und „Als Heimat wählen“.
- **⌘K** findet jetzt auch Geschosse, Schnitte, Blätter und Räume.
- **⌘[ und ⌘]** gehen zurück und vor, an dieselbe Stelle; dafür gibt es auch die Pfeile oben links.

**Geschosse**
- **Geschossleiste:** OKFF und Höhe tippen und ⏎. Die Geschosse darüber rücken mit.
- **⌘↑ und ⌘↓** wechseln das Geschoss; die Ansicht bleibt an der Stelle.
- **Geschoss darunter blass:** Haken in der Leiste oder im Menü Darstellung, je Grundriss.

**Schnitte und Ansichten**
- **SC zeichnet einen Schnitt** mit zwei Klicks, **AN eine Ansicht**. Vor dem zweiten Klick dreht die **Leertaste** die Blickrichtung, ein blauer Pfeil zeigt sie. Schnitte bekommen den nächsten freien Buchstaben, Ansichten den Namen der Fassade, etwa „Ansicht Süd“.
- **Ein Klick auf die Marke** wählt den Schnitt. Dann zeigen Griffe an den Enden und eine orange Linie, wie weit er blickt; Name, Buchstabe und Blicktiefe stehen im Inspektor.
- **Doppelklick auf die Marke** öffnet den Schnitt, ⌘[ führt zurück. Ansichten haben im Plan keine Marke; ihre Linie steht nur am Bildschirm, blau gestrichelt.
- **Im Schnitt** wählt ein Klick das Bauteil, das man sieht; ⇧ fügt hinzu, ⌫ löscht.

**Planblätter**
- **Neues Blatt** mit „+“ bei Planblätter.
- **Aufs Blatt legen:** Grundrisse, Schnitte, Ansichten und den Lageplan aus der Seitenleiste aufs Blatt ziehen. Eine Ansicht darf auf mehreren Blättern liegen.
- **Auf dem Blatt:** Ziehen verschiebt die Ansicht (auf ganze mm), Doppelklick öffnet sie, ⌫ nimmt sie vom Blatt. Im Inspektor stehen Maßstab und Heimat.
- **Heimat:** Liegt eine Ansicht auf mehreren Blättern, ist eine davon ihre Heimat; die Seitenleiste zeigt dort ein Haus.

**Nebeneinander** (⌥⌘N oder Knopf oben rechts)
- Zwei Ansichten mit derselben Auswahl, etwa links Grundriss und rechts Schnitt.
- Die angeklickte Hälfte ist aktiv; Seitenleiste, ⌘K und Inspektor gelten für sie. Oben links steht, was jede Hälfte zeigt.

## Fragen

1. **Geschoss darunter (Bild):** So hell, nur Bauteile, Öffnungen und Treppen, ohne Raumstempel? Vorschlag: so lassen.
2. **Geschossleiste:** Ändert sich Höhe oder OKFF eines Geschosses, rücken die Geschosse darüber mit (wie Archicad). Vorschlag: so lassen. Ok?
3. **Ansichten im Plan:** Lot zeichnet für Ansichten keine Marke in den Grundriss, nur am Bildschirm eine Linie. Vorschlag: so lassen, weil Ansichten nach Himmelsrichtung heißen. Oder willst du ein Ansichtssymbol im Plan?
4. **Name der Ansicht:** Blick nach Norden zeigt die Südfassade und heißt „Ansicht Süd“. Ok?
5. **Schnittmarke mit Plannummer:** Soll die Marke die Plannummer der Heimat tragen, etwa „A“ über „102“? Vorschlag: im Werkplan ja, im Bauantrag nein, sobald es Planarten gibt (Paket 4).
6. **Neuer Schnitt:** Von links nach rechts gezeichnet blickt er nach oben (wie Revit), bis 1 m hinter die letzte Wand. Ok?
7. **Schnitt anklicken:** Getroffen wird er nur an den Marken, nicht auf der Linie durchs Gebäude, damit die Wände dort anklickbar bleiben. Ok?
8. **Werkzeug in einer anderen Ansicht:** Wer im Schnitt oder auf dem Blatt ein Werkzeug wählt (Werkzeugleiste, Menü, ⌘K), landet im Grundriss; Kürzel wie WA wirken dort noch nicht. Vorschlag: auch die Kürzel führen in den Grundriss, bis wir im Schnitt zeichnen können. Ok?
9. **Neues Blatt:** A3 quer, nächste Nummer nach der höchsten (101, 102 …), Ansichten kommen in 1:100, der Lageplan in 1:500. Vorschlag: so, später aus dem Bürostandard. Ok?
10. **Nebeneinander:** Die rechte Hälfte öffnet zuerst den ersten Schnitt, sonst 3D, und wechselt Geschosse für sich. Oder soll ein zweiter Grundriss dem ersten im Geschoss folgen?

## Noch nicht gebaut

- Knick in der Schnittlinie; die Blicktiefe als Griff im Plan (heute im Inspektor).
- Plannummer an der Schnittmarke (Frage 5); gleiche Lage über mehrere Blätter per Anker.
- „Geschoss darunter“ merkt sich Lot nur bis zum Schließen. Gespeichert wird es mit den Ansichten aus ADR 0050 (Sichtbarkeit).
- 3D mit Schnittbox um die Auswahl (kommt mit 3D in Paket 4).

## Was ich nicht selbst prüfen konnte

Dein Bildschirm lag wieder in einer Vollbild-App. Aus dem Hintergrund ließen sich Menübefehle auslösen und die Fensterteile ansehen, aber die Zeichenfläche zeichnet dort nicht, und Klicks kommen nicht an.

**Geprüft in der App:**
- die Seitenleiste mit dem EFH (Bild)
- „Neues Planblatt“: Blatt 101 erscheint und öffnet sich, der Inspektor zeigt Plannummer und Format
- „Zurück“
- „Geschoss höher“: die Leiste springt auf DG
- „Nebeneinander“: links „Grundriss EG“, rechts „3D-Modell“

**Nicht selbst bedient:**
- Schnitte, Lageplan und Blätter auf der Zeichenfläche
- das Ziehen aufs Blatt
- Doppelklick auf die Marke
- die Auswahl im Schnitt

Diese Teile sind im Kern mit Tests abgesichert, auch am EFH. Die Bilder oben kommen aus denselben Funktionen.
