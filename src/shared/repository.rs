use async_trait::async_trait;

use super::errors::RepositoryError;

#[async_trait]
pub trait Repository<T: Send + Sync, ID: Send + Sync>: Send + Sync {
    async fn save(&self, entity: &T) -> Result<(), RepositoryError>;
    async fn find_by_id(&self, id: ID) -> Result<Option<T>, RepositoryError>;
    async fn delete(&self, id: ID) -> Result<(), RepositoryError>;
}

#[async_trait]
pub trait UnitOfWork: Send + Sync {
    async fn execute<F, Fut, T>(&self, f: F) -> Result<T, RepositoryError>
    where
        F: FnOnce() -> Fut + Send,
        Fut: std::future::Future<Output = Result<T, RepositoryError>> + Send;
}
