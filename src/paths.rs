use crate::UhpmError;
use std::path::PathBuf;

pub trait UhpmPaths: Send + Sync {
    fn base_dir(&self) -> PathBuf;

    fn packages_dir(&self) -> PathBuf {
        self.base_dir().join("packages")
    }

    fn db_path(&self) -> PathBuf {
        self.base_dir().join("packages.db")
    }

    fn config_path(&self) -> PathBuf;

    fn cache_dir(&self) -> PathBuf;

    fn temp_dir(&self) -> PathBuf;

    fn log_dir(&self) -> PathBuf {
        self.base_dir().join("logs")
    }

    async fn create_directories<FS: crate::ports::FileSystemOperations>(
        &self,
        fs: &FS,
    ) -> Result<(), UhpmError> {
        fs.create_dir_all(&self.base_dir()).await?;
        fs.create_dir_all(&self.packages_dir()).await?;
        fs.create_dir_all(&self.cache_dir()).await?;
        fs.create_dir_all(&self.temp_dir()).await?;
        fs.create_dir_all(&self.log_dir()).await?;

        if let Some(config_parent) = self.config_path().parent() {
            fs.create_dir_all(config_parent).await?;
        }

        Ok(())
    }
}
