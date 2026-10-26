# Paket Bedienung 3: Navigation, Sichtbarkeit, Decken, Dächer und Treppen

26.10.2026. Etwa 20 Minuten, nur lesen und antworten, kein Mac nötig. Drittes Paket nach deiner Entscheidung vom 25.10.2026 (ADR 0042). Pakete 1 und 2 liegen daneben in `pruefpakete/2026-10-25-bedienung-1` und `pruefpakete/2026-10-26-bedienung-2`.

## Was drin ist

- **`navigation.md`**, **`sichtbarkeit.md`**, **`bauteile.md`**: je Thema ein Kurzvergleich (Revit, Archicad, Vectorworks, Allplan, Lot heute), was Nutzer dort am meisten ärgert, und mein Vorschlag für Lot. Je eine Seite.
- `lot-heute.md`: was Lot bei diesen Themen heute kann. Hier ist die Lücke am größten: Schnitte, Planblätter, Treppen, Walm- und Flachdächer gibt es im Kern, aber nicht als Werkzeug oder Ansicht in der App.
- `recherche-navigation.md`, `recherche-sichtbarkeit.md`, `recherche-bauteile.md`: die Recherche mit Quellen. Nur zum Nachschlagen.

Revit und Archicad habe ich nicht selbst bedient; alles stammt aus den Hilfen und Foren. Wo dein Alltag etwas anderes sagt, gilt dein Alltag.

## Fragen

Antwort jeweils „Vorschlag ok“, „wie Revit“, „wie Archicad“ oder ein Satz, was anders sein soll.

**Navigation** (`navigation.md`)

1. **Seitenleiste und Sprünge:** links eine Seitenleiste „Projekt“ (Geschosse, Grundrisse, Schnitte und Ansichten, Lageplan, Planblätter, Listen) mit Suche; ⌘K springt auch zu Geschossen, Schnitten, Blättern und Räumen; ⌘[ und ⌘] gehen zurück und vor. Vorschlag ok?
2. **Geschosse:** eine Leiste mit EG, 1.OG, DG und editierbaren Höhen, ⌘↑ und ⌘↓ wechseln; das Geschoss darunter erscheint blass und läuft beim Wechsel mit, je Ansicht abschaltbar. Vorschlag ok?
3. **Schnitte und Blätter in der App:** Schnittwerkzeug im Grundriss (zwei Klicks, Leertaste dreht die Blickrichtung, Tiefe als Griff), Doppelklick auf die Marke öffnet den Schnitt; Ansichten per Ziehen aufs Blatt, eine Ansicht darf auf mehreren Blättern liegen, eine davon ist die „Heimat“ für die Schnittmarke; Grundriss und Schnitt nebeneinander mit gemeinsamer Auswahl. Vorschlag ok, oder lieber wie Revit eine Ansicht je Blatt?

**Sichtbarkeit** (`sichtbarkeit.md`)

4. **Herkunft und Vorlagen:** Darstellung kommt aus Bürostandard, Ansichtsvorlage, Ansicht und Bauteil, und jede Einstellung zeigt, woher sie kommt; Vorlagen aus Bausteinen (Maßstab, Bauzustand, Thema), die nur ihre Felder steuern, statt Revits ganz oder gar nicht. Vorschlag ok?
5. **Klarheit:** „Warum sehe ich das nicht?“ nennt die Gründe und behebt sie mit einem Klick; Regeln dürfen ausblenden und zeigen, wie viele Bauteile sie ausblenden; H blendet aus, I isoliert, mit Rahmen und nie im Druck; ein Schalter „Druckbild“. Vorschlag ok?
6. **Vorlagen ab Werk:** Welche Ansichtsarten brauchst du im Büro wirklich? Vorschlag: Grundriss Entwurf 1:200, Bauantrag 1:100, Werkplan 1:50, Präsentation, Umbauplan, Flächenplan, Brandschutzplan. Was fehlt, was ist zu viel?

**Decken, Dächer, Treppen** (`bauteile.md`)

7. **Decken:** per Klick in eine umschlossene Fläche, aus Wänden oder als Polygon; jede Kante zeigt ihren Bezug und folgt der Wand; scheitert das, bleibt die Decke und wird markiert, statt gelöscht zu werden. Fachlich: Bis wohin führst du die Rohdecke an der Außenwand (Außenkante Tragschicht, mit Randdämmung)? Vorschlag: Außenkante Tragschicht, Randdämmung mit den Details.
8. **Gefälle:** als Eigenschaft einer Schicht (Gefälledämmung, Gefälleestrich) mit Richtung, Prozent und Abläufen, die Dicke im Schnitt stimmt. Vorschlag ok, und welche Gefälle setzt du als Voreinstellung an (Flachdach, Balkon)?
9. **Dächer:** Klick auf eine Kante wechselt Traufe, Giebel, Walm; Neigung am Griff oder getippt; Giebelwände bis unter das Dach automatisch; Gauben und Dachfenster per Klick auf die Dachfläche; unmögliche Formen werden erklärt. Welche Dachformen kommen bei dir vor (Sattel, Walm, Krüppelwalm, Pult, Mansard, Flachdach)?
10. **Treppen:** Werkzeug im Grundriss (Antritt, Richtung, Form), DIN 18065 live als Ampel mit Vorschlägen, Treppenloch und Geländer automatisch und verbunden. Welche Formen brauchst du in Version 1? Vorschlag: gerade, zweiläufig mit Halbpodest, L-Form mit Viertelpodest jetzt; gewendelte Treppen danach.

## Wie es weitergeht

Mit diesem Paket ist der Vergleich der Grundfunktionen vollständig. Deine Antworten aus allen drei Paketen werden ADRs. Dann baue ich die Bedienung in dieser Reihenfolge: Auswählen und Bearbeiten, Zeichnen und Fangen, Wände und Öffnungen, Typen und Inspektor, Navigation mit Schnitten und Blättern in der App, Sichtbarkeit, Decken, Dächer und Treppen. Ich teste die App dabei selbst per Bildschirmsteuerung.
