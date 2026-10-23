# 0019 Werkzeuge, Fangen und Befehle im Kern

Status: umgesetzt in M1 · 23.10.2026

## Kontext

Wände zeichnen und verschieben (8.03), Befehlspalette (8.04) und später der KI-Assistent (8.06) brauchen dieselben Funktionen. Lot soll später auch auf Windows und dem iPad laufen.

## Entscheidung

Die Bearbeitung liegt im Kern, im Crate `lot-edit`, nicht in der Swift-App:

- **Werkzeuge:** Auswählen, Wand, Achse, Stütze. Die App reicht nur Zeiger- und Tastenereignisse in Modellkoordinaten weiter.
- **Fangen:** Wandenden, Achsschnittpunkte, Punkte auf Wänden und Achsen, Winkel in 15°-Schritten (mit ⇧ nur 90°), freie Punkte auf 1 cm.
- **Verschieben:** Ecken ziehen mit, an T-Stößen bleibt die anstoßende Wand in ihrer Richtung und wird verlängert. Alles zusammen ist eine Operation, ein Undo nimmt es zurück.
- **Befehle:** Jede Funktion ist ein `Command` mit Kennung (RON-Schreibweise, etwa `SetWallType(11)`). Die Befehlspalette zeigt sie; später ruft die KI dieselben Befehle auf.

Getestet ohne Fenster, auch unter Linux.

## Alternativen

- Werkzeuge in Swift: schneller für die erste Version, aber für Windows und iPad doppelt zu bauen und nicht ohne App testbar.

## Folgen

- Die Planansicht zeigt beim Ziehen eine Vorschau aus einem vorläufigen Modellstand; erst beim Loslassen entsteht die Operation.
- Bemaßung, Türen und Fenster (M2) kommen als weitere Werkzeuge in denselben Crate.
