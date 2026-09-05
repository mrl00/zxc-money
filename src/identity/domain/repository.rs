use super::user::User;
use crate::shared::errors::RepositoryError;
use crate::shared::ids::UserID;
use async_trait::async_trait;

/// Errors specific to the identity bounded context.
#[derive(Debug, thiserror::Error)]
pub enum IdentityError {
    /// A user with this email already exists.
    #[error("email already exists: {0}")]
    EmailAlreadyExists(String),

    /// The provided credentials are invalid.
    #[error("invalid credentials")]
    InvalidCredentials,

    /// An input validation error.
    #[error("invalid input: {0}")]
    InvalidInput(String),

    /// An error from the persistence layer.
    #[error("repository error: {0}")]
    Repository(#[from] RepositoryError),
}

/// Repository for persisting and querying [`User`] aggregates.
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Persists a new or updated user.
    async fn save(&self, user: &User) -> Result<(), RepositoryError>;
    /// Finds a user by their unique identifier.
    async fn find_by_id(&self, id: UserID) -> Result<Option<User>, RepositoryError>;
    /// Finds a user by their email address (used for login lookup).
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, RepositoryError>;
    /// Deletes a user by their unique identifier.
    async fn delete(&self, id: UserID) -> Result<(), RepositoryError>;
}
