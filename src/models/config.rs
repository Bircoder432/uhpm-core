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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repository_config_creation() {
        let repo =
            RepositoryConfig::new("test-repo", "https://example.com", RepositoryType::Binary);

        assert_eq!(repo.name, "test-repo");
        assert_eq!(repo.url, "https://example.com");
        assert_eq!(repo.repo_type, RepositoryType::Binary);
        assert_eq!(repo.enabled, true);
        assert_eq!(repo.priority, 100);
        assert_eq!(repo.authentication, None);
    }

    #[test]
    fn test_repository_config_builder_methods() {
        let auth = RepositoryAuth::token("test-token");
        let repo = RepositoryConfig::new("test-repo", "file:///local/path", RepositoryType::Source)
            .with_priority(50)
            .with_auth(auth.clone())
            .disabled();

        assert_eq!(repo.priority, 50);
        assert_eq!(repo.authentication, Some(auth));
        assert_eq!(repo.enabled, false);
    }

    #[test]
    fn test_repository_type_display() {
        assert_eq!(RepositoryType::Binary.to_string(), "binary");
        assert_eq!(RepositoryType::Source.to_string(), "source");
        assert_eq!(RepositoryType::Universal.to_string(), "universal");
        assert_eq!(RepositoryType::Mixed.to_string(), "mixed");
    }

    #[test]
    fn test_repository_type_default() {
        assert_eq!(RepositoryType::default(), RepositoryType::Binary);
    }

    #[test]
    fn test_repository_auth_creation() {
        let token_auth = RepositoryAuth::token("my-token");
        assert_eq!(token_auth.token, Some("my-token".to_string()));
        assert_eq!(token_auth.username, None);
        assert_eq!(token_auth.password, None);

        let basic_auth = RepositoryAuth::basic("user", "pass");
        assert_eq!(basic_auth.username, Some("user".to_string()));
        assert_eq!(basic_auth.password, Some("pass".to_string()));
        assert_eq!(basic_auth.token, None);
    }

    #[test]
    fn test_install_mode_methods() {
        assert!(InstallMode::Symlink.is_symlink());
        assert!(InstallMode::Direct.is_direct());
        assert!(InstallMode::Auto.is_auto());

        assert!(!InstallMode::Symlink.is_direct());
        assert!(!InstallMode::Direct.is_symlink());
        assert!(!InstallMode::Auto.is_direct());
    }

    #[test]
    fn test_install_mode_should_use_symlinks() {
        // Test with platform that supports symlinks
        assert_eq!(InstallMode::Symlink.should_use_symlinks(true), true);
        assert_eq!(InstallMode::Direct.should_use_symlinks(true), false);
        assert_eq!(InstallMode::Auto.should_use_symlinks(true), true);

        // Test with platform that doesn't support symlinks
        assert_eq!(InstallMode::Symlink.should_use_symlinks(false), true);
        assert_eq!(InstallMode::Direct.should_use_symlinks(false), false);
        assert_eq!(InstallMode::Auto.should_use_symlinks(false), false);
    }

    #[test]
    fn test_install_mode_display() {
        assert_eq!(InstallMode::Symlink.to_string(), "symlink");
        assert_eq!(InstallMode::Direct.to_string(), "direct");
        assert_eq!(InstallMode::Auto.to_string(), "auto");
    }

    #[test]
    fn test_install_mode_default() {
        assert_eq!(InstallMode::default(), InstallMode::Auto);
    }

    #[test]
    fn test_install_mode_try_from_valid() {
        assert_eq!(
            InstallMode::try_from("symlink").unwrap(),
            InstallMode::Symlink
        );
        assert_eq!(
            InstallMode::try_from("SYMLINK").unwrap(),
            InstallMode::Symlink
        );
        assert_eq!(
            InstallMode::try_from("symbolic").unwrap(),
            InstallMode::Symlink
        );
        assert_eq!(InstallMode::try_from("link").unwrap(), InstallMode::Symlink);

        assert_eq!(
            InstallMode::try_from("direct").unwrap(),
            InstallMode::Direct
        );
        assert_eq!(
            InstallMode::try_from("DIRECT").unwrap(),
            InstallMode::Direct
        );
        assert_eq!(InstallMode::try_from("copy").unwrap(), InstallMode::Direct);
        assert_eq!(InstallMode::try_from("hard").unwrap(), InstallMode::Direct);

        assert_eq!(InstallMode::try_from("auto").unwrap(), InstallMode::Auto);
        assert_eq!(InstallMode::try_from("AUTO").unwrap(), InstallMode::Auto);
        assert_eq!(
            InstallMode::try_from("automatic").unwrap(),
            InstallMode::Auto
        );
    }

    #[test]
    fn test_install_mode_try_from_invalid() {
        assert!(InstallMode::try_from("invalid").is_err());
        assert!(InstallMode::try_from("").is_err());
        assert!(InstallMode::try_from("unknown").is_err());
    }

    #[test]
    fn test_repository_config_local_detection() {
        // Local repositories
        let file_repo =
            RepositoryConfig::new("file-repo", "file:///path/to/repo", RepositoryType::Binary);
        assert!(file_repo.is_local());
        assert!(!file_repo.is_remote());

        let path_repo = RepositoryConfig::new("path-repo", "/path/to/repo", RepositoryType::Binary);
        assert!(path_repo.is_local());
        assert!(!path_repo.is_remote());

        // Remote repositories
        let http_repo =
            RepositoryConfig::new("http-repo", "http://example.com", RepositoryType::Binary);
        assert!(!http_repo.is_local());
        assert!(http_repo.is_remote());

        let https_repo =
            RepositoryConfig::new("https-repo", "https://example.com", RepositoryType::Binary);
        assert!(!https_repo.is_local());
        assert!(https_repo.is_remote());

        // Other protocols (should not be considered local or remote by current logic)
        let ftp_repo =
            RepositoryConfig::new("ftp-repo", "ftp://example.com", RepositoryType::Binary);
        assert!(!ftp_repo.is_local());
        assert!(!ftp_repo.is_remote());
    }

    #[test]
    fn test_repository_config_local_path() {
        // File protocol
        let file_repo =
            RepositoryConfig::new("file-repo", "file:///absolute/path", RepositoryType::Binary);
        assert_eq!(
            file_repo.local_path().unwrap(),
            std::path::PathBuf::from("/absolute/path")
        );

        // Simple path
        let path_repo = RepositoryConfig::new("path-repo", "relative/path", RepositoryType::Binary);
        assert_eq!(
            path_repo.local_path().unwrap(),
            std::path::PathBuf::from("relative/path")
        );

        // Remote URLs should return None
        let http_repo =
            RepositoryConfig::new("http-repo", "http://example.com", RepositoryType::Binary);
        assert!(http_repo.local_path().is_none());

        let https_repo =
            RepositoryConfig::new("https-repo", "https://example.com", RepositoryType::Binary);
        assert!(https_repo.local_path().is_none());
    }

    #[test]
    fn test_uhpm_config_serialization() {
        let config = UhpmConfig {
            update_source: "https://updates.example.com".to_string(),
            default_install_mode: InstallMode::Symlink,
            repositories: vec![
                RepositoryConfig::new("repo1", "https://repo1.example.com", RepositoryType::Binary),
                RepositoryConfig::new("repo2", "file:///local/repo", RepositoryType::Source)
                    .with_priority(200)
                    .disabled(),
            ],
        };

        // Test that serialization works without panicking
        let serialized = toml::to_string(&config).unwrap();
        assert!(!serialized.is_empty());

        // Test that deserialization works
        let deserialized: UhpmConfig = toml::from_str(&serialized).unwrap();
        assert_eq!(deserialized.update_source, config.update_source);
        assert_eq!(
            deserialized.default_install_mode,
            config.default_install_mode
        );
        assert_eq!(deserialized.repositories.len(), config.repositories.len());
    }

    #[test]
    fn test_repository_config_equality() {
        let repo1 = RepositoryConfig::new("repo", "url", RepositoryType::Binary);
        let repo2 = RepositoryConfig::new("repo", "url", RepositoryType::Binary);
        let repo3 = RepositoryConfig::new("different", "url", RepositoryType::Binary);

        assert_eq!(repo1, repo2);
        assert_ne!(repo1, repo3);
    }

    #[test]
    fn test_install_mode_equality() {
        assert_eq!(InstallMode::Symlink, InstallMode::Symlink);
        assert_ne!(InstallMode::Symlink, InstallMode::Direct);
    }
}
