use std::sync::Arc;

use crate::reporting::projections::account_balance::NetWorthSnapshot;
use crate::reporting::projections::net_worth::NetWorthStore;

/// Query to get a net worth snapshot.
pub struct GetNetWorthQuery {
    pub date: chrono::NaiveDate,
}

/// Handles [`GetNetWorthQuery`] by reading from the [`NetWorthStore`].
pub struct GetNetWorthHandler {
    net_worth_store: Arc<NetWorthStore>,
}

impl GetNetWorthHandler {
    /// Creates a new handler with the given net worth store.
    pub fn new(net_worth_store: Arc<NetWorthStore>) -> Self {
        Self { net_worth_store }
    }

    /// Executes the net worth query.
    pub fn handle(&self, query: GetNetWorthQuery) -> NetWorthSnapshot {
        self.net_worth_store.snapshot(query.date)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::domain::events::AccountOpened;
    use crate::shared::ids::UserID;
    use crate::shared::money::{Currency, Money};

    #[test]
    fn test_net_worth_handler() {
        let store = Arc::new(NetWorthStore::new());
        let handler = GetNetWorthHandler::new(store.clone());

        store.handle_event(&AccountOpened {
            account_id: crate::shared::ids::AccountID::new(),
            owner_id: UserID::new(),
            name: "Checking".into(),
            currency: Currency::BRL,
            opening_balance: Money::new(10000_00, Currency::BRL),
            timestamp: chrono::Utc::now(),
        });

        let snapshot = handler.handle(GetNetWorthQuery {
            date: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        });
        assert_eq!(snapshot.total_assets.amount(), 10000_00);
        assert_eq!(snapshot.net_worth.amount(), 10000_00);
    }

    #[test]
    fn test_net_worth_handler_empty() {
        let store = Arc::new(NetWorthStore::new());
        let handler = GetNetWorthHandler::new(store);

        let snapshot = handler.handle(GetNetWorthQuery {
            date: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        });
        assert!(snapshot.total_assets.is_zero());
        assert!(snapshot.net_worth.is_zero());
    }
}
