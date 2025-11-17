use crate::UhpmError;
use async_trait::async_trait;

#[async_trait]
pub trait NetworkOperations: Send + Sync {
    async fn get(&self, url: &str) -> Result<Vec<u8>, UhpmError>;

    async fn get_with_progress(
        &self,
        url: &str,
        on_progress: Option<Box<dyn Fn(u64, u64) + Send + Sync>>,
    ) -> Result<Vec<u8>, UhpmError>;

    async fn head(&self, url: &str) -> Result<reqwest::Response, UhpmError>;

    async fn is_url_available(&self, url: &str) -> bool;

    async fn download_with_checksum(
        &self,
        url: &str,
        expected_checksum: Option<(&str, &str)>,
        on_progress: Option<Box<dyn Fn(u64, u64) + Send + Sync>>,
    ) -> Result<Vec<u8>, UhpmError>;

    fn parse_url(&self, url: &str) -> Result<url::Url, UhpmError>;
}
