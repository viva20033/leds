use crate::contour::{ContourRing, Point2, RingKind, ShapeGroup};
use geo::{Contains, Coord, LineString, Point, Polygon};

/// Assign outer/hole kinds using even-odd nesting by area and containment.
pub fn build_topology(mut rings: Vec<ContourRing>) -> Vec<ShapeGroup> {
    rings.sort_by(|a, b| {
        b.area_mm2()
            .partial_cmp(&a.area_mm2())
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let polys: Vec<Option<Polygon<f64>>> = rings.iter().map(|r| r.to_polygon()).collect();
    let mut groups = Vec::new();
    let mut used = vec![false; rings.len()];

    for i in 0..rings.len() {
        if used[i] || polys[i].is_none() {
            continue;
        }
        let mut outer = rings[i].clone();
        outer.kind = RingKind::Outer;
        let mut holes = Vec::new();

        for j in (i + 1)..rings.len() {
            if used[j] || polys[j].is_none() {
                continue;
            }
            let outer_poly = polys[i].as_ref().unwrap();
            let inner_poly = polys[j].as_ref().unwrap();
            let sample = inner_poly.exterior().0.first().copied().unwrap_or(Coord {
                x: 0.0,
                y: 0.0,
            });
            if outer_poly.contains(&Point::from(sample)) {
                let mut hole = rings[j].clone();
                hole.kind = RingKind::Hole;
                holes.push(hole);
                used[j] = true;
            }
        }

        used[i] = true;
        groups.push(ShapeGroup { outer, holes });
    }

    groups
}

/// Minimum width estimate: bbox diagonal heuristic + ray cast samples.
pub fn estimate_min_width(group: &ShapeGroup, samples: usize) -> f64 {
    let outer = match group.outer.to_polygon() {
        Some(p) => p,
        None => return 0.0,
    };
    let c = group.outer.centroid();
    let mut min_w = f64::MAX;

    for k in 0..samples.max(8) {
        let angle = std::f64::consts::TAU * (k as f64) / (samples as f64);
        let dx = angle.cos();
        let dy = angle.sin();
        if let Some(w) = ray_width(&outer, &group.holes, c.x, c.y, dx, dy) {
            min_w = min_w.min(w);
        }
    }

    if min_w == f64::MAX {
        0.0
    } else {
        min_w
    }
}

fn ray_width(
    outer: &Polygon<f64>,
    holes: &[ContourRing],
    ox: f64,
    oy: f64,
    dx: f64,
    dy: f64,
) -> Option<f64> {
    let mut ts = Vec::new();
    collect_line_hits(&outer.exterior(), ox, oy, dx, dy, &mut ts);
    for hole in holes {
        if let Some(poly) = hole.to_polygon() {
            collect_line_hits(&poly.exterior(), ox, oy, dx, dy, &mut ts);
        }
    }
    ts.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    if ts.len() < 2 {
        return None;
    }
    // first segment inside outer without hole logic simplified: max gap between pairs
    let mut best = 0.0f64;
    for w in ts.windows(2) {
        best = best.max((w[1] - w[0]).abs());
    }
    Some(best)
}

fn collect_line_hits(line: &LineString<f64>, ox: f64, oy: f64, dx: f64, dy: f64, out: &mut Vec<f64>) {
    let pts = &line.0;
    for w in pts.windows(2) {
        if let Some(t) = intersect_ray_segment(ox, oy, dx, dy, w[0].x, w[0].y, w[1].x, w[1].y) {
            out.push(t);
        }
    }
}

fn intersect_ray_segment(
    ox: f64,
    oy: f64,
    dx: f64,
    dy: f64,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
) -> Option<f64> {
    let sx = x2 - x1;
    let sy = y2 - y1;
    let denom = dx * sy - dy * sx;
    if denom.abs() < 1e-9 {
        return None;
    }
    let t = ((x1 - ox) * sy - (y1 - oy) * sx) / denom;
    let u = ((x1 - ox) * dy - (y1 - oy) * dx) / denom;
    if t >= 0.0 && (0.0..=1.0).contains(&u) {
        Some(t)
    } else {
        None
    }
}

pub fn point_in_shape(group: &ShapeGroup, p: &Point2) -> bool {
    let pt = Point::new(p.x, p.y);
    let outer = match group.outer.to_polygon() {
        Some(p) => p,
        None => return false,
    };
    if !outer.contains(&pt) {
        return false;
    }
    for hole in &group.holes {
        if let Some(hp) = hole.to_polygon() {
            if hp.contains(&pt) {
                return false;
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contour::Point2;

    fn rect_ring(id: &str, x: f64, y: f64, w: f64, h: f64) -> ContourRing {
        ContourRing {
            id: id.into(),
            kind: RingKind::Outer,
            points: vec![
                Point2 { x, y },
                Point2 { x: x + w, y },
                Point2 { x: x + w, y: y + h },
                Point2 { x, y: y + h },
            ],
            closed: true,
            layer: None,
        }
    }

    #[test]
    fn detects_hole_inside_outer() {
        let outer = rect_ring("o", 0.0, 0.0, 200.0, 200.0);
        let hole = rect_ring("h", 60.0, 60.0, 80.0, 80.0);
        let groups = build_topology(vec![outer, hole]);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].holes.len(), 1);
        assert!(!point_in_shape(
            &groups[0],
            &Point2 { x: 100.0, y: 100.0 }
        ));
        assert!(point_in_shape(
            &groups[0],
            &Point2 { x: 20.0, y: 20.0 }
        ));
    }
}
