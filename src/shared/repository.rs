use crate::shared::errors::DomainError;
use async_trait::async_trait;

#[async_trait]
pub trait Repository<T: Send, ID: Send + Sync>: Send + Sync {
    async fn save(&self, entity: &T) -> Result<(), DomainError>;
    async fn find_by_id(&self, id: ID) -> Result<Option<T>, DomainError>;
    async fn delete(&self, id: ID) -> Result<(), DomainError>;
}

pub struct UnitOfWorkContext {
    pub events: std::sync::Arc<crate::shared::events::InMemoryEventDispatcher>,
}

impl UnitOfWorkContext {
    pub fn new(events: std::sync::Arc<crate::shared::events::InMemoryEventDispatcher>) -> Self {
        Self { events }
    }
}

pub trait UnitOfWork: Send + Sync {
    fn execute<F, R>(&self, f: F) -> Result<R, DomainError>
    where
        F: FnOnce() -> Result<R, DomainError>;
}
