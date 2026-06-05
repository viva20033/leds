use crate::{PlacementError, Result};
use leds_catalog::LedModule;
use leds_geometry::contour::{Point2, ProductParams, ShapeGroup};
use leds_geometry::{build_safe_zone, compute_distance_field, point_in_shape};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PlacementMode {
    Auto,
    SemiAuto,
    Expert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModulePlacement {
    pub id: String,
    pub module_id: String,
    pub x: f64,
    pub y: f64,
    pub angle_deg: f64,
    pub fixed: bool,
    pub user_placed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlacementResult {
    pub placements: Vec<ModulePlacement>,
    pub module_count: usize,
    pub pitch_used_mm: f64,
    pub coverage_estimate: f64,
    pub notes: Vec<String>,
}

pub struct PlacementOptions {
    pub mode: PlacementMode,
    pub cell_mm: f64,
    pub existing: Vec<ModulePlacement>,
}

impl Default for PlacementOptions {
    fn default() -> Self {
        Self {
            mode: PlacementMode::Auto,
            cell_mm: 5.0,
            existing: Vec::new(),
        }
    }
}

/// HAMP v0.1 — medial seeds from distance field + adaptive pitch along scan lines.
pub fn place_modules(
    group: &ShapeGroup,
    params: &ProductParams,
    module: &LedModule,
    options: &PlacementOptions,
) -> Result<PlacementResult> {
    let extent = module.footprint.max_extent_mm();
    let safe = build_safe_zone(
        group,
        params.rim_width_mm,
        extent,
        params.safety_margin_mm,
    )?;

    if safe.len() < 3 {
        return Err(PlacementError::NoSafeZone);
    }

    let pitch = module.placement.recommended_pitch_mm;
    let min_pitch = module.placement.min_pitch_mm;
    let field = compute_distance_field(group, &safe, options.cell_mm);
    let threshold = (min_pitch * 0.5) as f32;
    let mut seeds = field.local_maxima(threshold);

    if seeds.is_empty() {
        seeds.push(centroid_of(&safe));
    }

    let mut placements = match options.mode {
        PlacementMode::Expert => options.existing.clone(),
        _ => Vec::new(),
    };

    if options.mode != PlacementMode::Expert {
        placements.extend(seed_placements(&seeds, module, pitch, min_pitch));
        fill_gaps(&mut placements, group, &safe, module, min_pitch, &field);
    }

    // Respect fixed modules from semi-auto
    for fixed in &options.existing {
        if fixed.fixed {
            if let Some(p) = placements.iter_mut().find(|p| p.id == fixed.id) {
                p.x = fixed.x;
                p.y = fixed.y;
                p.angle_deg = fixed.angle_deg;
                p.fixed = true;
            } else {
                placements.push(fixed.clone());
            }
        }
    }

    prune_overcrowding(&mut placements, min_pitch * 0.7);

    let coverage = estimate_coverage(&placements, module, &safe);
    Ok(PlacementResult {
        module_count: placements.len(),
        pitch_used_mm: pitch,
        coverage_estimate: coverage,
        notes: vec![format!(
            "HAMP v0.1: {} seeds, pitch {} mm",
            seeds.len(),
            pitch
        )],
        placements,
    })
}

fn seed_placements(
    seeds: &[Point2],
    module: &LedModule,
    pitch: f64,
    min_pitch: f64,
) -> Vec<ModulePlacement> {
    let mut out = Vec::new();
    for (i, seed) in seeds.iter().enumerate() {
        out.push(ModulePlacement {
            id: Uuid::new_v4().to_string(),
            module_id: module.id.clone(),
            x: seed.x,
            y: seed.y,
            angle_deg: 0.0,
            fixed: false,
            user_placed: false,
        });
        // radial fill along 4 directions from seed
        for (dx, dy) in [(1.0, 0.0), (-1.0, 0.0), (0.0, 1.0), (0.0, -1.0)] {
            let mut t = pitch;
            while t < pitch * 3.0 {
                let p = Point2 {
                    x: seed.x + dx * t,
                    y: seed.y + dy * t,
                };
                if out.iter().any(|pl| dist(pl, &p) < min_pitch) {
                    t += pitch;
                    continue;
                }
                out.push(ModulePlacement {
                    id: Uuid::new_v4().to_string(),
                    module_id: module.id.clone(),
                    x: p.x,
                    y: p.y,
                    angle_deg: if dx != 0.0 { 0.0 } else { 90.0 },
                    fixed: false,
                    user_placed: false,
                });
                t += pitch;
            }
        }
        let _ = i;
    }
    out
}

fn fill_gaps(
    placements: &mut Vec<ModulePlacement>,
    group: &ShapeGroup,
    safe: &[Point2],
    module: &LedModule,
    min_pitch: f64,
    field: &leds_geometry::DistanceField,
) {
    let max_add = 50;
    let mut added = 0;
    for _ in 0..max_add {
        let mut best: Option<(Point2, f32)> = None;
        for y in 0..field.height {
            for x in 0..field.width {
                let wx = field.origin_x + x as f64 * field.cell_mm;
                let wy = field.origin_y + y as f64 * field.cell_mm;
                let p = Point2 { x: wx, y: wy };
                if !point_in_shape(group, &p) || !point_in_safe(safe, &p) {
                    continue;
                }
                if placements.iter().any(|pl| dist(pl, &p) < min_pitch) {
                    continue;
                }
                let d = field.values[y * field.width + x];
                if d < min_pitch as f32 * 0.5 {
                    continue;
                }
                let nearest = placements
                    .iter()
                    .map(|pl| dist(pl, &p))
                    .min_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap_or(f64::MAX);
                if nearest > module.placement.recommended_pitch_mm * 1.1 {
                    if best.map(|(_, bd)| d > bd).unwrap_or(true) {
                        best = Some((p, d));
                    }
                }
            }
        }
        let Some((p, _)) = best else { break };
        placements.push(ModulePlacement {
            id: Uuid::new_v4().to_string(),
            module_id: module.id.clone(),
            x: p.x,
            y: p.y,
            angle_deg: 0.0,
            fixed: false,
            user_placed: false,
        });
        added += 1;
        if added >= max_add {
            break;
        }
    }
}

fn prune_overcrowding(placements: &mut Vec<ModulePlacement>, min_dist: f64) {
    placements.sort_by(|a, b| a.id.cmp(&b.id));
    let mut kept: Vec<ModulePlacement> = Vec::new();
    'outer: for p in placements.drain(..) {
        if p.fixed {
            kept.push(p);
            continue;
        }
        for k in &kept {
            if dist(k, &Point2 { x: p.x, y: p.y }) < min_dist {
                continue 'outer;
            }
        }
        kept.push(p);
    }
    *placements = kept;
}

fn dist(p: &ModulePlacement, pt: &Point2) -> f64 {
    (p.x - pt.x).hypot(p.y - pt.y)
}

fn centroid_of(pts: &[Point2]) -> Point2 {
    let (x, y) = pts.iter().fold((0.0, 0.0), |(sx, sy), p| (sx + p.x, sy + p.y));
    let n = pts.len() as f64;
    Point2 { x: x / n, y: y / n }
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

fn estimate_coverage(placements: &[ModulePlacement], module: &LedModule, safe: &[Point2]) -> f64 {
    if safe.is_empty() {
        return 0.0;
    }
    let sigma = module
        .light_model
        .params
        .get("sigmaMm")
        .and_then(|v| v.as_f64())
        .unwrap_or(50.0);
    let r = sigma * 2.0;
    let step = 10.0;
    let (min_x, min_y, max_x, max_y) = bbox(safe);
    let mut covered = 0u32;
    let mut total = 0u32;
    let mut y = min_y;
    while y <= max_y {
        let mut x = min_x;
        while x <= max_x {
            let p = Point2 { x, y };
            if point_in_safe(safe, &p) {
                total += 1;
                if placements.iter().any(|pl| {
                    (pl.x - x).hypot(pl.y - y) <= r
                }) {
                    covered += 1;
                }
            }
            x += step;
        }
        y += step;
    }
    if total == 0 {
        0.0
    } else {
        covered as f64 / total as f64
    }
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
