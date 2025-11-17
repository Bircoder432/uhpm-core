use crate::UhpmError;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UhpmConfig {
    pub update_source: String,
    pub default_install_mode: InstallMode,
    pub repositories: Vec<RepositoryConfig>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct RepositoryConfig {
    pub name: String,
    pub url: String,
    pub repo_type: RepositoryType,
    pub enabled: bool,
    pub priority: u32,
    pub authentication: Option<RepositoryAuth>,
}

impl RepositoryConfig {
    pub fn new<S: Into<String>>(name: S, url: S, repo_type: RepositoryType) -> Self {
        Self {
            name: name.into(),
            url: url.into(),
            repo_type,
            enabled: true,
            priority: 100,
            authentication: None,
        }
    }

    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_auth(mut self, auth: RepositoryAuth) -> Self {
        self.authentication = Some(auth);
        self
    }

    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }

    pub fn is_local(&self) -> bool {
        self.url.starts_with("file://") || !self.url.contains("://")
    }

    pub fn is_remote(&self) -> bool {
        self.url.starts_with("http://") || self.url.starts_with("https://")
    }

    pub fn local_path(&self) -> Option<std::path::PathBuf> {
        if self.url.starts_with("file://") {
            Some(std::path::PathBuf::from(
                self.url.strip_prefix("file://").unwrap(),
            ))
        } else if !self.url.contains("://") {
            Some(std::path::PathBuf::from(&self.url))
        } else {
            None
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum RepositoryType {
    #[serde(rename = "binary")]
    Binary,
    #[serde(rename = "source")]
    Source,
    #[serde(rename = "universal")]
    Universal,
    #[serde(rename = "mixed")]
    Mixed,
}

impl fmt::Display for RepositoryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Binary => write!(f, "binary"),
            Self::Source => write!(f, "source"),
            Self::Universal => write!(f, "universal"),
            Self::Mixed => write!(f, "mixed"),
        }
    }
}

impl Default for RepositoryType {
    fn default() -> Self {
        Self::Binary
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct RepositoryAuth {
    pub username: Option<String>,
    pub password: Option<String>,
    pub token: Option<String>,
}

impl RepositoryAuth {
    pub fn token<S: Into<String>>(token: S) -> Self {
        Self {
            username: None,
            password: None,
            token: Some(token.into()),
        }
    }

    pub fn basic<S: Into<String>>(username: S, password: S) -> Self {
        Self {
            username: Some(username.into()),
            password: Some(password.into()),
            token: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InstallMode {
    #[serde(rename = "symlink")]
    Symlink,
    #[serde(rename = "direct")]
    Direct,
    #[serde(rename = "auto")]
    Auto,
}

impl InstallMode {
    pub fn is_direct(&self) -> bool {
        matches!(self, Self::Direct)
    }

    pub fn is_symlink(&self) -> bool {
        matches!(self, Self::Symlink)
    }

    pub fn is_auto(&self) -> bool {
        matches!(self, Self::Auto)
    }

    pub fn should_use_symlinks(&self, platform_supports_symlinks: bool) -> bool {
        match self {
            Self::Symlink => true,
            Self::Direct => false,
            Self::Auto => platform_supports_symlinks,
        }
    }
}

impl Default for InstallMode {
    fn default() -> Self {
        Self::Auto
    }
}

impl fmt::Display for InstallMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Symlink => write!(f, "symlink"),
            Self::Direct => write!(f, "direct"),
            Self::Auto => write!(f, "auto"),
        }
    }
}

impl TryFrom<&str> for InstallMode {
    type Error = UhpmError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "symlink" | "symbolic" | "link" => Ok(Self::Symlink),
            "direct" | "copy" | "hard" => Ok(Self::Direct),
            "auto" | "automatic" => Ok(Self::Auto),
            _ => Err(UhpmError::validation(format!(
                "Invalid install mode: '{}'. Use 'symlink', 'direct', or 'auto'",
                value
            ))),
        }
    }
}
