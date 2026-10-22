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

    for output in outputs(project_dir)? {
        if let Output::View3d { name } = &output {
            if with_3d {
                match render_3d(&project) {
                    Ok(png) => write(&out.join(format!("{name}.png")), &png)?,
                    Err(e) => eprintln!("Hinweis: kein 3D-Bild ({e})"),
                }
            }
            continue;
        }
        if let Output::PlanView { name, storey, scale } = &output {
            if with_3d {
                match render_plan_view(&project, lot_model::ElementId(*storey), *scale) {
                    Ok(png) => write(&out.join(format!("{name}.png")), &png)?,
                    Err(e) => eprintln!("Hinweis: keine Planansicht ({e})"),
                }
            }
            continue;
        }
        let pages = sheets(&project, &output, &source);
        write(&out.join(format!("{}.pdf", output.name())), &lot_io::pdf::to_pdf_pages(&pages)?)?;
        for (sheet, page) in pages.iter().zip(page_names(output.name(), pages.len())) {
            write(&out.join(format!("{page}.svg")), lot_io::svg::to_svg(sheet).as_bytes())?;
            write(&out.join(format!("{page}.png")), &lot_io::png::to_png(sheet, dpi)?)?;
        }
    }

    let doc = Document::new(project.clone());
    let lot_file = out.join("projekt.lot");
    lot_io::project_file::save(&doc, &lot_file)?;
    println!("geschrieben: {}", lot_file.display());
    println!("fertig in {:.0} ms", start.elapsed().as_secs_f64() * 1000.0);
    Ok(())
}

/// Ansicht des Modells als PNG, ohne Fenster gerechnet.
fn render_3d(project: &lot_model::Project) -> Result<Vec<u8>> {
    let (width, height) = (1600, 1000);
    let geometry = lot_geom::build(project)?;
    let mut renderer = lot_render::Renderer::headless(width, height)?;
    let standard = lot_plan::standard::OfficeStandard::default_standard();
    renderer.set_geometry(&geometry, &|m| standard.material(m).color);
    renderer.zoom_to_fit();
    encode_png(renderer.render_image()?, width, height)
}

/// Planansicht wie in der App, als PNG.
fn render_plan_view(project: &lot_model::Project, storey: lot_model::ElementId, scale: f64) -> Result<Vec<u8>> {
    let (width, height) = (1600, 1000);
    let mut renderer = lot_render::Renderer::headless(width, height)?;
    let standard = lot_plan::standard::OfficeStandard::default_standard();
    let view =
        lot_plan::floor_plan::FloorPlanView { storey, cut_height: lot_plan::floor_plan::DEFAULT_CUT_HEIGHT, scale };
    let viewport = lot_plan::Viewport { scale, origin: lot_plan::PaperPoint::new(0.0, 0.0) };
    renderer.set_plan(lot_plan::floor_plan::floor_plan(project, view, standard, viewport));
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
