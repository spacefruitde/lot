# 0025 Dächer

Status: umgesetzt in M3 (Vorschlag), Ben prüft · 23.10.2026

## Kontext

M3 verlangt Dächer (1.04). Das EFH hat ein Satteldach über einem Kniestock, die Wände im Dachgeschoss reichen bis unter die Dachschräge, und die Wohnfläche im Dachgeschoss hängt nach WoFlV § 4 an der lichten Höhe.

## Entscheidung

- **Dach** (`Roof`): Umriss an der Außenkante der Wände, je Umrisskante Traufe (`Eave`, mit Neigung und Überstand) oder Ortgang (`Verge`, mit Überstand), dazu die Traufhöhe über OKFF. An der Außenkante hat die Dachhaut die Traufhöhe.
- **Dachfläche** als untere Hülle der Traufebenen: An jedem Punkt gilt die niedrigste Ebene. Das ist für konvexe Umrisse exakt (Sattel-, Walm-, Pult-, Flachdach). First und Grate entstehen von selbst.
- **Dachaufbau** wie ein Deckenaufbau (`SlabType`), Schichten von oben nach unten, je Schicht ein Körper zwischen zwei parallelen Ebenen.
- **Wände bis unter das Dach**: Oberkante `Top::Roof`; im 3D wird an der Dachunterseite abgeschnitten, im Grundriss zählt die Wand wie bis zur Geschosshöhe.
- **Lichte Höhe**: Bereiche mit Dachunterseite über 1,00 m und 2,00 m. Daraus die Wohnfläche nach WoFlV § 4 Nr. 1 und 2 (voll ab 2 m, zur Hälfte zwischen 1 und 2 m) und im Grundriss die Linien gleicher lichter Höhe.
- **Befehl „Satteldach über dem Geschoss“**: Traufen an den Längsseiten, 40°, Überstände 50 cm an der Traufe und 30 cm am Ortgang (Vorschlag).

## Alternativen

- **Gewichtetes Straight Skeleton** für beliebige Umrisse: nötig für L- und T-förmige Häuser, deutlich aufwendiger. Kommt, wenn ein Projekt es braucht; bis dahin weist die Prüfung nicht konvexe Umrisse ab.
- **Dachflächen einzeln zeichnen** (wie in manchen CAD-Programmen): mehr Eingaben, und Anschlüsse an First und Grat müssten von Hand stimmen.

## Folgen

- Nur konvexe Dachumrisse. Verwinkelte Grundrisse brauchen vorerst mehrere Dächer.
- Gauben und Dachfenster schneiden in die Dachfläche ein (ADR 0029).
