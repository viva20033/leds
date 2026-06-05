use leds_catalog::LedModule;
use leds_geometry::{Point2, ShapeGroup};
use leds_placement::ModulePlacement;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationSettings {
    pub cell_mm: f64,
    pub depth_mm: f64,
    pub i_min_ratio: f64,
    pub i_max_ratio: f64,
}

impl Default for SimulationSettings {
    fn default() -> Self {
        Self {
            cell_mm: 5.0,
            depth_mm: 100.0,
            i_min_ratio: 0.7,
            i_max_ratio: 1.35,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightAlert {
    pub alert_type: String,
    pub x: f64,
    pub y: f64,
    pub severity: f64,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    pub width: usize,
    pub height: usize,
    pub origin_x: f64,
    pub origin_y: f64,
    pub cell_mm: f64,
    pub values: Vec<f32>,
    pub min_illuminance: f32,
    pub max_illuminance: f32,
    pub mean_illuminance: f32,
    pub uniformity_index: f32,
    pub alerts: Vec<LightAlert>,
}

pub fn simulate(
    _group: &ShapeGroup,
    safe: &[Point2],
    placements: &[ModulePlacement],
    modules: &[&LedModule],
    settings: &SimulationSettings,
) -> SimulationResult {
    let (min_x, min_y, max_x, max_y) = bbox(safe);
    let margin = 20.0;
    let origin_x = min_x - margin;
    let origin_y = min_y - margin;
    let cell = settings.cell_mm.max(10.0);
    let width = ((max_x - min_x + 2.0 * margin) / cell).ceil() as usize + 1;
    let height = ((max_y - min_y + 2.0 * margin) / cell).ceil() as usize + 1;
    let width = width.min(200);
    let height = height.min(200);
    let n = width * height;

    let values: Vec<f32> = (0..n)
        .map(|i| {
            let x = i % width;
            let y = i / width;
            let wx = origin_x + x as f64 * cell;
            let wy = origin_y + y as f64 * cell;
            let p = Point2 { x: wx, y: wy };
            if !point_in_safe(safe, &p) {
                return 0.0;
            }
            let mut sum = 0.0f32;
            for pl in placements {
                let module = modules
                    .iter()
                    .find(|m| m.id == pl.module_id)
                    .copied()
                    .unwrap_or(modules[0]);
                sum += spot_intensity(module, pl, &p, settings.depth_mm);
            }
            sum
        })
        .collect();

    let mask_vals: Vec<f32> = values.iter().copied().filter(|&v| v > 0.0).collect();
    let mean = if mask_vals.is_empty() {
        0.0
    } else {
        mask_vals.iter().sum::<f32>() / mask_vals.len() as f32
    };
    let min_i = mask_vals.iter().copied().fold(f32::MAX, f32::min);
    let max_i = mask_vals.iter().copied().fold(0.0f32, f32::max);
    let std = if mask_vals.len() > 1 {
        let var = mask_vals
            .iter()
            .map(|v| {
                let d = *v - mean;
                d * d
            })
            .sum::<f32>()
            / mask_vals.len() as f32;
        var.sqrt()
    } else {
        1.0
    };
    let uniformity = if std > 1e-6 { mean / std } else { 0.0 };

    let alerts = detect_alerts(
        &values,
        width,
        height,
        origin_x,
        origin_y,
        settings.cell_mm.max(10.0),
        mean,
        settings.i_min_ratio,
        settings.i_max_ratio,
        25,
    );

    SimulationResult {
        width,
        height,
        origin_x,
        origin_y,
        cell_mm: cell,
        values,
        min_illuminance: if min_i == f32::MAX { 0.0 } else { min_i },
        max_illuminance: max_i,
        mean_illuminance: mean,
        uniformity_index: uniformity,
        alerts,
    }
}

fn spot_intensity(module: &LedModule, pl: &ModulePlacement, p: &Point2, depth_mm: f64) -> f32 {
    let sigma0 = module
        .light_model
        .params
        .get("sigmaMm")
        .and_then(|v| v.as_f64())
        .unwrap_or(40.0) as f32;
    let k_sigma = module
        .light_model
        .params
        .get("kSigma")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.12) as f32;
    let sigma = sigma0 + k_sigma * depth_mm as f32;
    let lumens = module.electrical.lumens as f32;
    let dx = (p.x - pl.x) as f32;
    let dy = (p.y - pl.y) as f32;
    let r2 = dx * dx + dy * dy;
    let gauss = (-r2 / (2.0 * sigma * sigma)).exp();
    lumens * gauss
}

fn detect_alerts(
    values: &[f32],
    width: usize,
    height: usize,
    ox: f64,
    oy: f64,
    cell: f64,
    mean: f32,
    i_min: f64,
    i_max: f64,
    max_alerts: usize,
) -> Vec<LightAlert> {
    let mut alerts = Vec::new();
    if mean <= 0.0 {
        return alerts;
    }
    'scan: for y in (0..height).step_by(2) {
        for x in (0..width).step_by(2) {
            let v = values[y * width + x];
            if v <= 0.0 {
                continue;
            }
            let ratio = v / mean;
            let wx = ox + x as f64 * cell;
            let wy = oy + y as f64 * cell;
            if ratio < i_min as f32 {
                alerts.push(LightAlert {
                    alert_type: "underlit".into(),
                    x: wx,
                    y: wy,
                    severity: (i_min as f32 - ratio).into(),
                    message: "Недостаточная засветка".into(),
                });
            } else if ratio > i_max as f32 {
                alerts.push(LightAlert {
                    alert_type: "overlit".into(),
                    x: wx,
                    y: wy,
                    severity: (ratio - i_max as f32).into(),
                    message: "Переуплотнение / риск пятна".into(),
                });
            }
            if alerts.len() >= max_alerts {
                break 'scan;
            }
        }
    }
    alerts
}

fn bbox(pts: &[Point2]) -> (f64, f64, f64, f64) {
    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;
    for p in pts {
        min_x = min_x.min(p.x);
        min_y = min_y.min(p.y);
        max_x = max_x.max(p.x);
        max_y = max_y.max(p.y);
    }
    (min_x, min_y, max_x, max_y)
}

fn point_in_safe(poly: &[Point2], p: &Point2) -> bool {
    if poly.len() < 3 {
        return false;
    }
    let mut inside = false;
    let n = poly.len();
    for i in 0..n {
        let j = (i + 1) % n;
        let xi = poly[i].x;
        let yi = poly[i].y;
        let xj = poly[j].x;
        let yj = poly[j].y;
        if ((yi > p.y) != (yj > p.y))
            && (p.x < (xj - xi) * (p.y - yi) / (yj - yi + 1e-12) + xi)
        {
            inside = !inside;
        }
    }
    inside
}
