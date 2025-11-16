use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum Repository {
    Local { path: PathBuf },
    Http { index_url: String },
    Git { url: String, branch: Option<String> },
    Custom { kind: String, location: String },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct RepositoryIndex {
    pub name: String,
    pub description: Option<String>,
    pub url: String,
    pub packages: Vec<RepositoryPackageEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct RepositoryPackageEntry {
    pub name: String,
    pub versions: Vec<String>,
}
