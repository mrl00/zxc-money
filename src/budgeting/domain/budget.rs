use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::shared::errors::BudgetingError;
use crate::shared::ids::{BudgetID, CategoryID, UserID};
use crate::shared::money::Money;
use crate::shared::period::Period;

/// A spending limit for a specific category within a time period.
///
/// # Invariants
///
/// - `planned_amount` must be positive.
/// - At most one active budget per `(category_id, period)` (enforced by repository).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Budget {
    pub id: BudgetID,
    pub owner_id: UserID,
    pub category_id: CategoryID,
    pub period: Period,
    pub planned_amount: Money,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Budget {
    /// Creates a new [`Budget`] with the given parameters.
    ///
    /// # Errors
    ///
    /// Returns [`BudgetingError::InvalidAmount`] if `planned_amount` is zero or negative.
    pub fn new(
        id: BudgetID,
        owner_id: UserID,
        category_id: CategoryID,
        period: Period,
        planned_amount: Money,
    ) -> Result<Self, BudgetingError> {
        if planned_amount.amount() <= 0 {
            return Err(BudgetingError::InvalidAmount(
                "planned amount must be positive".into(),
            ));
        }

        let now = Utc::now();
        Ok(Self {
            id,
            owner_id,
            category_id,
            period,
            planned_amount,
            created_at: now,
            updated_at: now,
        })
    }

    /// Updates the planned amount for this budget.
    ///
    /// # Errors
    ///
    /// Returns [`BudgetingError::InvalidAmount`] if `new_amount` is zero or negative.
    pub fn update_amount(&mut self, new_amount: Money) -> Result<(), BudgetingError> {
        if new_amount.amount() <= 0 {
            return Err(BudgetingError::InvalidAmount(
                "planned amount must be positive".into(),
            ));
        }
        self.planned_amount = new_amount;
        self.updated_at = Utc::now();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::ids::{BudgetID, CategoryID, UserID};
    use crate::shared::money::{Currency, Money};
    use crate::shared::period::Period;
    use chrono::NaiveDate;

    fn sample_budget() -> Budget {
        Budget::new(
            BudgetID::new(),
            UserID::new(),
            CategoryID::new(),
            Period::new(
                NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            ),
            Money::new(500_00, Currency::BRL),
        )
        .unwrap()
    }

    #[test]
    fn test_budget_creation() {
        let b = sample_budget();
        assert_eq!(b.planned_amount.amount(), 500_00);
        assert!(b.updated_at >= b.created_at);
    }

    #[test]
    fn test_budget_zero_amount_fails() {
        let result = Budget::new(
            BudgetID::new(),
            UserID::new(),
            CategoryID::new(),
            Period::new(
                NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            ),
            Money::new(0, Currency::BRL),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_budget_negative_amount_fails() {
        let result = Budget::new(
            BudgetID::new(),
            UserID::new(),
            CategoryID::new(),
            Period::new(
                NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            ),
            Money::new(-100, Currency::BRL),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_budget_update_amount() {
        let mut b = sample_budget();
        b.update_amount(Money::new(800_00, Currency::BRL)).unwrap();
        assert_eq!(b.planned_amount.amount(), 800_00);
    }

    #[test]
    fn test_budget_update_zero_fails() {
        let mut b = sample_budget();
        let result = b.update_amount(Money::new(0, Currency::BRL));
        assert!(result.is_err());
    }
}
