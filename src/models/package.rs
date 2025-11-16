use semver::Version;
use serde::{Deserialize, Serialize};
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
