use std::collections::HashMap;
use std::sync::Mutex;

use crate::ledger::domain::events::{
    AccountDeleted, AccountOpened, TransactionDeleted, TransactionReconciled, TransactionRecorded,
    TransferCompleted,
};
use crate::ledger::domain::transaction::TransactionType;
use crate::reporting::projections::account_balance::AccountBalanceProjection;
use crate::shared::events::DomainEvent;
use crate::shared::ids::AccountID;

pub struct AccountBalanceProjectionStore {
    projections: Mutex<HashMap<AccountID, AccountBalanceProjection>>,
}

impl AccountBalanceProjectionStore {
    pub fn new() -> Self {
        Self {
            projections: Mutex::new(HashMap::new()),
        }
    }

    pub fn get(&self, account_id: AccountID) -> Option<AccountBalanceProjection> {
        let projections = self.projections.lock().unwrap();
        projections.get(&account_id).cloned()
    }

    pub fn get_all(&self) -> Vec<AccountBalanceProjection> {
        let projections = self.projections.lock().unwrap();
        projections.values().cloned().collect()
    }

    pub fn handle_event(&self, event: &dyn DomainEvent) {
        if let Some(e) = event.as_any().downcast_ref::<AccountOpened>() {
            let mut projections = self.projections.lock().unwrap();
            projections.insert(
                e.account_id,
                AccountBalanceProjection::new(e.account_id, e.opening_balance),
            );
        }

        if let Some(e) = event.as_any().downcast_ref::<TransactionRecorded>() {
            let mut projections = self.projections.lock().unwrap();
            if let Some(projection) = projections.get_mut(&e.account_id) {
                match e.tx_type {
                    TransactionType::Income | TransactionType::Transfer => {
                        projection.balance = projection.balance + e.amount;
                    }
                    TransactionType::Expense => {
                        projection.balance = projection.balance - e.amount;
                    }
                }
                projection.last_updated = chrono::Utc::now();
            }
        }

        if let Some(e) = event.as_any().downcast_ref::<TransactionDeleted>() {
            let mut projections = self.projections.lock().unwrap();
            if let Some(projection) = projections.get_mut(&e.account_id) {
                projection.last_updated = chrono::Utc::now();
            }
        }

        if let Some(e) = event.as_any().downcast_ref::<TransferCompleted>() {
            let mut projections = self.projections.lock().unwrap();
            if let Some(from) = projections.get_mut(&e.from_account_id) {
                from.balance = from.balance - e.amount;
                from.last_updated = chrono::Utc::now();
            }
            if let Some(to) = projections.get_mut(&e.to_account_id) {
                to.balance = to.balance + e.amount;
                to.last_updated = chrono::Utc::now();
            }
        }

        if let Some(e) = event.as_any().downcast_ref::<TransactionReconciled>() {
            let mut projections = self.projections.lock().unwrap();
            if let Some(projection) = projections.get_mut(&e.account_id) {
                projection.reconciled_balance = projection.balance;
                projection.last_updated = chrono::Utc::now();
            }
        }

        if let Some(e) = event.as_any().downcast_ref::<AccountDeleted>() {
            let mut projections = self.projections.lock().unwrap();
            projections.remove(&e.account_id);
        }
    }
}

impl Default for AccountBalanceProjectionStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::domain::events::AccountOpened;
    use crate::shared::ids::UserID;
    use crate::shared::money::{Currency, Money};

    #[test]
    fn test_projection_on_account_opened() {
        let store = AccountBalanceProjectionStore::new();
        let account_id = AccountID::new();

        let event = AccountOpened {
            account_id,
            owner_id: UserID::new(),
            name: "Test".into(),
            currency: Currency::BRL,
            opening_balance: Money::new(1000, Currency::BRL),
            timestamp: chrono::Utc::now(),
        };

        store.handle_event(&event);

        let projection = store.get(account_id).unwrap();
        assert_eq!(projection.balance, Money::new(1000, Currency::BRL));
        assert_eq!(
            projection.reconciled_balance,
            Money::new(1000, Currency::BRL)
        );
    }

    #[test]
    fn test_projection_on_income() {
        let store = AccountBalanceProjectionStore::new();
        let account_id = AccountID::new();

        store.handle_event(&AccountOpened {
            account_id,
            owner_id: UserID::new(),
            name: "Test".into(),
            currency: Currency::BRL,
            opening_balance: Money::new(1000, Currency::BRL),
            timestamp: chrono::Utc::now(),
        });

        store.handle_event(&TransactionRecorded {
            transaction_id: crate::shared::ids::TransactionID::new(),
            account_id,
            tx_type: TransactionType::Income,
            amount: Money::new(500, Currency::BRL),
            category_id: Some(crate::shared::ids::CategoryID::new()),
            description: "Salary".into(),
            date: chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            timestamp: chrono::Utc::now(),
        });

        let projection = store.get(account_id).unwrap();
        assert_eq!(projection.balance, Money::new(1500, Currency::BRL));
    }

    #[test]
    fn test_projection_on_expense() {
        let store = AccountBalanceProjectionStore::new();
        let account_id = AccountID::new();

        store.handle_event(&AccountOpened {
            account_id,
            owner_id: UserID::new(),
            name: "Test".into(),
            currency: Currency::BRL,
            opening_balance: Money::new(1000, Currency::BRL),
            timestamp: chrono::Utc::now(),
        });

        store.handle_event(&TransactionRecorded {
            transaction_id: crate::shared::ids::TransactionID::new(),
            account_id,
            tx_type: TransactionType::Expense,
            amount: Money::new(200, Currency::BRL),
            category_id: Some(crate::shared::ids::CategoryID::new()),
            description: "Food".into(),
            date: chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            timestamp: chrono::Utc::now(),
        });

        let projection = store.get(account_id).unwrap();
        assert_eq!(projection.balance, Money::new(800, Currency::BRL));
    }

    #[test]
    fn test_projection_on_transfer() {
        let store = AccountBalanceProjectionStore::new();
        let from_id = AccountID::new();
        let to_id = AccountID::new();

        store.handle_event(&AccountOpened {
            account_id: from_id,
            owner_id: UserID::new(),
            name: "From".into(),
            currency: Currency::BRL,
            opening_balance: Money::new(1000, Currency::BRL),
            timestamp: chrono::Utc::now(),
        });
        store.handle_event(&AccountOpened {
            account_id: to_id,
            owner_id: UserID::new(),
            name: "To".into(),
            currency: Currency::BRL,
            opening_balance: Money::new(500, Currency::BRL),
            timestamp: chrono::Utc::now(),
        });

        store.handle_event(&TransferCompleted {
            from_account_id: from_id,
            to_account_id: to_id,
            amount: Money::new(300, Currency::BRL),
            timestamp: chrono::Utc::now(),
        });

        assert_eq!(
            store.get(from_id).unwrap().balance,
            Money::new(700, Currency::BRL)
        );
        assert_eq!(
            store.get(to_id).unwrap().balance,
            Money::new(800, Currency::BRL)
        );
    }

    #[test]
    fn test_projection_on_delete() {
        let store = AccountBalanceProjectionStore::new();
        let account_id = AccountID::new();

        store.handle_event(&AccountOpened {
            account_id,
            owner_id: UserID::new(),
            name: "Test".into(),
            currency: Currency::BRL,
            opening_balance: Money::new(1000, Currency::BRL),
            timestamp: chrono::Utc::now(),
        });
        assert!(store.get(account_id).is_some());

        store.handle_event(&AccountDeleted {
            account_id,
            owner_id: UserID::new(),
            timestamp: chrono::Utc::now(),
        });
        assert!(store.get(account_id).is_none());
    }
}
