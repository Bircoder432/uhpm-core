use serde::{Deserialize, Serialize};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Symlink {
    pub source: PathBuf,
    pub target: PathBuf,
    pub link_type: SymlinkType,
    pub metadata: SymlinkMetadata,
}

impl Symlink {
    pub fn new<S, T>(source: S, target: T, link_type: SymlinkType) -> Self
    where
        S: Into<PathBuf>,
        T: Into<PathBuf>,
    {
        Self {
            source: source.into(),
            target: target.into(),
            link_type,
            metadata: SymlinkMetadata::default(),
        }
    }

    pub fn file<S, T>(source: S, target: T) -> Self
    where
        S: Into<PathBuf>,
        T: Into<PathBuf>,
    {
        Self::new(source, target, SymlinkType::File)
    }

    pub fn directory<S, T>(source: S, target: T) -> Self
    where
        S: Into<PathBuf>,
        T: Into<PathBuf>,
    {
        Self::new(source, target, SymlinkType::Directory)
    }

    pub fn with_metadata(mut self, metadata: SymlinkMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn is_file_link(&self) -> bool {
        matches!(self.link_type, SymlinkType::File)
    }

    pub fn is_directory_link(&self) -> bool {
        matches!(self.link_type, SymlinkType::Directory)
    }

    pub fn is_absolute(&self) -> bool {
        self.target.is_absolute()
    }

    pub fn is_relative(&self) -> bool {
        self.target.is_relative()
    }

    pub fn resolve_absolute_path(&self, base_dir: &Path) -> PathBuf {
        if self.target.is_absolute() {
            self.target.clone()
        } else {
            base_dir.join(&self.target)
        }
    }

    pub fn validate(&self) -> Result<(), crate::UhpmError> {
        if self.source.as_os_str().is_empty() {
            return Err(crate::UhpmError::validation(
                "Symlink source cannot be empty",
            ));
        }

        if self.target.as_os_str().is_empty() {
            return Err(crate::UhpmError::validation(
                "Symlink target cannot be empty",
            ));
        }

        if self.source == self.target {
            return Err(crate::UhpmError::validation(
                "Symlink source and target cannot be the same",
            ));
        }

        Ok(())
    }
}

impl Hash for Symlink {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.source.hash(state);
        self.target.hash(state);
        std::mem::discriminant(&self.link_type).hash(state);
        self.metadata.created_at.hash(state);
        self.metadata.owner.hash(state);
        self.metadata.group.hash(state);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SymlinkType {
    #[serde(rename = "file")]
    File,
    #[serde(rename = "directory")]
    Directory,
}

impl SymlinkType {
    pub fn is_file(&self) -> bool {
        matches!(self, Self::File)
    }

    pub fn is_directory(&self) -> bool {
        matches!(self, Self::Directory)
    }
}

impl Default for SymlinkType {
    fn default() -> Self {
        Self::File
    }
}

impl fmt::Display for SymlinkType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::File => write!(f, "file"),
            Self::Directory => write!(f, "directory"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SymlinkMetadata {
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub owner: Option<String>,
    pub group: Option<String>,
    pub description: Option<String>,
}

impl Default for SymlinkMetadata {
    fn default() -> Self {
        Self {
            created_at: chrono::Utc::now(),
            owner: None,
            group: None,
            description: None,
        }
    }
}

impl SymlinkMetadata {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_owner<S: Into<String>>(mut self, owner: S) -> Self {
        self.owner = Some(owner.into());
        self
    }

    pub fn with_group<S: Into<String>>(mut self, group: S) -> Self {
        self.group = Some(group.into());
        self
    }

    pub fn with_description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = Some(description.into());
        self
    }
}

#[derive(Debug, Clone)]
pub struct SymlinkBatch {
    pub links: Vec<Symlink>,
    pub base_directory: PathBuf,
}

impl SymlinkBatch {
    pub fn new(base_directory: PathBuf) -> Self {
        Self {
            links: Vec::new(),
            base_directory,
        }
    }

    pub fn add_link(&mut self, symlink: Symlink) -> Result<(), crate::UhpmError> {
        symlink.validate()?;
        self.links.push(symlink);
        Ok(())
    }

    pub fn add_file_link<S, T>(&mut self, source: S, target: T) -> Result<(), crate::UhpmError>
    where
        S: Into<PathBuf>,
        T: Into<PathBuf>,
    {
        let symlink = Symlink::file(source, target);
        self.add_link(symlink)
    }

    pub fn add_directory_link<S, T>(&mut self, source: S, target: T) -> Result<(), crate::UhpmError>
    where
        S: Into<PathBuf>,
        T: Into<PathBuf>,
    {
        let symlink = Symlink::directory(source, target);
        self.add_link(symlink)
    }

    pub fn validate_all(&self) -> Result<(), crate::UhpmError> {
        for link in &self.links {
            link.validate()?;
        }

        let mut targets = std::collections::HashSet::new();
        for link in &self.links {
            if !targets.insert(&link.target) {
                return Err(crate::UhpmError::validation(format!(
                    "Duplicate symlink target: {}",
                    link.target.display()
                )));
            }
        }

        Ok(())
    }
}
