use crate::contour::Point2;
use crate::topology::point_in_shape;
use crate::contour::ShapeGroup;

/// Raster distance transform (approximate EDT) on a grid inside shape bbox.
pub struct DistanceField {
    pub width: usize,
    pub height: usize,
    pub origin_x: f64,
    pub origin_y: f64,
    pub cell_mm: f64,
    pub values: Vec<f32>,
}

impl DistanceField {
    pub fn sample(&self, x: f64, y: f64) -> f32 {
        let fx = (x - self.origin_x) / self.cell_mm;
        let fy = (y - self.origin_y) / self.cell_mm;
        let ix = fx.floor() as isize;
        let iy = fy.floor() as isize;
        if ix < 0 || iy < 0 || ix as usize >= self.width.saturating_sub(1) || iy as usize >= self.height.saturating_sub(1) {
            return 0.0;
        }
        let ix = ix as usize;
        let iy = iy as usize;
        self.values[iy * self.width + ix]
    }

    pub fn local_maxima(&self, threshold: f32) -> Vec<Point2> {
        let mut out = Vec::new();
        for y in 1..self.height.saturating_sub(1) {
            for x in 1..self.width.saturating_sub(1) {
                let v = self.values[y * self.width + x];
                if v < threshold {
                    continue;
                }
                let mut is_max = true;
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let nv = self.values[(y as isize + dy) as usize * self.width
                            + (x as isize + dx) as usize];
                        if nv > v {
                            is_max = false;
                        }
                    }
                }
                if is_max {
                    out.push(Point2 {
                        x: self.origin_x + x as f64 * self.cell_mm,
                        y: self.origin_y + y as f64 * self.cell_mm,
                    });
                }
            }
        }
        out
    }
}

pub fn compute_distance_field(
    group: &ShapeGroup,
    safe_polygon: &[Point2],
    cell_mm: f64,
) -> DistanceField {
    let (min_x, min_y, max_x, max_y) = bbox(safe_polygon);
    let margin = cell_mm * 2.0;
    let origin_x = min_x - margin;
    let origin_y = min_y - margin;
    let width = ((max_x - min_x + 2.0 * margin) / cell_mm).ceil() as usize + 1;
    let height = ((max_y - min_y + 2.0 * margin) / cell_mm).ceil() as usize + 1;

    let mut inside = vec![false; width * height];
    for y in 0..height {
        for x in 0..width {
            let wx = origin_x + x as f64 * cell_mm;
            let wy = origin_y + y as f64 * cell_mm;
            let p = Point2 { x: wx, y: wy };
            inside[y * width + x] =
                point_in_shape(group, &p) && point_in_safe(safe_polygon, &p);
        }
    }

    // Two-pass chamfer DT
    let inf = 1e6f32;
    let mut dist = vec![inf; width * height];
    for i in 0..dist.len() {
        if inside[i] {
            dist[i] = if is_boundary(i, width, height, &inside) {
                0.0
            } else {
                inf
            };
        } else {
            dist[i] = 0.0;
        }
    }

    chamfer_pass(&mut dist, width, height, &inside, cell_mm as f32);
    chamfer_pass_reverse(&mut dist, width, height, &inside, cell_mm as f32);

    DistanceField {
        width,
        height,
        origin_x,
        origin_y,
        cell_mm,
        values: dist,
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

fn is_boundary(i: usize, w: usize, h: usize, inside: &[bool]) -> bool {
    if !inside[i] {
        return false;
    }
    let x = i % w;
    let y = i / w;
    if x == 0 || y == 0 || x + 1 == w || y + 1 == h {
        return true;
    }
    let neighbors = [
        i - 1,
        i + 1,
        i - w,
        i + w,
    ];
    neighbors.iter().any(|&ni| !inside[ni])
}

fn chamfer_pass(dist: &mut [f32], w: usize, h: usize, inside: &[bool], cell: f32) {
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            if !inside[i] {
                continue;
            }
            let mut v = dist[i];
            if x > 0 {
                v = v.min(dist[i - 1] + cell);
            }
            if y > 0 {
                v = v.min(dist[i - w] + cell);
            }
            if x > 0 && y > 0 {
                v = v.min(dist[i - w - 1] + cell * 1.4142);
            }
            if x + 1 < w && y > 0 {
                v = v.min(dist[i - w + 1] + cell * 1.4142);
            }
            dist[i] = v;
        }
    }
}

fn chamfer_pass_reverse(dist: &mut [f32], w: usize, h: usize, inside: &[bool], cell: f32) {
    for y in (0..h).rev() {
        for x in (0..w).rev() {
            let i = y * w + x;
            if !inside[i] {
                continue;
            }
            let mut v = dist[i];
            if x + 1 < w {
                v = v.min(dist[i + 1] + cell);
            }
            if y + 1 < h {
                v = v.min(dist[i + w] + cell);
            }
            if x + 1 < w && y + 1 < h {
                v = v.min(dist[i + w + 1] + cell * 1.4142);
            }
            if x > 0 && y + 1 < h {
                v = v.min(dist[i + w - 1] + cell * 1.4142);
            }
            dist[i] = v;
        }
    }
}
