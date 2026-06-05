use geo::{Coord, LineString, Polygon};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RingKind {
    Outer,
    Hole,
    Island,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Point2 {
    pub x: f64,
    pub y: f64,
}

impl From<Coord<f64>> for Point2 {
    fn from(c: Coord<f64>) -> Self {
        Self { x: c.x, y: c.y }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContourRing {
    pub id: String,
    pub kind: RingKind,
    pub points: Vec<Point2>,
    pub closed: bool,
    pub layer: Option<String>,
}

impl ContourRing {
    pub fn area_mm2(&self) -> f64 {
        if self.points.len() < 3 {
            return 0.0;
        }
        let mut a = 0.0;
        let n = self.points.len();
        for i in 0..n {
            let j = (i + 1) % n;
            a += self.points[i].x * self.points[j].y;
            a -= self.points[j].x * self.points[i].y;
        }
        (a * 0.5).abs()
    }

    pub fn to_polygon(&self) -> Option<Polygon<f64>> {
        if self.points.len() < 3 {
            return None;
        }
        let coords: Vec<Coord<f64>> = self
            .points
            .iter()
            .map(|p| Coord { x: p.x, y: p.y })
            .collect();
        let exterior = LineString::from(coords);
        Some(Polygon::new(exterior, vec![]))
    }

    pub fn centroid(&self) -> Point2 {
        if self.points.is_empty() {
            return Point2 { x: 0.0, y: 0.0 };
        }
        let (sx, sy) = self.points.iter().fold((0.0, 0.0), |(x, y), p| {
            (x + p.x, y + p.y)
        });
        let n = self.points.len() as f64;
        Point2 {
            x: sx / n,
            y: sy / n,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShapeGroup {
    pub outer: ContourRing,
    pub holes: Vec<ContourRing>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductParams {
    pub depth_mm: f64,
    pub rim_width_mm: f64,
    pub safety_margin_mm: f64,
}

impl Default for ProductParams {
    fn default() -> Self {
        Self {
            depth_mm: 100.0,
            rim_width_mm: 15.0,
            safety_margin_mm: 2.0,
        }
    }
}
