use crate::{CoreError, Result};
use leds_catalog::{load_catalog, module_for_depth, Catalog, LedModule};
use leds_electrical::{estimate_cost, plan_power};
use leds_geometry::contour::ProductParams;
use leds_geometry::{build_safe_zone, build_topology, estimate_min_width};
use leds_import::import_file;
use leds_lighting::{simulate, SimulationSettings};
use leds_placement::{place_modules, PlacementMode, PlacementOptions};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectReport {
    pub import: leds_import::ImportResult,
    pub shape_groups: Vec<leds_geometry::ShapeGroup>,
    pub min_width_mm: Vec<f64>,
    pub suggested_module_id: Option<String>,
    pub placement: leds_placement::PlacementResult,
    pub simulation: leds_lighting::SimulationResult,
    pub power: leds_electrical::PowerPlan,
    pub cost: leds_electrical::CostBreakdown,
}

pub struct RunOptions {
    pub depth_mm: f64,
    pub rim_width_mm: f64,
    pub module_id: Option<String>,
    pub mode: PlacementMode,
}

impl Default for RunOptions {
    fn default() -> Self {
        Self {
            depth_mm: 100.0,
            rim_width_mm: 15.0,
            module_id: None,
            mode: PlacementMode::Auto,
        }
    }
}

pub fn run_pipeline(
    svg_path: &str,
    catalog: &Catalog,
    options: &RunOptions,
) -> Result<ProjectReport> {
    let import = import_file(svg_path)?;
    let groups = build_topology(import.contours.clone());
    if groups.is_empty() {
        return Err(CoreError::Other("no shape groups after topology".into()));
    }

    let min_widths: Vec<f64> = groups
        .iter()
        .map(|g| estimate_min_width(g, 16))
        .collect();

    let module = resolve_module(catalog, options)?;
    let params = ProductParams {
        depth_mm: options.depth_mm,
        rim_width_mm: options.rim_width_mm,
        safety_margin_mm: 2.0,
    };

    let group = &groups[0];
    let extent = module.footprint.max_extent_mm();
    let safe = build_safe_zone(group, params.rim_width_mm, extent, params.safety_margin_mm)?;

    let placement = place_modules(
        group,
        &params,
        module,
        &PlacementOptions {
            mode: options.mode,
            ..Default::default()
        },
    )?;

    let sim = simulate(
        group,
        &safe,
        &placement.placements,
        &[module],
        &SimulationSettings {
            depth_mm: options.depth_mm,
            ..Default::default()
        },
    );

    let power = plan_power(&placement.placements, module);
    let cost = estimate_cost(&placement.placements, module, power.psu_count_estimate);

    Ok(ProjectReport {
        import,
        shape_groups: groups,
        min_width_mm: min_widths,
        suggested_module_id: Some(module.id.clone()),
        placement,
        simulation: sim,
        power,
        cost,
    })
}

fn resolve_module<'a>(catalog: &'a Catalog, options: &RunOptions) -> Result<&'a LedModule> {
    if let Some(id) = &options.module_id {
        return catalog
            .get(id)
            .ok_or_else(|| CoreError::Other(format!("module not found: {id}")));
    }
    module_for_depth(catalog, options.depth_mm)
        .ok_or_else(|| CoreError::Other("no module for depth".into()))
}

pub fn load_default_catalog() -> Result<Catalog> {
    Ok(load_catalog(leds_catalog::default_catalog_path())?)
}
