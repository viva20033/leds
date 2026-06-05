use thiserror::Error;

pub type Result<T> = std::result::Result<T, ImportError>;

#[derive(Debug, Error)]
pub enum ImportError {
    #[error("IO: {0}")]
    Io(#[from] std::io::Error),
    #[error("SVG parse: {0}")]
    Svg(String),
    #[error("unsupported format: {0}")]
    UnsupportedFormat(String),
}
