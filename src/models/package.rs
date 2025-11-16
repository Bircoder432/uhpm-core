use semver::Version;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, str};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Package {
    name: String,
    version: Version,
    author: String,
    source: PackageSource,
    target: Target,
    checksum: Option<Checksum>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Target {
    pub os: OperatingSystem,
    pub arch: Architecture,
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
pub enum Architecture {
    X86_64,
    Aarch64,
    Custom(String),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum OperatingSystem {
    Linux,
    MacOS,
    Custom(String),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Checksum {
    pub algorithm: String,
    pub hash: String,
}
