# Eigene Bauteile und Bestand: Vergleich und Vorschlag

Quellen und Einzelheiten: `recherche-bauteile-bauen.md`. Was Lot heute kann: `lot-heute.md`.

## Kurzvergleich

| Thema | Revit | Archicad | Vectorworks | Allplan | Lot heute |
|---|---|---|---|---|---|
| eigenes Bauteil | Familie mit Referenzebenen und Abhängigkeiten | GDL-Skript oder starres Objekt | Hybridsymbol, Plug-in, Marionette | Makro, SmartPart- oder Python-Skript | keins (M6) |
| einmalige Form | Modellieren vor Ort, bremst, nicht umwandelbar | Morph, nicht assoziativ | Volumenkörper | 3D-Körper, wandelbar | keine |
| 2D je Maßstab | symbolische Linien je Detailgrad, von Hand | Projektion oder Skript | 2D je Ansicht und Detailgrad | Folien je Maßstab | fest je Bauteilart |
| Wanddicke wechselnd | nein, überlappende Wände | trapezförmig, polygonal | nein | über Ebenen | nein |
| schiefe Wand | geneigt, konisch | geneigt, doppelt geneigt | nein | über Ebenen | nein |
| Nische, Schlitz | Abzugskörper vor Ort | Solid-Operation, nicht im Grundriss | „Wall Recess“ | aus 3D-Körpern | Durchbruch nur in Decken |
| Teilabbruch | Wand teilen oder „Teile“ | Wand teilen | eigener Wandstil | Wand teilen | Wand teilen |

**Was Nutzer am meisten ärgert:** Altbauwände werden in Revit aus überlappenden Wänden, Stützen und Modellen vor Ort zusammengestückelt [54 bis 56]; Modelle vor Ort lassen sich nicht in Familien umwandeln (Revit Ideas, 170 Stimmen [61]), nicht alle Kategorien werden im Grundriss geschnitten (533 Stimmen [62]); Archicads Solid-Operationen erscheinen nie im Grundriss [65], Morphs liefern in Listen nur Volumen [66]; eigene Objekte brauchen Skripte (GDL, SmartParts) oder brüchige Referenzebenen [57, 58, 71, 82].

**Wo Lot schon vorn ist:** Lot leitet jeden Grundriss aus der Geometrie ab (Regel 6) und speichert jede Änderung als Operation (Regel 7). Darauf lassen sich Körper, die im Grundriss richtig geschnitten werden, und Bauzustände je Teilbereich bauen, ohne Wände zu zerstückeln.

## Vorschlag für Lot

1. **Wand aus dem Aufmaß:** Eine Bestandswand darf je Seite eigene Punkte haben: wechselnde Dicke, schiefe Fluchten, ein Knick in der Flucht, eine nicht lotrechte Fläche. Schichten folgen der Fläche (Putz bleibt Putz), Öffnungen bleiben Öffnungen. Gezeichnet über die Unterlage: Punkte der Außen- und Innenfläche anklicken. Keine Stückelung aus mehreren Wänden.
2. **Körper an Ort und Stelle, gleich ein Bauteil:** Umriss im Grundriss oder Schnitt zeichnen, dann extrudieren, drehen oder entlang eines Pfads ziehen. Der Körper bekommt sofort Art (Kamin, Unterzug, Vorsprung, Gewölbe, Einbau), Material, Bauzustand und zählt in Listen und Mengen. Als **Abzugskörper** schneidet er Wände und Decken (Nische, Schlitz, Gewölbeaussparung), und der Grundriss zeigt das Ergebnis, weil er ein echter Schnitt ist.
3. **Aus einem Körper wird ein Typ, ohne Umwandlung:** „als Bauteil sichern“ macht ihn wiederverwendbar; Parameter entstehen, indem man im Modell ein Maß setzt und es benennt („Breite“), statt Referenzebenen zu bauen. Alternativ beschreibst du das Bauteil in Worten, und Claude schreibt die Definition (M6).
4. **2D-Bild abgeleitet, je Maßstab übersteuerbar:** Standard ist der Schnitt durch die Geometrie plus Ansicht von oben in Linien nach Bürostandard. Wer will, zeichnet für 1:100 ein vereinfachtes und für 1:50 ein genaues Symbol.
5. **Bauzustand je Teilbereich und Schicht, ohne die Wand zu teilen:** Ein Rechteck in der Ansicht oder im Grundriss markiert einen Abbruchbereich in einer Bestandswand; eine Schicht lässt sich einzeln abbrechen (alter Putz ab, neuer Putz an). Bauantrag und Umbauplan zeigen es nach Bürostandard gelb und rot.
6. **Vorlagen für den Bestand:** Kamin mit Zügen, Unterzug, Holzbalkendecke, Kappendecke, Tonnen- und Kreuzgewölbe, Fachwerkwand, Bestandstreppe aus dem Aufmaß (jede Stufe einzeln). Jede Vorlage ist ein Körper aus Punkt 2 mit Parametern.
