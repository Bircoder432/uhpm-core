use semver::Version;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

use crate::Dependency;
use crate::Target;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Package {
    pub name: String,
    pub version: Version,
    pub author: String,
    pub source: PackageSource,
    pub target: Target,
    pub checksum: Option<Checksum>,

    #[serde(default)]
    pub dependencies: Vec<Dependency>,
}

impl Package {
    pub fn id(&self) -> String {
        format!("{}@{}", self.name, self.version)
    }

    pub fn matches_target(&self, target: &Target) -> bool {
        self.target.matches(target)
    }

    pub fn has_dependency(&self, name: &str) -> bool {
        self.dependencies.iter().any(|d| d.name == name)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
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

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
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
