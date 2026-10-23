# Prüfpaket M2: Räume, Türen, Fenster, Maße und Flächen

23.10.2026. Etwa 15 Minuten, davon 5 am Mac. M2 baut auf M1 auf und liegt im Branch `m2-raeume-oeffnungen`. Was in M1 offen ist (Prüfpaket M1), gilt hier unverändert.

## Was drin ist

- `eg-100.pdf`: Erdgeschoss des EFH mit Raumstempeln, Türen, Fenstern und Außenmaßketten, 1:100, A4 quer. **In Originalgröße drucken (100 %).** Dazu `eg-100.png` fürs iPhone.
- `eg-50.pdf` und `eg-200.pdf` (mit PNG): dasselbe in 1:50 und 1:200, zum Vergleich der Detailstufen
- `kg-100.pdf`, `dg-100.pdf` (mit PNG): Keller und Dachgeschoss mit Raumstempeln
- `flaechen.pdf` und `flaechen.png`: Flächenaufstellung nach DIN 277 und WoFlV, `flaechen.csv` dieselbe Raumliste für Excel oder Numbers
- `eg-planansicht.png`: das EG so, wie es die App zeigt
- `efh-3d.png`: das Haus in 3D
- Entscheidungen: ADR 0021 (Räume und Flächen), 0022 (Türen und Fenster), 0023 (Maßketten) in `docs/entscheidungen/`

## App

Neu gebaut liegt die App in `~/Applications/Lot.app`. Das EFH öffnest du über Ablage, Testprojekt öffnen (⇧⌘O), Ordner `testprojekte/efh`.

| Taste | Was |
|---|---|
| T | Tür: auf die Wand klicken, auf der Seite, zu der sie aufschlagen soll. Die Bänder sitzen am näheren Wandende. |
| F | Fenster: auf die Wand klicken. **Neu:** „Wand umdrehen“ liegt jetzt auf ⇧F. |
| R | Raum: in den Bereich klicken. Werkzeug, „Räume erkennen“ setzt in jeden umschlossenen Bereich einen Raum. |
| M | Bemaßen: Maßketten außen an den gewählten Wänden |
| V | Auswählen. Türen und Fenster gleiten beim Ziehen entlang ihrer Wand, Raumstempel und Maßketten lassen sich verschieben. |

Rechts im Inspektor: bei Räumen Nummer, Name, Nutzung nach DIN 277, „zählt zur Wohnfläche“ und die Flächen; bei Türen Anschlag und Aufschlag mit Knopf zum Wechseln; ohne Auswahl die Flächen je Geschoss.

## Prüffragen

Antwort jeweils Ja oder Nein und ein Satz. Wo ein Vorschlag steht, reicht „Vorschlag ok“.

1. **Abnahme:** `eg-100.pdf` mit Raumstempeln und Maßketten: Ist das EG so komplett, wie du es für einen Entwurfsplan erwartest? Was fehlt dir zuerst?
2. Raumstempel: Nummer klein über dem Namen, darunter die Netto-Raumfläche; in 1:100 Name 2,5 mm, Nummer und Fläche 1,8 mm, kein Rahmen. Vorschlag: so lassen.
3. Flächenaufstellung `flaechen.pdf`: je Raum NRF und Wohnfläche, je Geschoss KGF und BGF, dann die Zusammenstellung nach DIN 277 und die Wohnfläche. Reicht das für LP 2 und 3? Vorschlag: ja, lichte Höhe und Umfang je Raum kommen mit den Dächern in M3.
4. BGF messe ich an den Außenkanten einschließlich Dämmung (WDVS im EG, Perimeterdämmung im KG), wie DIN 277 Bekleidungen mitzählt. Stimmt das mit deiner Praxis überein? Vorschlag: so lassen.
5. Türen: Türblatt als Linie und Aufschlag als Viertelkreis, in 1:50 das Türblatt mit 4 cm Dicke, ohne Zarge. Vorschlag: in 1:50 die Zarge ergänzen, sonst so lassen.
6. Fenster: Rahmen 7 cm tief an der Außenkante des Mauerwerks (bei WDVS), in 1:100 nur der Rahmen, in 1:50 mit Glaslinie, Fensterbänke fehlen. Vorschlag: in 1:50 Fensterbank innen (2 cm über Putz) und außen (4 cm über WDVS) ergänzen.
7. Das hohe WC-Fenster (Brüstung 1,385 m, über der Schnittebene) ist gestrichelt. Vorschlag: so lassen.
8. Maßketten: Schrägstrich als Maßbegrenzung, kurze Maßhilfslinien, Maßzahl 1,8 mm, halbe Zentimeter hochgestellt (1,13⁵), Öffnungshöhe unter der Linie, Ketten 1,00 m und 1,70 m vor der Wand. Vorschlag: so lassen. Oder lieber Maßhilfslinien bis kurz vor das Bauteil?
9. Rohbaumaße beim Setzen: Tür 1,01 × 2,135 m, Fenster 1,26 × 1,385 m mit Brüstung 0,885 m (DIN 18100). Vorschlag: so lassen, eigene Standardmaße kommen mit dem Bürostandard in M5.
10. **Abnahme:** In der App im EG eine Tür und ein Fenster setzen und verschieben, „Räume erkennen“, eine Wand bemaßen. Geht das ohne Anleitung?

## Was ich geprüft habe und was nicht

- Kern: 117 Tests grün, clippy ohne Warnung. Neu getestet: Raumerkennung (Türen verbinden keine Räume, Stützen werden abgezogen, offene Wände ergeben keinen Raum), DIN 277 und WoFlV (kleine Stützen bleiben in der Wohnfläche, Keller zählt nicht), Anschlag DIN links und rechts je Aufschlagseite, Türsymbol, Maßketten an L-Ecken außen und innen, Maßketten wachsen mit der Wand.
- Von Hand nachgerechnet: WC 6,21 m² und Keller 22,76 m² aus den Wandmaßen, Summen der Maßketten (Süden 11,00 m, Westen 9,00 m).
- App per Bildschirmsteuerung: Räume erkennen, Raumstempel wählen, Name ändern, Wohnfläche abschalten; Tür und Fenster setzen, Tür wählen, Aufschlag wechseln. Dabei einen Fehler gefunden und behoben: Bei Türen blieb der Inspektor leer.
- Nicht prüfen konnte ich wie in M1 das Ziehen mit der Maus und das Tippen in die Befehlspalette (im Kern getestet).
- Eine Umbenennung nach dem Glossar: Raum heißt im Code `Space`, Maßkette `DimensionChain`.

## Messwerte

| Was | Gemessen | Ziel aus PLAN.md |
|---|---|---|
| Wandlänge ändern, danach Grundriss mit Raumstempeln und Maßketten, 3D und Flächen neu | 23 ms | unter 50 ms |

## Bekannte Lücken

- Decken und Bodenplatten (1.03) und Durchbrüche in Decken (1.11) fehlen noch. Deshalb ist der Brutto-Rauminhalt vorläufig BGF mal Geschosshöhe.
- Dachschrägen (M3) fehlen: Die Wohnfläche im DG zählt deshalb noch voll.
- Raumtrennlinien für offene Grundrisse fehlen; ein Bereich ohne trennende Wand ist ein Raum.
- Bogenwände lassen sich noch nicht bemaßen, Innenmaße und Höhenkoten fehlen.
- Zweiflügelige Türen, Schiebetüren und eigene Symbole für Fenstertüren fehlen.
- Die Testdaten des EFH sind ausgedacht: Die Garderobe hat keine Tür, die Nutzungsgruppen sind nicht geprüft.
