use std::sync::Arc;

use chrono::Utc;
use rust_decimal::Decimal;

use crate::investment::domain::events::AssetSold;
use crate::investment::domain::repository::PortfolioRepository;
use crate::shared::errors::InvestmentError;
use crate::shared::events::EventPublisher;
use crate::shared::ids::{AssetID, PortfolioID};
use crate::shared::money::Money;

/// Command to record the sale of an asset from a portfolio.
pub struct RecordSellCommand {
    /// The portfolio to sell from.
    pub portfolio_id: PortfolioID,
    /// The asset being sold.
    pub asset_id: AssetID,
    /// Quantity sold (must be positive and ≤ available).
    pub quantity: Decimal,
    /// Per-unit price at time of sale.
    pub price: Money,
}

/// Handler that records an asset sale, updating the portfolio and
/// publishing an [`AssetSold`] event.
///
/// # Flow
///
/// 1. Load the portfolio from the repository.
/// 2. Delegate to [`Portfolio::record_sell`](crate::investment::domain::portfolio::Portfolio::record_sell)
///    for quantity validation and profit calculation.
/// 3. Persist the updated portfolio.
/// 4. Publish [`AssetSold`] for downstream consumers (e.g. Reporting).
///
/// # Errors
///
/// - [`InvestmentError::PortfolioNotFound`] if the portfolio does not exist.
/// - [`InvestmentError::AssetNotFound`] if the asset is not held.
/// - [`InvestmentError::InsufficientQuantity`] if selling more than held.
pub struct RecordSellHandler<P: PortfolioRepository, EP: EventPublisher> {
    portfolio_repository: Arc<P>,
    event_publisher: Arc<EP>,
}

impl<P: PortfolioRepository, EP: EventPublisher> RecordSellHandler<P, EP> {
    /// Creates a new handler with the given dependencies.
    pub fn new(portfolio_repository: Arc<P>, event_publisher: Arc<EP>) -> Self {
        Self {
            portfolio_repository,
            event_publisher,
        }
    }

    /// Executes the record-sell use case.
    pub async fn handle(&self, cmd: RecordSellCommand) -> Result<(), InvestmentError> {
        let mut portfolio = self
            .portfolio_repository
            .find_by_id(cmd.portfolio_id)
            .await?
            .ok_or_else(|| InvestmentError::PortfolioNotFound(cmd.portfolio_id.to_string()))?;

        portfolio.record_sell(cmd.asset_id, cmd.quantity, cmd.price)?;

        self.portfolio_repository.save(&portfolio).await?;

        let event = AssetSold {
            portfolio_id: cmd.portfolio_id,
            owner_id: portfolio.owner_id,
            asset_id: cmd.asset_id,
            quantity: cmd.quantity,
            price: cmd.price,
            timestamp: Utc::now(),
        };

        self.event_publisher
            .publish(vec![&event])
            .await
            .map_err(InvestmentError::Publish)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::investment::domain::asset::AssetClass;
    use crate::investment::domain::portfolio::Portfolio;
    use crate::shared::events::InMemoryEventDispatcher;
    use crate::shared::ids::UserID;
    use crate::shared::mock::MockPortfolioRepository;
    use crate::shared::money::Currency;

    fn brl(amount: i64) -> Money {
        Money::from_cents(amount, Currency::BRL)
    }

    async fn setup_with_position() -> Arc<MockPortfolioRepository> {
        let repo = Arc::new(MockPortfolioRepository::new());
        let mut portfolio = Portfolio::new(PortfolioID::new(), UserID::new());
        portfolio
            .record_buy(
                AssetID::new(),
                Decimal::from(10),
                brl(2500),
                AssetClass::Stock,
            )
            .unwrap();
        repo.save(&portfolio).await.unwrap();
        repo
    }

    #[tokio::test]
    async fn test_record_sell_success() {
        let repo = setup_with_position().await;
        let portfolio = repo.find_all().await.unwrap().into_iter().next().unwrap();
        let asset_id = portfolio.positions[0].asset_id;

        let handler = RecordSellHandler::new(repo, Arc::new(InMemoryEventDispatcher::new()));

        let cmd = RecordSellCommand {
            portfolio_id: portfolio.id,
            asset_id,
            quantity: Decimal::from(5),
            price: brl(3000),
        };

        handler.handle(cmd).await.unwrap();
    }

    #[tokio::test]
    async fn test_record_sell_full_position() {
        let repo = setup_with_position().await;
        let portfolio = repo.find_all().await.unwrap().into_iter().next().unwrap();
        let asset_id = portfolio.positions[0].asset_id;

        let handler =
            RecordSellHandler::new(repo.clone(), Arc::new(InMemoryEventDispatcher::new()));

        let cmd = RecordSellCommand {
            portfolio_id: portfolio.id,
            asset_id,
            quantity: Decimal::from(10),
            price: brl(2500),
        };

        handler.handle(cmd).await.unwrap();

        let portfolio = repo.find_by_id(portfolio.id).await.unwrap().unwrap();
        assert!(portfolio.positions.is_empty());
    }

    #[tokio::test]
    async fn test_record_sell_insufficient_quantity() {
        let repo = setup_with_position().await;
        let portfolio = repo.find_all().await.unwrap().into_iter().next().unwrap();
        let asset_id = portfolio.positions[0].asset_id;

        let handler = RecordSellHandler::new(repo, Arc::new(InMemoryEventDispatcher::new()));

        let cmd = RecordSellCommand {
            portfolio_id: portfolio.id,
            asset_id,
            quantity: Decimal::from(20),
            price: brl(3000),
        };

        let result = handler.handle(cmd).await;
        assert!(matches!(
            result,
            Err(InvestmentError::InsufficientQuantity { .. })
        ));
    }

    #[tokio::test]
    async fn test_record_sell_portfolio_not_found() {
        let handler = RecordSellHandler::new(
            Arc::new(MockPortfolioRepository::new()),
            Arc::new(InMemoryEventDispatcher::new()),
        );

        let cmd = RecordSellCommand {
            portfolio_id: PortfolioID::new(),
            asset_id: AssetID::new(),
            quantity: Decimal::from(5),
            price: brl(3000),
        };

        let result = handler.handle(cmd).await;
        assert!(matches!(result, Err(InvestmentError::PortfolioNotFound(_))));
    }
}
