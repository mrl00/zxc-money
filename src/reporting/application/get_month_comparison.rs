use std::sync::Arc;

use crate::reporting::projections::cash_flow::CashFlowStore;
use crate::shared::money::Money;
use crate::shared::period::YearMonth;

/// Summary of a single month's income, expenses, and balance.
#[derive(Debug, Clone)]
pub struct MonthSummary {
    pub year: i32,
    pub month: u32,
    pub total_income: Money,
    pub total_expense: Money,
    pub balance: Money,
}

/// Query to compare two months side by side.
pub struct GetMonthComparisonQuery {
    pub left: (i32, u32),
    pub right: (i32, u32),
}

/// Handles [`GetMonthComparisonQuery`] by aggregating cash flow data for both months.
pub struct GetMonthComparisonHandler {
    cash_flow_store: Arc<CashFlowStore>,
}

impl GetMonthComparisonHandler {
    /// Creates a new handler with the given cash flow store.
    pub fn new(cash_flow_store: Arc<CashFlowStore>) -> Self {
        Self { cash_flow_store }
    }

    /// Returns summaries for both months.
    pub fn handle(&self, query: GetMonthComparisonQuery) -> (MonthSummary, MonthSummary) {
        let left = self.summarize(query.left.0, query.left.1);
        let right = self.summarize(query.right.0, query.right.1);
        (left, right)
    }

    fn summarize(&self, year: i32, month: u32) -> MonthSummary {
        let ym = YearMonth::new(year, month);
        let period = ym.period();
        let entries = self.cash_flow_store.get_period(period);

        let mut total_income = Money::zero(crate::shared::money::Currency::BRL);
        let mut total_expense = Money::zero(crate::shared::money::Currency::BRL);

        for entry in &entries {
            total_income = (total_income + entry.income).unwrap();
            total_expense = (total_expense + entry.expense).unwrap();
        }

        let balance = (total_income - total_expense).unwrap();

        MonthSummary {
            year,
            month,
            total_income,
            total_expense,
            balance,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::domain::events::TransactionRecorded;
    use crate::ledger::domain::transaction::TransactionType;
    use crate::shared::ids::{AccountID, CategoryID, TransactionID};
    use crate::shared::money::Currency;

    #[test]
    fn test_comparison_two_months() {
        let store = Arc::new(CashFlowStore::new());
        let handler = GetMonthComparisonHandler::new(store.clone());

        // January: income 5000, expense 3000
        store.handle_event(
            &TransactionRecorded {
                transaction_id: TransactionID::new(),
                account_id: AccountID::new(),
                tx_type: TransactionType::Income,
                amount: Money::from_cents(5000_00, Currency::BRL),
                category_id: Some(CategoryID::new()),
                description: "Salary".into(),
                date: chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
                timestamp: chrono::Utc::now(),
            },
            &std::sync::Mutex::new(std::collections::HashMap::new()),
        );

        store.handle_event(
            &TransactionRecorded {
                transaction_id: TransactionID::new(),
                account_id: AccountID::new(),
                tx_type: TransactionType::Expense,
                amount: Money::from_cents(3000_00, Currency::BRL),
                category_id: Some(CategoryID::new()),
                description: "Rent".into(),
                date: chrono::NaiveDate::from_ymd_opt(2026, 1, 10).unwrap(),
                timestamp: chrono::Utc::now(),
            },
            &std::sync::Mutex::new(std::collections::HashMap::new()),
        );

        // February: income 6000, expense 2000
        store.handle_event(
            &TransactionRecorded {
                transaction_id: TransactionID::new(),
                account_id: AccountID::new(),
                tx_type: TransactionType::Income,
                amount: Money::from_cents(6000_00, Currency::BRL),
                category_id: Some(CategoryID::new()),
                description: "Bonus".into(),
                date: chrono::NaiveDate::from_ymd_opt(2026, 2, 10).unwrap(),
                timestamp: chrono::Utc::now(),
            },
            &std::sync::Mutex::new(std::collections::HashMap::new()),
        );

        store.handle_event(
            &TransactionRecorded {
                transaction_id: TransactionID::new(),
                account_id: AccountID::new(),
                tx_type: TransactionType::Expense,
                amount: Money::from_cents(2000_00, Currency::BRL),
                category_id: Some(CategoryID::new()),
                description: "Food".into(),
                date: chrono::NaiveDate::from_ymd_opt(2026, 2, 20).unwrap(),
                timestamp: chrono::Utc::now(),
            },
            &std::sync::Mutex::new(std::collections::HashMap::new()),
        );

        let (jan, feb) = handler.handle(GetMonthComparisonQuery {
            left: (2026, 1),
            right: (2026, 2),
        });

        assert_eq!(
            jan.total_income.amount(),
            rust_decimal::Decimal::from(5000_00) / rust_decimal::Decimal::from(100)
        );
        assert_eq!(
            jan.total_expense.amount(),
            rust_decimal::Decimal::from(3000_00) / rust_decimal::Decimal::from(100)
        );
        assert_eq!(
            jan.balance.amount(),
            rust_decimal::Decimal::from(2000_00) / rust_decimal::Decimal::from(100)
        );

        assert_eq!(
            feb.total_income.amount(),
            rust_decimal::Decimal::from(6000_00) / rust_decimal::Decimal::from(100)
        );
        assert_eq!(
            feb.total_expense.amount(),
            rust_decimal::Decimal::from(2000_00) / rust_decimal::Decimal::from(100)
        );
        assert_eq!(
            feb.balance.amount(),
            rust_decimal::Decimal::from(4000_00) / rust_decimal::Decimal::from(100)
        );
    }

    #[test]
    fn test_comparison_empty_months() {
        let store = Arc::new(CashFlowStore::new());
        let handler = GetMonthComparisonHandler::new(store);

        let (left, right) = handler.handle(GetMonthComparisonQuery {
            left: (2026, 1),
            right: (2026, 6),
        });

        assert!(left.total_income.is_zero());
        assert!(right.total_income.is_zero());
    }
}
