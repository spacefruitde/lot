# 0051 Decken, Dächer und Treppen in der App

Status: entschieden von Ben · 26.10.2026; umgesetzt am 28.10.2026 bis auf Gauben und die offenen Punkte unter „Folgen“; Gefällewerte sind mein Vorschlag, Ben prüft

## Kontext

Vergleich in `pruefpakete/2026-10-26-bedienung-3/bauteile.md` (ADR 0042). Decken, Dächer und Treppen gibt es im Kern seit M2 und M3; in der App nur „Decke aus Wänden“ und „Satteldach über dem Geschoss“.

## Entscheidung

Ben am 26.10.2026: „27 ok“, zum Gefälle „28 ok mach einen vorschlag“, zu den Dachformen „29 ok for now - meist giebel und flach - häufig mit gauben was in revit sehr schlecht umgesetzt ist“, zu den Treppen „30 gewendelt wär auch wichtig - vor allem spindeltreppen“.

- **Decken folgen den Wänden,** sichtbar je Kante; scheitert die Anpassung, bleibt die Decke und wird markiert. Die Rohdecke geht bis Außenkante Tragschicht, Randdämmung kommt mit den Details.
- **Gefälle als Eigenschaft einer Schicht** mit Richtung, Prozent und Abläufen. Voreinstellung (Vorschlag, als Daten mit Quelle):
  - Flachdach 2 % in der Fläche (Flachdachrichtlinie, DIN 18531); darunter Hinweis „Sonderkonstruktion“.
  - Balkon, Loggia, Terrasse 2 %.
  - Bodengleiche Dusche 2 %.
  - Jede Dachfläche mit Ablauf und Notüberlauf, sonst Hinweis.
- **Dächer:** Klick auf eine Kante wechselt Traufe, Giebel, Walm; Neigung am Griff; Giebelwände bis unters Dach. Zuerst Satteldach, Flachdach und **Gauben**, die Ben besonders wichtig sind, weil Revit sie schlecht kann.
- **Treppen:** Werkzeug mit DIN 18065 als Ampel und Vorschlägen; Treppenloch und Geländer automatisch. Formen in Version 1: gerade, zweiläufig mit Halbpodest, L-Form mit Viertelpodest, **gewendelt und Spindeltreppe**.

## Alternativen

- **Gewendelte Treppen nach Version 1** (mein früherer Vorschlag): von Ben verworfen.

## Folgen

- Gauben bekommen vor dem Bauen einen kurzen eigenen Vergleich (Formen, Anschluss an das Hauptdach), weil Ben sie als Revit-Schwäche nennt.
- Die DIN-18065-Werte für gewendelte und Spindeltreppen kommen als Daten mit Quelle; offene Werte gehen als Frage ins Prüfpaket.
- Umgesetzt (28.10.2026): Decken folgen den Wänden (`Slab::follows_walls`, Nachführen über `Document::set_follow`), Gefälle als Schicht mit Abläufen und Notüberläufen, Prüfung nach `normen/gefaelle.ron`, Walm- und Flachdach, Dachkanten umschalten, Treppenwerkzeug für gerade, Halb- und Viertelpodest, viertel- und halbgewendelt (Verhältnismethode) und Spindeltreppe, DIN-18065-Ampel, Treppenloch und Umwehrung verbunden. Gauben-Vergleich in `pruefpakete/2026-10-28-bauteile/gauben.md`, gebaut wird nach Bens Antwort.
- Nachgezogen (28.10.2026): Treppen mit Inspektor und Griffen bearbeiten, Geländer über die Stufen gewendelter und Spindeltreppen, Podestregeln der DIN 18065 (Tabelle 1, Nr. 6.3.1 bis 6.3.3), Dachneigung am Griff.
- Offen: Gauben, Attika, Gefälle zu mehreren Abläufen.
