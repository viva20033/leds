mod error;
mod svg;

pub use error::{ImportError, Result};
pub use svg::*;

use leds_geometry::ContourRing;
use std::path::Path;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ImportResult {
    pub source: String,
    pub format: String,
    pub layers: Vec<ImportedLayer>,
    pub contours: Vec<ContourRing>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ImportedLayer {
    pub name: String,
    pub visible: bool,
}

pub fn import_file(path: impl AsRef<Path>) -> Result<ImportResult> {
    let path = path.as_ref();
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "svg" => svg::import_svg(path),
        "dxf" => Err(ImportError::UnsupportedFormat(
            "DXF import scheduled for Phase 1.1 — use SVG export from Corel".into(),
        )),
        other => Err(ImportError::UnsupportedFormat(other.into())),
    }
}
