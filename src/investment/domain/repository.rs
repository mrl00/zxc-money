use crate::shared::errors::RepositoryError;
use crate::shared::ids::{AssetID, PortfolioID, UserID};
use async_trait::async_trait;

use super::asset::Asset;
use super::portfolio::Portfolio;

/// Persistence trait for [`Asset`] entities.
#[async_trait]
pub trait AssetRepository: Send + Sync {
    /// Persists an asset.
    async fn save(&self, asset: &Asset) -> Result<(), RepositoryError>;
    /// Retrieves an asset by its unique identifier.
    async fn find_by_id(&self, id: AssetID) -> Result<Option<Asset>, RepositoryError>;
    /// Retrieves an asset by its ticker symbol.
    async fn find_by_ticker(&self, ticker: &str) -> Result<Option<Asset>, RepositoryError>;
    /// Deletes an asset by its unique identifier.
    async fn delete(&self, id: AssetID) -> Result<(), RepositoryError>;
}

/// Persistence trait for [`Portfolio`] entities.
#[async_trait]
pub trait PortfolioRepository: Send + Sync {
    /// Persists a portfolio.
    async fn save(&self, portfolio: &Portfolio) -> Result<(), RepositoryError>;
    /// Retrieves a portfolio by its unique identifier.
    async fn find_by_id(&self, id: PortfolioID) -> Result<Option<Portfolio>, RepositoryError>;
    /// Retrieves all portfolios belonging to a specific user.
    async fn find_by_owner(&self, owner: UserID) -> Result<Vec<Portfolio>, RepositoryError>;
    /// Deletes a portfolio by its unique identifier.
    async fn delete(&self, id: PortfolioID) -> Result<(), RepositoryError>;
}
