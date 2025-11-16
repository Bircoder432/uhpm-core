use semver::VersionReq;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Dependency {
    pub name: String,
    pub constraint: VersionConstraint,
    pub kind: DependencyKind,

    #[serde(default)]
    pub provides: Option<String>,

    #[serde(default)]
    pub features: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum DependencyKind {
    Required,
    Optional,
    Build,
    Dev,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct VersionConstraint {
    pub requirement: VersionReq,
}

impl Dependency {
    pub fn matches_version(&self, version: &semver::Version) -> bool {
        self.constraint.requirement.matches(version)
    }
}
