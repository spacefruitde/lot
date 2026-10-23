# 0021 Räume und Flächen

Status: umgesetzt in M2 (Vorschlag), Ben prüft · 23.10.2026

## Kontext

M2 verlangt Räume (2.03), Raumstempel (3.06) und Flächen nach DIN 277 (4.01), WoFlV (4.02) und den Brutto-Rauminhalt (4.03). Flächen müssen nach jeder Wandänderung stimmen, ohne dass jemand Raumgrenzen nachzieht.

## Entscheidung

- **Ein Raum speichert nur einen Punkt** (zugleich Lage des Raumstempels), Nummer, Name, Nutzungsgruppe nach DIN 277 und das Merkmal „zählt zur Wohnfläche“. Die Fläche ist der von Wänden und Stützen umschlossene Bereich um den Punkt (`lot-geom/src/spaces.rs`), Regel 6.
- **Gemessen wird in Fußbodenhöhe an den fertigen Oberflächen, ohne Öffnungen.** Türen verbinden also keine Räume, Tür- und Fensternischen gehören nicht dazu. Das entspricht DIN 277:2021-08 (NRF) und WoFlV § 3 Abs. 1 und 3.
- **Flächen in einem eigenen Crate `lot-rules`**: NRF je Raum und Nutzungsgruppe, BGF innerhalb der Außenkanten, KGF als BGF minus NRF, BRI vorläufig BGF mal Geschosshöhe. Wohnfläche: Zubehörräume zählen nicht (§ 2 Abs. 3), Stützen bis 0,1 m² werden nicht abgezogen (§ 3 Abs. 3 Nr. 1).
- **Raumnummer** automatisch: Ebene, Punkt, laufende Nummer (0.01 im EG, 1.01 im 1. OG, -1.01 im Keller). Räume im Keller zählen anfangs nicht zur Wohnfläche.
- **Ausgaben**: Raumstempel im Grundriss (Nummer, Name, NRF), Flächenaufstellung als PDF-Blatt und als CSV (Semikolon, Dezimalkomma).
- **Prüfungen**: Räume ohne geschlossene Wände („Fläche offen“) und zwei Räume im selben Bereich erscheinen als Hinweis in der Aufstellung.

## Alternativen

- **Raumpolygon speichern** (wie gezeichnete Raumgrenzen in manchen Programmen): veraltet bei jeder Wandänderung, widerspricht Regel 6.
- **Räume in Höhe der Schnittebene ermitteln**: Türen würden Räume verbinden.

## Folgen

- Offene Grundrisse ohne trennende Wände (Wohnen und Flur in einem) brauchen später Raumtrennlinien. Bis dahin ist der ganze Bereich ein Raum.
- Dachschrägen (M3) fehlen noch: Die Anrechnung nach WoFlV § 4 (unter 1 m gar nicht, 1 bis 2 m zur Hälfte) greift erst mit Dächern. Ebenso fehlt beim BRI die Höhe bis Unterkante Bodenplatte und Oberkante Dach.
- Innenhöfe, die von Wänden umschlossen sind, zählen bis dahin zur NRF „ohne Raum“ und zur BGF.
