use crate::contour::{Point2, ShapeGroup};
use crate::topology::point_in_shape;
use crate::{GeometryError, Result};

pub fn build_safe_zone(
    group: &ShapeGroup,
    rim_width_mm: f64,
    module_extent_mm: f64,
    safety_margin_mm: f64,
) -> Result<Vec<Point2>> {
    let inset = rim_width_mm + module_extent_mm * 0.5 + safety_margin_mm;
    let outer_inset = inset_ring(&group.outer.points, -inset)?;
    if outer_inset.len() < 3 {
        return Err(GeometryError::OffsetCollapsed(
            "outer safe zone collapsed — increase depth or reduce rim".into(),
        ));
    }

    let mut result: Vec<Point2> = outer_inset
        .iter()
        .copied()
        .filter(|p| {
            for hole in &group.holes {
                let expanded = inset_ring(&hole.points, inset).unwrap_or_default();
                if expanded.len() >= 3 && point_in_ring(p, &expanded) {
                    return false;
                }
            }
            true
        })
        .collect();

    if result.len() < 3 {
        result = filter_safe_points(&outer_inset, group, inset);
    }

    if result.len() < 3 {
        return Err(GeometryError::OffsetCollapsed(
            "safe zone empty after holes".into(),
        ));
    }
    Ok(result)
}

fn filter_safe_points(outer: &[Point2], group: &ShapeGroup, hole_delta: f64) -> Vec<Point2> {
    outer
        .iter()
        .copied()
        .filter(|p| point_in_shape(group, p))
        .filter(|p| {
            !group.holes.iter().any(|h| {
                let exp = inset_ring(&h.points, hole_delta).unwrap_or_default();
                exp.len() >= 3 && point_in_ring(p, &exp)
            })
        })
        .collect()
}

fn inset_ring(points: &[Point2], delta: f64) -> Result<Vec<Point2>> {
    let n = points.len();
    if n < 3 {
        return Err(GeometryError::EmptyContour);
    }
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let prev = &points[(i + n - 1) % n];
        let curr = &points[i];
        let next = &points[(i + 1) % n];
        let e1 = normalize(Point2 {
            x: curr.x - prev.x,
            y: curr.y - prev.y,
        });
        let e2 = normalize(Point2 {
            x: next.x - curr.x,
            y: next.y - curr.y,
        });
        let mut nx = -(e1.y + e2.y);
        let mut ny = e1.x + e2.x;
        let len = (nx * nx + ny * ny).sqrt().max(1e-9);
        nx /= len;
        ny /= len;
        out.push(Point2 {
            x: curr.x + nx * delta,
            y: curr.y + ny * delta,
        });
    }
    Ok(out)
}

fn normalize(v: Point2) -> Point2 {
    let l = (v.x * v.x + v.y * v.y).sqrt().max(1e-9);
    Point2 {
        x: v.x / l,
        y: v.y / l,
    }
}

fn point_in_ring(p: &Point2, ring: &[Point2]) -> bool {
    let mut inside = false;
    let n = ring.len();
    for i in 0..n {
        let j = (i + 1) % n;
        let xi = ring[i].x;
        let yi = ring[i].y;
        let xj = ring[j].x;
        let yj = ring[j].y;
        if ((yi > p.y) != (yj > p.y))
            && (p.x < (xj - xi) * (p.y - yi) / (yj - yi + 1e-12) + xi)
        {
            inside = !inside;
        }
    }
    inside
}
