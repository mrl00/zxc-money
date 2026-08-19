use std::sync::Arc;

use crate::reporting::projections::cash_flow::CashFlowStore;
use crate::shared::money::Money;
use crate::shared::period::YearMonth;

/// Summary of a single year's income, expenses, and balance.
#[derive(Debug, Clone)]
pub struct YearSummary {
    pub year: i32,
    pub total_income: Money,
    pub total_expense: Money,
    pub balance: Money,
}

/// Query to compare multiple years.
pub struct GetYearComparisonQuery {
    pub years: Vec<i32>,
}

/// Handles [`GetYearComparisonQuery`] by aggregating cash flow data for each year.
pub struct GetYearComparisonHandler {
    cash_flow_store: Arc<CashFlowStore>,
}

impl GetYearComparisonHandler {
    /// Creates a new handler with the given cash flow store.
    pub fn new(cash_flow_store: Arc<CashFlowStore>) -> Self {
        Self { cash_flow_store }
    }

    /// Returns summaries for each requested year.
    pub fn handle(&self, query: GetYearComparisonQuery) -> Vec<YearSummary> {
        query
            .years
            .iter()
            .map(|&year| self.summarize_year(year))
            .collect()
    }

    fn summarize_year(&self, year: i32) -> YearSummary {
        let start = YearMonth::new(year, 1).first_day();
        let end = YearMonth::new(year, 12).last_day();
        let period = crate::shared::period::Period::new(start, end);
        let entries = self.cash_flow_store.get_period(period);

        let mut total_income = Money::zero(crate::shared::money::Currency::BRL);
        let mut total_expense = Money::zero(crate::shared::money::Currency::BRL);

        for entry in &entries {
            total_income = total_income + entry.income;
            total_expense = total_expense + entry.expense;
        }

        let balance = total_income - total_expense;

        YearSummary {
            year,
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
    fn test_year_comparison() {
        let store = Arc::new(CashFlowStore::new());
        let handler = GetYearComparisonHandler::new(store.clone());

        // 2025 income
        store.handle_event(
            &TransactionRecorded {
                transaction_id: TransactionID::new(),
                account_id: AccountID::new(),
                tx_type: TransactionType::Income,
                amount: Money::new(50000_00, Currency::BRL),
                category_id: Some(CategoryID::new()),
                description: "2025 Salary".into(),
                date: chrono::NaiveDate::from_ymd_opt(2025, 6, 15).unwrap(),
                timestamp: chrono::Utc::now(),
            },
            &std::sync::Mutex::new(std::collections::HashMap::new()),
        );

        // 2026 income
        store.handle_event(
            &TransactionRecorded {
                transaction_id: TransactionID::new(),
                account_id: AccountID::new(),
                tx_type: TransactionType::Income,
                amount: Money::new(60000_00, Currency::BRL),
                category_id: Some(CategoryID::new()),
                description: "2026 Salary".into(),
                date: chrono::NaiveDate::from_ymd_opt(2026, 6, 15).unwrap(),
                timestamp: chrono::Utc::now(),
            },
            &std::sync::Mutex::new(std::collections::HashMap::new()),
        );

        // 2026 expense
        store.handle_event(
            &TransactionRecorded {
                transaction_id: TransactionID::new(),
                account_id: AccountID::new(),
                tx_type: TransactionType::Expense,
                amount: Money::new(40000_00, Currency::BRL),
                category_id: Some(CategoryID::new()),
                description: "2026 Rent".into(),
                date: chrono::NaiveDate::from_ymd_opt(2026, 3, 10).unwrap(),
                timestamp: chrono::Utc::now(),
            },
            &std::sync::Mutex::new(std::collections::HashMap::new()),
        );

        let summaries = handler.handle(GetYearComparisonQuery {
            years: vec![2025, 2026],
        });

        assert_eq!(summaries.len(), 2);
        assert_eq!(summaries[0].year, 2025);
        assert_eq!(summaries[0].total_income.amount(), 50000_00);
        assert!(summaries[0].total_expense.is_zero());

        assert_eq!(summaries[1].year, 2026);
        assert_eq!(summaries[1].total_income.amount(), 60000_00);
        assert_eq!(summaries[1].total_expense.amount(), 40000_00);
        assert_eq!(summaries[1].balance.amount(), 20000_00);
    }

    #[test]
    fn test_year_comparison_empty() {
        let store = Arc::new(CashFlowStore::new());
        let handler = GetYearComparisonHandler::new(store);

        let summaries = handler.handle(GetYearComparisonQuery {
            years: vec![2024, 2025],
        });

        assert_eq!(summaries.len(), 2);
        for s in &summaries {
            assert!(s.total_income.is_zero());
            assert!(s.total_expense.is_zero());
        }
    }
}
