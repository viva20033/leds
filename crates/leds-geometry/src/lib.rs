mod contour;
mod distance;
mod offset;
mod topology;

pub use contour::*;
pub use distance::*;
pub use offset::*;
pub use topology::*;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum GeometryError {
    #[error("empty contour")]
    EmptyContour,
    #[error("offset collapsed: {0}")]
    OffsetCollapsed(String),
    #[error("invalid ring: {0}")]
    InvalidRing(String),
}

pub type Result<T> = std::result::Result<T, GeometryError>;
