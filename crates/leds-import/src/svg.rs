use crate::{ImportError, ImportResult, ImportedLayer, Result};
use leds_geometry::contour::{ContourRing, Point2, RingKind};
use std::path::Path;
use usvg::tiny_skia_path::{PathSegment, Transform};
use usvg::{Node, Tree};

pub fn import_svg(path: &Path) -> Result<ImportResult> {
    let data = std::fs::read(path)?;
    let tree = Tree::from_data(&data, &usvg::Options::default())
        .map_err(|e| ImportError::Svg(e.to_string()))?;

    let mut contours = Vec::new();
    let mut warnings = Vec::new();
    let mut idx = 0usize;

    walk_node(
        tree.root(),
        Transform::identity(),
        "root",
        &mut contours,
        &mut idx,
        &mut warnings,
    );

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

fn walk_node(
    node: &usvg::Node,
    transform: Transform,
    layer: &str,
    out: &mut Vec<ContourRing>,
    idx: &mut usize,
    warnings: &mut Vec<String>,
) {
    match node {
        Node::Group(g) => {
            let t = transform.pre_concat(g.transform());
            for child in g.children() {
                walk_node(child, t, layer, out, idx, warnings);
            }
        }
        Node::Path(p) => {
            let t = transform.pre_concat(p.transform());
            if let Some(path) = p.data().transform(t) {
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

fn path_to_ring(path: &usvg::tiny_skia_path::Path, layer: &str, idx: &usize) -> Option<ContourRing> {
    let mut points = Vec::new();
    for seg in path.segments() {
        match seg {
            PathSegment::LineTo(p) => points.push(Point2 { x: p.x as f64, y: p.y as f64 }),
            PathSegment::QuadTo(p, _) => points.push(Point2 { x: p.x as f64, y: p.y as f64 }),
            PathSegment::CubicTo(p, _, _) => points.push(Point2 { x: p.x as f64, y: p.y as f64 }),
            PathSegment::MoveTo(p) => {
                if !points.is_empty() {
                    // new subpath — for v0 take first closed only
                }
                points.push(Point2 { x: p.x as f64, y: p.y as f64 });
            }
        }
    }

    if points.len() < 3 {
        return None;
    }

    let first = points.first().copied().unwrap();
    let last = points.last().copied().unwrap();
    let closed = (first.x - last.x).hypot(first.y - last.y) < 0.5 || path.contains_close();
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
    let mut out = Vec::with_capacity(pts.len());
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
