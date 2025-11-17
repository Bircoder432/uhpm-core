use crate::UhpmError;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;
use std::path::PathBuf;

use crate::Dependency;
use crate::Target;

#[derive(Debug, Clone, Eq)]
pub struct Package {
    id: PackageId,
    name: String,
    version: Version,
    author: String,
    source: PackageSource,
    target: Target,
    checksum: Option<Checksum>,
    dependencies: HashSet<Dependency>,
    installed: bool,
    active: bool,
}

impl Package {
    pub fn new(
        name: String,
        version: semver::Version,
        author: String,
        source: PackageSource,
        target: Target,
        checksum: Option<Checksum>,
        dependencies: Vec<Dependency>,
    ) -> Result<Self, crate::UhpmError> {
        if name.is_empty() {
            return Err(UhpmError::ValidationError(
                "Package name cannot be empty".into(),
            ));
        }

        let id = PackageId::new(&name, &version);
        let dependencies_set: HashSet<Dependency> = dependencies.into_iter().collect();

        Ok(Self {
            id,
            name,
            version,
            author,
            source,
            target,
            checksum,
            dependencies: dependencies_set,
            installed: false,
            active: false,
        })
    }

    pub fn mark_installed(&mut self) {
        self.installed = true;
    }

    pub fn mark_removed(&mut self) {
        self.installed = false;
        self.active = false;
    }

    pub fn activate(&mut self) {
        if self.installed {
            self.active = true;
        }
    }

    pub fn deactivate(&mut self) {
        self.active = false;
    }

    pub fn matches_target(&self, target: &Target) -> bool {
        self.target.matches(target)
    }

    pub fn has_dependency(&self, name: &str) -> bool {
        self.dependencies.iter().any(|d| d.name == name)
    }
    pub fn id(&self) -> &PackageId {
        &self.id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn version(&self) -> &semver::Version {
        &self.version
    }
    pub fn is_installed(&self) -> bool {
        self.installed
    }
    pub fn is_active(&self) -> bool {
        self.active
    }
    pub fn author(&self) -> &String {
        &self.author
    }
}

impl PartialEq for Package {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum PackageSource {
    Git {
        url: String,
        release: Option<String>,
    },
    Http {
        url: String,
    },
    Local {
        path: PathBuf,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PackageId(String);

impl PackageId {
    pub fn new(name: &str, version: &semver::Version) -> Self {
        Self(format!("{}@{}", name, version))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Checksum {
    pub algorithm: String,
    pub hash: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PackageReference {
    pub name: String,
    pub version: Version,
}

impl PackageReference {
    pub fn new(name: String, version: Version) -> Self {
        Self { name, version }
    }

    pub fn from_package(package: &Package) -> Self {
        Self {
            name: package.name.clone(),
            version: package.version.clone(),
        }
    }

    pub fn id(&self) -> String {
        format!("{}@{}", self.name, self.version)
    }

    pub fn matches(&self, other: &PackageReference) -> bool {
        self.name == other.name && self.version == other.version
    }
}

impl fmt::Display for PackageReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.name, self.version)
    }
}

impl From<&Package> for PackageReference {
    fn from(package: &Package) -> Self {
        PackageReference::from_package(package)
    }
}

impl TryFrom<&str> for PackageReference {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = s.split('@').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid package reference format: {}", s));
        }

        let name = parts[0].to_string();
        let version = Version::parse(parts[1])
            .map_err(|e| format!("Invalid version in package reference: {}", e))?;

        Ok(PackageReference::new(name, version))
    }
}
