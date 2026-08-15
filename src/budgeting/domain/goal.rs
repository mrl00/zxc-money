use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use crate::shared::ids::AccountID;
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
    pub target_date: NaiveDate,
    pub linked_account_id: Option<AccountID>,
    pub status: GoalStatus,
    pub created_at: DateTime<Utc>,
}

impl FinancialGoal {
    pub fn new(id: GoalID, name: String, target_amount: Money, target_date: NaiveDate) -> Self {
        Self {
            id,
            name,
            target_amount,
            current_amount: Money::zero(target_amount.currency()),
            target_date,
            linked_account_id: None,
            status: GoalStatus::InProgress,
            created_at: Utc::now(),
        }
    }

    pub fn with_linked_account(mut self, account_id: AccountID) -> Self {
        self.linked_account_id = Some(account_id);
        self
    }

    pub fn contribute(
        &mut self,
        amount: Money,
    ) -> Result<(), crate::shared::errors::BudgetingError> {
        if self.status != GoalStatus::InProgress {
            return Err(crate::shared::errors::BudgetingError::InvariantViolation(
                "can only contribute to in-progress goals".into(),
            ));
        }

        self.current_amount = self.current_amount.checked_add(amount)?;

        if self.current_amount >= self.target_amount {
            self.status = GoalStatus::Achieved;
        }

        Ok(())
    }

    pub fn progress(&self) -> f64 {
        if self.target_amount.is_zero() {
            return 100.0;
        }
        (self.current_amount.amount() as f64 / self.target_amount.amount() as f64) * 100.0
    }
}
