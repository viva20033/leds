use crate::contour::{ContourRing, Point2, ShapeGroup};
use crate::{GeometryError, Result};
use i_float::float::FloatNumber;
use i_float::line_float::LineFloat;
use i_float::point::Point as IPoint;
use i_float::rect::Rect;
use i_overlay::float::overlay::FloatOverlay;
use i_overlay::float::style::FillRule;
use i_overlay::mesh::style::OffsetStyle;
use i_overlay::mesh::MeshBuilder;
use i_overlay::mesh::MeshOffset;

const SCALE: f64 = 100.0;

pub fn build_safe_zone(
    group: &ShapeGroup,
    rim_width_mm: f64,
    module_extent_mm: f64,
    safety_margin_mm: f64,
) -> Result<Vec<Point2>> {
    let inset = rim_width_mm + module_extent_mm * 0.5 + safety_margin_mm;
    let outer_inset = inset_ring(&group.outer, -inset)?;
    if outer_inset.len() < 3 {
        return Err(GeometryError::OffsetCollapsed(
            "outer safe zone collapsed — increase depth or reduce rim".into(),
        ));
    }

    let mut result = outer_inset;
    for hole in &group.holes {
        let hole_outset = inset_ring(hole, inset)?;
        if hole_outset.len() >= 3 {
            result = subtract_polygon(&result, &hole_outset);
        }
    }

    if result.len() < 3 {
        return Err(GeometryError::OffsetCollapsed(
            "safe zone empty after holes".into(),
        ));
    }
    Ok(result)
}

fn inset_ring(ring: &ContourRing, delta: f64) -> Result<Vec<Point2>> {
    let path = ring_to_float_path(ring);
    if path.is_empty() {
        return Err(GeometryError::EmptyContour);
    }

    let style = OffsetStyle {
        offset: delta as f32,
        ..Default::default()
    };
    let mesh = MeshBuilder::from_subj_path(&path).offset(style);
    let contours = mesh.contours();
    let first = contours.first().ok_or_else(|| {
        GeometryError::OffsetCollapsed("offset returned empty mesh".into())
    })?;
    let pts = contour_to_points(first);
    if pts.len() < 3 {
        return Err(GeometryError::OffsetCollapsed(
            "offset collapsed to degenerate polygon".into(),
        ));
    }
    Ok(pts)
}

fn subtract_polygon(subject: &[Point2], clip: &[Point2]) -> Vec<Point2> {
    let subj = vec![points_to_path(subject)];
    let clp = vec![points_to_path(clip)];
    let mesh = FloatOverlay::with_subj_and_clip(&subj, &clp)
        .overlay(FillRule::EvenOdd);
    mesh.contours()
        .first()
        .map(contour_to_points)
        .unwrap_or_else(|| subject.to_vec())
}

fn ring_to_float_path(ring: &ContourRing) -> LineFloat<f32> {
    let mut path = LineFloat::with_capacity(ring.points.len());
    for p in &ring.points {
        path.push(IPoint::new(
            (p.x * SCALE) as f32,
            (p.y * SCALE) as f32,
        ));
    }
    if let (Some(first), Some(last)) = (ring.points.first(), ring.points.last()) {
        if (first.x - last.x).hypot(first.y - last.y) > 0.01 {
            path.push(IPoint::new(
                (first.x * SCALE) as f32,
                (first.y * SCALE) as f32,
            ));
        }
    }
    path
}

fn points_to_path(pts: &[Point2]) -> LineFloat<f32> {
    let mut path = LineFloat::with_capacity(pts.len());
    for p in pts {
        path.push(IPoint::new(
            (p.x * SCALE) as f32,
            (p.y * SCALE) as f32,
        ));
    }
    path
}

fn contour_to_points(contour: &[IPoint<f32>]) -> Vec<Point2> {
    contour
        .iter()
        .map(|p| Point2 {
            x: p.x as f64 / SCALE,
            y: p.y as f64 / SCALE,
        })
        .collect()
}
