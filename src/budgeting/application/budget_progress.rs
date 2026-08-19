use crate::shared::ids::{BudgetID, CategoryID};
use crate::shared::money::Money;
use crate::shared::period::Period;

/// Read model showing planned vs. spent amounts for a budget.
#[derive(Debug, Clone)]
pub struct BudgetProgress {
    pub budget_id: BudgetID,
    pub category_id: CategoryID,
    pub period: Period,
    pub planned: Money,
    pub spent: Money,
}

impl BudgetProgress {
    /// Creates a new [`BudgetProgress`] from a budget's planned amount and the computed spent amount.
    pub fn new(
        budget_id: BudgetID,
        category_id: CategoryID,
        period: Period,
        planned: Money,
        spent: Money,
    ) -> Self {
        Self {
            budget_id,
            category_id,
            period,
            planned,
            spent,
        }
    }

    /// Returns `true` if spending has exceeded the planned amount.
    pub fn is_over(&self) -> bool {
        self.spent.amount() > self.planned.amount()
    }

    /// Returns the remaining amount before the budget is exceeded.
    ///
    /// May be negative if the budget is already over.
    pub fn remaining(&self) -> Money {
        self.planned
            .checked_sub(self.spent)
            .unwrap_or_else(|_| Money::zero(self.planned.currency()))
    }

    /// Returns the percentage of the budget that has been used (0.0–100.0+).
    pub fn pct_used(&self) -> f64 {
        if self.planned.is_zero() {
            return if self.spent.is_zero() { 0.0 } else { 100.0 };
        }
        (self.spent.amount() as f64 / self.planned.amount() as f64) * 100.0
    }
}

/// Utility to compute [`BudgetProgress`] from a planned amount and spent amount.
pub struct BudgetProgressCalculator;

impl BudgetProgressCalculator {
    /// Computes [`BudgetProgress`] for a given budget.
    pub fn compute(
        budget_id: BudgetID,
        category_id: CategoryID,
        period: Period,
        planned: Money,
        spent: Money,
    ) -> BudgetProgress {
        BudgetProgress::new(budget_id, category_id, period, planned, spent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::ids::{BudgetID, CategoryID};
    use crate::shared::money::{Currency, Money};
    use crate::shared::period::Period;
    use chrono::NaiveDate;

    fn sample_period() -> Period {
        Period::new(
            NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
        )
    }

    #[test]
    fn test_budget_progress_under() {
        let bp = BudgetProgress::new(
            BudgetID::new(),
            CategoryID::new(),
            sample_period(),
            Money::new(1000_00, Currency::BRL),
            Money::new(500_00, Currency::BRL),
        );
        assert!(!bp.is_over());
        assert_eq!(bp.remaining().amount(), 500_00);
        assert!((bp.pct_used() - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_budget_progress_exact() {
        let bp = BudgetProgress::new(
            BudgetID::new(),
            CategoryID::new(),
            sample_period(),
            Money::new(1000_00, Currency::BRL),
            Money::new(1000_00, Currency::BRL),
        );
        assert!(!bp.is_over());
        assert_eq!(bp.remaining().amount(), 0);
        assert!((bp.pct_used() - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_budget_progress_over() {
        let bp = BudgetProgress::new(
            BudgetID::new(),
            CategoryID::new(),
            sample_period(),
            Money::new(1000_00, Currency::BRL),
            Money::new(1200_00, Currency::BRL),
        );
        assert!(bp.is_over());
        assert!((bp.pct_used() - 120.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_budget_progress_zero_planned() {
        let bp = BudgetProgress::new(
            BudgetID::new(),
            CategoryID::new(),
            sample_period(),
            Money::new(0, Currency::BRL),
            Money::new(0, Currency::BRL),
        );
        assert!(!bp.is_over());
        assert!((bp.pct_used()).abs() < f64::EPSILON);
    }
}
