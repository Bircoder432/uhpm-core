use crate::PackageEvent;
use crate::UhpmError;
use async_trait::async_trait;

#[async_trait]
pub trait EventPublisher: Send + Sync {
    async fn publish(&self, event: PackageEvent) -> Result<(), UhpmError>;

    async fn subscribe(
        &self,
        callback: Box<dyn Fn(PackageEvent) + Send + Sync>,
    ) -> Result<String, UhpmError>;

    async fn unsubscribe(&self, subscription_id: &str) -> Result<(), UhpmError>;

    async fn get_event_history(&self, limit: Option<usize>)
    -> Result<Vec<PackageEvent>, UhpmError>;

    async fn clear_event_history(&self) -> Result<(), UhpmError>;
}
