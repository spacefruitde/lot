# Prüfpaket M0: Technik bestätigen

22.10.2026. Etwa 15 Minuten, davon 5 am Mac für die App.

## Was drin ist

- `m0-pruefblatt.pdf`: Prüfblatt A4 quer, 1:100. **In Originalgröße drucken (100 %).**
- `m0-pruefblatt.png`: dasselbe Blatt zum Ansehen auf dem iPhone
- `3d.png`: die Wand mit Fenster, ohne Fenster gerechnet vom selben Renderer wie die App
- `lizenzen.txt`: Lizenzen aller 250 Abhängigkeiten (nur freizügige, dazu UniFFI unter MPL-2.0)
- Entscheidungen zu jeder Bibliothek: `docs/entscheidungen/` (16 kurze ADRs)

## App starten

In Xcode `app-mac/Lot.xcodeproj` öffnen und ⌘R. Oder im Terminal:

```
cd app-mac && xcodebuild -project Lot.xcodeproj -scheme Lot -derivedDataPath build/DerivedData build && open build/DerivedData/Build/Products/Debug/Lot.app
```

Bedienung: zwei Finger ziehen dreht, mit ⇧ verschieben, spreizen zoomt. ⌘0 zeigt alles. Rechts stehen die Parameter von Wand und Fenster.

## Prüffragen

Antwort jeweils Ja oder Nein und ein Satz.

1. **Abnahme:** Misst das Rechteck A auf dem Ausdruck 50 × 30 mm?
2. **Abnahme:** Sind in Reihe C die Linien 0,25 und 0,50 klar unterscheidbar?
3. Ist die Maßstabsleiste B genau 50 mm lang, jedes Feld 10 mm?
4. **Abnahme:** Startet die App, und lässt sich die Wand mit dem Trackpad flüssig drehen?
5. Passt die Gestenbelegung (zwei Finger drehen, mit ⇧ verschieben)? Vorschlag: so lassen. Alternative wie Archicad: zwei Finger verschieben, mit ⌥ drehen.
6. Bleibt der Wert „Bild“ in der Statuszeile unten links beim Drehen unter 8 ms? (8 ms reichen für 120 Hz.)
7. Rechts die Fensterbreite auf 2,00 m stellen, dann ⌘Z. Springt das Fenster sofort auf 1,50 m zurück?
8. ⌘⇧S speichern, ⌘N, dann ⌘O mit der gespeicherten Datei. Kommt die Wand mit deiner Änderung zurück?
9. Ist osifont (Reihe F) als Planschrift für Lot in Ordnung? Vorschlag: ja, bis der Bürostandard in M5 eine eigene Wahl erlaubt.
10. `3d.png`: Taugt der Look (helles Arbeitsmodell mit dunklen Kanten) als Ausgangspunkt? Vorschlag: ja, Materialien und Schatten kommen später.

## Entscheidungen, die ich von dir brauche

1. **DWG ohne ODA File Converter** (ADR 0012). Der kostenlose Converter ist laut ODA nur für nichtkommerzielle Nutzung frei; für den Büroalltag ist das unklar, mit Lot ausliefern ist verboten. Die Rust-Bibliothek acadrust (MPL-2.0) liest und schreibt DXF und DWG und hat im Test echte Pläne samt Schraffuren gelesen. **Vorschlag:** acadrust für DXF und DWG, der ODA-Converter entfällt. Betrifft erst M4.
2. **rustup statt Homebrew-Rust** (ADR 0001). Mit Homebrew-Rust läuft die App nur ab macOS 26 und nur auf Apple Silicon. rustup ändert den Suchpfad in deiner Shell. **Vorschlag:** jetzt umstellen; ich mache das auf dein Ja.

## Was ich nicht prüfen konnte

- Die App selbst habe ich nicht gesehen, der Bildschirmzugriff war gesperrt. Laut Protokoll startet die 3D-Ansicht ohne Fehler (1240 × 1000 Pixel, Metal).
- Unter Linux habe ich nicht gebaut. Alle Bibliotheken im Kern sind reines Rust oder C (SQLite), Probleme erwarte ich keine. Die erste Cloud-Session zeigt es.
- 120 Hz auf ProMotion-Displays: Die Ansicht zeichnet nur bei Änderungen. Ob macOS dabei die Bildrate senkt, zeigt Frage 6.

## Messwerte

| Was | Gemessen | Ziel aus PLAN.md |
|---|---|---|
| Wandänderung als Operation plus neue Geometrie | 0,09 ms | unter 50 ms |
| Ein 3D-Bild 2400 × 1600 Pixel, 4-fach geglättet | 2,7 ms | 60 bis 120 Bilder pro Sekunde |
| Aufruf von Swift in den Kern | 6 bis 12 ns | |
| Testprojekt rechnen, PDF, SVG, PNG und 3D-Bild schreiben | 0,1 s | |
