//! Golden-Tests: Jede von Ben freigegebene Planausgabe liegt als Referenz in
//! `testprojekte/<name>/referenz/`. Dieser Test rechnet das Projekt neu und
//! vergleicht das SVG zeichengenau. Eine Referenz wird nur nach Bens
//! ausdrücklicher Freigabe geändert (CLAUDE.md, Regel 3).

use std::path::Path;

#[test]
fn approved_plans_are_unchanged() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../testprojekte");
    let mut checked = 0;
    for entry in std::fs::read_dir(&root).unwrap() {
        let dir = entry.unwrap().path();
        let reference = dir.join("referenz/m0-pruefblatt.svg");
        if !reference.is_file() {
            continue;
        }
        let project = lot_io::testproject::load(&dir).unwrap();
        let name = dir.file_name().unwrap().to_string_lossy();
        let sheet = lot_plan::m0::sheet(&project, &format!("testprojekte/{name}"));
        let actual = lot_io::svg::to_svg(&sheet);
        let expected = std::fs::read_to_string(&reference).unwrap();
        assert!(
            actual == expected,
            "{name}: Plan weicht von der freigegebenen Referenz ab. Unterschied mit \
             `cargo run -p lot-cli -- render ../testprojekte/{name} --out ../out/{name}` und \
             `diff ../out/{name}/m0-pruefblatt.svg ../testprojekte/{name}/referenz/m0-pruefblatt.svg` ansehen."
        );
        checked += 1;
    }
    println!("{checked} Referenzen geprüft");
}
