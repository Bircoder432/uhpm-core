#[derive(Debug, thiserror::Error)]
pub enum FsError {
    #[error("file not found: {0}")]
    NotFound(String),

    #[error("permission denied: {0}")]
    PermissionDenied(String),

    #[error("io error: {0}")]
    Io(String),

    #[error("invalid path: {0}")]
    InvalidPath(String),

    #[error("not a directory: {0}")]
    NotADirectory(String),

    #[error("unsupported operation: {0}")]
    Unsupported(String),

    #[error("extraction error: {0}")]
    ExtractionError(String),
}
