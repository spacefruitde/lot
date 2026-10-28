# Treppen bearbeiten, Geländer an Wendeltreppen, Podestregeln, Dachneigung am Griff

28.10.2026. Etwa 15 Minuten, davon 3 auf dem iPhone. Offene Punkte aus ADR 0051, die keine Entscheidung von dir brauchten. Branch `bedienung-bauteile-offen`, baut auf `bedienung-bauteile` auf.

## Erst auf dem iPhone

`viertelgewendelt-gelaender.png`, `halbgewendelt-gelaender.png`, `spindeltreppe-gelaender.png`: Treppen aus dem Werkzeug, jetzt mit Geländer über die Stufen.
- Gewendelte Treppen: innen und außen, wo keine Wand steht.
- Spindeltreppe: außen.

## Dann am Mac

App starten, im EFH oder einem neuen Projekt eine Treppe mit TP setzen und wählen.

**Inspektor**
- Form wechseln: gerade, gerade mit Zwischenpodest, Halb- und Viertelpodest, viertel- und halbgewendelt, Spindeltreppe. Die Zahl der Steigungen bleibt.
- Weitere Einstellungen: Wendung links oder rechts, Laufbreite, Auftritt, Steigungen des ersten Laufs, Treppenauge, Außen- und Spindelradius, baurechtlich notwendig.
- „Steigungen und Auftritt nach DIN 18065 vorschlagen“ rechnet beides neu.
- Die Geländer an den Läufen setzt Lot nach jeder Änderung neu an die freien Seiten; Loch und Umwehrung folgen ohnehin.

**Griffe im Plan**
- Am Antritt verschieben.
- Außen am Antritt die Laufbreite ziehen, auf 5 cm.
- In der Mitte des Podests oder der Wendelung die Aufteilung der Läufe verschieben.
- Bei der Spindeltreppe am Antritt den Radius ziehen, in der Mitte verschieben.
- Am Zeiger stehen Wert und Ampel.

**Podestregeln** (DIN 18065:2020-08, Tabelle 1)
- Mehr als 18 Steigungen ohne Zwischenpodest gibt einen Hinweis (Nr. 6.3.2, nur sonstige Gebäude; die Norm erlaubt begründete Ausnahmen).
- Podesttiefe unter der Laufbreite ist ein Fehler (Nr. 6.3.1), ebenso ein Podestauftritt auf der Lauflinie unter 3 · a, in Wohngebäuden 2,5 · a (Nr. 6.3.3).

**Dachneigung am Griff:** Dach wählen. An jeder Traufe sitzt ein Griff dort, wo das Dach 1 m steigt. Zur Traufe hin gezogen wird es steiler, auf 0,5° gerundet.

## Fragen

1. **Geländer an Spindeltreppen:** nur außen. Innen an der Spindel ein Handlauf? Vorschlag: nein, bis du ihn brauchst.
2. **Geländer nach einer Änderung:** Lot setzt die Geländer an den Läufen neu. Von Hand angelegte Geländer an den Läufen gehen dabei verloren. Vorschlag: so, solange es keine eigenen Geländertypen gibt. Ok?
3. **Halbgewendelt in sonstigen Gebäuden:** Mit 10 cm innen und 1,20 m Laufbreite verzieht Lot fast alle Stufen (Bild). Ist das so üblich, oder soll Lot eher ein größeres Treppenauge vorschlagen?
4. **Podestregeln:** 18 Steigungen als Hinweis, Podestmaße als Fehler. Ok?
5. **Dachneigung am Griff:** Griff auf 1 m Steigung, Neigung auf 0,5°. Ok?

## Noch offen aus ADR 0051

- Gauben (wartet auf deine Antworten).
- Attika (Frage 5 im letzten Prüfpaket).
- Gefälle zu mehreren Abläufen mit Kehlen und Graten.

## Was ich nicht selbst prüfen konnte

Wie zuletzt: Die App baut, Bedienen ging nicht, weil dein Bildschirm in einer Vollbild-App lag. Formwechsel, Griffe, Podestregeln, Dachneigung und die Geländer sind im Kern getestet, die Geländer auch als 3D-Körper.
