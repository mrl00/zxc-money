//! Generic repository and unit-of-work traits.
//!
//! These are the fundamental persistence ports. Domain-specific repositories
//! (e.g. `AccountRepository`, `TransactionRepository`) are defined in each
//! module's `repository.rs` file.

use async_trait::async_trait;

use super::errors::RepositoryError;
use super::ids::IdempotencyKey;

/// Generic async CRUD repository trait.
///
/// Domain-specific repositories extend this with additional query methods
/// (e.g. `find_by_owner`, `find_by_account`).
///
/// # Type Parameters
///
/// - `T` — The aggregate root or entity type
/// - `ID` — The identifier type (e.g. `AccountID`)
#[async_trait]
pub trait Repository<T: Send + Sync, ID: Send + Sync>: Send + Sync {
    /// Persist or update an entity.
    async fn save(&self, entity: &T) -> Result<(), RepositoryError>;

    /// Find an entity by its identifier. Returns `None` if not found.
    async fn find_by_id(&self, id: ID) -> Result<Option<T>, RepositoryError>;

    /// Delete an entity by its identifier.
    async fn delete(&self, id: ID) -> Result<(), RepositoryError>;
}

/// Port for transactional execution of a batch of operations.
///
/// Implementations ensure that the enclosed closure runs atomically —
/// either all operations succeed or all are rolled back.
#[async_trait]
pub trait UnitOfWork: Send + Sync {
    /// Execute the given closure within a transaction.
    ///
    /// Returns the closure's result on success, or a [`RepositoryError`]
    /// if the transaction fails.
    async fn execute<F, Fut, T>(&self, f: F) -> Result<T, RepositoryError>
    where
        F: FnOnce() -> Fut + Send,
        Fut: std::future::Future<Output = Result<T, RepositoryError>> + Send;
}

/// Port for idempotency tracking on sensitive commands.
///
/// Ensures that a command with a given [`IdempotencyKey`] is processed
/// at most once. Adapters implement this with a persistent store;
/// the in-memory [`super::mock::MockIdempotencyRepository`] is used
/// for testing.
#[async_trait]
pub trait IdempotencyRepository: Send + Sync {
    /// Returns `true` if the key has already been used.
    async fn exists(&self, key: IdempotencyKey) -> Result<bool, RepositoryError>;
    /// Marks the key as used. Must be called after successful processing.
    async fn mark_used(&self, key: IdempotencyKey) -> Result<(), RepositoryError>;
}
