# 0037 Landesrecht als Daten, Abstandsflächen

Status: umgesetzt in M5 (Vorschlag), Ben prüft · 24.10.2026

## Kontext

M5 verlangt Abstandsflächen (5.01), geprüft gegen eine Handrechnung. Die Regeln stehen in den Landesbauordnungen und unterscheiden sich je Land (Tiefe, Mindestmaß, Dächer, Privilegien). CLAUDE.md, Regel 8: Landesrecht liegt als Daten vor, nicht als Code. Bens Bundesland ist noch offen (PLAN.md, Frage 2).

## Entscheidung

- **Landesrecht als RON-Datei je Land** in `core/crates/lot-rules/landesrecht/`. Bis Ben sein Land nennt, gilt die Musterbauordnung (`mbo.ron`). Jeder Wert steht mit Paragraf in der Datei. Das Land wählt `ProjectInfo.state` (ISO-Kürzel, etwa „NW“); fehlt eine Datei, gilt die MBO mit Hinweis im Prüfbericht.
- **Wandhöhe H aus der 3D-Geometrie**: Über jedem Punkt einer Außenwand (alle 25 cm) die höchste Kante des Gebäudes, 1 cm hinter der Außenfläche gemessen und auf die Fläche hochgerechnet. Damit zählen Attika, Giebeldreieck und Schnittpunkt mit der Dachhaut ohne Sonderfälle; Dachüberstände und Gesimse vor der Wand nicht. Steile Dächer (über 45° ein Drittel, über 70° ganz) kommen aus dem Dachmodell dazu.
- **Gelände je Punkt**: H beginnt am Gelände direkt vor der Wand. Bei geneigtem Gelände ändert sich T entlang der Wand, die Abstandsfläche wird ein Trapez.
- **Tiefe T = 0,4 H, mindestens 3 m** (MBO § 6 Abs. 5), für kleine Wohngebäude (Gebäudeklasse 1 und 2, bis drei oberirdische Geschosse) fest 3 m. Rechtecke vor den Wänden, Ecken frei.
- **Außenwände** sind die Außenkanten des Umrisses jedes Geschosses. Wände tieferer Geschosse in derselben Flucht und nicht höher fallen weg; ein Staffelgeschoss hat seine eigene Abstandsfläche.
- **Zulässige Fläche**: das Baugrundstück (`Plot`), auf öffentlichen Verkehrs-, Grün- und Wasserflächen bis zu deren Mitte (§ 6 Abs. 2). Was darüber hinausragt, ist rot.
- **Gebäudeklasse** (§ 2 Abs. 3) wird mitgerechnet, weil die kleine Abstandsfläche, die Rettungswege und der Bericht sie brauchen. Das Gebäude gilt als freistehend.

## Alternativen

- **H aus den Wandparametern** (Oberkante, Dach): kürzer, aber jeder Sonderfall (Attika, Giebel, Gaube, Pultdach) wäre eigener Code.
- **H im Mittel je Wand**: einfacher zu prüfen, aber nicht, was die MBO sagt; manche Länder verlangen es, dann wird es ein Schalter in der Landesdatei.
- **Regeln in Code je Land**: widerspricht Regel 8 und wäre für jedes Land neu zu prüfen.

## Folgen

- Handrechnung für das MFH im Prüfpaket M5, als Test in `tests/mfh.rs` festgehalten.
- Noch nicht geprüft: Überdeckung eigener Abstandsflächen (§ 6 Abs. 3), Vorbauten und Dachaufbauten (§ 6 Abs. 6), Abstandsflächen mehrerer Gebäude auf einem Grundstück, Gebäude an der Grenze, Baulasten. Der Bericht sagt das.
