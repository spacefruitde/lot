//! IFC lesen (6.02) über IFClite (ADR 0013): Bauteile eines Fachplanermodells
//! als Dreiecksnetze in Metern, mit ausgeschnittenen Öffnungen. In Lot sind
//! sie eine Referenz: sichtbar und schneidbar, aber nicht bearbeitbar.

use ifc_lite_core::{EntityDecoder, EntityScanner};
use ifc_lite_geometry::{GeometryRouter, VoidIndex};
use rustc_hash::FxHashMap;

use crate::IoError;

/// Ein Bauteil aus der IFC-Datei.
#[derive(Debug, Clone, PartialEq)]
pub struct IfcPart {
    /// IFC-Klasse in Großbuchstaben, etwa „IFCWALL“.
    pub class: String,
    pub name: String,
    /// Eckpunkte in Metern, im Koordinatensystem der Datei.
    pub positions: Vec<[f64; 3]>,
    pub triangles: Vec<[u32; 3]>,
}

/// Die Bauteile, die Lot als Referenz zeigt: Bauteilklassen nach IFC4, ohne
/// Öffnungen, Räume und die räumliche Gliederung.
const CLASSES: [&str; 26] = [
    "IFCWALL",
    "IFCWALLSTANDARDCASE",
    "IFCSLAB",
    "IFCCOLUMN",
    "IFCBEAM",
    "IFCROOF",
    "IFCSTAIR",
    "IFCSTAIRFLIGHT",
    "IFCRAMP",
    "IFCRAMPFLIGHT",
    "IFCRAILING",
    "IFCDOOR",
    "IFCWINDOW",
    "IFCCOVERING",
    "IFCMEMBER",
    "IFCPLATE",
    "IFCFOOTING",
    "IFCPILE",
    "IFCCHIMNEY",
    "IFCCURTAINWALL",
    "IFCBUILDINGELEMENTPROXY",
    "IFCFURNISHINGELEMENT",
    "IFCFLOWSEGMENT",
    "IFCFLOWTERMINAL",
    "IFCFLOWFITTING",
    "IFCDISTRIBUTIONELEMENT",
];

/// Liest alle Bauteile einer IFC-Datei. Bauteile, deren Geometrie IFClite
/// nicht erzeugen kann, fehlen; ihre Zahl steht in `skipped`.
pub fn read_ifc(bytes: &[u8]) -> Result<(Vec<IfcPart>, usize), IoError> {
    if !bytes.starts_with(b"ISO-10303-21") {
        return Err(IoError::Ifc("keine IFC-Datei im STEP-Format".into()));
    }
    let mut decoder = EntityDecoder::new(bytes);
    let router = GeometryRouter::with_units(bytes, &mut decoder);
    let voids: FxHashMap<u32, Vec<u32>> =
        VoidIndex::from_content(bytes, &mut decoder).iter().map(|(h, v)| (h, v.to_vec())).collect();
    let mut wanted = Vec::new();
    let mut scanner = EntityScanner::new(bytes);
    while let Some((id, type_name, _, _)) = scanner.next_entity() {
        let upper = type_name.to_ascii_uppercase();
        if CLASSES.contains(&upper.as_str()) {
            wanted.push((id, upper));
        }
    }
    let mut parts = Vec::new();
    let mut skipped = 0;
    for (id, class) in wanted {
        let Ok(entity) = decoder.decode_by_id(id) else {
            skipped += 1;
            continue;
        };
        let name = entity.attributes.get(2).and_then(|a| a.as_string()).unwrap_or("").to_string();
        let mesh = match router.process_element_with_voids(&entity, &mut decoder, &voids) {
            Ok(m) => m,
            Err(_) => {
                skipped += 1;
                continue;
            }
        };
        if mesh.indices.len() < 3 {
            continue;
        }
        let o = mesh.origin;
        let raw: Vec<[f64; 3]> = mesh
            .positions
            .chunks_exact(3)
            .map(|p| [o[0] + p[0] as f64, o[1] + p[1] as f64, o[2] + p[2] as f64])
            .collect();
        // Gleiche Eckpunkte zusammenführen (IFClite legt sie je Fläche an), damit
        // Schnitte die Kanten der Nachbardreiecke wiederfinden.
        let mut index: std::collections::HashMap<[i64; 3], u32> = std::collections::HashMap::new();
        let mut positions: Vec<[f64; 3]> = Vec::new();
        let remap: Vec<u32> = raw
            .iter()
            .map(|p| {
                let key = p.map(|v| (v * 1e6).round() as i64);
                *index.entry(key).or_insert_with(|| {
                    positions.push(*p);
                    (positions.len() - 1) as u32
                })
            })
            .collect();
        let triangles: Vec<[u32; 3]> = mesh
            .indices
            .chunks_exact(3)
            .map(|t| [remap[t[0] as usize], remap[t[1] as usize], remap[t[2] as usize]])
            .filter(|t| t[0] != t[1] && t[1] != t[2] && t[0] != t[2])
            .collect();
        parts.push(IfcPart { class, name, positions, triangles });
    }
    Ok((parts, skipped))
}

/// Material der Referenzbauteile im Bürostandard.
pub const REFERENCE_MATERIAL: &str = "referenz";

/// Die Bauteile eines Referenzmodells als Geometrie im Lot-Modell, je Bauteil
/// eine Schicht.
pub fn reference_geometry(
    id: lot_model::ElementId,
    model: &lot_model::ReferenceModel,
    parts: &[IfcPart],
) -> Vec<lot_geom::ElementGeometry> {
    parts
        .iter()
        .enumerate()
        .map(|(layer, p)| lot_geom::ElementGeometry {
            id,
            layer,
            material: REFERENCE_MATERIAL.to_string(),
            mesh: lot_geom::TriMesh {
                positions: p.positions.iter().map(|q| model.to_model(*q)).collect(),
                triangles: p.triangles.clone(),
            },
        })
        .collect()
}

/// Geometrie aller Referenzmodelle des Projekts aus den Dateien des Projekts.
pub fn references(
    project: &lot_model::Project,
    attachments: &crate::project_file::Attachments,
) -> Vec<lot_geom::ElementGeometry> {
    project
        .reference_models()
        .filter_map(|(id, r)| {
            let (parts, _) = read_ifc(attachments.get(&r.source)?).ok()?;
            Some(reference_geometry(id, r, &parts))
        })
        .flatten()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_structural_model_of_the_extension_in_metres() {
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../testprojekte/umbau/tragwerk-anbau.ifc");
        let (parts, skipped) = read_ifc(&std::fs::read(path).unwrap()).unwrap();
        assert_eq!(skipped, 0);
        let classes: Vec<&str> = parts.iter().map(|p| p.class.as_str()).collect();
        assert_eq!(classes, ["IFCCOLUMN", "IFCSLAB", "IFCBEAM"]);
        let column = &parts[0];
        assert_eq!(column.name, "Stuetze S1");
        let (lo, hi) = column.positions.iter().fold(([f64::MAX; 3], [f64::MIN; 3]), |(lo, hi), p| {
            ([lo[0].min(p[0]), lo[1].min(p[1]), lo[2].min(p[2])], [hi[0].max(p[0]), hi[1].max(p[1]), hi[2].max(p[2])])
        });
        // 20/20 um (13,0 | 1,1), 2,78 m hoch.
        assert!((lo[0] - 12.9).abs() < 1e-3 && (hi[0] - 13.1).abs() < 1e-3, "{lo:?} {hi:?}");
        assert!(lo[2].abs() < 1e-3 && (hi[2] - 2.78).abs() < 1e-3, "{lo:?} {hi:?}");
        // Ein Quader: 8 gemeinsame Ecken, 12 Dreiecke.
        assert_eq!((column.positions.len(), column.triangles.len()), (8, 12));
        assert!(read_ifc(b"kein IFC").is_err());
    }
}
