use std::sync::Arc;

use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

use crate::investment::domain::repository::PortfolioRepository;
use crate::shared::errors::InvestmentError;
use crate::shared::ids::{AssetID, PortfolioID, Principal};
use crate::shared::money::Money;

/// Query to retrieve profitability for a single asset position.
pub struct GetProfitabilityQuery {
    /// The portfolio containing the position.
    pub portfolio_id: PortfolioID,
    /// The asset to calculate profitability for.
    pub asset_id: AssetID,
    /// Current market price per unit (obtained from [`QuoteProvider`](crate::investment::domain::quote::QuoteProvider)).
    pub current_price: Money,
    /// The authenticated principal.
    pub principal: Principal,
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

        if portfolio.owner_id != query.principal.user_id {
            return Err(InvestmentError::Forbidden(
                "not the owner of this portfolio".into(),
            ));
        }

        let position = portfolio
            .positions
            .iter()
            .find(|p| p.asset_id == query.asset_id)
            .ok_or_else(|| InvestmentError::AssetNotFound(query.asset_id.to_string()))?;

        let invested = position.average_cost * position.quantity;
        let current_value = query.current_price * position.quantity;
        let profit = (current_value - invested)
            .map_err(|_| InvestmentError::InvariantViolation("currency mismatch".into()))?;

        let profit_pct = if invested.amount() == Decimal::ZERO {
            0.0
        } else {
            (profit.amount().to_f64().unwrap_or(0.0) / invested.amount().to_f64().unwrap_or(0.0))
                * 100.0
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
    use crate::shared::ids::{PortfolioID, Principal, UserID};
    use crate::shared::mock::MockPortfolioRepository;
    use crate::shared::money::Currency;

    fn brl(amount: i64) -> Money {
        Money::from_cents(amount, Currency::BRL)
    }

    async fn setup_with_position() -> (Arc<MockPortfolioRepository>, PortfolioID, AssetID, UserID) {
        let repo = Arc::new(MockPortfolioRepository::new());
        let owner = UserID::new();
        let mut portfolio = Portfolio::new(PortfolioID::new(), owner);
        let asset_id = AssetID::new();
        portfolio
            .record_buy(asset_id, Decimal::from(10), brl(2500), AssetClass::Stock)
            .unwrap();
        let pid = portfolio.id;
        repo.save(&portfolio).await.unwrap();
        (repo, pid, asset_id, owner)
    }

    #[tokio::test]
    async fn test_profitability_positive() {
        let (repo, pid, aid, owner) = setup_with_position().await;
        let handler = GetProfitabilityHandler::new(repo);

        let result = handler
            .handle(GetProfitabilityQuery {
                portfolio_id: pid,
                asset_id: aid,
                current_price: brl(3000),
                principal: Principal::new(owner),
            })
            .await
            .unwrap();

        assert_eq!(result.quantity, Decimal::from(10));
        assert_eq!(result.invested.amount(), Decimal::from(250)); // 10 × 25.00
        assert_eq!(result.current_value.amount(), Decimal::from(300)); // 10 × 30.00
        assert_eq!(result.profit.amount(), Decimal::from(50));
        assert!((result.profit_pct - 20.0).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_profitability_negative() {
        let (repo, pid, aid, owner) = setup_with_position().await;
        let handler = GetProfitabilityHandler::new(repo);

        let result = handler
            .handle(GetProfitabilityQuery {
                portfolio_id: pid,
                asset_id: aid,
                current_price: brl(2000),
                principal: Principal::new(owner),
            })
            .await
            .unwrap();

        assert_eq!(result.profit.amount(), Decimal::from(-50));
        assert!((result.profit_pct - (-20.0)).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_profitability_asset_not_found() {
        let repo = Arc::new(MockPortfolioRepository::new());
        let owner = UserID::new();
        let mut portfolio = Portfolio::new(PortfolioID::new(), owner);
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
                principal: Principal::new(owner),
            })
            .await;

        assert!(matches!(result, Err(InvestmentError::AssetNotFound(_))));
    }

    #[tokio::test]
    async fn test_profitability_wrong_owner() {
        let (repo, pid, aid, _owner) = setup_with_position().await;
        let handler = GetProfitabilityHandler::new(repo);

        let wrong_owner = UserID::new();

        let result = handler
            .handle(GetProfitabilityQuery {
                principal: Principal::new(wrong_owner),
                portfolio_id: pid,
                asset_id: aid,
                current_price: brl(3000),
            })
            .await;

        assert!(matches!(result, Err(InvestmentError::Forbidden(_))));
    }
}
