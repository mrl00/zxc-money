use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::shared::ids::AssetID;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetClass {
    Stock,
    Fund,
    FixedIncome,
    Crypto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub id: AssetID,
    pub ticker: String,
    pub name: String,
    pub class: AssetClass,
    pub currency: crate::shared::money::Currency,
    pub created_at: DateTime<Utc>,
}
