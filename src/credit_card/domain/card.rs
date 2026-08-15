use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::shared::ids::{CreditCardID, UserID};
use crate::shared::money::Money;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditCard {
    pub id: CreditCardID,
    pub owner_id: UserID,
    pub name: String,
    pub brand: String,
    pub limit: Money,
    pub closing_day: u32,
    pub due_day: u32,
    pub created_at: DateTime<Utc>,
}

impl CreditCard {
    pub fn new(
        id: CreditCardID,
        owner_id: UserID,
        name: String,
        brand: String,
        limit: Money,
        closing_day: u32,
        due_day: u32,
    ) -> Self {
        Self {
            id,
            owner_id,
            name,
            brand,
            limit,
            closing_day,
            due_day,
            created_at: Utc::now(),
        }
    }

    pub fn available_limit(
        &self,
        used: Money,
    ) -> Result<Money, crate::shared::errors::CreditCardError> {
        self.limit.checked_sub(used).map_err(|_| {
            crate::shared::errors::CreditCardError::InvariantViolation(
                "used amount exceeds limit".into(),
            )
        })
    }
}
