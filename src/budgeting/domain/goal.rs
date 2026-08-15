use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::shared::ids::GoalID;
use crate::shared::money::Money;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GoalStatus {
    InProgress,
    Achieved,
    Abandoned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialGoal {
    pub id: GoalID,
    pub name: String,
    pub target_amount: Money,
    pub current_amount: Money,
    pub target_date: chrono::NaiveDate,
    pub status: GoalStatus,
    pub created_at: DateTime<Utc>,
}
