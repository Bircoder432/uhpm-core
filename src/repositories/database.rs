use crate::{FileMetadata, Installation, Package, PackageId, UhpmError};
use rusqlite::Connection;
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

        Ok(())
    }

    pub fn save_package(&self, package: &Package) -> Result<(), UhpmError> {
        // TODO

        Ok(())
    }

    pub fn get_package(&self, package_id: &PackageId) -> Result<Option<Package>, UhpmError> {
        // TODO
        Ok(None)
    }

    pub fn get_installed_packages(&self) -> Result<Vec<Package>, UhpmError> {
        // TODO
        Ok(Vec::new())
    }

    pub fn save_installation(&self, installation: &Installation) -> Result<(), UhpmError> {
        // TODO
        Ok(())
    }

    pub fn get_installation(
        &self,
        installation_id: &str,
    ) -> Result<Option<Installation>, UhpmError> {
        // TODO
        Ok(None)
    }

    pub fn save_installed_files(
        &self,
        installation_id: &str,
        files: &[(PathBuf, FileMetadata)],
    ) -> Result<(), UhpmError> {
        // TODO
        Ok(())
    }

    pub fn get_installed_files(
        &self,
        installation_id: &str,
    ) -> Result<Vec<(PathBuf, FileMetadata)>, UhpmError> {
        // TODO
        Ok(Vec::new())
    }
}
