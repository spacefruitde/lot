# 0047 Öffnungen: Rohbauöffnung zuerst, zwei Brüstungshöhen

Status: entschieden von Ben · 26.10.2026; das Setzen testet Ben in der App

## Kontext

Vergleich in `pruefpakete/2026-10-26-bedienung-2/oeffnungen.md` (ADR 0042). Türen und Fenster sind seit M2 Bauteile mit Rohbaumaß (ADR 0022).

## Entscheidung

Ben am 26.10.2026: „15 an sich klingt das gut“, zum Setzen „16 muss ich austesten“, zur Brüstung „17 über okff - aber es gibt ja einen unterschied zwischen rohbaubrüstung und fertiger brh den es so in revit nicht gibt“.

- **Die Öffnung ist das Bauteil, die Füllung optional.** Wandöffnung mit Rohbaumaß zuerst, Tür oder Fenster füllen sie später, ohne dass sich die Öffnung ändert. Dasselbe Bauteil kann Durchgang oder Nische sein.
- **Setzen mit Abstand und Bezug** an der Rohbauöffnung, Tab wechselt den Bezug, „mittig zwischen“ als Fang, Leertaste schaltet die vier Lagen von Aufschlag und Anschlag. Ben probiert es in der App aus, danach entscheiden wir nach.
- **Zwei Brüstungshöhen:**
  - **Fertige BRH über OKFF** ist führend und wird gespeichert; sie zählt für den Bauantrag und die Absturzsicherung nach Landesrecht.
  - **Rohbaubrüstung über Oberkante Rohdecke** rechnet Lot aus dem Fußbodenaufbau des Raums vor der Öffnung und dem unteren Anschluss des Fensters.
  - Beide stehen im Inspektor und lassen sich tippen; getippt in der Rohbaubrüstung, rechnet Lot die BRH zurück.
  - Der Werkplan zeigt die Rohbaubrüstung, der Bauantrag die BRH. Ändert sich der Fußbodenaufbau, bleibt die BRH, die Rohbaubrüstung wandert, und Lot zeigt die Änderung an.

## Alternativen

- **Ein Wert wie Revit** (Brüstung über Ebene): der Unterschied zwischen Rohbau und Fertigmaß bleibt Kopfrechnen.
- **Rohbaubrüstung führend:** passt zur Ausführung (LP 5), aber in LP 1 bis 4 ist die fertige Höhe die Planungsabsicht.

## Folgen

- Der untere Anschluss (Fensterbank, Ausgleich) wird ein Wert am Fenstertyp; der Fußbodenaufbau kommt aus der Decke.
