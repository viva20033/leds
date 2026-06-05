use clap::{Parser, Subcommand};
use leds_core::{load_default_catalog, run_pipeline, RunOptions};
use leds_placement::PlacementMode;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "leds", about = "LED Engineering & Design System CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Import SVG/DXF and print geometry summary
    Import {
        path: PathBuf,
        #[arg(long, default_value = "json")]
        format: String,
    },
    /// List catalog modules
    Catalog {
        #[arg(long, default_value = "table")]
        format: String,
    },
    /// Full pipeline: import → place → simulate → power → cost
    Run {
        path: PathBuf,
        #[arg(long, default_value_t = 100.0)]
        depth: f64,
        #[arg(long, default_value_t = 15.0)]
        rim: f64,
        #[arg(long)]
        module: Option<String>,
    },
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .try_init()
        .ok();

    let cli = Cli::parse();
    if let Err(e) = dispatch(cli) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn dispatch(cli: Cli) -> leds_core::Result<()> {
    match cli.command {
        Commands::Import { path, format } => {
            let result = leds_import::import_file(&path)?;
            if format == "json" {
                println!("{}", serde_json::to_string_pretty(&result).unwrap());
            } else {
                println!(
                    "imported {} contours from {}",
                    result.contours.len(),
                    result.source
                );
                for c in &result.contours {
                    println!("  {} — {} pts, area {:.0} mm²", c.id, c.points.len(), c.area_mm2());
                }
            }
        }
        Commands::Catalog { format } => {
            let catalog = load_default_catalog()?;
            if format == "json" {
                println!("{}", serde_json::to_string_pretty(&catalog.modules).unwrap());
            } else {
                println!("{:<40} {:>6} {:>8} {:>6} {:>8}", "ID", "LED", "₽", "pitch", "depth");
                for m in &catalog.modules {
                    let led = m.description.as_deref().unwrap_or("?");
                    let price = m.pricing.unit_price.unwrap_or(0.0);
                    println!(
                        "{:<40} {:>6} {:>8.0} {:>6.0} {:>3.0}-{:>3.0}",
                        m.id,
                        m.footprint.length_mm as u32,
                        price,
                        m.placement.recommended_pitch_mm,
                        m.placement.depth_min_mm,
                        m.placement.depth_max_mm
                    );
                    let _ = led;
                }
            }
        }
        Commands::Run {
            path,
            depth,
            rim,
            module,
        } => {
            let catalog = load_default_catalog()?;
            let report = run_pipeline(
                path.to_str().unwrap(),
                &catalog,
                &RunOptions {
                    depth_mm: depth,
                    rim_width_mm: rim,
                    module_id: module,
                    mode: PlacementMode::Auto,
                },
            )?;
            let summary = serde_json::json!({
                "source": report.import.source,
                "contours": report.import.contours.len(),
                "groups": report.shape_groups.len(),
                "min_width_mm": report.min_width_mm,
                "module": report.suggested_module_id,
                "modules_placed": report.placement.module_count,
                "coverage": report.placement.coverage_estimate,
                "uniformity": report.simulation.uniformity_index,
                "alerts": report.simulation.alerts.len(),
                "power_w": report.power.total_power_w,
                "psu_count": report.power.psu_count_estimate,
                "cost_rub": report.cost.total_cost,
                "cost_breakdown": report.cost,
            });
            println!("{}", serde_json::to_string_pretty(&summary).unwrap());
        }
    }
    Ok(())
}
