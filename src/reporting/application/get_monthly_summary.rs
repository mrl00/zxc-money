use std::sync::Arc;

use crate::reporting::projections::cash_flow::CashFlowStore;
use crate::shared::money::Money;
use crate::shared::period::YearMonth;

/// Query to get a monthly summary of income, expenses, and balance.
pub struct GetMonthlySummaryQuery {
    pub year: i32,
    pub month: u32,
}

/// Result of a monthly summary query.
#[derive(Debug, Clone)]
pub struct MonthlySummary {
    pub year: i32,
    pub month: u32,
    pub total_income: Money,
    pub total_expense: Money,
    pub balance: Money,
}

/// Handles [`GetMonthlySummaryQuery`] by aggregating cash flow data for a month.
pub struct GetMonthlySummaryHandler {
    cash_flow_store: Arc<CashFlowStore>,
}

impl GetMonthlySummaryHandler {
    /// Creates a new handler with the given cash flow store.
    pub fn new(cash_flow_store: Arc<CashFlowStore>) -> Self {
        Self { cash_flow_store }
    }

    /// Executes the monthly summary query.
    pub fn handle(&self, query: GetMonthlySummaryQuery) -> MonthlySummary {
        let ym = YearMonth::new(query.year, query.month);
        let period = ym.period();
        let entries = self.cash_flow_store.get_period(period);

        let mut total_income = Money::zero(crate::shared::money::Currency::BRL);
        let mut total_expense = Money::zero(crate::shared::money::Currency::BRL);

        for entry in &entries {
            total_income = total_income + entry.income;
            total_expense = total_expense + entry.expense;
        }

        let balance = total_income - total_expense;

        MonthlySummary {
            year: query.year,
            month: query.month,
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
    fn test_monthly_summary_with_data() {
        let store = Arc::new(CashFlowStore::new());
        let handler = GetMonthlySummaryHandler::new(store.clone());

        let date = chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        store.handle_event(
            &TransactionRecorded {
                transaction_id: TransactionID::new(),
                account_id: AccountID::new(),
                tx_type: TransactionType::Income,
                amount: Money::new(5000_00, Currency::BRL),
                category_id: Some(CategoryID::new()),
                description: "Salary".into(),
                date,
                timestamp: chrono::Utc::now(),
            },
            &std::sync::Mutex::new(std::collections::HashMap::new()),
        );

        store.handle_event(
            &TransactionRecorded {
                transaction_id: TransactionID::new(),
                account_id: AccountID::new(),
                tx_type: TransactionType::Expense,
                amount: Money::new(1500_00, Currency::BRL),
                category_id: Some(CategoryID::new()),
                description: "Rent".into(),
                date,
                timestamp: chrono::Utc::now(),
            },
            &std::sync::Mutex::new(std::collections::HashMap::new()),
        );

        let summary = handler.handle(GetMonthlySummaryQuery {
            year: 2026,
            month: 1,
        });
        assert_eq!(summary.total_income.amount(), 5000_00);
        assert_eq!(summary.total_expense.amount(), 1500_00);
        assert_eq!(summary.balance.amount(), 3500_00);
    }

    #[test]
    fn test_monthly_summary_empty() {
        let store = Arc::new(CashFlowStore::new());
        let handler = GetMonthlySummaryHandler::new(store);

        let summary = handler.handle(GetMonthlySummaryQuery {
            year: 2026,
            month: 3,
        });
        assert!(summary.total_income.is_zero());
        assert!(summary.total_expense.is_zero());
        assert!(summary.balance.is_zero());
    }
}
