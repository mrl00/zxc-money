use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::investment::domain::asset::AssetClass;
use crate::investment::domain::position::Position;
use crate::shared::ids::{AssetID, PortfolioID, UserID};
use crate::shared::money::Money;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Portfolio {
    pub id: PortfolioID,
    pub owner_id: UserID,
    pub positions: Vec<Position>,
    pub created_at: DateTime<Utc>,
}

impl Portfolio {
    pub fn new(id: PortfolioID, owner_id: UserID) -> Self {
        Self {
            id,
            owner_id,
            positions: Vec::new(),
            created_at: Utc::now(),
        }
    }

    pub fn record_buy(
        &mut self,
        asset_id: AssetID,
        quantity: Decimal,
        price: Money,
        asset_class: AssetClass,
    ) -> Result<(), crate::shared::errors::InvestmentError> {
        let total_cost = price * quantity;

        if let Some(position) = self.positions.iter_mut().find(|p| p.asset_id == asset_id) {
            let old_total = position.average_cost * position.quantity;
            let new_total = old_total + total_cost;
            let new_quantity = position.quantity + quantity;

            let new_avg_cost = (new_total.amount() as f64
                / new_quantity.to_string().parse::<f64>().unwrap())
                as i64;
            position.average_cost = Money::new(new_avg_cost, price.currency());
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

    pub fn record_sell(
        &mut self,
        asset_id: AssetID,
        quantity: Decimal,
        price: Money,
    ) -> Result<Money, crate::shared::errors::InvestmentError> {
        let position = self
            .positions
            .iter_mut()
            .find(|p| p.asset_id == asset_id)
            .ok_or_else(|| {
                crate::shared::errors::InvestmentError::AssetNotFound(asset_id.to_string())
            })?;

        if position.quantity < quantity {
            return Err(
                crate::shared::errors::InvestmentError::InsufficientQuantity {
                    available: position.quantity.to_string(),
                    requested: quantity.to_string(),
                },
            );
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
}
