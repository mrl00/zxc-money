use crate::shared::errors::RepositoryError;
use crate::shared::ids::BillID;
use async_trait::async_trait;

use super::bill::Bill;

#[async_trait]
pub trait BillRepository: Send + Sync {
    async fn save(&self, bill: &Bill) -> Result<(), RepositoryError>;
    async fn find_by_id(&self, id: BillID) -> Result<Option<Bill>, RepositoryError>;
    async fn find_pending(&self) -> Result<Vec<Bill>, RepositoryError>;
    async fn find_overdue(&self) -> Result<Vec<Bill>, RepositoryError>;
    async fn delete(&self, id: BillID) -> Result<(), RepositoryError>;
}
