# 0039 Bauantragspläne, Lageplan, Tür- und Fensterliste

Status: umgesetzt in M5 (Vorschlag), Ben prüft · 24.10.2026

## Kontext

Abnahme M5: Für das MFH entstehen vollständige Bauantragsunterlagen. Was in die Bauvorlagen gehört, regelt die Bauvorlagenverordnung des Landes; als Vorschlag gilt die Muster-Bauvorlagenverordnung (MBauVorlV 2007), §§ 7 und 8.

## Entscheidung

- **Lageplan** als eigene Ansicht (`ViewContent::SitePlan`), 1:250 (nicht kleiner als 1:500): Baugrundstück mit Fläche, Nachbargrundstücke, öffentliche Fläche mit Breite und Mitte, Gebäude rot schraffiert mit Geschosszahl und Dachform, ±0,00 und Oberkante über NHN, Gebäudeklasse, Abstandsflächen mit H und T, Abstände zu den Grenzen, Geländehöhen, Nordpfeil, Legende.
- **Grundrisse**: Aufzug mit Fahrkorbfläche, Rohbaumaße aller Türen ohne Maßkette als „Breite/Höhe“, Raumnutzung im Raumstempel, Treppen mit Steigungsverhältnis.
- **Schnitte**: Wandhöhe H nach § 6 Abs. 4 an beiden Außenseiten, lichte Raumhöhe je Geschoss, Steigungsverhältnis über jeder Treppe, vorhandenes Gelände.
- **Plankopf**: Bauvorhaben, Bauherr, Bauort mit Flurstück, Entwurfsverfasser aus dem Bürostandard, Maßstab, Datum, Plannummer, Unterschriftsfelder.
- **Berechnungen** als Auszug des Prüfberichts (Gebäudeklasse, Abstandsflächen, GRZ und GFZ), dazu Flächenaufstellung mit Wohnfläche je Wohnung.
- **Tür- und Fensterliste** (4.05): Position je Geschoss („T 0.01“, „F 0.01“), Typ nach Art und Rohbaumaß, Raum und angrenzender Raum, Brüstung, lichte Breite, Anschlag; als Blatt und CSV.
- **Projektangaben** um Bauvorhaben, Flurstück, Bundesland, Höhe über NHN und Nordrichtung erweitert.

## Alternativen

- **Amtlicher Lageplan als Unterlage**: In vielen Ländern liefert ihn der Vermesser; Lot zeichnet den Abstandsflächenplan darauf. Die Unterlage (ADR 0032) geht schon, der Abgleich der Grenzen kommt, wenn Ben einen echten hat.
- **Farbige Bauantragspläne für Neubau** (alles rot): Die MBauVorlV verlangt Farben nur bei Änderungen; Neubau bleibt schwarz-weiß.

## Folgen

- Baubeschreibung, Standsicherheits- und Brandschutznachweis, Stellplatznachweis und Entwässerung bleiben bei Fachplanern oder im Formular des Landes.
- Bebauungsplan (Baugrenzen, zulässige GRZ und GFZ) ist nicht im Modell; der Vergleich fehlt.
