mod pipeline;

pub use pipeline::*;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("catalog: {0}")]
    Catalog(#[from] leds_catalog::CatalogError),
    #[error("import: {0}")]
    Import(#[from] leds_import::ImportError),
    #[error("placement: {0}")]
    Placement(#[from] leds_placement::PlacementError),
    #[error("geometry: {0}")]
    Geometry(#[from] leds_geometry::GeometryError),
    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, CoreError>;
