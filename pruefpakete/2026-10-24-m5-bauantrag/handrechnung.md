# Handrechnung Abstandsflächen MFH

Testprojekt `testprojekte/mfh`, Musterbauordnung § 6, 24.10.2026. Dieselben Zahlen prüft `cargo test -p lot-cli --test mfh`.

## Grundlagen

- Gebäude 24,00 x 12,00 m (Außenkanten), Ecken bei x = 0 und 24, y = 0 (Süd) und 12 (Nord).
- Oberer Abschluss der Außenwände: Attika +12,10 über ±0,00 (OKFF 3.OG +8,70 plus 3,40). Flachdach, kein Dachanteil.
- Gelände eben geneigt: z = -0,40 + y / 30, also Süd -0,40, Nord ±0,00.
- MBO § 6 Abs. 4: H = Gelände bis oberer Abschluss der Wand. § 6 Abs. 5: T = 0,4 H, mindestens 3 m.
- Grundstück x = -6 bis 30, y = -9 bis 16 (900 m²). Im Norden die Musterstraße, 10 m breit: Die Abstandsfläche darf bis zur Straßenmitte reichen (§ 6 Abs. 2 Satz 2), also bis y = 21.

## Je Seite

| Seite | Gelände | H | T = 0,4 H | Fläche | Abstand bis Grenze | Ergebnis |
|---|---|---|---|---|---|---|
| Süd (y = 0) | -0,40 | 12,10 + 0,40 = **12,50** | **5,00** | 24,00 x 5,00 = 120,00 m² | 9,00 | eingehalten |
| Nord (y = 12) | ±0,00 | **12,10** | **4,84** | 24,00 x 4,84 = 116,16 m² | 4,00 bis Grenze + 5,00 bis Straßenmitte = 9,00 | eingehalten |
| Ost (x = 24) | -0,40 bis ±0,00 | **12,50 bis 12,10** | **5,00 bis 4,84** | 12,00 x (5,00 + 4,84) / 2 = 59,04 m² | 6,00 | eingehalten |
| West (x = 0) | wie Ost | **12,50 bis 12,10** | **5,00 bis 4,84** | 59,04 m² | 6,00 | eingehalten |

Ost und West in der Mitte (y = 6): Gelände -0,20, H = 12,30, T = 4,92.

Lot rechnet H alle 25 cm aus dem 3D-Modell und kommt auf dieselben Werte (Abweichung unter 5 mm).

## Probe mit zu kleinem Abstand

Variante „Ostgrenze 4,50 m“ (Grenze bei x = 28,50): Die Abstandsfläche Ost ragt im Süden 5,00 - 4,50 = 0,50 m und im Norden 4,84 - 4,50 = 0,34 m über die Grenze. Rot im Lageplan: Trapez 12,00 x (0,50 + 0,34) / 2 = **5,04 m²**. Lot: 5,04 m².

## Gebäudeklasse und Maß der Nutzung

- Gelände im Mittel um das Gebäude (Umfang 72 m): (24 x -0,40 + 24 x 0 + 24 x -0,20) / 72 = **-0,20**
- Höhe nach § 2 Abs. 3: OKFF 3.OG 8,70 + 0,20 = **8,90 m**, über 7 m und bis 13 m, Wohnungen je unter 400 m²: **Gebäudeklasse 4**
- GRZ = 288,00 / 900,00 = **0,32**; vier Vollgeschosse, GFZ = 4 x 288,00 / 900,00 = **1,28**
