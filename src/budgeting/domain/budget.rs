use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::shared::ids::{BudgetID, CategoryID, UserID};
use crate::shared::money::Money;
use crate::shared::period::Period;

/// A spending limit for a specific category within a time period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Budget {
    pub id: BudgetID,
    pub owner_id: UserID,
    pub category_id: CategoryID,
    pub period: Period,
    pub planned_amount: Money,
    pub created_at: DateTime<Utc>,
}

impl Budget {
    /// Creates a new [`Budget`] with the given parameters.
    pub fn new(
        id: BudgetID,
        owner_id: UserID,
        category_id: CategoryID,
        period: Period,
        planned_amount: Money,
    ) -> Self {
        Self {
            id,
            owner_id,
            category_id,
            period,
            planned_amount,
            created_at: Utc::now(),
        }
    }
}
