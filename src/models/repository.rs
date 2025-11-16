use crate::Dependency;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum Repository {
    Local { path: PathBuf },
    Http { index_url: String },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct RepositoryIndex {
    pub name: String,
    pub url: String,
    pub packages: Vec<RepositoryPackageEntry>,
}

impl RepositoryIndex {
    pub fn get_versions(&self, pkg: &str) -> Option<&[String]> {
        self.packages
            .iter()
            .find(|p| p.name == pkg)
            .map(|p| p.versions.as_slice())
    }

    pub fn latest_satisfying(&self, dep: &Dependency) -> Option<String> {
        let versions = self.get_versions(&dep.name)?;
        let mut parsed: Vec<Version> = versions
            .iter()
            .filter_map(|v| Version::parse(v).ok())
            .collect();
        parsed.sort();
        parsed
            .into_iter()
            .rev()
            .find(|v| dep.matches_version(v))
            .map(|v| v.to_string())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct RepositoryPackageEntry {
    pub name: String,
    pub versions: Vec<String>,
}
