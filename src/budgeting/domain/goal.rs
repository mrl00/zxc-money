use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};

use crate::shared::errors::BudgetingError;
use crate::shared::ids::{AccountID, GoalID, UserID};
use crate::shared::money::Money;

/// Status of a financial goal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GoalStatus {
    /// Goal is actively being funded.
    InProgress,
    /// Goal has been reached.
    Achieved,
    /// Goal is no longer being pursued.
    Abandoned,
}

/// A savings target with a deadline and optional linked account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialGoal {
    /// Unique identifier.
    pub id: GoalID,
    /// Owner of this goal.
    pub owner_id: UserID,
    /// User-defined name (e.g. "Emergency Fund").
    pub name: String,
    /// Target amount to save.
    pub target_amount: Money,
    /// Amount saved so far.
    pub current_amount: Money,
    /// Deadline for reaching the target.
    pub target_date: NaiveDate,
    /// Optional account linked for automatic tracking.
    pub linked_account_id: Option<AccountID>,
    /// Current status of the goal.
    pub status: GoalStatus,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Last modification timestamp.
    pub updated_at: DateTime<Utc>,
}

impl FinancialGoal {
    /// Creates a new [`FinancialGoal`] in [`GoalStatus::InProgress`] with zero current amount.
    pub fn new(
        id: GoalID,
        owner_id: UserID,
        name: String,
        target_amount: Money,
        target_date: NaiveDate,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            owner_id,
            name,
            target_amount,
            current_amount: Money::zero(target_amount.currency()),
            target_date,
            linked_account_id: None,
            status: GoalStatus::InProgress,
            created_at: now,
            updated_at: now,
        }
    }

    /// Links an account to this goal for automatic tracking.
    pub fn with_linked_account(mut self, account_id: AccountID) -> Self {
        self.linked_account_id = Some(account_id);
        self
    }

    /// Adds a contribution toward the goal.
    ///
    /// Automatically marks the goal as [`GoalStatus::Achieved`] when the target is reached.
    /// Returns an error if the goal is not in progress.
    pub fn contribute(&mut self, amount: Money) -> Result<(), BudgetingError> {
        if self.status != GoalStatus::InProgress {
            return Err(BudgetingError::InvariantViolation(
                "can only contribute to in-progress goals".into(),
            ));
        }

        self.current_amount = self.current_amount.checked_add(amount)?;

        if self.current_amount >= self.target_amount {
            self.status = GoalStatus::Achieved;
        }

        self.updated_at = Utc::now();
        Ok(())
    }

    /// Abandons the goal. Only allowed if the goal is in progress.
    ///
    /// # Errors
    ///
    /// Returns [`BudgetingError::InvariantViolation`] if the goal is not in progress.
    pub fn abandon(&mut self) -> Result<(), BudgetingError> {
        if self.status != GoalStatus::InProgress {
            return Err(BudgetingError::InvariantViolation(
                "can only abandon in-progress goals".into(),
            ));
        }
        self.status = GoalStatus::Abandoned;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Returns the goal completion percentage (0.0–100.0).
    pub fn progress(&self) -> f64 {
        if self.target_amount.is_zero() {
            return 100.0;
        }
        (self.current_amount.amount().to_f64().unwrap_or(0.0)
            / self.target_amount.amount().to_f64().unwrap_or(1.0))
            * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::ids::{GoalID, UserID};
    use crate::shared::money::{Currency, Money};
    use rust_decimal::Decimal;

    fn sample_goal() -> FinancialGoal {
        FinancialGoal::new(
            GoalID::new(),
            UserID::new(),
            "Emergency Fund".into(),
            Money::from_cents(1000000, Currency::BRL),
            NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        )
    }

    #[test]
    fn test_goal_creation() {
        let g = sample_goal();
        assert_eq!(g.status, GoalStatus::InProgress);
        assert!(g.current_amount.is_zero());
        assert!(g.linked_account_id.is_none());
    }

    #[test]
    fn test_goal_contribute_partial() {
        let mut g = sample_goal();
        g.contribute(Money::from_cents(500000, Currency::BRL))
            .unwrap();
        assert_eq!(g.current_amount.amount(), Decimal::from(5000));
        assert_eq!(g.status, GoalStatus::InProgress);
        assert!((g.progress() - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_goal_contribute_exact_target() {
        let mut g = sample_goal();
        g.contribute(Money::from_cents(1000000, Currency::BRL))
            .unwrap();
        assert_eq!(g.status, GoalStatus::Achieved);
        assert!((g.progress() - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_goal_contribute_over_target() {
        let mut g = sample_goal();
        g.contribute(Money::from_cents(1200000, Currency::BRL))
            .unwrap();
        assert_eq!(g.status, GoalStatus::Achieved);
    }

    #[test]
    fn test_goal_contribute_when_achieved_fails() {
        let mut g = sample_goal();
        g.contribute(Money::from_cents(1000000, Currency::BRL))
            .unwrap();
        let result = g.contribute(Money::from_cents(100000, Currency::BRL));
        assert!(result.is_err());
    }

    #[test]
    fn test_goal_contribute_when_abandoned_fails() {
        let mut g = sample_goal();
        g.abandon().unwrap();
        let result = g.contribute(Money::from_cents(100000, Currency::BRL));
        assert!(result.is_err());
    }

    #[test]
    fn test_goal_abandon() {
        let mut g = sample_goal();
        g.abandon().unwrap();
        assert_eq!(g.status, GoalStatus::Abandoned);
    }

    #[test]
    fn test_goal_abandon_when_achieved_fails() {
        let mut g = sample_goal();
        g.contribute(Money::from_cents(1000000, Currency::BRL))
            .unwrap();
        let result = g.abandon();
        assert!(result.is_err());
    }

    #[test]
    fn test_goal_progress_zero_target() {
        let g = FinancialGoal::new(
            GoalID::new(),
            UserID::new(),
            "Zero Goal".into(),
            Money::zero(Currency::BRL),
            NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        );
        assert!((g.progress() - 100.0).abs() < f64::EPSILON);
        // Cannot contribute to zero-target goal (amount 0 is invalid for Money)
        // But progress should still be 100%
    }

    #[test]
    fn test_goal_progress_calculation() {
        let mut g = sample_goal();
        g.contribute(Money::from_cents(300000, Currency::BRL))
            .unwrap();
        assert!((g.progress() - 30.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_goal_with_linked_account() {
        let g = sample_goal().with_linked_account(AccountID::new());
        assert!(g.linked_account_id.is_some());
    }
}
