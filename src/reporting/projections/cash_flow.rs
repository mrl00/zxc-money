use std::collections::HashMap;
use std::sync::Mutex;

use crate::ledger::domain::events::{TransactionRecorded, TransferCompleted};
use crate::ledger::domain::transaction::TransactionType;
use crate::reporting::projections::account_balance::CashFlowEntry;
use crate::shared::events::DomainEvent;
use crate::shared::ids::AccountID;
use crate::shared::money::Money;
use crate::shared::period::Period;

/// In-memory store that maintains daily cash flow aggregations from domain events.
pub struct CashFlowStore {
    entries: Mutex<HashMap<chrono::NaiveDate, CashFlowEntry>>,
}

impl CashFlowStore {
    /// Creates an empty store.
    pub fn new() -> Self {
        Self {
            entries: Mutex::new(HashMap::new()),
        }
    }

    /// Returns the cash flow entry for a specific date, if it exists.
    pub fn get(&self, date: chrono::NaiveDate) -> Option<CashFlowEntry> {
        let entries = self.entries.lock().unwrap();
        entries.get(&date).cloned()
    }

    /// Returns all cash flow entries within a period (inclusive).
    pub fn get_period(&self, period: Period) -> Vec<CashFlowEntry> {
        let entries = self.entries.lock().unwrap();
        entries
            .values()
            .filter(|e| period.contains(e.date))
            .cloned()
            .collect()
    }

    /// Returns all cash flow entries.
    pub fn get_all(&self) -> Vec<CashFlowEntry> {
        let entries = self.entries.lock().unwrap();
        entries.values().cloned().collect()
    }

    /// Applies a domain event to update the cash flow projection.
    pub fn handle_event(
        &self,
        event: &dyn DomainEvent,
        accounts: &Mutex<HashMap<AccountID, crate::shared::money::Currency>>,
    ) {
        if let Some(e) = event.as_any().downcast_ref::<TransactionRecorded>() {
            // Only track income and expense (transfers are handled by TransferCompleted)
            match e.tx_type {
                TransactionType::Income | TransactionType::Expense => {
                    let mut entries = self.entries.lock().unwrap();
                    let entry = entries.entry(e.date).or_insert_with(|| CashFlowEntry {
                        date: e.date,
                        income: Money::zero(crate::shared::money::Currency::BRL),
                        expense: Money::zero(crate::shared::money::Currency::BRL),
                        net: Money::zero(crate::shared::money::Currency::BRL),
                    });

                    match e.tx_type {
                        TransactionType::Income => {
                            entry.income = (entry.income + e.amount).unwrap();
                        }
                        TransactionType::Expense => {
                            entry.expense = (entry.expense + e.amount).unwrap();
                        }
                        _ => {}
                    }
                    entry.net = (entry.income - entry.expense).unwrap();
                }
                _ => {}
            }
        }

        if let Some(e) = event.as_any().downcast_ref::<TransferCompleted>() {
            // Transfers don't affect cash flow (money moves between accounts, not income/expense)
            // But we track the transfer date for completeness
            let _ = e;
            let _ = accounts;
        }
    }
}

impl Default for CashFlowStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::domain::events::TransactionRecorded;
    use crate::shared::ids::{AccountID, CategoryID, TransactionID};
    use crate::shared::money::{Currency, Money};
    use crate::shared::period::YearMonth;

    #[test]
    fn test_income_recorded() {
        let store = CashFlowStore::new();
        let date = chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();

        store.handle_event(
            &TransactionRecorded {
                transaction_id: TransactionID::new(),
                account_id: AccountID::new(),
                tx_type: TransactionType::Income,
                amount: Money::from_cents(5000_00, Currency::BRL),
                category_id: Some(CategoryID::new()),
                description: "Salary".into(),
                date,
                timestamp: chrono::Utc::now(),
            },
            &Mutex::new(HashMap::new()),
        );

        let entry = store.get(date).unwrap();
        assert_eq!(
            entry.income.amount(),
            rust_decimal::Decimal::from(5000_00) / rust_decimal::Decimal::from(100)
        );
        assert!(entry.expense.is_zero());
        assert_eq!(
            entry.net.amount(),
            rust_decimal::Decimal::from(5000_00) / rust_decimal::Decimal::from(100)
        );
    }

    #[test]
    fn test_expense_recorded() {
        let store = CashFlowStore::new();
        let date = chrono::NaiveDate::from_ymd_opt(2026, 1, 20).unwrap();

        store.handle_event(
            &TransactionRecorded {
                transaction_id: TransactionID::new(),
                account_id: AccountID::new(),
                tx_type: TransactionType::Expense,
                amount: Money::from_cents(150_00, Currency::BRL),
                category_id: Some(CategoryID::new()),
                description: "Groceries".into(),
                date,
                timestamp: chrono::Utc::now(),
            },
            &Mutex::new(HashMap::new()),
        );

        let entry = store.get(date).unwrap();
        assert!(entry.income.is_zero());
        assert_eq!(
            entry.expense.amount(),
            rust_decimal::Decimal::from(150_00) / rust_decimal::Decimal::from(100)
        );
        assert_eq!(
            entry.net.amount(),
            rust_decimal::Decimal::from(-150_00) / rust_decimal::Decimal::from(100)
        );
    }

    #[test]
    fn test_multiple_transactions_same_day() {
        let store = CashFlowStore::new();
        let date = chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();

        store.handle_event(
            &TransactionRecorded {
                transaction_id: TransactionID::new(),
                account_id: AccountID::new(),
                tx_type: TransactionType::Income,
                amount: Money::from_cents(5000_00, Currency::BRL),
                category_id: None,
                description: "Salary".into(),
                date,
                timestamp: chrono::Utc::now(),
            },
            &Mutex::new(HashMap::new()),
        );

        store.handle_event(
            &TransactionRecorded {
                transaction_id: TransactionID::new(),
                account_id: AccountID::new(),
                tx_type: TransactionType::Expense,
                amount: Money::from_cents(200_00, Currency::BRL),
                category_id: None,
                description: "Food".into(),
                date,
                timestamp: chrono::Utc::now(),
            },
            &Mutex::new(HashMap::new()),
        );

        let entry = store.get(date).unwrap();
        assert_eq!(
            entry.income.amount(),
            rust_decimal::Decimal::from(5000_00) / rust_decimal::Decimal::from(100)
        );
        assert_eq!(
            entry.expense.amount(),
            rust_decimal::Decimal::from(200_00) / rust_decimal::Decimal::from(100)
        );
        assert_eq!(
            entry.net.amount(),
            rust_decimal::Decimal::from(4800_00) / rust_decimal::Decimal::from(100)
        );
    }

    #[test]
    fn test_transfer_ignored() {
        let store = CashFlowStore::new();
        let date = chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();

        store.handle_event(
            &TransferCompleted {
                from_account_id: AccountID::new(),
                to_account_id: AccountID::new(),
                amount: Money::from_cents(300_00, Currency::BRL),
                timestamp: chrono::Utc::now(),
            },
            &Mutex::new(HashMap::new()),
        );

        assert!(store.get(date).is_none());
    }

    #[test]
    fn test_get_period() {
        let store = CashFlowStore::new();
        let accounts = Mutex::new(HashMap::new());

        store.handle_event(
            &TransactionRecorded {
                transaction_id: TransactionID::new(),
                account_id: AccountID::new(),
                tx_type: TransactionType::Expense,
                amount: Money::from_cents(100_00, Currency::BRL),
                category_id: None,
                description: "Day 1".into(),
                date: chrono::NaiveDate::from_ymd_opt(2026, 1, 5).unwrap(),
                timestamp: chrono::Utc::now(),
            },
            &accounts,
        );

        store.handle_event(
            &TransactionRecorded {
                transaction_id: TransactionID::new(),
                account_id: AccountID::new(),
                tx_type: TransactionType::Expense,
                amount: Money::from_cents(200_00, Currency::BRL),
                category_id: None,
                description: "Day 15".into(),
                date: chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
                timestamp: chrono::Utc::now(),
            },
            &accounts,
        );

        let jan = YearMonth::new(2026, 1).period();
        let entries = store.get_period(jan);
        assert_eq!(entries.len(), 2);

        let feb = YearMonth::new(2026, 2).period();
        let entries = store.get_period(feb);
        assert!(entries.is_empty());
    }
}
