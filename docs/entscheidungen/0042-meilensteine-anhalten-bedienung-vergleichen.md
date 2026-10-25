# 0042 Meilensteine anhalten, erst die Bedienung vergleichen

Status: entschieden von Ben · 25.10.2026

## Kontext

Nach M5 kann der Kern viel (Pläne, Bauantrag, Prüfungen), die App bildet davon wenig ab. Auswählen kennt nur den Klick und ⇧-Klick, Bearbeiten nur Ziehen und Wandenden; Kopieren, Drehen, Spiegeln, Versetzen, Stutzen, Teilen und temporäre Maße fehlen. Treppen, Dächer, Schnitte, Planblätter, Gelände und Grundstück entstehen bisher nur in der Textdatei der Testprojekte. M6 (eigene Bauteile) baut auf genau diese Grundfunktionen auf. Fünf PRs (M0 bis M5) sind ungeprüft.

Ben am 25.10.2026: „es fehlt noch sehr viel in den grundfunktionen, der ui, der logik hinter dem aufbau - zb wandauswahl, etc.. maxht es sinn schon weiterzumachen oder willst du erst bei den bisherigen themen schauen, wie es etablierte bim programme umsetzen und dann jeweils entscheiden wie wir es machen wollen - mindestens gleich gut - oder besser weil sinnvoller, schneller, klarer, schöner, ...“

## Entscheidung

Ben: „Ja, halte die Meilensteine an und mach den Vergleich. Fang mit Auswählen, Bearbeiten und Wandlogik an.“

- M6 und folgende ruhen.
- Je Thema vergleiche ich, wie Revit, Archicad, Vectorworks und Allplan es lösen, dazu neuere Werkzeuge als Ideengeber. Quellen sind die offiziellen Hilfen der Hersteller und belegte Kritik aus Foren.
- Je Thema ein Vorschlag für Lot: mindestens gleich gut, oder besser, weil sinnvoller, schneller, klarer oder schöner. Ben entscheidet in Paketen mit höchstens 10 Fragen.
- Danach baue ich die Entscheidungen und teste die App selbst per Bildschirmsteuerung.
- Reihenfolge: Auswählen, Bearbeiten, Wandlogik; danach Zeichnen mit Zahleneingabe und Fangen, Öffnungen, Typen und Eigenschaften, Navigation, Sichtbarkeit, Decken, Dächer und Treppen in der App.

## Alternativen

- **Mit M6 weitermachen**: mehr Breite auf schwachen Grundfunktionen, das Risiko für Nacharbeit wächst mit jedem Meilenstein.
- **Grundfunktionen ohne Vergleich nachbauen**: schneller, aber ohne Maßstab, ob es mindestens so gut ist wie das, was Ben aus 15 Jahren Revit kennt.

## Folgen

- Der Vergleich liegt in `pruefpakete/` als Pakete je Themengruppe, Entscheidungen werden ADRs.
- PLAN.md bekommt einen Zwischenschritt „Bedienung und Grundfunktionen“ vor M6.
