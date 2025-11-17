use crate::{Package, PackageReference};
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

#[derive(Debug, Clone)]
pub struct ResolutionResult {
    pub packages_to_install: Vec<Package>,

    pub packages_to_update: Vec<PackageReference>,

    pub packages_to_remove: Vec<PackageReference>,

    pub conflicts: Vec<DependencyConflict>,
}

#[derive(Debug, Clone)]
pub struct DependencyConflict {
    pub package: String,

    pub required: String,

    pub installed: String,

    pub message: String,
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
