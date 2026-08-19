use std::sync::Arc;

use rust_decimal::Decimal;

use crate::investment::domain::repository::PortfolioRepository;
use crate::shared::errors::InvestmentError;
use crate::shared::ids::{AssetID, PortfolioID};
use crate::shared::money::Money;

/// Query to retrieve profitability for a single asset position.
pub struct GetProfitabilityQuery {
    /// The portfolio containing the position.
    pub portfolio_id: PortfolioID,
    /// The asset to calculate profitability for.
    pub asset_id: AssetID,
    /// Current market price per unit (obtained from [`QuoteProvider`](crate::investment::domain::quote::QuoteProvider)).
    pub current_price: Money,
}

/// Result of a profitability calculation for a single position.
#[derive(Debug)]
pub struct ProfitabilityResult {
    /// The asset this result refers to.
    pub asset_id: AssetID,
    /// Quantity held.
    pub quantity: Decimal,
    /// Total amount invested (cost basis = average_cost × quantity).
    pub invested: Money,
    /// Current market value (current_price × quantity).
    pub current_value: Money,
    /// Absolute profit or loss (current_value − invested).
    pub profit: Money,
    /// Percentage return on investment (profit / invested × 100).
    pub profit_pct: f64,
}

/// Handler that calculates profitability for a single asset position.
///
/// Requires the current market price to be injected by the caller (typically
/// obtained via [`QuoteProvider`](crate::investment::domain::quote::QuoteProvider)).
///
/// # Errors
///
/// - [`InvestmentError::PortfolioNotFound`] if the portfolio does not exist.
/// - [`InvestmentError::AssetNotFound`] if the asset is not held in the portfolio.
pub struct GetProfitabilityHandler<P: PortfolioRepository> {
    portfolio_repository: Arc<P>,
}

impl<P: PortfolioRepository> GetProfitabilityHandler<P> {
    /// Creates a new handler with the given repository.
    pub fn new(portfolio_repository: Arc<P>) -> Self {
        Self {
            portfolio_repository,
        }
    }

    /// Executes the profitability query.
    pub async fn handle(
        &self,
        query: GetProfitabilityQuery,
    ) -> Result<ProfitabilityResult, InvestmentError> {
        let portfolio = self
            .portfolio_repository
            .find_by_id(query.portfolio_id)
            .await?
            .ok_or_else(|| InvestmentError::PortfolioNotFound(query.portfolio_id.to_string()))?;

        let position = portfolio
            .positions
            .iter()
            .find(|p| p.asset_id == query.asset_id)
            .ok_or_else(|| InvestmentError::AssetNotFound(query.asset_id.to_string()))?;

        let invested = position.average_cost * position.quantity;
        let current_value = query.current_price * position.quantity;
        let profit = current_value - invested;

        let profit_pct = if invested.amount() == 0 {
            0.0
        } else {
            (profit.amount() as f64 / invested.amount() as f64) * 100.0
        };

        Ok(ProfitabilityResult {
            asset_id: query.asset_id,
            quantity: position.quantity,
            invested,
            current_value,
            profit,
            profit_pct,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::investment::domain::asset::AssetClass;
    use crate::investment::domain::portfolio::Portfolio;
    use crate::shared::ids::{PortfolioID, UserID};
    use crate::shared::mock::MockPortfolioRepository;
    use crate::shared::money::Currency;

    fn brl(amount: i64) -> Money {
        Money::new(amount, Currency::BRL)
    }

    async fn setup_with_position() -> (Arc<MockPortfolioRepository>, PortfolioID, AssetID) {
        let repo = Arc::new(MockPortfolioRepository::new());
        let mut portfolio = Portfolio::new(PortfolioID::new(), UserID::new());
        let asset_id = AssetID::new();
        portfolio
            .record_buy(asset_id, Decimal::from(10), brl(2500), AssetClass::Stock)
            .unwrap();
        let pid = portfolio.id;
        repo.save(&portfolio).await.unwrap();
        (repo, pid, asset_id)
    }

    #[tokio::test]
    async fn test_profitability_positive() {
        let (repo, pid, aid) = setup_with_position().await;
        let handler = GetProfitabilityHandler::new(repo);

        let result = handler
            .handle(GetProfitabilityQuery {
                portfolio_id: pid,
                asset_id: aid,
                current_price: brl(3000),
            })
            .await
            .unwrap();

        assert_eq!(result.quantity, Decimal::from(10));
        assert_eq!(result.invested.amount(), 25000); // 10 × 2500
        assert_eq!(result.current_value.amount(), 30000); // 10 × 3000
        assert_eq!(result.profit.amount(), 5000);
        assert!((result.profit_pct - 20.0).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_profitability_negative() {
        let (repo, pid, aid) = setup_with_position().await;
        let handler = GetProfitabilityHandler::new(repo);

        let result = handler
            .handle(GetProfitabilityQuery {
                portfolio_id: pid,
                asset_id: aid,
                current_price: brl(2000),
            })
            .await
            .unwrap();

        assert_eq!(result.profit.amount(), -5000);
        assert!((result.profit_pct - (-20.0)).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_profitability_asset_not_found() {
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
        let pid = portfolio.id;
        repo.save(&portfolio).await.unwrap();

        let handler = GetProfitabilityHandler::new(repo);

        let result = handler
            .handle(GetProfitabilityQuery {
                portfolio_id: pid,
                asset_id: AssetID::new(),
                current_price: brl(3000),
            })
            .await;

        assert!(matches!(result, Err(InvestmentError::AssetNotFound(_))));
    }
}
