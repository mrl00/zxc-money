use crate::shared::errors::RepositoryError;
use crate::shared::ids::{BillID, UserID};
use async_trait::async_trait;

use super::bill::Bill;

/// Persistence trait for [`Bill`](super::bill::Bill) entities.
#[async_trait]
pub trait BillRepository: Send + Sync {
    /// Persists a bill.
    async fn save(&self, bill: &Bill) -> Result<(), RepositoryError>;
    /// Retrieves a bill by its unique identifier.
    async fn find_by_id(&self, id: BillID) -> Result<Option<Bill>, RepositoryError>;
    /// Retrieves all bills belonging to a specific user.
    async fn find_by_owner(&self, owner: UserID) -> Result<Vec<Bill>, RepositoryError>;
    /// Retrieves all bills with [`BillStatus::Pending`].
    async fn find_pending(&self) -> Result<Vec<Bill>, RepositoryError>;
    /// Retrieves all bills with [`BillStatus::Overdue`].
    async fn find_overdue(&self) -> Result<Vec<Bill>, RepositoryError>;
    /// Deletes a bill by its unique identifier.
    async fn delete(&self, id: BillID) -> Result<(), RepositoryError>;
}
