use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
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
    /// Unique identifier.
    pub id: BudgetID,
    /// Owner of this budget.
    pub owner_id: UserID,
    /// Category this budget tracks.
    pub category_id: CategoryID,
    /// Time period this budget covers.
    pub period: Period,
    /// Maximum planned spending for this period.
    pub planned_amount: Money,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Last modification timestamp.
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
        if planned_amount.amount() <= Decimal::ZERO {
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
        if new_amount.amount() <= Decimal::ZERO {
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
    use rust_decimal::Decimal;

    fn sample_budget() -> Budget {
        Budget::new(
            BudgetID::new(),
            UserID::new(),
            CategoryID::new(),
            Period::new(
                NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            ),
            Money::from_cents(50000, Currency::BRL),
        )
        .unwrap()
    }

    #[test]
    fn test_budget_creation() {
        let b = sample_budget();
        assert_eq!(b.planned_amount.amount(), Decimal::from(500));
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
            Money::zero(Currency::BRL),
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
            Money::new(Decimal::from(-100), Currency::BRL),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_budget_update_amount() {
        let mut b = sample_budget();
        b.update_amount(Money::from_cents(80000, Currency::BRL))
            .unwrap();
        assert_eq!(b.planned_amount.amount(), Decimal::from(800));
    }

    #[test]
    fn test_budget_update_zero_fails() {
        let mut b = sample_budget();
        let result = b.update_amount(Money::zero(Currency::BRL));
        assert!(result.is_err());
    }
}
