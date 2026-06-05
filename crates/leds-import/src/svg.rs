use crate::{ImportError, ImportResult, ImportedLayer, Result};
use leds_geometry::{ContourRing, Point2, RingKind};
use std::path::Path;
use usvg::tiny_skia_path::{PathSegment, Transform};
use usvg::{Group, Node, Tree};

pub fn import_svg(path: &Path) -> Result<ImportResult> {
    let data = std::fs::read(path)?;
    let tree = Tree::from_data(&data, &usvg::Options::default())
        .map_err(|e| ImportError::Svg(e.to_string()))?;

    let mut contours = Vec::new();
    let mut warnings = Vec::new();
    let mut idx = 0usize;

    walk_group(
        tree.root(),
        Transform::identity(),
        "root",
        &mut contours,
        &mut idx,
        &mut warnings,
    );

    normalize_to_viewbox(&mut contours, &String::from_utf8_lossy(&data));

    if contours.is_empty() {
        warnings.push("no closed contours found — check SVG paths".into());
    }

    let layers = vec![ImportedLayer {
        name: "default".into(),
        visible: true,
    }];

    Ok(ImportResult {
        source: path.display().to_string(),
        format: "svg".into(),
        layers,
        contours,
        warnings,
    })
}

fn walk_group(
    group: &Group,
    transform: Transform,
    layer: &str,
    out: &mut Vec<ContourRing>,
    idx: &mut usize,
    warnings: &mut Vec<String>,
) {
    let t = transform.pre_concat(group.abs_transform());
    for child in group.children() {
        match child {
            Node::Group(g) => walk_group(g, t, layer, out, idx, warnings),
            Node::Path(p) => {
                let pt = t.pre_concat(p.abs_transform());
                if let Some(path) = p.data().clone().transform(pt) {
                    if let Some(ring) = path_to_ring(&path, layer, idx) {
                        *idx += 1;
                        out.push(ring);
                    } else {
                        warnings.push(format!("path {} skipped (open or too short)", p.id()));
                    }
                }
            }
            _ => {}
        }
    }
}

fn path_to_ring(path: &usvg::tiny_skia_path::Path, layer: &str, idx: &usize) -> Option<ContourRing> {
    let mut points = Vec::new();
    let mut has_close = false;
    for seg in path.segments() {
        match seg {
            PathSegment::LineTo(p) => points.push(Point2 { x: p.x as f64, y: p.y as f64 }),
            PathSegment::QuadTo(p, _) => points.push(Point2 { x: p.x as f64, y: p.y as f64 }),
            PathSegment::CubicTo(p, _, _) => points.push(Point2 { x: p.x as f64, y: p.y as f64 }),
            PathSegment::MoveTo(p) => points.push(Point2 { x: p.x as f64, y: p.y as f64 }),
            PathSegment::Close => has_close = true,
        }
    }

    if points.len() < 3 {
        return None;
    }

    let first = points.first().copied().unwrap();
    let last = points.last().copied().unwrap();
    let closed = has_close || (first.x - last.x).hypot(first.y - last.y) < 0.5;
    if closed && (first.x - last.x).hypot(first.y - last.y) > 0.01 {
        points.push(first);
    }

    if !closed {
        return None;
    }

    dedupe_points(&mut points);
    if points.len() < 3 {
        return None;
    }

    Some(ContourRing {
        id: format!("ring-{idx}"),
        kind: RingKind::Outer,
        points,
        closed: true,
        layer: Some(layer.into()),
    })
}

fn dedupe_points(pts: &mut Vec<Point2>) {
    const EPS: f64 = 0.01;
    let mut out: Vec<Point2> = Vec::with_capacity(pts.len());
    for p in pts.drain(..) {
        if let Some(last) = out.last() {
            if (last.x - p.x).hypot(last.y - p.y) < EPS {
                continue;
            }
        }
        out.push(p);
    }
    *pts = out;
}

/// usvg may convert mm/pt into large pixel coords — rescale to viewBox (typically mm).
fn normalize_to_viewbox(contours: &mut [ContourRing], svg_text: &str) {
    if contours.is_empty() {
        return;
    }
    let Some((vb_x, vb_y, vb_w, vb_h)) = parse_viewbox(svg_text) else {
        return;
    };
    let (min_x, min_y, max_x, max_y) = global_bbox(contours);
    let w = max_x - min_x;
    let h = max_y - min_y;
    if w <= 0.0 || h <= 0.0 {
        return;
    }
    // Already in viewBox scale (with tolerance)
    if w <= vb_w * 1.2 && h <= vb_h * 1.2 {
        return;
    }
    let sx = vb_w / w;
    let sy = vb_h / h;
    for c in contours.iter_mut() {
        for p in &mut c.points {
            p.x = vb_x + (p.x - min_x) * sx;
            p.y = vb_y + (p.y - min_y) * sy;
        }
    }
}

fn parse_viewbox(svg: &str) -> Option<(f64, f64, f64, f64)> {
    let re = regex_lite::Regex::new(r#"viewBox\s*=\s*"([^"]+)""#).ok()?;
    let caps = re.captures(svg)?;
    let nums: Vec<f64> = caps[1]
        .split_whitespace()
        .filter_map(|s| s.parse().ok())
        .collect();
    if nums.len() == 4 {
        Some((nums[0], nums[1], nums[2], nums[3]))
    } else {
        None
    }
}

fn global_bbox(contours: &[ContourRing]) -> (f64, f64, f64, f64) {
    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;
    for c in contours {
        for p in &c.points {
            min_x = min_x.min(p.x);
            min_y = min_y.min(p.y);
            max_x = max_x.max(p.x);
            max_y = max_y.max(p.y);
        }
    }
    (min_x, min_y, max_x, max_y)
}
