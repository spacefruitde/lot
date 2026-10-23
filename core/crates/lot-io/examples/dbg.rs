fn main() {
    let p = load();
    let g = lot_geom::build(&p).unwrap();
    let mut worst: Vec<(f64, String)> = Vec::new();
    for e in &g.elements {
        if let Some((lo, hi)) = e.mesh.bounds() {
            worst.push((
                lo[2].min(-hi[2] + 0.0),
                format!("{:?} {} {:?} {:?} tris {}", e.id, e.material, lo, hi, e.mesh.triangles.len()),
            ));
        }
    }
    worst.sort_by(|a, b| a.0.total_cmp(&b.0));
    for w in worst.iter().take(5) {
        println!("{}", w.1);
    }
    println!("{:?}", g.bounds());
}
fn load() -> lot_model::Project {
    lot_io::testproject::load(std::path::Path::new("../testprojekte/mfh")).unwrap()
}
