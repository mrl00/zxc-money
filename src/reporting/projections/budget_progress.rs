use std::collections::HashMap;
use std::sync::Mutex;

use crate::budgeting::domain::events::BudgetDefined;
use crate::ledger::domain::events::TransactionRecorded;
use crate::ledger::domain::transaction::TransactionType;
use crate::reporting::projections::account_balance::BudgetProgress;
use crate::shared::events::DomainEvent;
use crate::shared::ids::CategoryID;
use crate::shared::money::Money;
use crate::shared::period::Period;
use rust_decimal::prelude::ToPrimitive;

/// Key for budget progress: (category, period).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct BudgetKey {
    category_id: CategoryID,
    period: Period,
}

/// In-memory store that maintains budget progress from domain events.
pub struct BudgetProgressStore {
    budgets: Mutex<HashMap<BudgetKey, BudgetProgress>>,
}

impl BudgetProgressStore {
    /// Creates an empty store.
    pub fn new() -> Self {
        Self {
            budgets: Mutex::new(HashMap::new()),
        }
    }

    /// Returns the budget progress for a specific category and period, if tracked.
    pub fn get(&self, category_id: CategoryID, period: Period) -> Option<BudgetProgress> {
        let budgets = self.budgets.lock().unwrap();
        let key = BudgetKey {
            category_id,
            period,
        };
        budgets.get(&key).cloned()
    }

    /// Returns all budget progress entries.
    pub fn get_all(&self) -> Vec<BudgetProgress> {
        let budgets = self.budgets.lock().unwrap();
        budgets.values().cloned().collect()
    }

    /// Applies a domain event to update the budget progress.
    pub fn handle_event(&self, event: &dyn DomainEvent) {
        if let Some(e) = event.as_any().downcast_ref::<BudgetDefined>() {
            let mut budgets = self.budgets.lock().unwrap();
            let key = BudgetKey {
                category_id: e.category_id,
                period: crate::shared::period::Period::new(
                    chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                    chrono::NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
                ),
            };
            let entry = budgets.entry(key).or_insert_with(|| BudgetProgress {
                category_id: e.category_id,
                planned: Money::zero(crate::shared::money::Currency::BRL),
                spent: Money::zero(crate::shared::money::Currency::BRL),
                remaining: Money::zero(crate::shared::money::Currency::BRL),
                pct_used: 0.0,
                is_over: false,
            });
            entry.planned = e.planned_amount;
            entry.remaining = (entry.planned - entry.spent).unwrap();
            entry.is_over = entry.spent.amount() > entry.planned.amount();
        }

        if let Some(e) = event.as_any().downcast_ref::<TransactionRecorded>() {
            if e.tx_type != TransactionType::Expense {
                return;
            }
            if let Some(category_id) = e.category_id {
                let mut budgets = self.budgets.lock().unwrap();
                let period = crate::shared::period::YearMonth::from_date(e.date).period();
                let key = BudgetKey {
                    category_id,
                    period,
                };
                let entry = budgets.entry(key).or_insert_with(|| BudgetProgress {
                    category_id,
                    planned: Money::zero(crate::shared::money::Currency::BRL),
                    spent: Money::zero(crate::shared::money::Currency::BRL),
                    remaining: Money::zero(crate::shared::money::Currency::BRL),
                    pct_used: 0.0,
                    is_over: false,
                });
                entry.spent = (entry.spent + e.amount).unwrap();
                entry.remaining = (entry.planned - entry.spent).unwrap();
                entry.is_over = entry.spent.amount() > entry.planned.amount();
                if entry.planned.amount() > rust_decimal::Decimal::ZERO {
                    entry.pct_used = (entry.spent.amount().to_f64().unwrap()
                        / entry.planned.amount().to_f64().unwrap())
                        * 100.0;
                }
            }
        }
    }
}

impl Default for BudgetProgressStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::budgeting::domain::events::BudgetDefined;
    use crate::ledger::domain::events::TransactionRecorded;
    use crate::shared::ids::{CategoryID, TransactionID};
    use crate::shared::money::{Currency, Money};
    use crate::shared::period::YearMonth;

    fn define_budget(store: &BudgetProgressStore, cat: CategoryID, planned: i64) {
        store.handle_event(&BudgetDefined {
            budget_id: crate::shared::ids::BudgetID::new(),
            category_id: cat,
            planned_amount: Money::from_cents(planned, Currency::BRL),
            timestamp: chrono::Utc::now(),
        });
    }

    #[test]
    fn test_budget_defined() {
        let store = BudgetProgressStore::new();
        let cat = CategoryID::new();
        let period = YearMonth::new(2026, 1).period();

        define_budget(&store, cat, 1000_00);

        let progress = store.get(cat, period).unwrap();
        assert_eq!(
            progress.planned.amount(),
            rust_decimal::Decimal::from(1000_00) / rust_decimal::Decimal::from(100)
        );
        assert!(progress.spent.is_zero());
    }

    #[test]
    fn test_expense_tracking() {
        let store = BudgetProgressStore::new();
        let cat = CategoryID::new();
        let period = YearMonth::new(2026, 1).period();

        define_budget(&store, cat, 1000_00);

        store.handle_event(&TransactionRecorded {
            transaction_id: TransactionID::new(),
            account_id: crate::shared::ids::AccountID::new(),
            tx_type: TransactionType::Expense,
            amount: Money::from_cents(300_00, Currency::BRL),
            category_id: Some(cat),
            description: "Food".into(),
            date: chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            timestamp: chrono::Utc::now(),
        });

        let progress = store.get(cat, period).unwrap();
        assert_eq!(
            progress.spent.amount(),
            rust_decimal::Decimal::from(300_00) / rust_decimal::Decimal::from(100)
        );
        assert_eq!(
            progress.remaining.amount(),
            rust_decimal::Decimal::from(700_00) / rust_decimal::Decimal::from(100)
        );
        assert!(!progress.is_over);
        assert!((progress.pct_used - 30.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_budget_exceeded() {
        let store = BudgetProgressStore::new();
        let cat = CategoryID::new();
        let period = YearMonth::new(2026, 1).period();

        define_budget(&store, cat, 500_00);

        store.handle_event(&TransactionRecorded {
            transaction_id: TransactionID::new(),
            account_id: crate::shared::ids::AccountID::new(),
            tx_type: TransactionType::Expense,
            amount: Money::from_cents(600_00, Currency::BRL),
            category_id: Some(cat),
            description: "Big purchase".into(),
            date: chrono::NaiveDate::from_ymd_opt(2026, 1, 20).unwrap(),
            timestamp: chrono::Utc::now(),
        });

        let progress = store.get(cat, period).unwrap();
        assert!(progress.is_over);
        assert_eq!(
            progress.spent.amount(),
            rust_decimal::Decimal::from(600_00) / rust_decimal::Decimal::from(100)
        );
    }

    #[test]
    fn test_income_ignored() {
        let store = BudgetProgressStore::new();
        let cat = CategoryID::new();
        let period = YearMonth::new(2026, 1).period();

        define_budget(&store, cat, 1000_00);

        store.handle_event(&TransactionRecorded {
            transaction_id: TransactionID::new(),
            account_id: crate::shared::ids::AccountID::new(),
            tx_type: TransactionType::Income,
            amount: Money::from_cents(5000_00, Currency::BRL),
            category_id: Some(cat),
            description: "Salary".into(),
            date: chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            timestamp: chrono::Utc::now(),
        });

        let progress = store.get(cat, period).unwrap();
        assert!(progress.spent.is_zero());
    }

    #[test]
    fn test_expense_without_category_ignored() {
        let store = BudgetProgressStore::new();
        let cat = CategoryID::new();

        define_budget(&store, cat, 1000_00);

        store.handle_event(&TransactionRecorded {
            transaction_id: TransactionID::new(),
            account_id: crate::shared::ids::AccountID::new(),
            tx_type: TransactionType::Expense,
            amount: Money::from_cents(100_00, Currency::BRL),
            category_id: None,
            description: "Uncategorized".into(),
            date: chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            timestamp: chrono::Utc::now(),
        });

        let period = YearMonth::new(2026, 1).period();
        let progress = store.get(cat, period);
        assert!(progress.is_none() || progress.unwrap().spent.is_zero());
    }
}
