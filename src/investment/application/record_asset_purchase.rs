use std::sync::Arc;

use chrono::Utc;
use rust_decimal::Decimal;

use crate::investment::domain::events::AssetBought;
use crate::investment::domain::repository::{AssetRepository, PortfolioRepository};
use crate::shared::errors::InvestmentError;
use crate::shared::events::EventPublisher;
use crate::shared::ids::{AssetID, PortfolioID};
use crate::shared::money::Money;

/// Command to record the purchase of an asset within a portfolio.
pub struct RecordBuyCommand {
    /// The portfolio to add the position to.
    pub portfolio_id: PortfolioID,
    /// The asset being purchased.
    pub asset_id: AssetID,
    /// Quantity purchased (must be positive).
    pub quantity: Decimal,
    /// Per-unit price at time of purchase.
    pub price: Money,
}

/// Handler that records an asset purchase, updating the portfolio and
/// publishing an [`AssetBought`] event.
///
/// # Flow
///
/// 1. Load the portfolio from the repository.
/// 2. Load the asset to obtain its [`AssetClass`](crate::investment::domain::asset::AssetClass).
/// 3. Delegate to [`Portfolio::record_buy`](crate::investment::domain::portfolio::Portfolio::record_buy)
///    for average-cost recalculation.
/// 4. Persist the updated portfolio.
/// 5. Publish [`AssetBought`] for downstream consumers (e.g. Reporting).
///
/// # Errors
///
/// - [`InvestmentError::PortfolioNotFound`] if the portfolio does not exist.
/// - [`InvestmentError::AssetNotFound`] if the asset does not exist.
/// - [`InvestmentError::InvariantViolation`] on domain rule violations.
pub struct RecordBuyHandler<P: PortfolioRepository, A: AssetRepository, EP: EventPublisher> {
    portfolio_repository: Arc<P>,
    asset_repository: Arc<A>,
    event_publisher: Arc<EP>,
}

impl<P: PortfolioRepository, A: AssetRepository, EP: EventPublisher> RecordBuyHandler<P, A, EP> {
    /// Creates a new handler with the given dependencies.
    pub fn new(
        portfolio_repository: Arc<P>,
        asset_repository: Arc<A>,
        event_publisher: Arc<EP>,
    ) -> Self {
        Self {
            portfolio_repository,
            asset_repository,
            event_publisher,
        }
    }

    /// Executes the record-buy use case.
    pub async fn handle(&self, cmd: RecordBuyCommand) -> Result<(), InvestmentError> {
        let mut portfolio = self
            .portfolio_repository
            .find_by_id(cmd.portfolio_id)
            .await?
            .ok_or_else(|| InvestmentError::PortfolioNotFound(cmd.portfolio_id.to_string()))?;

        let asset = self
            .asset_repository
            .find_by_id(cmd.asset_id)
            .await?
            .ok_or_else(|| InvestmentError::AssetNotFound(cmd.asset_id.to_string()))?;

        portfolio.record_buy(cmd.asset_id, cmd.quantity, cmd.price, asset.class)?;

        self.portfolio_repository.save(&portfolio).await?;

        let event = AssetBought {
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
    use crate::shared::mock::{MockAssetRepository, MockPortfolioRepository};
    use crate::shared::money::Currency;

    fn brl(amount: i64) -> Money {
        Money::new(amount, Currency::BRL)
    }

    async fn setup() -> (
        Arc<MockPortfolioRepository>,
        Arc<MockAssetRepository>,
        Arc<InMemoryEventDispatcher>,
    ) {
        let portfolio_repo = Arc::new(MockPortfolioRepository::new());
        let asset_repo = Arc::new(MockAssetRepository::new());
        let publisher = Arc::new(InMemoryEventDispatcher::new());

        // Create a portfolio
        let mut portfolio = Portfolio::new(PortfolioID::new(), UserID::new());
        portfolio_repo.save(&portfolio).await.unwrap();

        // Register an asset
        let asset = crate::investment::domain::asset::Asset::new(
            AssetID::new(),
            "PETR4".into(),
            "Petrobras".into(),
            AssetClass::Stock,
            Currency::BRL,
        )
        .unwrap();
        asset_repo.save(&asset).await.unwrap();

        (portfolio_repo, asset_repo, publisher)
    }

    #[tokio::test]
    async fn test_record_buy_success() {
        let (portfolio_repo, asset_repo, publisher) = setup().await;

        let portfolio = portfolio_repo
            .find_all()
            .await
            .unwrap()
            .into_iter()
            .next()
            .unwrap();
        let asset = asset_repo.find_by_ticker("PETR4").await.unwrap().unwrap();

        let handler = RecordBuyHandler::new(portfolio_repo, asset_repo, publisher);

        let cmd = RecordBuyCommand {
            portfolio_id: portfolio.id,
            asset_id: asset.id,
            quantity: Decimal::from(10),
            price: brl(2500),
        };

        handler.handle(cmd).await.unwrap();
    }

    #[tokio::test]
    async fn test_record_buy_portfolio_not_found() {
        let asset_repo = Arc::new(MockAssetRepository::new());
        let asset = crate::investment::domain::asset::Asset::new(
            AssetID::new(),
            "PETR4".into(),
            "Petrobras".into(),
            AssetClass::Stock,
            Currency::BRL,
        )
        .unwrap();
        asset_repo.save(&asset).await.unwrap();

        let handler = RecordBuyHandler::new(
            Arc::new(MockPortfolioRepository::new()),
            asset_repo,
            Arc::new(InMemoryEventDispatcher::new()),
        );

        let cmd = RecordBuyCommand {
            portfolio_id: PortfolioID::new(),
            asset_id: asset.id,
            quantity: Decimal::from(10),
            price: brl(2500),
        };

        let result = handler.handle(cmd).await;
        assert!(matches!(result, Err(InvestmentError::PortfolioNotFound(_))));
    }

    #[tokio::test]
    async fn test_record_buy_asset_not_found() {
        let portfolio_repo = Arc::new(MockPortfolioRepository::new());
        let portfolio = Portfolio::new(PortfolioID::new(), UserID::new());
        portfolio_repo.save(&portfolio).await.unwrap();

        let handler = RecordBuyHandler::new(
            portfolio_repo,
            Arc::new(MockAssetRepository::new()),
            Arc::new(InMemoryEventDispatcher::new()),
        );

        let cmd = RecordBuyCommand {
            portfolio_id: portfolio.id,
            asset_id: AssetID::new(),
            quantity: Decimal::from(10),
            price: brl(2500),
        };

        let result = handler.handle(cmd).await;
        assert!(matches!(result, Err(InvestmentError::AssetNotFound(_))));
    }
}
