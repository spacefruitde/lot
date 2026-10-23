# 0038 Prüfbericht und Hinweise im Modell

Status: umgesetzt in M5 (Vorschlag), Ben prüft · 24.10.2026

## Kontext

PLAN.md, Abschnitt 4: Ergebnisse der Prüfungen erscheinen als Hinweise direkt im Modell und als Prüfbericht; jede Regel nennt Norm, Paragraf und Fassung. M5 verlangt Barrierefreiheit (5.03) und Rettungswege (5.05).

## Entscheidung

- **Ein Prüfbericht** (`lot_rules::check_project`) mit Abschnitten: Gebäudeklasse, Abstandsflächen, Maß der baulichen Nutzung (GRZ, GFZ), Barrierefreiheit, Rettungswege, Treppen. Jeder Befund hat Status (erfüllt, Hinweis, nicht erfüllt), Text, Quelle und, wo sinnvoll, einen Ort im Grundriss.
- **Hinweise im Modell**: Befunde mit Ort, die nicht erfüllt sind oder Hinweise, erscheinen im Grundriss als farbiger Kreis mit Text. In der App über das Menü Prüfen ein- und ausschaltbar, in den Bauantragsplänen nie.
- **Raumverbund** (`lot_rules::rooms`): Jede Öffnung kennt die Räume links und rechts oder „außen“. Darauf bauen Tür- und Fensterliste, Barrierefreiheit und Rettungswege.
- **Barrierefreiheit nach DIN 18040-2**, Werte als Daten (`normen/din18040-2.ron`): Erreichbarkeit über Eingang und Aufzug, Fahrkorb und Fläche vor dem Aufzug, lichte Türbreiten, Flurbreiten (Öffnen der Fläche: schrumpfen und wachsen), Bewegungsflächen (passt ein Quadrat in den Raum), Handläufe an beiden Seiten, Badtüren schlagen nach außen auf. Ob es verlangt ist, sagt das Landesrecht (MBO § 50: mehr als zwei Wohnungen).
- **Rettungswege nach MBO §§ 33 bis 35**: Ausgang je Nutzungseinheit in den Treppenraum mit Weglänge (Luftlinie im Raum, von Tür zu Tür, höchstens 35 m), Fenster für den zweiten Rettungsweg mit Anleiterhöhe (über 8 m nur mit Hubrettungsfahrzeug), Treppenraum mit Ausgang ins Freie und Fenster je Geschoss.
- **Nutzungseinheit** am Raum (`Space::unit`), leer bei gemeinsamen Räumen. Der Treppenraum ergibt sich aus den Treppen.

## Alternativen

- **Einzelne Prüfungen ohne gemeinsamen Bericht**: einfacher, aber Ben müsste mehrere Listen lesen.
- **Wegsuche auf einem Raster** für Rettungswege: genauer um Hindernisse, aber ohne Möbel im Modell kein Gewinn.

## Folgen

- Lichte Maße aus Rohbaumaßen über feste Abzüge (Tür 0,11 m, Fenster 0,12 m), bis Türen und Fenster Typen mit eigenen Maßen haben.
- Nicht im Modell und deshalb nicht geprüft: Schwellen, Rampen, Bedienelemente, Ausstattung, Rauchableitung oben im Treppenraum, Aufstellflächen der Feuerwehr.
