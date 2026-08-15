use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::shared::ids::BudgetID;
use crate::shared::money::Money;
use crate::shared::period::Period;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Budget {
    pub id: BudgetID,
    pub category_id: crate::shared::ids::CategoryID,
    pub period: Period,
    pub planned_amount: Money,
    pub created_at: DateTime<Utc>,
}
