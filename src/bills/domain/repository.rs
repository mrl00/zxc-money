use crate::shared::errors::RepositoryError;
use crate::shared::ids::{BillID, UserID};
use async_trait::async_trait;

use super::bill::Bill;

/// Persistence trait for [`Bill`] entities.
#[async_trait]
pub trait BillRepository: Send + Sync {
    /// Persists a bill.
    async fn save(&self, bill: &Bill) -> Result<(), RepositoryError>;
    /// Retrieves a bill by its unique identifier.
    async fn find_by_id(&self, id: BillID) -> Result<Option<Bill>, RepositoryError>;
    /// Retrieves all bills belonging to a specific user.
    async fn find_by_owner(&self, owner: UserID) -> Result<Vec<Bill>, RepositoryError>;
    /// Retrieves all bills with `Pending` status.
    async fn find_pending(&self) -> Result<Vec<Bill>, RepositoryError>;
    /// Retrieves all bills with `Overdue` status.
    async fn find_overdue(&self) -> Result<Vec<Bill>, RepositoryError>;
    /// Deletes a bill by its unique identifier.
    async fn delete(&self, id: BillID) -> Result<(), RepositoryError>;
}
