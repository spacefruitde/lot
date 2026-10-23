//! Kommandozeile von Lot: Testprojekte rechnen und Pläne ausgeben.
//!
//! Beispiel: `cargo run -p lot-cli -- render ../testprojekte/m0-wand --out ../out/m0-wand`

use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use lot_cli::{Output, outputs, page_names, sheets};
use lot_model::Document;

#[derive(Parser)]
#[command(name = "lot-cli", version, about = "Lot: Testprojekte rechnen und Pläne ausgeben")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Rechnet ein Testprojekt und schreibt seine Ausgaben (siehe ausgaben.ron):
    /// Pläne als PDF, SVG und PNG, 3D-Bilder und die Projektdatei.
    Render {
        /// Ordner des Testprojekts mit `projekt.ron`
        project: PathBuf,
        /// Zielordner für die Ausgaben
        #[arg(long)]
        out: PathBuf,
        /// Auflösung der PNG-Pläne in Pixel je Zoll
        #[arg(long, default_value_t = 200.0)]
        dpi: f64,
        /// Keine 3D-Bilder rechnen (etwa auf Rechnern ohne Grafikkarte)
        #[arg(long)]
        no_3d: bool,
    },
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Command::Render { project, out, dpi, no_3d } => render(&project, &out, dpi, !no_3d),
    }
}

fn render(project_dir: &Path, out: &Path, dpi: f64, with_3d: bool) -> Result<()> {
    let start = Instant::now();
    let project = lot_io::testproject::load(project_dir)
        .with_context(|| format!("Testprojekt {} nicht lesbar", project_dir.display()))?;
    std::fs::create_dir_all(out).with_context(|| format!("Ordner {} nicht anlegbar", out.display()))?;
    let source = project_label(project_dir);
    // Unterlagen liegen als Dateien im Projektordner.
    let attachments = lot_io::testproject::attachments(&project, project_dir);

    for output in outputs(project_dir)? {
        if let Output::View3d { name } = &output {
            if with_3d {
                match render_3d(&project, &attachments) {
                    Ok(png) => write(&out.join(format!("{name}.png")), &png)?,
                    Err(e) => eprintln!("Hinweis: kein 3D-Bild ({e})"),
                }
            }
            continue;
        }
        if let Output::PlanView { name, storey, scale, findings } = &output {
            if with_3d {
                match render_plan_view(&project, &attachments, lot_model::ElementId(*storey), *scale, *findings) {
                    Ok(png) => write(&out.join(format!("{name}.png")), &png)?,
                    Err(e) => eprintln!("Hinweis: keine Planansicht ({e})"),
                }
            }
            continue;
        }
        if let Output::OpeningSchedule { name } = &output {
            let (rows, _) = lot_rules::opening_schedule(&project.variant_view());
            write(&out.join(format!("{name}.csv")), lot_plan::table::schedule_csv(&rows).as_bytes())?;
        }
        if let Output::AreaTable { name, phase } = &output {
            let csv =
                lot_plan::area_table::area_csv(&lot_rules::area_report(&lot_cli::phase_project(&project, *phase)));
            write(&out.join(format!("{name}.csv")), csv.as_bytes())?;
        }
        for (file, bytes) in lot_cli::cad_files(&project, &output, &attachments)? {
            write(&out.join(file), &bytes)?;
        }
        let pages = sheets(&project, &output, &source, &attachments);
        write(&out.join(format!("{}.pdf", output.name())), &lot_io::pdf::to_pdf_pages(&pages)?)?;
        for (sheet, page) in pages.iter().zip(page_names(output.name(), pages.len())) {
            write(&out.join(format!("{page}.svg")), lot_io::svg::to_svg(sheet).as_bytes())?;
            write(&out.join(format!("{page}.png")), &lot_io::png::to_png(sheet, dpi)?)?;
        }
    }

    let doc = Document::new(project.clone());
    let lot_file = out.join("projekt.lot");
    lot_io::project_file::save_with(&doc, &attachments, &lot_file)?;
    println!("geschrieben: {}", lot_file.display());
    // Prüfung der Treppen nach DIN 18065.
    for c in lot_rules::check_stairs(&project) {
        let de = |v: f64, d: usize| format!("{v:.d$}").replace('.', ",");
        let clear = c.headroom.map_or("frei".to_string(), |h| format!("{} m", de(h, 2)));
        let result = if c.problems.is_empty() { "in Ordnung".to_string() } else { c.problems.join("; ") };
        println!(
            "Treppe {}: {} Stg. {}/{}, 2s + a = {} cm, Durchgangshöhe {clear}: {result}",
            c.stair.0,
            c.risers,
            de(c.rise * 100.0, 1),
            de(c.tread * 100.0, 1),
            de(c.step * 100.0, 1)
        );
    }
    println!("fertig in {:.0} ms", start.elapsed().as_secs_f64() * 1000.0);
    Ok(())
}

/// Ansicht des Modells als PNG, ohne Fenster gerechnet.
fn render_3d(project: &lot_model::Project, attachments: &lot_io::project_file::Attachments) -> Result<Vec<u8>> {
    let (width, height) = (1600, 1000);
    // Bei Umbauten der Zustand nach dem Umbau, dazu die Referenzmodelle.
    let shown = project.variant_view().phase_view(lot_model::PhaseFilter::Planned);
    let mut geometry = lot_geom::build(&shown)?;
    geometry.elements.extend(lot_io::ifc::references(&shown, attachments));
    let mut renderer = lot_render::Renderer::headless(width, height)?;
    let standard = lot_plan::standard::OfficeStandard::default_standard();
    renderer.set_geometry(&geometry, &|m| standard.material(m).color);
    renderer.zoom_to_fit();
    encode_png(renderer.render_image()?, width, height)
}

/// Planansicht wie in der App, als PNG, mit den Unterlagen des Geschosses (nur PNG).
fn render_plan_view(
    project: &lot_model::Project,
    attachments: &lot_io::project_file::Attachments,
    storey: lot_model::ElementId,
    scale: f64,
    findings: bool,
) -> Result<Vec<u8>> {
    let (width, height) = (1600, 1000);
    let mut renderer = lot_render::Renderer::headless(width, height)?;
    let standard = lot_plan::standard::OfficeStandard::default_standard();
    let view =
        lot_plan::floor_plan::FloorPlanView { storey, cut_height: lot_plan::floor_plan::DEFAULT_CUT_HEIGHT, scale };
    let viewport = lot_plan::Viewport { scale, origin: lot_plan::PaperPoint::new(0.0, 0.0) };
    // Unterlagen aus DXF und DWG als graue Linien unter dem Plan.
    let mut objects = Vec::new();
    for (_, u) in project.underlays_on(storey).filter(|(_, u)| lot_io::cad::is_cad(&u.source)) {
        if let Some(v) = attachments.get(&u.source).and_then(|b| lot_io::cad::read_vectors(b).ok()) {
            objects.extend(lot_io::cad::underlay_objects(u, &v, viewport));
        }
    }
    // Referenzmodelle in der Schnittebene.
    let z = project.storey(storey).map_or(0.0, |s| s.elevation) + lot_plan::floor_plan::DEFAULT_CUT_HEIGHT;
    lot_plan::reference::draw_reference_cut(&mut objects, &lot_io::ifc::references(project, attachments), z, viewport);
    objects.extend(lot_plan::floor_plan::floor_plan(&project.variant_view(), view, standard, viewport));
    if findings {
        let shown = project.variant_view();
        let geometry = lot_cli::geometry(&shown, attachments)?;
        let report = lot_rules::check_project(&shown, &geometry);
        lot_plan::findings::draw_findings(&mut objects, &report, storey, viewport);
    }
    renderer.set_plan(objects);
    let quads: Vec<lot_render::UnderlayQuad> = project
        .underlays_on(storey)
        .filter_map(|(_, u)| {
            let r = lot_io::raster::decode_png(attachments.get(&u.source)?).ok()?;
            let image =
                std::sync::Arc::new(lot_render::UnderlayImage { width: r.width, height: r.height, rgba: r.rgba });
            let corners = u.corners().map(|p| {
                let q = viewport.to_paper(p.x, p.y);
                [q.x, q.y]
            });
            Some(lot_render::UnderlayQuad { corners, image, opacity: u.opacity as f32 })
        })
        .collect();
    renderer.set_underlays(&quads);
    renderer.set_mode(lot_render::ViewMode::Plan);
    renderer.plan_zoom_to_fit();
    encode_png(renderer.render_image()?, width, height)
}

fn encode_png(pixels: Vec<u8>, width: u32, height: u32) -> Result<Vec<u8>> {
    let mut png_data = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut png_data, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.set_source_srgb(png::SrgbRenderingIntent::Perceptual);
        encoder.write_header()?.write_image_data(&pixels)?;
    }
    Ok(png_data)
}

/// „testprojekte/m0-wand“ statt eines absoluten Pfads, damit die Ausgabe auf jedem Rechner gleich ist.
fn project_label(dir: &Path) -> String {
    let name = dir
        .canonicalize()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .unwrap_or_else(|| dir.display().to_string());
    format!("testprojekte/{name}")
}

fn write(path: &Path, data: &[u8]) -> Result<()> {
    std::fs::write(path, data).with_context(|| format!("{} nicht schreibbar", path.display()))?;
    println!("geschrieben: {}", path.display());
    Ok(())
}
