# Decken, Dächer und Treppen: zum Ausprobieren in der App

28.10.2026. Etwa 25 Minuten, davon 10 auf dem iPhone (Bilder, Gauben-Vergleich, Fragen) und 15 **am Mac** in der App. Umsetzung von ADR 0051. Branch `bedienung-bauteile`, baut auf `bedienung-sichtbarkeit` auf.

## Erst auf dem iPhone

- `mfh-flachdach-aufsicht.png`: MFH mit Flachdach statt Satteldach, Aufsicht. Pfeil in Fließrichtung mit „2,0 %“, ein Ablauf (Kreis mit Kreuz) und ein Notüberlauf („NÜ“).
- `mfh-flachdach-schnitt.png`: Schnitt durch das Gefälle 1:50. Die Gefälledämmung wird gegen die Fließrichtung dicker, Kies und Abdichtung steigen mit, die Rohdecke bleibt eben.
- `treppe-eg.png` und `treppe-og.png`: Treppe zweiläufig mit Halbpodest aus dem Werkzeug. Unten die Geländer an den freien Seiten, oben das Treppenloch mit Umwehrung, die Seite des Austritts bleibt offen.
- `treppe-viertelgewendelt.png`, `treppe-halbgewendelt.png`, `spindeltreppe.png`: gewendelte Treppen und eine Spindeltreppe aus dem Werkzeug, alle nach DIN 18065 geprüft.
- `gauben.md`: der Vergleich der Gauben in Revit, Archicad, Vectorworks und Allplan mit Vorschlag (ADR 0051 verlangt ihn vor dem Bauen). Quellen in `recherche-gauben.md`.
- `recherche-din18065-wendel.md`: die Werte für gewendelte und Spindeltreppen mit Fundstellen. Den Normtext gibt es frei als PDF; bitte die Fundstellen einmal gegen dein Exemplar halten.

## Dann am Mac

App bauen und starten wie in `CLAUDE.md` (Xcode ⌘R), Ablage, Testprojekt öffnen (⇧⌘O), `testprojekte/efh`. Neue Werkzeuge im Menü Werkzeug und in der Leiste unter „Treppe“ und „Dach und Decke“.

**Decken folgen den Wänden**
- „Decke aus Wänden“ legt die Decke an die Außenkante der Tragschicht und merkt sich die Wände.
- Wand verschieben: Die Decke folgt, im selben Schritt beim Widerrufen.
- Decke wählen: Im Plan sind die Kanten blau, wo sie einer Wand folgen, orange und gestrichelt, wo sie frei sind.
- Wand löschen oder anbauen: Der Umriss bleibt, die Decke ist im Inspektor markiert, „Umriss neu aus den Wänden“ übernimmt ihn.

**Gefälle** (Decke wählen, Inspektor)
- Gefälle an, Verwendung (Flachdach, Balkon, Dusche), Prozent, Fließrichtung in Grad, in welcher Schicht.
- AB setzt einen Ablauf, NU einen Notüberlauf.
- Der Prüfbericht hat einen neuen Abschnitt „Gefälle“.

**Dächer**
- Satteldach, Walmdach oder Flachdach über dem Geschoss. Das Flachdach kommt als Decke der Art Flachdach ins Geschoss darüber, das Lot bei Bedarf als „Dach“ anlegt, mit 2 % Gefälle in der Dämmung.
- DK und Klick auf eine Dachkante wechselt Traufe und Giebel. Im Inspektor stehen je Kante Traufe oder Giebel, Neigung und Überstand.
- Die Wände reichen danach bis unter das Dach.

**Treppen** (TP oder Leiste „Treppe“)
- Formen: gerade, zweiläufig mit Halbpodest, L-Form mit Viertelpodest, viertelgewendelt, halbgewendelt, Spindeltreppe.
- Antritt klicken, dann die Richtung. Bei der Spindeltreppe erst den Mittelpunkt, dann den Antritt außen; der Abstand ist der Radius. Die Leertaste wechselt die Seite der Wendung beziehungsweise den Drehsinn.
- Steigungen und Auftritt rechnet Lot aus der Geschosshöhe. Am Zeiger steht die Ampel nach DIN 18065, grün oder rot mit Grund, etwa „16 Stg. 17,2/28,5 · DIN 18065 erfüllt“.
- Die Treppe schneidet ihr Loch in die Decke darüber, so groß, wie die Durchgangshöhe es verlangt. Geländer kommen an die freien Seiten, eine Umwehrung um das Loch.
- Loch und Umwehrung folgen der Treppe, wenn du sie verschiebst, und gehen mit, wenn du sie löschst.

## Fragen

1. **Gauben, Formen (gauben.md):** Vorschlag zuerst Schlepp, Flach, Satteldach, Walm und Spitz, danach Trapez, Tonne, Fledermaus, Ochsenauge. Passt die Reihenfolge?
2. **Gauben, Höhe:** Traufhöhe vorn und Neigung führen, die Anschlusshöhe am Hauptdach zeigt Lot an und lässt sie tippen. Ok?
3. **Gauben im Umbauplan:** neue Gaube im Bestandsdach mit gelbem Ausbruch und roter Gaube, wie eine neue Öffnung in einer Bestandswand. Passt das?
4. **Gefälle:** Flachdach 2 %, darunter Hinweis „Sonderkonstruktion“; Balkon, Loggia, Terrasse 2 %; bodengleiche Dusche 2 %. Jede Dachfläche braucht Ablauf und Notüberlauf. Werte so übernehmen?
5. **Flachdach:** Decke der Art Flachdach im Geschoss darüber. Aufbau ab Werk: Kies 5, Abdichtung 1, Gefälledämmung 16, Stahlbeton 20, Putz 1,5 cm. Die Attika fehlt noch. Vorschlag: Attika als nächster Schritt. Ok?
6. **Decke folgt den Wänden:** Kommt eine Wand dazu oder fehlt eine, bleibt der Umriss und die Decke ist markiert, statt still zu wachsen oder zu schrumpfen. Ok?
7. **Treppenvorschlag:**
   - Steigung nahe 17,5 cm (Wohngebäude) oder 17 cm (sonst), Auftritt aus 63 cm Schrittmaß.
   - Laufbreite 1,00 m beziehungsweise 1,20 m.
   - Geländer 0,90 m an freien Seiten.
   
   Passt das?
8. **Gewendelte Treppen:**
   - Lot verzieht nach der Verhältnismethode, die Lauflinie liegt in der Mitte, bei mehr als 1,00 m Laufbreite 50 cm von innen.
   - Halbgewendelte Treppen bekommen ein Treppenauge von 20 cm.
   
   Welche Methode nutzt dein Treppenbauer, und passt das Auge?
9. **Durchgangshöhe in Wohngebäuden:** DIN 18065 Bild A.8b lässt an den Rändern 1,75 m zu (25 × 25 cm Ecke), Lot prüft heute überall 2,00 m. Vorschlag: nach A.8b prüfen. Ok?
10. **Spindeltreppe:** Steigung bis 19 cm, Lauflinie 4/10 der Laufbreite von außen. In sonstigen Gebäuden beginnt die nutzbare Laufbreite erst, wo der Auftritt 10 cm erreicht (6.2.4), dort muss die Treppe also größer werden. Brauchst du Spindeltreppen als notwendige Treppe außerhalb von Wohngebäuden?

## Noch nicht gebaut

- **Gauben:** Gebaut wird nach deinen Antworten auf den Vergleich.
- **Dächer:** Attika am Flachdach. Neigung am Griff im Plan (heute im Inspektor). Pultdach als eigene Form (geht über Kanten: eine Traufe, drei Giebel).
- **Gefälle:** zu mehreren Abläufen mit Kehlen und Graten (heute eine Richtung je Decke).
- **Treppen bearbeiten mit Griffen:** Laufbreite ziehen, Podest einfügen. Heute ändert man sie in der Projektdatei.
- **Treppen:** Geländer an gewendelten und Spindeltreppen. Podestregeln der DIN (18 Steigungen, Podesttiefe).

## Was ich nicht selbst prüfen konnte

Dein Bildschirm lag wieder in einer Vollbild-App, deshalb habe ich die App nicht selbst bedient. Die neuen Werkzeuge, das Folgen der Decken, das Gefälle, die Dachkanten und die Treppen mit Loch, Geländer und DIN-Prüfung sind im Kern mit Tests abgesichert, auch am MFH. Die Bilder kommen aus denselben Funktionen.
