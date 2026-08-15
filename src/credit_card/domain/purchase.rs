use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use crate::shared::ids::PurchaseID;
use crate::shared::money::Money;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Purchase {
    pub id: PurchaseID,
    pub description: String,
    pub total_amount: Money,
    pub installments_count: u32,
    pub purchased_at: NaiveDate,
    pub created_at: DateTime<Utc>,
}
