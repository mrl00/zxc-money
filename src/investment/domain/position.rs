use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::investment::domain::asset::AssetClass;
use crate::shared::ids::AssetID;
use crate::shared::money::Money;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub asset_id: AssetID,
    pub quantity: Decimal,
    pub average_cost: Money,
    pub asset_class: AssetClass,
}
