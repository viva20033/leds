use thiserror::Error;

pub type Result<T> = std::result::Result<T, CatalogError>;

#[derive(Debug, Error)]
pub enum CatalogError {
    #[error("IO error reading {path}: {source}")]
    Io {
        path: String,
        source: std::io::Error,
    },
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("module not found: {0}")]
    NotFound(String),
}
