use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

use crate::shared::ids::{AssetID, PortfolioID, UserID};
use crate::shared::money::Money;

/// Event emitted when an asset is purchased within a portfolio.
#[derive(Debug)]
pub struct AssetBought {
    pub portfolio_id: PortfolioID,
    pub owner_id: UserID,
    pub asset_id: AssetID,
    pub quantity: Decimal,
    pub price: Money,
    pub timestamp: DateTime<Utc>,
}

impl crate::shared::events::DomainEvent for AssetBought {
    fn event_type(&self) -> &'static str {
        "AssetBought"
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Event emitted when an asset is sold from a portfolio.
#[derive(Debug)]
pub struct AssetSold {
    pub portfolio_id: PortfolioID,
    pub owner_id: UserID,
    pub asset_id: AssetID,
    pub quantity: Decimal,
    pub price: Money,
    pub timestamp: DateTime<Utc>,
}

impl crate::shared::events::DomainEvent for AssetSold {
    fn event_type(&self) -> &'static str {
        "AssetSold"
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
