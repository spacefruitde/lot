# 0029 Gauben und Dachfenster

Status: umgesetzt in M3 (Vorschlag), Ben prüft · 23.10.2026

## Kontext

M3 verlangt Gauben und Dachfenster (1.05). Das EFH ist laut Plan ein „Satteldach mit Gauben“.

## Entscheidung

- Beide sitzen in der Dachfläche einer Traufe. Ihre Lage wird entlang der Traufkante und waagerecht nach innen angegeben.
- **Dachfenster** (`RoofWindow`): Mitte entlang der Traufe, Breite, Länge in der Dachfläche und Höhe der Unterkante über OKFF. Im 3D ein Ausschnitt im Dach mit Rahmen und Glas in der Dachfläche.
- **Gaube** (`Dormer`), vorerst als Schleppgaube: Mitte und Breite entlang der Traufe, Abstand der Vorderwand vom Dachumriss, Traufhöhe der Gaube über OKFF, Neigung des Gaubendachs, Wandaufbau für Vorderwand und Wangen, wahlweise ein Fenster in der Vorderwand. Wo das Gaubendach die Dachfläche trifft, wird abgeleitet. Das Hauptdach ist innerhalb der Gaube ausgeschnitten.
- Wände mit `Top::Roof` reichen unter der Gaube bis unter das Gaubendach, und die lichte Höhe unter der Gaube zählt für die Wohnfläche.
- **Grundriss**: Umriss von Gaube und Dachfenster gestrichelt wie der Dachumriss (Vorschlag).

## Alternativen

- **Gaube als Zusammenbau aus Wänden und kleinem Dach**: flexibel, aber Ausschnitt und Anschlüsse müsste man von Hand pflegen.

## Folgen

- Satteldach-, Walm- und Fledermausgauben fehlen noch, ebenso Zwerchgiebel. Die Vorderwand steht immer auf der Dachfläche; eine Gaube in der Flucht der Außenwand mit unterbrochener Traufe geht noch nicht.
