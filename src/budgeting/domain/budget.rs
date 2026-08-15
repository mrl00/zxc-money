use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::shared::ids::BudgetID;
use crate::shared::ids::CategoryID;
use crate::shared::money::Money;
use crate::shared::period::Period;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Budget {
    pub id: BudgetID,
    pub category_id: CategoryID,
    pub period: Period,
    pub planned_amount: Money,
    pub created_at: DateTime<Utc>,
}

impl Budget {
    pub fn new(
        id: BudgetID,
        category_id: CategoryID,
        period: Period,
        planned_amount: Money,
    ) -> Self {
        Self {
            id,
            category_id,
            period,
            planned_amount,
            created_at: Utc::now(),
        }
    }
}
