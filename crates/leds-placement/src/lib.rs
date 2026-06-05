mod hamp;

pub use hamp::*;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum PlacementError {
    #[error("geometry: {0}")]
    Geometry(#[from] leds_geometry::GeometryError),
    #[error("no safe zone")]
    NoSafeZone,
    #[error("module not found: {0}")]
    ModuleNotFound(String),
}

pub type Result<T> = std::result::Result<T, PlacementError>;
