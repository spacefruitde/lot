//! Golden-Tests: Jede von Ben freigegebene Planausgabe liegt als Referenz in
//! `testprojekte/<name>/referenz/<seite>.svg`. Dieser Test rechnet alle
//! Ausgaben neu und vergleicht jedes SVG, zu dem es eine Referenz gibt,
//! zeichengenau. Eine Referenz wird nur nach Bens ausdrücklicher Freigabe
//! geändert (CLAUDE.md, Regel 3).

use std::path::Path;

#[test]
fn approved_plans_are_unchanged() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../testprojekte");
    let mut checked = 0;
    for entry in std::fs::read_dir(&root).unwrap() {
        let dir = entry.unwrap().path();
        let references = dir.join("referenz");
        if !references.is_dir() {
            continue;
        }
        let name = dir.file_name().unwrap().to_string_lossy().into_owned();
        let project = lot_io::testproject::load(&dir).unwrap();
        for output in lot_cli::outputs(&dir).unwrap() {
            let files = lot_io::testproject::attachments(&project, &dir);
            let standard = lot_cli::office_standard(&dir).expect("Bürostandard");
            let pages = lot_cli::sheets(&project, &output, &format!("testprojekte/{name}"), &files, &standard);
            for (sheet, page) in pages.iter().zip(lot_cli::page_names(output.name(), pages.len())) {
                let reference = references.join(format!("{page}.svg"));
                if !reference.is_file() {
                    continue;
                }
                let expected = std::fs::read_to_string(&reference).unwrap();
                assert!(
                    lot_io::svg::to_svg(sheet) == expected,
                    "{name}/{page}: Plan weicht von der freigegebenen Referenz ab. Unterschied mit \
                     `cargo run -p lot-cli -- render ../testprojekte/{name} --out ../out/{name}` und \
                     `diff ../out/{name}/{page}.svg ../testprojekte/{name}/referenz/{page}.svg` ansehen."
                );
                checked += 1;
            }
        }
    }
    println!("{checked} Referenzen geprüft");
}
