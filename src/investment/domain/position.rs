use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::investment::domain::asset::AssetClass;
use crate::shared::ids::AssetID;
use crate::shared::money::Money;

/// A holding of a specific [`Asset`](super::asset::Asset) within a portfolio.
///
/// Each position tracks the quantity held, the weighted-average cost per unit,
/// and the asset class. Positions are created and updated by
/// [`Portfolio::record_buy`](super::portfolio::Portfolio::record_buy) and
/// removed when the full quantity is sold.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    /// The asset this position refers to.
    pub asset_id: AssetID,
    /// Quantity held (in units of the asset, e.g. shares, coins).
    pub quantity: Decimal,
    /// Weighted-average cost per unit at time of purchase.
    pub average_cost: Money,
    /// Asset classification (stock, fund, fixed income, crypto).
    pub asset_class: AssetClass,
}
