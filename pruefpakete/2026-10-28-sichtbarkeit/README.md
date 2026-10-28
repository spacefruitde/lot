# Sichtbarkeit: zum Ausprobieren in der App

28.10.2026. Etwa 20 Minuten, davon 5 auf dem iPhone (Bilder und Fragen) und 15 **am Mac** in der App. Umsetzung von ADR 0050. Branch `bedienung-sichtbarkeit`, baut auf `bedienung-navigation` auf.

## Erst auf dem iPhone

- `inspektor-ansicht.jpg`: der neue Abschnitt „Ansicht“ im Inspektor, aus der App. Jede Einstellung sagt, woher sie kommt: „aus dem Bürostandard“, „aus Vorlage Bauantrag 1:100“ oder orange „in dieser Ansicht gesetzt“, dann mit ↺ zum Zurücksetzen.
- `mfh-brandschutzplan-eg.png`: MFH EG mit der Vorlage Brandschutzplan. Die Werte für den Feuerwiderstand habe ich nur für das Bild gesetzt (Treppenraum F90, Wohnungstrennwand F60, tragende Innenwände F30), sie sind nicht geprüft. Übrige Wände grau, Maßketten ausgeblendet.
- `efh-flaechenplan-eg.png`: EFH EG mit der Vorlage Flächenplan. Räume nach DIN 277: Nutzungsflächen gelb, Verkehrsflächen blau, Technikflächen grau.

## Dann am Mac

App bauen und starten wie in `CLAUDE.md` (Xcode ⌘R), Ablage, Testprojekt öffnen (⇧⌘O), `testprojekte/efh`.

**Die Kette** (Inspektor, nichts gewählt)
- **Reihenfolge:** Bürostandard, dann Vorlage, dann Ansicht, dann Bauteil. Jede Zeile sagt, woher der Wert kommt.
- **Vorlage wählen**, etwa „Werkplan 1:50“: Maßstab und Detailgrad wechseln, der Maßstab steht auf „aus Vorlage Werkplan 1:50“.
- **Wert selbst setzen**, etwa Detailgrad grob: die Zeile wird orange, ↺ übernimmt wieder.
- **Einstellungen:** Maßstab, Detailgrad, Bauzustand (Umbauplan, Bestand, Neuzustand), Thema, Schnitthöhe und Geschoss darunter. Sie gelten je Ansicht und werden mit dem Projekt gesichert. Die Planblätter zeigen die Ansicht genauso.
- **Maßstab-Menü:** Der Maßstab im Menü Geschoss ist jetzt der Maßstab der Ansicht.

**Vorlagen ab Werk:** Entwurf 1:200, Bauantrag 1:100, Werkplan 1:50, Präsentation, Umbauplan, Flächenplan, Brandschutzplan.
- Jede Vorlage besteht aus Bausteinen (Maßstab mit Detailgrad, Bauzustand, Thema) und steuert nur ihre eigenen. Werkplan 1:50 ändert also den Bauzustand nicht.
- Vorlagen und Themen stehen im Bürostandard (`buerostandard.ron`), dein Büro kann eigene ergänzen.

**Regeln** (bei Vorlage Brandschutzplan oder Flächenplan)
- Der Abschnitt „Regeln“ zeigt jede Regel mit Farbe und Zähler, etwa „4 Bauteile farbig“ oder „12 Bauteile ausgeblendet“.
- Mit dem Haken schaltest du eine Regel in dieser Ansicht ab.
- Je Bauteil gilt die erste Regel, die passt.

**Bauteile ausblenden**
- **Dauerhaft:** Bauteil wählen, „In dieser Ansicht ausblenden“. Der Abschnitt „In dieser Ansicht ausgeblendet“ zählt sie, „Alle wieder zeigen“ holt sie zurück.
- **Vorübergehend:** H blendet die Auswahl aus, I isoliert sie. Ein türkiser Rahmen mit Zähler zeigt es an. H oder I ohne Auswahl zeigt wieder alles. Das gilt je Ansicht und wird nie gedruckt.

**Warum sehe ich das nicht?**
- **Wo:** Bauteile, die du gewählt hast, die die Ansicht aber nicht zeigt, stehen oben im Inspektor unter „Nicht zu sehen in dieser Ansicht“, mit jedem Grund und einem Knopf, der ihn behebt.
- **Gründe:** anderes Geschoss (Dorthin), Bauzustand (Umbauplan zeigen), Regel (Regel abschalten), in der Ansicht ausgeblendet (Wieder zeigen), H oder I (Alles zeigen), andere Variante, über der Schnitthöhe.
- **Zum Probieren:**
  1. Den Grundriss auf Bestand stellen; die Wände des EFH sind Neubau und verschwinden.
  2. Mit ⌥⌘N rechts daneben einen Schnitt öffnen und dort eine Wand anklicken.
  3. Einmal in den Grundriss klicken. Der erste Klick in eine Hälfte macht sie nur aktiv, die Auswahl bleibt.

**Druckbild** (Drucker-Knopf oder ⌥⌘P): nur, was gedruckt wird, ohne Hinweise, Unterlagen, blasses Geschoss und ohne H und I.

## Fragen

1. **Vorlagen ab Werk:** Entwurf 1:200 und Werkplan 1:50 setzen nur den Maßstab, Bauantrag 1:100 dazu den Umbauplan. Soll Werkplan auch den Neuzustand zeigen? Vorschlag: nein, Bauzustand bleibt eigene Wahl.
2. **Vorlage wählen:** Lot nimmt der Ansicht dabei ihre eigenen Werte für die Felder ab, die die Vorlage steuert, damit die Vorlage sichtbar wirkt. Danach selbst abweichen geht wie gewohnt. Ok?
3. **Brandschutz, Schreibweise:** Lot erkennt den Wert im Feld Brandschutz wörtlich („F90“). Schreibst du F90 (DIN 4102), REI 90 (DIN EN 13501) oder „feuerbeständig“? Vorschlag: alle drei als gleich erkennen.
4. **Brandschutz, Farben:** F90 rot, F60 orange, F30 grün, Brandwand dunkelrot, übrige Wände grau, Türen noch ohne Farbe. Gibt es in deinem Büro feste Farben?
5. **Flächenplan:** Nutzungsfläche gelb, Technikfläche grau, Verkehrsfläche blau, Maßketten aus. Brauchst du auch einen Plan nach Wohnungen (Nutzungseinheiten)? Vorschlag: ja, als zweites Thema „Wohnungen“.
6. **Regeln:** Je Bauteil gilt die erste Regel, die passt („übrige Wände grau“ steht deshalb zuletzt). Ok?
7. **H und I:** H blendet aus, I isoliert, H oder I ohne Auswahl zeigt alles; gilt je Ansicht, bis die App schließt (wie Revit). Ok?
8. **Dauerhaft ausblenden:** nur über den Inspektor, ohne Kürzel. Vorschlag: so; Revit hat dafür EH.
9. **Druckbild:** Unterlagen (alte Pläne, DXF) erscheinen nie im Druck. Ok?
10. **Mehrere Grundrisse je Geschoss:** Heute hat jedes Geschoss einen Grundriss mit einer Darstellung. Brauchst du etwa EG Entwurf 1:200 und EG Werkplan 1:50 nebeneinander im Projekt? Vorschlag: ja, als nächster Schritt mit „Ansicht duplizieren“.

## Noch nicht gebaut

- **Ansicht duplizieren:** mehrere Grundrisse je Geschoss (Frage 10).
- **Regeln in der App:** Eigene Regeln lassen sich in der App noch nicht anlegen, nur abschalten und löschen; anlegen geht im Bürostandard oder in der Projektdatei.
- **Vorlagen bearbeiten in der App:** heute nur in `buerostandard.ron`.
- **„Verborgenes zeigen“:** alles Ausgeblendete blass anzeigen und anklicken, wie Revits Glühbirne.
- **Farben je Bauteil:** in einer Ansicht über die Regeln hinaus.
- **Türen in Regeln:** Türen mit T30 färben (die Regel trifft sie, gezeichnet werden heute nur Wände und Räume farbig).

## Was ich nicht selbst prüfen konnte

Dein Bildschirm lag wieder in einer Vollbild-App. Der Abschnitt „Ansicht“ erscheint in der App mit dem EFH (Bild), Klicks kamen aber nicht an. Vorlagen, Regeln, H und I, die Gründe mit Lösungen und die Planblätter sind im Kern mit Tests abgesichert, auch am MFH und EFH. Die Bilder kommen aus denselben Funktionen, die die App beim Zeichnen nutzt.
