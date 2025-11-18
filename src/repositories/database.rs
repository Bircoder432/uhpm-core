// В файле ./repositories/database.rs
use crate::{
    Checksum, Dependency, DependencyKind, FileMetadata, Installation, Package, PackageId,
    PackageSource, Target, UhpmError, VersionConstraint,
};
use rusqlite::{Connection, params};
use semver::{Version, VersionReq};
use std::collections::HashSet;
use std::path::PathBuf;

pub struct DatabaseRepository {
    connection: Connection,
}

impl DatabaseRepository {
    pub fn new(db_path: PathBuf) -> Result<Self, UhpmError> {
        let connection =
            Connection::open(db_path).map_err(|e| UhpmError::DatabaseError(e.to_string()))?;

        let repo = Self { connection };
        repo.init_tables()?;

        Ok(repo)
    }

    fn init_tables(&self) -> Result<(), UhpmError> {
        self.connection
            .execute(
                "CREATE TABLE IF NOT EXISTS packages (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                version TEXT NOT NULL,
                author TEXT NOT NULL,
                source_type TEXT NOT NULL,
                source_path TEXT,
                target_os TEXT NOT NULL,
                target_arch TEXT NOT NULL,
                checksum_algorithm TEXT,
                checksum_hash TEXT,
                installed BOOLEAN NOT NULL DEFAULT 0,
                active BOOLEAN NOT NULL DEFAULT 0,
                installed_at DATETIME,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
                [],
            )
            .map_err(|e| UhpmError::DatabaseError(e.to_string()))?;

        self.connection
            .execute(
                "CREATE TABLE IF NOT EXISTS installed_files (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                package_id TEXT NOT NULL,
                installation_id TEXT NOT NULL,
                file_path TEXT NOT NULL,
                file_size INTEGER NOT NULL,
                checksum_algorithm TEXT,
                checksum_hash TEXT,
                permissions_read BOOLEAN NOT NULL,
                permissions_write BOOLEAN NOT NULL,
                permissions_execute BOOLEAN NOT NULL,
                file_type TEXT NOT NULL,
                created_at DATETIME NOT NULL,
                modified_at DATETIME NOT NULL,
                FOREIGN KEY (package_id) REFERENCES packages (id),
                FOREIGN KEY (installation_id) REFERENCES installations (installation_id)
            )",
                [],
            )
            .map_err(|e| UhpmError::DatabaseError(e.to_string()))?;

        self.connection
            .execute(
                "CREATE TABLE IF NOT EXISTS installations (
                installation_id TEXT PRIMARY KEY,
                package_id TEXT NOT NULL,
                installed_at DATETIME NOT NULL,
                active BOOLEAN NOT NULL DEFAULT 0,
                install_mode TEXT NOT NULL,
                FOREIGN KEY (package_id) REFERENCES packages (id)
            )",
                [],
            )
            .map_err(|e| UhpmError::DatabaseError(e.to_string()))?;

        self.connection
            .execute(
                "CREATE TABLE IF NOT EXISTS symlinks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                installation_id TEXT NOT NULL,
                source_path TEXT NOT NULL,
                target_path TEXT NOT NULL,
                link_type TEXT NOT NULL,
                created_at DATETIME NOT NULL,
                FOREIGN KEY (installation_id) REFERENCES installations (installation_id)
            )",
                [],
            )
            .map_err(|e| UhpmError::DatabaseError(e.to_string()))?;

        self.connection
            .execute(
                "CREATE TABLE IF NOT EXISTS dependencies (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                package_id TEXT NOT NULL,
                dependency_name TEXT NOT NULL,
                version_constraint TEXT NOT NULL,
                dependency_kind TEXT NOT NULL,
                provides TEXT,
                features TEXT,
                FOREIGN KEY (package_id) REFERENCES packages (id)
            )",
                [],
            )
            .map_err(|e| UhpmError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub fn save_package(&mut self, package: &Package) -> Result<(), UhpmError> {
        let tx = self.connection.transaction()?;

        let (source_type, source_path) = Self::source_to_strings(package.source());
        let (target_os, target_arch) = Self::target_to_strings(package.target());

        tx.execute(
            "INSERT OR REPLACE INTO packages (
                id, name, version, author, source_type, source_path,
                target_os, target_arch, checksum_algorithm, checksum_hash,
                installed, active, installed_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                package.id().as_str(),
                package.name(),
                package.version().to_string(),
                package.author(),
                source_type,
                source_path,
                target_os,
                target_arch,
                package.checksum().as_ref().map(|c| c.algorithm.as_str()),
                package.checksum().as_ref().map(|c| c.hash.as_str()),
                package.is_installed(),
                package.is_active(),
                chrono::Utc::now().to_rfc3339(),
            ],
        )?;

        // Вызываем save_dependencies через tx, а не self
        Self::save_dependencies(&tx, package.id().as_str(), package.dependencies())?;

        tx.commit()?;

        Ok(())
    }

    fn save_dependencies(
        tx: &rusqlite::Transaction,
        package_id: &str,
        dependencies: &HashSet<Dependency>,
    ) -> Result<(), UhpmError> {
        tx.execute(
            "DELETE FROM dependencies WHERE package_id = ?1",
            params![package_id],
        )?;

        for dependency in dependencies {
            let features_str = if dependency.features.is_empty() {
                None
            } else {
                Some(dependency.features.join(","))
            };

            tx.execute(
                "INSERT INTO dependencies (package_id, dependency_name, version_constraint, dependency_kind, provides, features)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    package_id,
                    &dependency.name,
                    &dependency.constraint.requirement.to_string(),
                    Self::dependency_kind_to_string(&dependency.kind),
                    dependency.provides.as_ref(),
                    features_str.as_deref(),
                ],
            )?;
        }

        Ok(())
    }

    pub fn get_package(&self, package_id: &PackageId) -> Result<Option<Package>, UhpmError> {
        let mut stmt = self.connection.prepare(
            "SELECT id, name, version, author, source_type, source_path,
                    target_os, target_arch, checksum_algorithm, checksum_hash,
                    installed, active, installed_at
             FROM packages WHERE id = ?1",
        )?;

        let mut rows = stmt.query(params![package_id.as_str()])?;

        if let Some(row) = rows.next()? {
            let name: String = row.get(1)?;
            let version_str: String = row.get(2)?;
            let version = Version::parse(&version_str)
                .map_err(|e| UhpmError::ValidationError(e.to_string()))?;
            let author: String = row.get(3)?;

            let source_type: String = row.get(4)?;
            let source_path: Option<String> = row.get(5)?;
            let source = Self::strings_to_source(source_type, source_path);

            let target_os: String = row.get(6)?;
            let target_arch: String = row.get(7)?;
            let target = Self::strings_to_target(target_os, target_arch);

            let checksum = match (
                row.get::<_, Option<String>>(8)?,
                row.get::<_, Option<String>>(9)?,
            ) {
                (Some(algorithm), Some(hash)) => Some(Checksum { algorithm, hash }),
                _ => None,
            };

            let dependencies = self.load_dependencies(package_id.as_str())?;

            let mut package = Package::new(
                name,
                version,
                author,
                source,
                target,
                checksum,
                dependencies.into_iter().collect(),
            )?;

            if row.get::<_, bool>(10)? {
                package.mark_installed();
            }
            if row.get::<_, bool>(11)? {
                package.activate();
            }

            Ok(Some(package))
        } else {
            Ok(None)
        }
    }

    fn load_dependencies(&self, package_id: &str) -> Result<Vec<Dependency>, UhpmError> {
        let mut stmt = self.connection.prepare(
            "SELECT dependency_name, version_constraint, dependency_kind, provides, features
             FROM dependencies WHERE package_id = ?1",
        )?;

        let rows = stmt.query_map(params![package_id], |row| {
            let name: String = row.get(0)?;
            let constraint_str: String = row.get(1)?;
            let kind_str: String = row.get(2)?;
            let provides: Option<String> = row.get(3)?;
            let features_str: Option<String> = row.get(4)?;

            let constraint = VersionConstraint {
                requirement: VersionReq::parse(&constraint_str).map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        1,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?,
            };

            let kind = Self::string_to_dependency_kind(&kind_str);

            let features = features_str
                .map(|s| s.split(',').map(|f| f.trim().to_string()).collect())
                .unwrap_or_else(Vec::new);

            Ok(Dependency {
                name,
                constraint,
                kind,
                provides,
                features,
            })
        })?;

        let mut dependencies = Vec::new();
        for row in rows {
            dependencies.push(row?);
        }

        Ok(dependencies)
    }

    pub fn get_installed_packages(&self) -> Result<Vec<Package>, UhpmError> {
        let mut stmt = self.connection.prepare(
            "SELECT id, name, version, author, source_type, source_path,
                    target_os, target_arch, checksum_algorithm, checksum_hash,
                    installed, active, installed_at
             FROM packages WHERE installed = 1",
        )?;

        let rows = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let name: String = row.get(1)?;
            let version_str: String = row.get(2)?;
            let version = Version::parse(&version_str).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    2,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;
            let author: String = row.get(3)?;

            let source_type: String = row.get(4)?;
            let source_path: Option<String> = row.get(5)?;
            let source = Self::strings_to_source(source_type, source_path);

            let target_os: String = row.get(6)?;
            let target_arch: String = row.get(7)?;
            let target = Self::strings_to_target(target_os, target_arch);

            let checksum = match (
                row.get::<_, Option<String>>(8)?,
                row.get::<_, Option<String>>(9)?,
            ) {
                (Some(algorithm), Some(hash)) => Some(Checksum { algorithm, hash }),
                _ => None,
            };

            let dependencies = self
                .load_dependencies(&id)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

            let mut package = Package::new(
                name,
                version,
                author,
                source,
                target,
                checksum,
                dependencies.into_iter().collect(),
            )
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

            if row.get::<_, bool>(10)? {
                package.mark_installed();
            }
            if row.get::<_, bool>(11)? {
                package.activate();
            }

            Ok(package)
        })?;

        let mut packages = Vec::new();
        for row in rows {
            packages.push(row?);
        }

        Ok(packages)
    }

    pub fn save_installation(&mut self, installation: &Installation) -> Result<(), UhpmError> {
        let tx = self.connection.transaction()?;

        tx.execute(
            "INSERT OR REPLACE INTO installations (installation_id, package_id, installed_at, active, install_mode)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                installation.id().to_string(),
                installation.package_id().as_str(),
                installation.installed_at().to_rfc3339(),
                installation.is_active(),
                "symlink",
            ],
        )?;

        // Вызываем методы напрямую через Self, а не self
        Self::save_installation_files(
            &tx,
            installation.id().to_string().as_str(),
            installation.installed_files(),
        )?;
        Self::save_symlinks(
            &tx,
            installation.id().to_string().as_str(),
            installation.symlinks(),
        )?;

        tx.commit()?;

        Ok(())
    }

    fn save_installation_files(
        tx: &rusqlite::Transaction,
        installation_id: &str,
        files: &std::collections::HashMap<PathBuf, FileMetadata>,
    ) -> Result<(), UhpmError> {
        tx.execute(
            "DELETE FROM installed_files WHERE installation_id = ?1",
            params![installation_id],
        )?;

        for (path, metadata) in files {
            tx.execute(
                "INSERT INTO installed_files (
                    package_id, installation_id, file_path, file_size, checksum_algorithm, checksum_hash,
                    permissions_read, permissions_write, permissions_execute, file_type, created_at, modified_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                params![
                    "", // TODO: package_id должен быть доступен
                    installation_id,
                    path.to_string_lossy().to_string(),
                    metadata.size,
                    metadata.checksum.as_ref().map(|c| c.algorithm.as_str()),
                    metadata.checksum.as_ref().map(|c| c.hash.as_str()),
                    metadata.permissions.read,
                    metadata.permissions.write,
                    metadata.permissions.execute,
                    Self::file_type_to_string(&metadata.file_type),
                    metadata.created_at.to_rfc3339(),
                    metadata.modified_at.to_rfc3339(),
                ],
            )?;
        }

        Ok(())
    }

    fn save_symlinks(
        tx: &rusqlite::Transaction,
        installation_id: &str,
        symlinks: &[crate::Symlink],
    ) -> Result<(), UhpmError> {
        tx.execute(
            "DELETE FROM symlinks WHERE installation_id = ?1",
            params![installation_id],
        )?;

        for symlink in symlinks {
            tx.execute(
                "INSERT INTO symlinks (installation_id, source_path, target_path, link_type, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    installation_id,
                    symlink.source.to_string_lossy().to_string(),
                    symlink.target.to_string_lossy().to_string(),
                    Self::symlink_type_to_string(&symlink.link_type),
                    symlink.metadata.created_at.to_rfc3339(),
                ],
            )?;
        }

        Ok(())
    }

    pub fn get_installation(
        &self,
        installation_id: &str,
    ) -> Result<Option<Installation>, UhpmError> {
        let mut stmt = self.connection.prepare(
            "SELECT installation_id, package_id, installed_at, active, install_mode
             FROM installations WHERE installation_id = ?1",
        )?;

        let mut rows = stmt.query(params![installation_id])?;

        if let Some(row) = rows.next()? {
            let installation_id_str: String = row.get(0)?;
            let package_id_str: String = row.get(1)?;
            let installed_at_str: String = row.get(2)?;
            let active: bool = row.get(3)?;

            let package_id = PackageId::new(
                package_id_str.split('@').next().unwrap_or(""),
                &Version::parse(package_id_str.split('@').nth(1).unwrap_or("0.0.0")).unwrap(),
            );

            let mut installation = Installation::new(package_id);
            installation.set_id(crate::InstallationId::try_from(
                installation_id_str.as_str(),
            )?);
            installation.set_installed_at(
                chrono::DateTime::parse_from_rfc3339(&installed_at_str)
                    .map_err(|e| UhpmError::DatabaseError(e.to_string()))?
                    .with_timezone(&chrono::Utc),
            );

            if active {
                installation.activate();
            }

            let installed_files = self.load_installed_files(installation_id)?;
            for (path, metadata) in installed_files {
                installation.add_installed_file(path, metadata);
            }

            let symlinks = self.load_symlinks(installation_id)?;
            for symlink in symlinks {
                installation.add_symlink(symlink);
            }

            Ok(Some(installation))
        } else {
            Ok(None)
        }
    }

    fn load_installed_files(
        &self,
        installation_id: &str,
    ) -> Result<Vec<(PathBuf, FileMetadata)>, UhpmError> {
        let mut stmt = self.connection.prepare(
            "SELECT file_path, file_size, checksum_algorithm, checksum_hash,
                    permissions_read, permissions_write, permissions_execute, file_type,
                    created_at, modified_at
             FROM installed_files WHERE installation_id = ?1",
        )?;

        let rows = stmt.query_map(params![installation_id], |row| {
            let file_path: String = row.get(0)?;
            let file_size: u64 = row.get(1)?;
            let checksum_algorithm: Option<String> = row.get(2)?;
            let checksum_hash: Option<String> = row.get(3)?;
            let permissions_read: bool = row.get(4)?;
            let permissions_write: bool = row.get(5)?;
            let permissions_execute: bool = row.get(6)?;
            let file_type_str: String = row.get(7)?;
            let created_at_str: String = row.get(8)?;
            let modified_at_str: String = row.get(9)?;

            let mut metadata = FileMetadata::new(PathBuf::from(file_path), file_size);

            if let (Some(algorithm), Some(hash)) = (checksum_algorithm, checksum_hash) {
                metadata.checksum = Some(crate::FileChecksum { algorithm, hash });
            }

            metadata.permissions = crate::FilePermissions {
                read: permissions_read,
                write: permissions_write,
                execute: permissions_execute,
            };

            metadata.file_type = Self::string_to_file_type(&file_type_str);
            metadata.created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        8,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?
                .with_timezone(&chrono::Utc);
            metadata.modified_at = chrono::DateTime::parse_from_rfc3339(&modified_at_str)
                .map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        9,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?
                .with_timezone(&chrono::Utc);

            Ok(metadata)
        })?;

        let mut files = Vec::new();
        for row in rows {
            let metadata = row?;
            files.push((metadata.path.clone(), metadata));
        }

        Ok(files)
    }

    fn load_symlinks(&self, installation_id: &str) -> Result<Vec<crate::Symlink>, UhpmError> {
        let mut stmt = self.connection.prepare(
            "SELECT source_path, target_path, link_type, created_at
             FROM symlinks WHERE installation_id = ?1",
        )?;

        let rows = stmt.query_map(params![installation_id], |row| {
            let source_path: String = row.get(0)?;
            let target_path: String = row.get(1)?;
            let link_type_str: String = row.get(2)?;
            let created_at_str: String = row.get(3)?;

            let symlink = crate::Symlink::new(
                PathBuf::from(source_path),
                PathBuf::from(target_path),
                Self::string_to_symlink_type(&link_type_str),
            )
            .with_metadata(crate::SymlinkMetadata {
                created_at: chrono::DateTime::parse_from_rfc3339(&created_at_str)
                    .map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            3,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        )
                    })?
                    .with_timezone(&chrono::Utc),
                ..Default::default()
            });

            Ok(symlink)
        })?;

        let mut symlinks = Vec::new();
        for row in rows {
            symlinks.push(row?);
        }

        Ok(symlinks)
    }

    pub fn save_installed_files(
        &mut self,
        installation_id: &str,
        files: &[(PathBuf, FileMetadata)],
    ) -> Result<(), UhpmError> {
        let tx = self.connection.transaction()?;

        let files_map: std::collections::HashMap<_, _> = files.iter().cloned().collect();
        Self::save_installation_files(&tx, installation_id, &files_map)?;

        tx.commit()?;

        Ok(())
    }

    pub fn get_installed_files(
        &self,
        installation_id: &str,
    ) -> Result<Vec<(PathBuf, FileMetadata)>, UhpmError> {
        self.load_installed_files(installation_id)
    }

    // Вспомогательные методы для преобразования типов

    fn source_to_strings(source: &PackageSource) -> (String, Option<String>) {
        match source {
            PackageSource::Git { url, release: _ } => ("git".to_string(), Some(url.clone())),
            PackageSource::Http { url } => ("http".to_string(), Some(url.clone())),
            PackageSource::Local { path } => (
                "local".to_string(),
                Some(path.to_string_lossy().to_string()),
            ),
        }
    }

    fn strings_to_source(source_type: String, source_path: Option<String>) -> PackageSource {
        match source_type.as_str() {
            "git" => PackageSource::Git {
                url: source_path.unwrap_or_default(),
                release: None,
            },
            "http" => PackageSource::Http {
                url: source_path.unwrap_or_default(),
            },
            "local" => PackageSource::Local {
                path: PathBuf::from(source_path.unwrap_or_default()),
            },
            _ => PackageSource::Local {
                path: PathBuf::from("/unknown"),
            },
        }
    }

    fn target_to_strings(target: &Target) -> (String, String) {
        match (&target.os, &target.arch) {
            (crate::OperatingSystem::Linux, crate::Architecture::X86_64) => {
                ("linux".to_string(), "x86_64".to_string())
            }
            (crate::OperatingSystem::Linux, crate::Architecture::Aarch64) => {
                ("linux".to_string(), "aarch64".to_string())
            }
            (crate::OperatingSystem::MacOS, crate::Architecture::X86_64) => {
                ("macos".to_string(), "x86_64".to_string())
            }
            (crate::OperatingSystem::MacOS, crate::Architecture::Aarch64) => {
                ("macos".to_string(), "aarch64".to_string())
            }
            (crate::OperatingSystem::Custom(os), crate::Architecture::Custom(arch)) => {
                (os.clone(), arch.clone())
            }
            _ => ("unknown".to_string(), "unknown".to_string()),
        }
    }

    fn strings_to_target(os: String, arch: String) -> Target {
        match (os.as_str(), arch.as_str()) {
            ("linux", "x86_64") => Target {
                os: crate::OperatingSystem::Linux,
                arch: crate::Architecture::X86_64,
            },
            ("linux", "aarch64") => Target {
                os: crate::OperatingSystem::Linux,
                arch: crate::Architecture::Aarch64,
            },
            ("macos", "x86_64") => Target {
                os: crate::OperatingSystem::MacOS,
                arch: crate::Architecture::X86_64,
            },
            ("macos", "aarch64") => Target {
                os: crate::OperatingSystem::MacOS,
                arch: crate::Architecture::Aarch64,
            },
            _ => Target {
                os: crate::OperatingSystem::Custom(os),
                arch: crate::Architecture::Custom(arch),
            },
        }
    }

    fn dependency_kind_to_string(kind: &DependencyKind) -> String {
        match kind {
            DependencyKind::Required => "required".to_string(),
            DependencyKind::Optional => "optional".to_string(),
            DependencyKind::Build => "build".to_string(),
            DependencyKind::Dev => "dev".to_string(),
        }
    }

    fn string_to_dependency_kind(kind_str: &str) -> DependencyKind {
        match kind_str {
            "optional" => DependencyKind::Optional,
            "build" => DependencyKind::Build,
            "dev" => DependencyKind::Dev,
            _ => DependencyKind::Required,
        }
    }

    fn file_type_to_string(file_type: &crate::FileType) -> String {
        match file_type {
            crate::FileType::Regular => "regular",
            crate::FileType::Directory => "directory",
            crate::FileType::Symlink => "symlink",
            crate::FileType::Executable => "executable",
        }
        .to_string()
    }

    fn string_to_file_type(file_type_str: &str) -> crate::FileType {
        match file_type_str {
            "directory" => crate::FileType::Directory,
            "symlink" => crate::FileType::Symlink,
            "executable" => crate::FileType::Executable,
            _ => crate::FileType::Regular,
        }
    }

    fn symlink_type_to_string(link_type: &crate::SymlinkType) -> String {
        match link_type {
            crate::SymlinkType::File => "file",
            crate::SymlinkType::Directory => "directory",
        }
        .to_string()
    }

    fn string_to_symlink_type(link_type_str: &str) -> crate::SymlinkType {
        match link_type_str {
            "directory" => crate::SymlinkType::Directory,
            _ => crate::SymlinkType::File,
        }
    }
}
