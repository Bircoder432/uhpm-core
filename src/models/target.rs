use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Target {
    pub os: OperatingSystem,
    pub arch: Architecture,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum OperatingSystem {
    Linux,
    MacOS,
    Custom(String),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum Architecture {
    X86_64,
    Aarch64,
    Custom(String),
}

impl Target {
    pub fn current() -> Self {
        Self {
            os: OperatingSystem::Linux,
            arch: Architecture::X86_64,
        }
    }

    pub fn matches(&self, other: &Target) -> bool {
        self.os == other.os && self.arch == other.arch
    }
}
