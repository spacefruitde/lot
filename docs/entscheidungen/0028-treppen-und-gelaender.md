# 0028 Treppen und Geländer

Status: umgesetzt in M3 (Vorschlag), Ben prüft · 23.10.2026

## Kontext

M3 verlangt Treppen mit Prüfung des Steigungsverhältnisses (1.07) und Geländer (1.08).

## Entscheidung

- **Treppe** (`Stair`) aus geraden Läufen: Antritt (Mitte der Vorderkante der ersten Stufe), Laufrichtung, Laufbreite, Auftritt und je Lauf die Zahl der Steigungen. Nach jedem Lauf außer dem letzten liegt ein Podest: Zwischenpodest, Viertelpodest links oder rechts, Halbpodest links oder rechts. Die Treppe überwindet die Geschosshöhe ihres Geschosses; die Steigung ist Geschosshöhe durch Zahl der Steigungen. Stufen, Podeste und Lauflinie werden abgeleitet.
- **3D**: Laufplatte mit Stufen (Profil quer über die Laufbreite gezogen, Unterseite parallel zur Linie der Stufenvorderkanten), Podeste als Platten, Dicke 18 cm, Stahlbeton.
- **Grundriss** (Vorschlag nach DIN 1356-1): im eigenen Geschoss bis zur Schnittebene durchgezogen, eine schräge Linie quer über den Lauf, darüber gestrichelt. Im Geschoss darüber die ganze Treppe durch das Treppenloch, soweit sie keine Treppe dieses Geschosses verdeckt. Lauflinie mit Kreis am Antritt und Pfeil am Austritt, über Podeste im Bogen. Vor dem Antritt „16 Stg. 17,2/27,0“.
- **Prüfung** nach DIN 18065:2020-08: Steigung und Auftritt nach Tabelle 1 je Gebäudeart (Wohngebäude mit höchstens zwei Wohnungen oder sonstige) und Treppenart (notwendig oder nicht), Schrittmaß 59 bis 65 cm, Laufbreite, lichte Durchgangshöhe ab 2,00 m über Stufenvorderkanten und Podesten unter Decken, anderen Läufen und Dächern.
- **Geländer** (`Railing`): frei verlegt auf OKFF (etwa am Treppenloch) oder an einer Seite eines Treppenlaufs; Handlauf 4 × 4 cm, Pfosten an den Enden, Stäbe mit höchstens 12 cm Abstand, Höhe über OKFF oder über der Stufenvorderkante.

## Alternativen

- **Treppe aus einzelnen Stufen**: frei, aber ohne Prüfung und mit viel Eingabe.
- **Gewendelte Treppen** (verzogene Stufen): im Einfamilienhaus häufig, Verziehung nach DIN 18065 Anhang aufwendig. Folgt, wenn Ben sie braucht.

## Folgen

- Treppenlöcher legt man noch selbst als Deckendurchbruch an; die Prüfung meldet zu wenig Kopfhöhe.
- Die Laufbreite ist der Abstand der Laufseiten; Handläufe zählen noch nicht als Einengung.
