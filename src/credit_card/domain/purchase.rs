use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use crate::shared::ids::{CategoryID, PurchaseID};
use crate::shared::money::Money;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Purchase {
    pub id: PurchaseID,
    pub description: String,
    pub total_amount: Money,
    pub installments_count: u32,
    pub category_id: CategoryID,
    pub purchased_at: NaiveDate,
    pub created_at: DateTime<Utc>,
}

impl Purchase {
    pub fn new(
        id: PurchaseID,
        description: String,
        total_amount: Money,
        installments_count: u32,
        category_id: CategoryID,
        purchased_at: NaiveDate,
    ) -> Self {
        Self {
            id,
            description,
            total_amount,
            installments_count,
            category_id,
            purchased_at,
            created_at: Utc::now(),
        }
    }

    pub fn installment_amount(&self) -> Money {
        let per_installment = self.total_amount.amount() / self.installments_count as i64;
        Money::new(per_installment, self.total_amount.currency())
    }
}
