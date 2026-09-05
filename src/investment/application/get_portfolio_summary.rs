use std::collections::HashMap;
use std::sync::Arc;

use rust_decimal::Decimal;

use crate::investment::domain::repository::PortfolioRepository;
use crate::shared::errors::InvestmentError;
use crate::shared::ids::{AssetID, PortfolioID, Principal};
use crate::shared::money::Money;

/// Query to retrieve a summary of all positions in a portfolio.
pub struct GetPortfolioSummaryQuery {
    /// The portfolio to summarize.
    pub portfolio_id: PortfolioID,
    /// Current market prices keyed by asset ID
    /// (obtained from [`QuoteProvider`](crate::investment::domain::quote::QuoteProvider)).
    pub prices: HashMap<AssetID, Money>,
    /// The authenticated principal.
    pub principal: Principal,
}

/// Summary of a single position within a portfolio.
#[derive(Debug)]
pub struct PositionSummary {
    pub asset_id: AssetID,
    pub quantity: Decimal,
    pub average_cost: Money,
    pub invested: Money,
    pub current_value: Money,
    pub profit: Money,
}

/// Consolidated summary of an entire portfolio.
#[derive(Debug)]
pub struct PortfolioSummary {
    pub portfolio_id: PortfolioID,
    /// Total cost basis across all positions.
    pub total_invested: Money,
    /// Total current market value across all positions.
    pub total_current_value: Money,
    /// Per-position breakdown.
    pub positions: Vec<PositionSummary>,
}

/// Handler that returns a consolidated summary of all positions in a portfolio.
///
/// Requires current market prices to be injected by the caller.
///
/// # Errors
///
/// - [`InvestmentError::PortfolioNotFound`] if the portfolio does not exist.
pub struct GetPortfolioSummaryHandler<P: PortfolioRepository> {
    portfolio_repository: Arc<P>,
}

impl<P: PortfolioRepository> GetPortfolioSummaryHandler<P> {
    /// Creates a new handler with the given repository.
    pub fn new(portfolio_repository: Arc<P>) -> Self {
        Self {
            portfolio_repository,
        }
    }

    /// Executes the portfolio summary query.
    pub async fn handle(
        &self,
        query: GetPortfolioSummaryQuery,
    ) -> Result<PortfolioSummary, InvestmentError> {
        let portfolio = self
            .portfolio_repository
            .find_by_id(query.portfolio_id)
            .await?
            .ok_or_else(|| InvestmentError::PortfolioNotFound(query.portfolio_id.to_string()))?;

        if portfolio.owner_id != query.principal.user_id {
            return Err(InvestmentError::Forbidden(
                "not the owner of this portfolio".into(),
            ));
        }

        let mut total_invested = Money::zero(crate::shared::money::Currency::BRL);
        let mut total_current_value = Money::zero(crate::shared::money::Currency::BRL);
        let mut positions = Vec::new();

        for pos in &portfolio.positions {
            let invested = pos.average_cost * pos.quantity;
            let current_value = query
                .prices
                .get(&pos.asset_id)
                .map(|price| *price * pos.quantity)
                .unwrap_or_else(|| Money::zero(pos.average_cost.currency()));

            let profit = (current_value - invested)
                .map_err(|_| InvestmentError::InvariantViolation("currency mismatch".into()))?;

            total_invested = (total_invested + invested)
                .map_err(|_| InvestmentError::InvariantViolation("currency mismatch".into()))?;
            total_current_value = (total_current_value + current_value)
                .map_err(|_| InvestmentError::InvariantViolation("currency mismatch".into()))?;

            positions.push(PositionSummary {
                asset_id: pos.asset_id,
                quantity: pos.quantity,
                average_cost: pos.average_cost,
                invested,
                current_value,
                profit,
            });
        }

        Ok(PortfolioSummary {
            portfolio_id: query.portfolio_id,
            total_invested,
            total_current_value,
            positions,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::investment::domain::asset::AssetClass;
    use crate::investment::domain::portfolio::Portfolio;
    use crate::shared::ids::{PortfolioID, Principal, UserID};
    use crate::shared::mock::MockPortfolioRepository;
    use crate::shared::money::Currency;

    fn brl(amount: i64) -> Money {
        Money::from_cents(amount, Currency::BRL)
    }

    async fn setup_two_positions() -> (
        Arc<MockPortfolioRepository>,
        PortfolioID,
        AssetID,
        AssetID,
        UserID,
    ) {
        let repo = Arc::new(MockPortfolioRepository::new());
        let owner = UserID::new();
        let mut portfolio = Portfolio::new(PortfolioID::new(), owner);
        let a1 = AssetID::new();
        let a2 = AssetID::new();
        portfolio
            .record_buy(a1, Decimal::from(10), brl(2500), AssetClass::Stock)
            .unwrap();
        portfolio
            .record_buy(a2, Decimal::from(5), brl(5000), AssetClass::Fund)
            .unwrap();
        let pid = portfolio.id;
        repo.save(&portfolio).await.unwrap();
        (repo, pid, a1, a2, owner)
    }

    #[tokio::test]
    async fn test_portfolio_summary_two_positions() {
        let (repo, pid, a1, a2, owner) = setup_two_positions().await;
        let handler = GetPortfolioSummaryHandler::new(repo);

        let mut prices = HashMap::new();
        prices.insert(a1, brl(3000));
        prices.insert(a2, brl(4800));

        let summary = handler
            .handle(GetPortfolioSummaryQuery {
                portfolio_id: pid,
                prices,
                principal: Principal::new(owner),
            })
            .await
            .unwrap();

        assert_eq!(summary.positions.len(), 2);
        // invested: 10*25.00 + 5*50.00 = 500.00
        assert_eq!(summary.total_invested.amount(), Decimal::from(500));
        // current: 10*30.00 + 5*48.00 = 540.00
        assert_eq!(summary.total_current_value.amount(), Decimal::from(540));
    }

    #[tokio::test]
    async fn test_portfolio_summary_empty() {
        let repo = Arc::new(MockPortfolioRepository::new());
        let owner = UserID::new();
        let portfolio = Portfolio::new(PortfolioID::new(), owner);
        let pid = portfolio.id;
        repo.save(&portfolio).await.unwrap();

        let handler = GetPortfolioSummaryHandler::new(repo);

        let summary = handler
            .handle(GetPortfolioSummaryQuery {
                portfolio_id: pid,
                prices: HashMap::new(),
                principal: Principal::new(owner),
            })
            .await
            .unwrap();

        assert!(summary.positions.is_empty());
        assert_eq!(summary.total_invested.amount(), Decimal::ZERO);
        assert_eq!(summary.total_current_value.amount(), Decimal::ZERO);
    }

    #[tokio::test]
    async fn test_portfolio_summary_missing_prices() {
        let (repo, pid, a1, _a2, owner) = setup_two_positions().await;
        let handler = GetPortfolioSummaryHandler::new(repo);

        let mut prices = HashMap::new();
        prices.insert(a1, brl(3000));
        // a2 price missing — should default to zero

        let summary = handler
            .handle(GetPortfolioSummaryQuery {
                portfolio_id: pid,
                prices,
                principal: Principal::new(owner),
            })
            .await
            .unwrap();

        assert_eq!(summary.positions.len(), 2);
        // Only a1 contributes to current_value
        assert_eq!(summary.total_current_value.amount(), Decimal::from(300));
    }

    #[tokio::test]
    async fn test_portfolio_summary_not_found() {
        let repo = Arc::new(MockPortfolioRepository::new());
        let handler = GetPortfolioSummaryHandler::new(repo);

        let result = handler
            .handle(GetPortfolioSummaryQuery {
                portfolio_id: PortfolioID::new(),
                prices: HashMap::new(),
                principal: Principal::new(UserID::new()),
            })
            .await;

        assert!(matches!(result, Err(InvestmentError::PortfolioNotFound(_))));
    }

    #[tokio::test]
    async fn test_portfolio_summary_wrong_owner() {
        let (repo, pid, _a1, _a2, _owner) = setup_two_positions().await;
        let handler = GetPortfolioSummaryHandler::new(repo);

        let wrong_owner = UserID::new();

        let result = handler
            .handle(GetPortfolioSummaryQuery {
                principal: Principal::new(wrong_owner),
                portfolio_id: pid,
                prices: HashMap::new(),
            })
            .await;

        assert!(matches!(result, Err(InvestmentError::Forbidden(_))));
    }
}
