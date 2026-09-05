use std::sync::Arc;

use crate::ledger::domain::events::TransactionRecorded;
use crate::ledger::domain::transaction::TransactionType;
use crate::shared::events::DomainEvent;
use crate::shared::ids::{Principal, TransactionID};
use crate::shared::money::Money;
use crate::shared::period::Period;

/// A transaction with its display data for top expenses.
#[derive(Debug, Clone)]
pub struct TopExpenseEntry {
    pub transaction_id: TransactionID,
    pub amount: Money,
    pub description: String,
    pub date: chrono::NaiveDate,
}

/// Query to get the top N expenses within a date range.
pub struct GetTopExpensesQuery {
    pub principal: Principal,
    pub from: chrono::NaiveDate,
    pub to: chrono::NaiveDate,
    pub limit: usize,
}

/// Handles [`GetTopExpensesQuery`] by filtering and sorting expense transactions.
pub struct GetTopExpensesHandler {
    events: Arc<Vec<Box<dyn DomainEvent + Send + Sync>>>,
}

impl GetTopExpensesHandler {
    /// Creates a new handler with a set of pre-collected events.
    pub fn new(events: Arc<Vec<Box<dyn DomainEvent + Send + Sync>>>) -> Self {
        Self { events }
    }

    /// Executes the top expenses query.
    pub fn handle(&self, query: GetTopExpensesQuery) -> Vec<TopExpenseEntry> {
        let period = Period::new(query.from, query.to);

        let mut expenses: Vec<TopExpenseEntry> = self
            .events
            .iter()
            .filter_map(|event| {
                let e = event.as_any().downcast_ref::<TransactionRecorded>()?;
                if e.tx_type != TransactionType::Expense {
                    return None;
                }
                if !period.contains(e.date) {
                    return None;
                }
                Some(TopExpenseEntry {
                    transaction_id: e.transaction_id,
                    amount: e.amount,
                    description: e.description.clone(),
                    date: e.date,
                })
            })
            .collect();

        expenses.sort_by_key(|b| std::cmp::Reverse(b.amount.amount()));
        expenses.truncate(query.limit);
        expenses
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::ids::{AccountID, CategoryID, UserID};
    use crate::shared::money::Currency;

    fn expense_event(amount: i64, date: chrono::NaiveDate, desc: &str) -> Box<TransactionRecorded> {
        Box::new(TransactionRecorded {
            transaction_id: TransactionID::new(),
            account_id: AccountID::new(),
            owner_id: crate::shared::ids::UserID::new(),
            tx_type: TransactionType::Expense,
            amount: Money::from_cents(amount, Currency::BRL),
            category_id: Some(CategoryID::new()),
            description: desc.into(),
            date,
            timestamp: chrono::Utc::now(),
        })
    }

    #[test]
    fn test_top_expenses_sorted() {
        let events: Vec<Box<dyn DomainEvent + Send + Sync>> = vec![
            expense_event(
                10000,
                chrono::NaiveDate::from_ymd_opt(2026, 1, 5).unwrap(),
                "Cheap",
            ),
            expense_event(
                50000,
                chrono::NaiveDate::from_ymd_opt(2026, 1, 10).unwrap(),
                "Medium",
            ),
            expense_event(
                100000,
                chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
                "Expensive",
            ),
        ];

        let handler = GetTopExpensesHandler::new(Arc::new(events));
        let top = handler.handle(GetTopExpensesQuery {
            principal: Principal::new(UserID::new()),
            from: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            to: chrono::NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            limit: 2,
        });

        assert_eq!(top.len(), 2);
        assert_eq!(top[0].description, "Expensive");
        assert_eq!(top[1].description, "Medium");
    }

    #[test]
    fn test_top_expenses_limit() {
        let events: Vec<Box<dyn DomainEvent + Send + Sync>> = vec![
            expense_event(
                10000,
                chrono::NaiveDate::from_ymd_opt(2026, 1, 5).unwrap(),
                "A",
            ),
            expense_event(
                20000,
                chrono::NaiveDate::from_ymd_opt(2026, 1, 6).unwrap(),
                "B",
            ),
            expense_event(
                30000,
                chrono::NaiveDate::from_ymd_opt(2026, 1, 7).unwrap(),
                "C",
            ),
        ];

        let handler = GetTopExpensesHandler::new(Arc::new(events));
        let top = handler.handle(GetTopExpensesQuery {
            principal: Principal::new(UserID::new()),
            from: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            to: chrono::NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            limit: 1,
        });

        assert_eq!(top.len(), 1);
        assert_eq!(top[0].description, "C");
    }
}
