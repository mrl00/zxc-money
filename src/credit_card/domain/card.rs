use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::shared::ids::CreditCardID;
use crate::shared::money::Money;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditCard {
    pub id: CreditCardID,
    pub name: String,
    pub brand: String,
    pub limit: Money,
    pub closing_day: u32,
    pub due_day: u32,
    pub created_at: DateTime<Utc>,
}
