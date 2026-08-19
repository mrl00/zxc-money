use std::collections::HashMap;
use std::sync::Mutex;

use crate::ledger::domain::events::{
    AccountDeleted, AccountOpened, TransactionRecorded, TransferCompleted,
};
use crate::ledger::domain::transaction::TransactionType;
use crate::reporting::projections::account_balance::NetWorthSnapshot;
use crate::shared::events::DomainEvent;
use crate::shared::ids::AccountID;
use crate::shared::money::Money;

/// In-memory store that maintains net worth snapshots from domain events.
pub struct NetWorthStore {
    balances: Mutex<HashMap<AccountID, Money>>,
}

impl NetWorthStore {
    /// Creates an empty store.
    pub fn new() -> Self {
        Self {
            balances: Mutex::new(HashMap::new()),
        }
    }

    /// Returns a net worth snapshot for the current state.
    pub fn snapshot(&self, date: chrono::NaiveDate) -> NetWorthSnapshot {
        let balances = self.balances.lock().unwrap();
        let total_assets: Money = balances.values().fold(
            Money::zero(crate::shared::money::Currency::BRL),
            |acc, &m| acc + m,
        );

        NetWorthSnapshot {
            date,
            total_assets,
            total_liabilities: Money::zero(crate::shared::money::Currency::BRL),
            net_worth: total_assets,
        }
    }

    /// Returns the balance for a specific account, if tracked.
    pub fn get_balance(&self, account_id: AccountID) -> Option<Money> {
        let balances = self.balances.lock().unwrap();
        balances.get(&account_id).copied()
    }

    /// Returns all account balances.
    pub fn get_all_balances(&self) -> Vec<(AccountID, Money)> {
        let balances = self.balances.lock().unwrap();
        balances.iter().map(|(&id, &amount)| (id, amount)).collect()
    }

    /// Applies a domain event to update the net worth projection.
    pub fn handle_event(&self, event: &dyn DomainEvent) {
        if let Some(e) = event.as_any().downcast_ref::<AccountOpened>() {
            let mut balances = self.balances.lock().unwrap();
            balances.insert(e.account_id, e.opening_balance);
        }

        if let Some(e) = event.as_any().downcast_ref::<TransactionRecorded>() {
            let mut balances = self.balances.lock().unwrap();
            if let Some(balance) = balances.get_mut(&e.account_id) {
                match e.tx_type {
                    TransactionType::Income => {
                        *balance = *balance + e.amount;
                    }
                    TransactionType::Expense => {
                        *balance = *balance - e.amount;
                    }
                    TransactionType::Transfer => {
                        // Transfers don't change net worth (money moves between accounts)
                    }
                }
            }
        }

        if let Some(e) = event.as_any().downcast_ref::<TransferCompleted>() {
            let mut balances = self.balances.lock().unwrap();
            if let Some(from) = balances.get_mut(&e.from_account_id) {
                *from = *from - e.amount;
            }
            if let Some(to) = balances.get_mut(&e.to_account_id) {
                *to = *to + e.amount;
            }
        }

        if let Some(e) = event.as_any().downcast_ref::<AccountDeleted>() {
            let mut balances = self.balances.lock().unwrap();
            balances.remove(&e.account_id);
        }
    }
}

impl Default for NetWorthStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::domain::events::{AccountOpened, TransactionRecorded};
    use crate::shared::ids::{CategoryID, TransactionID, UserID};
    use crate::shared::money::{Currency, Money};

    fn open_account(store: &NetWorthStore, id: AccountID, balance: i64) {
        store.handle_event(&AccountOpened {
            account_id: id,
            owner_id: UserID::new(),
            name: "Test".into(),
            currency: Currency::BRL,
            opening_balance: Money::new(balance, Currency::BRL),
            timestamp: chrono::Utc::now(),
        });
    }

    #[test]
    fn test_net_worth_single_account() {
        let store = NetWorthStore::new();
        let id = AccountID::new();
        open_account(&store, id, 1000_00);

        let snapshot = store.snapshot(chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
        assert_eq!(snapshot.total_assets.amount(), 1000_00);
        assert_eq!(snapshot.net_worth.amount(), 1000_00);
    }

    #[test]
    fn test_net_worth_multiple_accounts() {
        let store = NetWorthStore::new();
        let id1 = AccountID::new();
        let id2 = AccountID::new();
        open_account(&store, id1, 5000_00);
        open_account(&store, id2, 3000_00);

        let snapshot = store.snapshot(chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
        assert_eq!(snapshot.total_assets.amount(), 8000_00);
    }

    #[test]
    fn test_net_worth_after_income() {
        let store = NetWorthStore::new();
        let id = AccountID::new();
        open_account(&store, id, 1000_00);

        store.handle_event(&TransactionRecorded {
            transaction_id: TransactionID::new(),
            account_id: id,
            tx_type: TransactionType::Income,
            amount: Money::new(500_00, Currency::BRL),
            category_id: Some(CategoryID::new()),
            description: "Salary".into(),
            date: chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            timestamp: chrono::Utc::now(),
        });

        let snapshot = store.snapshot(chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap());
        assert_eq!(snapshot.total_assets.amount(), 1500_00);
    }

    #[test]
    fn test_net_worth_after_expense() {
        let store = NetWorthStore::new();
        let id = AccountID::new();
        open_account(&store, id, 1000_00);

        store.handle_event(&TransactionRecorded {
            transaction_id: TransactionID::new(),
            account_id: id,
            tx_type: TransactionType::Expense,
            amount: Money::new(200_00, Currency::BRL),
            category_id: Some(CategoryID::new()),
            description: "Food".into(),
            date: chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            timestamp: chrono::Utc::now(),
        });

        let snapshot = store.snapshot(chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap());
        assert_eq!(snapshot.total_assets.amount(), 800_00);
    }

    #[test]
    fn test_net_worth_transfer_unchanged() {
        let store = NetWorthStore::new();
        let from_id = AccountID::new();
        let to_id = AccountID::new();
        open_account(&store, from_id, 1000_00);
        open_account(&store, to_id, 500_00);

        store.handle_event(&TransferCompleted {
            from_account_id: from_id,
            to_account_id: to_id,
            amount: Money::new(300_00, Currency::BRL),
            timestamp: chrono::Utc::now(),
        });

        let snapshot = store.snapshot(chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
        assert_eq!(snapshot.total_assets.amount(), 1500_00);
        assert_eq!(snapshot.net_worth.amount(), 1500_00);
    }

    #[test]
    fn test_net_worth_account_deleted() {
        let store = NetWorthStore::new();
        let id = AccountID::new();
        open_account(&store, id, 1000_00);

        store.handle_event(&AccountDeleted {
            account_id: id,
            owner_id: UserID::new(),
            timestamp: chrono::Utc::now(),
        });

        let snapshot = store.snapshot(chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
        assert_eq!(snapshot.total_assets.amount(), 0);
    }
}
