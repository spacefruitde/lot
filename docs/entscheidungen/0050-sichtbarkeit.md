# 0050 Sichtbarkeit und Ansichtsvorlagen

Status: entschieden von Ben · 26.10.2026; umgesetzt am 28.10.2026 bis auf die offenen Punkte unter „Folgen“, Ben testet in der App

## Kontext

Vergleich in `pruefpakete/2026-10-26-bedienung-3/sichtbarkeit.md` (ADR 0042). Je Ansicht lässt sich heute nichts einstellen; die Darstellung folgt dem Bürostandard.

## Entscheidung

Ben am 26.10.2026: „24 ok“, „25 ok“, „26 ok“.

- **Sichtbare Kette:** Bürostandard, Ansichtsvorlage, Ansicht, Bauteil; jede Einstellung zeigt ihre Herkunft.
- **Vorlagen aus Bausteinen** (Maßstab mit Detailgrad, Bauzustand, Thema), die nur ihre Felder steuern.
- **„Warum sehe ich das nicht?“** nennt die Gründe und behebt sie mit einem Klick.
- **Regeln dürfen ausblenden und zeigen einen Zähler.**
- **H blendet aus, I isoliert,** mit Rahmen und nie im Druck; ein Schalter „Druckbild“.
- **Vorlagen ab Werk:** Grundriss Entwurf 1:200, Bauantrag 1:100, Werkplan 1:50, Präsentation, Umbauplan, Flächenplan, Brandschutzplan.

## Alternativen

- **Revits Ansichtsvorlagen** (ganz oder gar nicht, Herkunft unsichtbar).

## Folgen

- Die Vorlagen ab Werk werden Teil des eingebauten Bürostandards (ADR 0040).
- Umgesetzt (28.10.2026): Einstellungen je Ansicht als Element `ViewSettings` (nur, was die Ansicht selbst setzt), Vorlagen aus Bausteinen (`ViewTemplate`, `ScaleBlock`), Themen mit Regeln nach Art, Bauzustand, Eigenschaft, Aufbau und Nutzung (`Theme`, `Rule`), die Kette mit Herkunft (`views::look`), Zähler je Regel, „Warum sehe ich das nicht?“ mit Lösung je Grund, dauerhaft in der Ansicht ausblenden, H und I je Ansicht mit Rahmen, Druckbild, sieben Vorlagen und drei Themen ab Werk im Bürostandard. Planblätter zeigen jede Ansicht so, wie sie eingestellt ist.
- Offen: mehrere Grundrisse je Geschoss („Ansicht duplizieren“), Regeln und Vorlagen in der App anlegen, „Verborgenes zeigen“, Farben je Bauteil über Regeln hinaus, Türen und Fenster farbig nach Regeln.
