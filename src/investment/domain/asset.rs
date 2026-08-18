use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::shared::ids::AssetID;

/// Classification of a tradeable asset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetClass {
    /// Equity share in a company.
    Stock,
    /// Mutual fund or ETF.
    Fund,
    /// Bond or other fixed-income instrument.
    FixedIncome,
    /// Cryptocurrency.
    Crypto,
}

/// A tradeable financial instrument (e.g. stock, fund, bond, crypto).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub id: AssetID,
    pub ticker: String,
    pub name: String,
    pub class: AssetClass,
    pub currency: crate::shared::money::Currency,
    pub created_at: DateTime<Utc>,
}
