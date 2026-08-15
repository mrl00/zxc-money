use crate::shared::errors::RepositoryError;
use crate::shared::ids::{AssetID, PortfolioID};
use async_trait::async_trait;

use super::asset::Asset;
use super::portfolio::Portfolio;

#[async_trait]
pub trait AssetRepository: Send + Sync {
    async fn save(&self, asset: &Asset) -> Result<(), RepositoryError>;
    async fn find_by_id(&self, id: AssetID) -> Result<Option<Asset>, RepositoryError>;
    async fn find_by_ticker(&self, ticker: &str) -> Result<Option<Asset>, RepositoryError>;
    async fn delete(&self, id: AssetID) -> Result<(), RepositoryError>;
}

#[async_trait]
pub trait PortfolioRepository: Send + Sync {
    async fn save(&self, portfolio: &Portfolio) -> Result<(), RepositoryError>;
    async fn find_by_id(&self, id: PortfolioID) -> Result<Option<Portfolio>, RepositoryError>;
    async fn delete(&self, id: PortfolioID) -> Result<(), RepositoryError>;
}
