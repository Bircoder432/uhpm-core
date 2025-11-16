use semver::VersionReq;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UhpmError {
    #[error("Package `{0}` not found")]
    PackageNotFound(String),
    #[error("No version of `{package}` matches `{required}`")]
    VersionMismatch {
        package: String,
        required: VersionReq,
    },
    #[error("Dependency resolution failed: {0}")]
    ResolutionError(String),
    #[error("Repository unreachable: {0}")]
    RepositoryUnavailable(String),
    #[error("Failed to read file: {0}")]
    IoError(String),
    #[error("Invalid package format in `{0}`")]
    InvalidPackage(PathBuf),
}
