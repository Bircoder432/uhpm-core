use crate::{Dependency, Target};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;
use std::path::PathBuf;

/// Package entity representing a software package.
///
/// This is a pure data structure with no business logic.
/// All validation and business rules are handled by factories and services.
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
    ) -> Self {
        Self {
            id: id,
            name: name,
            version: version,
            author: author,
            source: source,
            target: target,
            checksum: checksum,
            dependencies: dependencies,
            installed: installed,
            active: active,
        }
    }

    /// Returns package ID.
    pub fn id(&self) -> &PackageId {
        &self.id
    }

    /// Returns package name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns package version.
    pub fn version(&self) -> &Version {
        &self.version
    }

    /// Returns package author.
    pub fn author(&self) -> &str {
        &self.author
    }

    /// Returns package source.
    pub fn source(&self) -> &PackageSource {
        &self.source
    }

    /// Returns target platform.
    pub fn target(&self) -> &Target {
        &self.target
    }

    /// Returns package checksum.
    pub fn checksum(&self) -> &Option<Checksum> {
        &self.checksum
    }

    /// Returns package dependencies.
    pub fn dependencies(&self) -> &HashSet<Dependency> {
        &self.dependencies
    }

    /// Checks if package is installed.
    pub fn is_installed(&self) -> bool {
        self.installed
    }

    /// Checks if package is active.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Sets installed status.
    pub fn set_installed(&mut self, installed: bool) {
        self.installed = installed;
    }

    /// Sets active status.
    pub fn set_active(&mut self, active: bool) {
        self.active = active;
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
