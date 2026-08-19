use std::sync::Arc;

use crate::investment::application::get_portfolio_summary::{
    GetPortfolioSummaryHandler, GetPortfolioSummaryQuery, PortfolioSummary,
};
use crate::investment::application::get_profitability::{
    GetProfitabilityHandler, GetProfitabilityQuery, ProfitabilityResult,
};
use crate::investment::application::record_asset_purchase::{RecordBuyCommand, RecordBuyHandler};
use crate::investment::application::record_asset_sale::{RecordSellCommand, RecordSellHandler};
use crate::investment::application::register_asset::{RegisterAssetCommand, RegisterAssetHandler};
use crate::investment::domain::asset::Asset;
use crate::investment::domain::portfolio::Portfolio;
use crate::investment::domain::repository::{AssetRepository, PortfolioRepository};
use crate::provider::id::IdGenerator;
use crate::shared::errors::InvestmentError;
use crate::shared::events::EventPublisher;
use crate::shared::ids::{AssetID, PortfolioID};

/// Facade for the Investment bounded context.
///
/// Aggregates all command and query handlers behind a single entry point,
/// following the facade pattern used across all bounded contexts.
///
/// # Example
///
/// ```ignore
/// let facade = InvestmentFacade::new(asset_repo, portfolio_repo, event_publisher, id_gen);
///
/// let asset_id = facade.register_asset(RegisterAssetCommand { ... }).await?;
/// facade.record_buy(RecordBuyCommand { ... }).await?;
/// ```
pub struct InvestmentFacade<
    A: AssetRepository,
    P: PortfolioRepository,
    EP: EventPublisher,
    I: IdGenerator,
> {
    register_asset: RegisterAssetHandler<A, I>,
    record_buy: RecordBuyHandler<P, A, EP>,
    record_sell: RecordSellHandler<P, EP>,
    get_profitability: GetProfitabilityHandler<P>,
    get_portfolio_summary: GetPortfolioSummaryHandler<P>,
    asset_repository: Arc<A>,
    portfolio_repository: Arc<P>,
}

impl<A: AssetRepository, P: PortfolioRepository, EP: EventPublisher, I: IdGenerator>
    InvestmentFacade<A, P, EP, I>
{
    /// Creates a new facade with shared dependencies.
    pub fn new(
        asset_repository: Arc<A>,
        portfolio_repository: Arc<P>,
        event_publisher: Arc<EP>,
        id_generator: Arc<I>,
    ) -> Self {
        Self {
            register_asset: RegisterAssetHandler::new(
                asset_repository.clone(),
                id_generator.clone(),
            ),
            record_buy: RecordBuyHandler::new(
                portfolio_repository.clone(),
                asset_repository.clone(),
                event_publisher.clone(),
            ),
            record_sell: RecordSellHandler::new(portfolio_repository.clone(), event_publisher),
            get_profitability: GetProfitabilityHandler::new(portfolio_repository.clone()),
            get_portfolio_summary: GetPortfolioSummaryHandler::new(portfolio_repository.clone()),
            asset_repository,
            portfolio_repository,
        }
    }

    /// Registers a new asset in the shared catalog.
    pub async fn register_asset(
        &self,
        cmd: RegisterAssetCommand,
    ) -> Result<AssetID, InvestmentError> {
        self.register_asset.handle(cmd).await
    }

    /// Records the purchase of an asset within a portfolio.
    pub async fn record_buy(&self, cmd: RecordBuyCommand) -> Result<(), InvestmentError> {
        self.record_buy.handle(cmd).await
    }

    /// Records the sale of an asset from a portfolio.
    pub async fn record_sell(&self, cmd: RecordSellCommand) -> Result<(), InvestmentError> {
        self.record_sell.handle(cmd).await
    }

    /// Calculates profitability for a single asset position.
    pub async fn get_profitability(
        &self,
        query: GetProfitabilityQuery,
    ) -> Result<ProfitabilityResult, InvestmentError> {
        self.get_profitability.handle(query).await
    }

    /// Returns a consolidated summary of all positions in a portfolio.
    pub async fn get_portfolio_summary(
        &self,
        query: GetPortfolioSummaryQuery,
    ) -> Result<PortfolioSummary, InvestmentError> {
        self.get_portfolio_summary.handle(query).await
    }

    /// Finds an asset by its ticker symbol.
    pub async fn find_asset_by_ticker(
        &self,
        ticker: &str,
    ) -> Result<Option<Asset>, InvestmentError> {
        Ok(self.asset_repository.find_by_ticker(ticker).await?)
    }

    /// Finds a portfolio by its ID.
    pub async fn get_portfolio(
        &self,
        portfolio_id: PortfolioID,
    ) -> Result<Option<Portfolio>, InvestmentError> {
        Ok(self.portfolio_repository.find_by_id(portfolio_id).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::investment::domain::asset::AssetClass;
    use crate::investment::domain::portfolio::Portfolio;
    use crate::provider::id::MockIdGenerator;
    use crate::shared::events::InMemoryEventDispatcher;
    use crate::shared::ids::{PortfolioID, UserID};
    use crate::shared::mock::{MockAssetRepository, MockPortfolioRepository};
    use crate::shared::money::{Currency, Money};

    fn brl(amount: i64) -> Money {
        Money::new(amount, Currency::BRL)
    }

    async fn setup() -> (
        Arc<MockAssetRepository>,
        Arc<MockPortfolioRepository>,
        Arc<InMemoryEventDispatcher>,
    ) {
        let asset_repo = Arc::new(MockAssetRepository::new());
        let portfolio_repo = Arc::new(MockPortfolioRepository::new());
        let publisher = Arc::new(InMemoryEventDispatcher::new());

        let asset = crate::investment::domain::asset::Asset::new(
            AssetID::new(),
            "PETR4".into(),
            "Petrobras".into(),
            AssetClass::Stock,
            Currency::BRL,
        )
        .unwrap();
        asset_repo.save(&asset).await.unwrap();

        let mut portfolio = Portfolio::new(PortfolioID::new(), UserID::new());
        portfolio_repo.save(&portfolio).await.unwrap();

        (asset_repo, portfolio_repo, publisher)
    }

    #[tokio::test]
    async fn test_facade_register_then_buy() {
        let (asset_repo, portfolio_repo, publisher) = setup().await;
        let id_gen = Arc::new(MockIdGenerator::new(uuid::Uuid::new_v4()));

        let facade = InvestmentFacade::new(
            asset_repo.clone(),
            portfolio_repo.clone(),
            publisher,
            id_gen,
        );

        // Register a new asset
        let asset_id = facade
            .register_asset(RegisterAssetCommand {
                ticker: "VALE3".into(),
                name: "Vale".into(),
                class: AssetClass::Stock,
                currency: Currency::BRL,
            })
            .await
            .unwrap();

        let portfolio = portfolio_repo
            .find_all()
            .await
            .unwrap()
            .into_iter()
            .next()
            .unwrap();

        // Record a buy
        facade
            .record_buy(RecordBuyCommand {
                portfolio_id: portfolio.id,
                asset_id,
                quantity: rust_decimal::Decimal::from(10),
                price: brl(6500),
            })
            .await
            .unwrap();

        // Verify profitability
        let result = facade
            .get_profitability(GetProfitabilityQuery {
                portfolio_id: portfolio.id,
                asset_id,
                current_price: brl(7000),
            })
            .await
            .unwrap();

        assert_eq!(result.invested.amount(), 65000);
        assert_eq!(result.current_value.amount(), 70000);
    }
}
