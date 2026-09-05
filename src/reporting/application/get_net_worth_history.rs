use std::sync::Arc;

use crate::reporting::projections::account_balance::NetWorthSnapshot;
use crate::reporting::projections::net_worth::NetWorthStore;
use crate::shared::period::YearMonth;

/// Query to get net worth snapshots over a date range.
pub struct GetNetWorthHistoryQuery {
    pub start: chrono::NaiveDate,
    pub end: chrono::NaiveDate,
}

/// Handles [`GetNetWorthHistoryQuery`] by sampling net worth at each month boundary.
pub struct GetNetWorthHistoryHandler {
    net_worth_store: Arc<NetWorthStore>,
}

impl GetNetWorthHistoryHandler {
    /// Creates a new handler with the given net worth store.
    pub fn new(net_worth_store: Arc<NetWorthStore>) -> Self {
        Self { net_worth_store }
    }

    /// Returns net worth snapshots for each month in the given range.
    pub fn handle(&self, query: GetNetWorthHistoryQuery) -> Vec<NetWorthSnapshot> {
        let start_month = YearMonth::from_date(query.start);
        let end_month = YearMonth::from_date(query.end);

        let mut snapshots = Vec::new();
        let mut current = start_month;

        loop {
            let date = current.last_day().min(query.end);
            snapshots.push(self.net_worth_store.snapshot(date));

            if current >= end_month {
                break;
            }
            current = current.next();
        }

        snapshots
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::domain::events::AccountOpened;

    use crate::shared::ids::{AccountID, UserID};
    use crate::shared::money::{Currency, Money};
    use rust_decimal::Decimal;

    fn brl(amount: i64) -> Money {
        Money::from_cents(amount, Currency::BRL)
    }

    #[test]
    fn test_history_single_month() {
        let store = Arc::new(NetWorthStore::new());
        let handler = GetNetWorthHistoryHandler::new(store.clone());

        store.handle_event(&AccountOpened {
            account_id: AccountID::new(),
            owner_id: UserID::new(),
            name: "Checking".into(),
            currency: Currency::BRL,
            opening_balance: brl(1000_00),
            timestamp: chrono::Utc::now(),
        });

        let snapshots = handler.handle(GetNetWorthHistoryQuery {
            start: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            end: chrono::NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
        });

        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0].total_assets.amount(), Decimal::from(1000));
    }

    #[test]
    fn test_history_multiple_months() {
        let store = Arc::new(NetWorthStore::new());
        let handler = GetNetWorthHistoryHandler::new(store.clone());

        store.handle_event(&AccountOpened {
            account_id: AccountID::new(),
            owner_id: UserID::new(),
            name: "Checking".into(),
            currency: Currency::BRL,
            opening_balance: brl(5000_00),
            timestamp: chrono::Utc::now(),
        });

        let snapshots = handler.handle(GetNetWorthHistoryQuery {
            start: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            end: chrono::NaiveDate::from_ymd_opt(2026, 3, 31).unwrap(),
        });

        assert_eq!(snapshots.len(), 3);
        for snap in &snapshots {
            assert_eq!(snap.total_assets.amount(), Decimal::from(5000));
        }
    }

    #[test]
    fn test_history_empty() {
        let store = Arc::new(NetWorthStore::new());
        let handler = GetNetWorthHistoryHandler::new(store);

        let snapshots = handler.handle(GetNetWorthHistoryQuery {
            start: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            end: chrono::NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        });

        assert_eq!(snapshots.len(), 12);
        for snap in &snapshots {
            assert!(snap.total_assets.is_zero());
        }
    }
}
