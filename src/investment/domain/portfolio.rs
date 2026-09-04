use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};

use crate::investment::domain::asset::AssetClass;
use crate::investment::domain::position::Position;
use crate::shared::errors::InvestmentError;
use crate::shared::ids::{AssetID, PortfolioID, UserID};
use crate::shared::money::Money;

/// A collection of investment positions owned by a user.
///
/// The portfolio is the aggregate root for investment holdings. It enforces
/// invariants on position quantities and average-cost calculations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Portfolio {
    /// Unique identifier.
    pub id: PortfolioID,
    /// Owner of this portfolio.
    pub owner_id: UserID,
    /// Current positions held.
    pub positions: Vec<Position>,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
}

impl Portfolio {
    /// Creates a new empty [`Portfolio`].
    pub fn new(id: PortfolioID, owner_id: UserID) -> Self {
        Self {
            id,
            owner_id,
            positions: Vec::new(),
            created_at: Utc::now(),
        }
    }

    /// Records the purchase of an asset.
    ///
    /// If the asset is already held, the position's average cost is
    /// recalculated as a weighted average. Otherwise a new position is created.
    ///
    /// # Errors
    ///
    /// Returns [`InvestmentError::InvariantViolation`] if `quantity` is zero
    /// or negative, or if the price currency does not match the existing
    /// position's currency.
    pub fn record_buy(
        &mut self,
        asset_id: AssetID,
        quantity: Decimal,
        price: Money,
        asset_class: AssetClass,
    ) -> Result<(), InvestmentError> {
        if quantity.is_zero() || quantity.is_sign_negative() {
            return Err(InvestmentError::InvariantViolation(
                "quantity must be positive".into(),
            ));
        }

        if let Some(position) = self.positions.iter_mut().find(|p| p.asset_id == asset_id) {
            // Validate currency consistency
            if position.average_cost.currency() != price.currency() {
                return Err(InvestmentError::InvariantViolation(format!(
                    "currency mismatch: position uses {}, got {}",
                    position.average_cost.currency().code(),
                    price.currency().code(),
                )));
            }

            let old_total_cents = Decimal::from(position.average_cost.amount()) * position.quantity;
            let new_total_cents = old_total_cents + Decimal::from(price.amount()) * quantity;
            let new_quantity = position.quantity + quantity;

            // new average cost per unit in cents
            let new_avg_cents = new_total_cents / new_quantity;
            let new_avg_i64 = new_avg_cents.to_i64().ok_or_else(|| {
                InvestmentError::InvariantViolation("average cost overflow".into())
            })?;

            position.average_cost = Money::new(new_avg_i64, price.currency());
            position.quantity = new_quantity;
        } else {
            self.positions.push(Position {
                asset_id,
                quantity,
                average_cost: price,
                asset_class,
            });
        }

        Ok(())
    }

    /// Records the sale of an asset.
    ///
    /// Returns the realized profit/loss (sale proceeds minus cost basis).
    /// Removes the position entirely if the full quantity is sold.
    ///
    /// # Errors
    ///
    /// Returns [`InvestmentError::AssetNotFound`] if the asset is not held,
    /// or [`InvestmentError::InsufficientQuantity`] if trying to sell more
    /// than available.
    pub fn record_sell(
        &mut self,
        asset_id: AssetID,
        quantity: Decimal,
        price: Money,
    ) -> Result<Money, InvestmentError> {
        if quantity.is_zero() || quantity.is_sign_negative() {
            return Err(InvestmentError::InvariantViolation(
                "quantity must be positive".into(),
            ));
        }

        let position = self
            .positions
            .iter_mut()
            .find(|p| p.asset_id == asset_id)
            .ok_or_else(|| InvestmentError::AssetNotFound(asset_id.to_string()))?;

        if position.quantity < quantity {
            return Err(InvestmentError::InsufficientQuantity {
                available: position.quantity.to_string(),
                requested: quantity.to_string(),
            });
        }

        let sale_proceeds = price * quantity;
        let cost_basis = position.average_cost * quantity;
        let profit = sale_proceeds - cost_basis;

        position.quantity -= quantity;

        if position.quantity.is_zero() {
            self.positions.retain(|p| p.asset_id != asset_id);
        }

        Ok(profit)
    }

    /// Calculates the total portfolio value given current market prices.
    ///
    /// Returns `None` if the portfolio has no positions with available prices.
    pub fn total_value(&self, prices: &std::collections::HashMap<AssetID, Money>) -> Option<Money> {
        let mut total = None;

        for position in &self.positions {
            if let Some(price) = prices.get(&position.asset_id) {
                let value = *price * position.quantity;
                total = Some(match total {
                    None => value,
                    Some(t) => t + value,
                });
            }
        }

        total
    }

    /// Returns the total amount invested (cost basis) across all positions.
    pub fn total_invested(&self) -> Option<Money> {
        let mut total = None;

        for position in &self.positions {
            let cost = position.average_cost * position.quantity;
            total = Some(match total {
                None => cost,
                Some(t) => t + cost,
            });
        }

        total
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::ids::{AssetID, PortfolioID, UserID};
    use crate::shared::money::{Currency, Money};

    fn brl(amount: i64) -> Money {
        Money::new(amount, Currency::BRL)
    }

    fn usd(amount: i64) -> Money {
        Money::new(amount, Currency::USD)
    }

    #[test]
    fn test_record_buy_new_position() {
        let mut portfolio = Portfolio::new(PortfolioID::new(), UserID::new());
        let asset_id = AssetID::new();

        portfolio
            .record_buy(asset_id, Decimal::from(10), brl(2500), AssetClass::Stock)
            .unwrap();

        assert_eq!(portfolio.positions.len(), 1);
        let pos = &portfolio.positions[0];
        assert_eq!(pos.asset_id, asset_id);
        assert_eq!(pos.quantity, Decimal::from(10));
        assert_eq!(pos.average_cost, brl(2500));
    }

    #[test]
    fn test_record_buy_updates_average_cost() {
        let mut portfolio = Portfolio::new(PortfolioID::new(), UserID::new());
        let asset_id = AssetID::new();

        // Buy 10 at R$ 25.00
        portfolio
            .record_buy(asset_id, Decimal::from(10), brl(2500), AssetClass::Stock)
            .unwrap();
        // Buy 5 at R$ 30.00
        portfolio
            .record_buy(asset_id, Decimal::from(5), brl(3000), AssetClass::Stock)
            .unwrap();

        let pos = &portfolio.positions[0];
        assert_eq!(pos.quantity, Decimal::from(15));
        // avg = (10*2500 + 5*3000) / 15 = 40000/15 = 2666.666... → 2666
        assert_eq!(pos.average_cost.amount(), 2666);
    }

    #[test]
    fn test_record_buy_zero_quantity_fails() {
        let mut portfolio = Portfolio::new(PortfolioID::new(), UserID::new());
        let result =
            portfolio.record_buy(AssetID::new(), Decimal::ZERO, brl(2500), AssetClass::Stock);
        assert!(result.is_err());
    }

    #[test]
    fn test_record_buy_currency_mismatch_fails() {
        let mut portfolio = Portfolio::new(PortfolioID::new(), UserID::new());
        let asset_id = AssetID::new();

        portfolio
            .record_buy(asset_id, Decimal::from(10), brl(2500), AssetClass::Stock)
            .unwrap();
        let result = portfolio.record_buy(asset_id, Decimal::from(5), usd(3000), AssetClass::Stock);
        assert!(result.is_err());
    }

    #[test]
    fn test_record_sell_partial() {
        let mut portfolio = Portfolio::new(PortfolioID::new(), UserID::new());
        let asset_id = AssetID::new();

        portfolio
            .record_buy(asset_id, Decimal::from(10), brl(2500), AssetClass::Stock)
            .unwrap();
        let profit = portfolio
            .record_sell(asset_id, Decimal::from(5), brl(3000))
            .unwrap();

        // profit = (3000 - 2500) * 5 = 2500
        assert_eq!(profit.amount(), 2500);

        let pos = &portfolio.positions[0];
        assert_eq!(pos.quantity, Decimal::from(5));
    }

    #[test]
    fn test_record_sell_full_removes_position() {
        let mut portfolio = Portfolio::new(PortfolioID::new(), UserID::new());
        let asset_id = AssetID::new();

        portfolio
            .record_buy(asset_id, Decimal::from(10), brl(2500), AssetClass::Stock)
            .unwrap();
        portfolio
            .record_sell(asset_id, Decimal::from(10), brl(2500))
            .unwrap();

        assert!(portfolio.positions.is_empty());
    }

    #[test]
    fn test_record_sell_insufficient_quantity() {
        let mut portfolio = Portfolio::new(PortfolioID::new(), UserID::new());
        let asset_id = AssetID::new();

        portfolio
            .record_buy(asset_id, Decimal::from(10), brl(2500), AssetClass::Stock)
            .unwrap();
        let result = portfolio.record_sell(asset_id, Decimal::from(20), brl(3000));

        assert!(result.is_err());
    }

    #[test]
    fn test_record_sell_asset_not_found() {
        let mut portfolio = Portfolio::new(PortfolioID::new(), UserID::new());
        let result = portfolio.record_sell(AssetID::new(), Decimal::from(5), brl(2500));
        assert!(result.is_err());
    }

    #[test]
    fn test_total_value() {
        let mut portfolio = Portfolio::new(PortfolioID::new(), UserID::new());
        let a1 = AssetID::new();
        let a2 = AssetID::new();

        portfolio
            .record_buy(a1, Decimal::from(10), brl(2500), AssetClass::Stock)
            .unwrap();
        portfolio
            .record_buy(a2, Decimal::from(5), brl(5000), AssetClass::Fund)
            .unwrap();

        let mut prices = std::collections::HashMap::new();
        prices.insert(a1, brl(3000));
        prices.insert(a2, brl(4800));

        let total = portfolio.total_value(&prices).unwrap();
        // 10*3000 + 5*4800 = 30000 + 24000 = 54000
        assert_eq!(total.amount(), 54000);
    }

    #[test]
    fn test_total_invested() {
        let mut portfolio = Portfolio::new(PortfolioID::new(), UserID::new());
        let a1 = AssetID::new();

        portfolio
            .record_buy(a1, Decimal::from(10), brl(2500), AssetClass::Stock)
            .unwrap();

        let invested = portfolio.total_invested().unwrap();
        assert_eq!(invested.amount(), 25000);
    }

    #[test]
    fn test_total_invested_empty() {
        let portfolio = Portfolio::new(PortfolioID::new(), UserID::new());
        assert!(portfolio.total_invested().is_none());
    }
}
