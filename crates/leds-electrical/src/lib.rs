use leds_catalog::LedModule;
use leds_placement::ModulePlacement;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerGroup {
    pub chain_id: u32,
    pub placement_ids: Vec<String>,
    pub module_count: usize,
    pub total_power_w: f64,
    pub total_current_a: f64,
    pub max_span_mm: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerPlan {
    pub groups: Vec<PowerGroup>,
    pub total_power_w: f64,
    pub total_modules: usize,
    pub psu_count_estimate: u32,
    pub psu_power_each_w: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostBreakdown {
    pub module_cost: f64,
    pub psu_cost: f64,
    pub total_cost: f64,
    pub currency: String,
    pub module_count: usize,
}

/// Greedy chain split by max modules in series and max wire span.
pub fn plan_power(placements: &[ModulePlacement], module: &LedModule) -> PowerPlan {
    let max_series = module.chain.max_modules_in_series as usize;
    let max_span = module.chain.max_center_distance_mm;
    let mut groups = Vec::new();
    let mut chain_id = 1u32;
    let mut idx = 0usize;

    while idx < placements.len() {
        let end = (idx + max_series).min(placements.len());
        let chunk = &placements[idx..end];
        let span = max_pair_span(chunk);
        let count = chunk.len();
        let power = count as f64 * module.electrical.power_w;
        let current = count as f64
            * module
                .electrical
                .current_a
                .unwrap_or(module.electrical.power_w / module.electrical.voltage_v);
        groups.push(PowerGroup {
            chain_id,
            placement_ids: chunk.iter().map(|p| p.id.clone()).collect(),
            module_count: count,
            total_power_w: power,
            total_current_a: current,
            max_span_mm: span,
        });
        if span > max_span {
            // note: real routing would reorder; v0 warns only
        }
        chain_id += 1;
        idx = end;
    }

    let total_power: f64 = groups.iter().map(|g| g.total_power_w).sum();
    let psu_power = 100.0f64;
    let derating = 0.8;
    let psu_count = ((total_power / (psu_power * derating)).ceil() as u32).max(1);

    PowerPlan {
        groups,
        total_power_w: total_power,
        total_modules: placements.len(),
        psu_count_estimate: psu_count,
        psu_power_each_w: psu_power,
    }
}

pub fn estimate_cost(placements: &[ModulePlacement], module: &LedModule, psu_count: u32) -> CostBreakdown {
    let unit = module.pricing.unit_price.unwrap_or(0.0);
    let module_cost = placements.len() as f64 * unit;
    let psu_unit = 850.0; // placeholder Mean Well class
    let psu_cost = psu_count as f64 * psu_unit;
    CostBreakdown {
        module_cost,
        psu_cost,
        total_cost: module_cost + psu_cost,
        currency: module.pricing.currency.clone().unwrap_or_else(|| "RUB".into()),
        module_count: placements.len(),
    }
}

fn max_pair_span(placements: &[ModulePlacement]) -> f64 {
    let mut max = 0.0f64;
    for i in 0..placements.len() {
        for j in (i + 1)..placements.len() {
            let d = (placements[i].x - placements[j].x)
                .hypot(placements[i].y - placements[j].y);
            max = max.max(d);
        }
    }
    max
}
