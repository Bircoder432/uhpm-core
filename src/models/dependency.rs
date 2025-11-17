use semver::VersionReq;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum DependencyKind {
    Required,
    Optional,
    Build,
    Dev,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct VersionConstraint {
    pub requirement: VersionReq,
}

impl Dependency {
    pub fn matches_version(&self, version: &semver::Version) -> bool {
        self.constraint.requirement.matches(version)
    }
}

impl Hash for Dependency {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.constraint.hash(state);
        std::mem::discriminant(&self.kind).hash(state);
        self.provides.hash(state);
        self.features.hash(state);
    }
}
