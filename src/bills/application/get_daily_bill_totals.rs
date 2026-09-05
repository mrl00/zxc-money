use std::sync::Arc;

use crate::bills::projections::bill_calendar::{BillCalendarStore, DayBillTotal};

/// Query to retrieve the total pending amount per day in a given month.
pub struct GetDailyBillTotalsQuery {
    /// Calendar year (e.g. `2026`).
    pub year: i32,
    /// Calendar month (1–12).
    pub month: u32,
}

/// Handler that returns daily aggregated bill totals for a month.
///
/// Only includes bills with `Pending` status
/// that have a known amount. Results are sorted chronologically.
pub struct GetDailyBillTotalsHandler {
    calendar_store: Arc<BillCalendarStore>,
}

impl GetDailyBillTotalsHandler {
    /// Creates a new handler backed by the given calendar store.
    pub fn new(calendar_store: Arc<BillCalendarStore>) -> Self {
        Self { calendar_store }
    }

    /// Executes the query and returns [`DayBillTotal`] items sorted by date.
    pub async fn handle(&self, query: GetDailyBillTotalsQuery) -> Vec<DayBillTotal> {
        self.calendar_store.daily_totals(query.year, query.month)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bills::domain::events::BillScheduled;
    use crate::shared::ids::BillID;
    use crate::shared::money::{Currency, Money};

    #[tokio::test]
    async fn test_get_daily_bill_totals() {
        let store = Arc::new(BillCalendarStore::new());
        let handler = GetDailyBillTotalsHandler::new(store.clone());

        store.handle_bill_scheduled(&BillScheduled {
            bill_id: BillID::new(),
            name: "Bill A".into(),
            amount: Some(Money::from_cents(100_00, Currency::BRL)),
            due_date: chrono::NaiveDate::from_ymd_opt(2026, 4, 5).unwrap(),
            timestamp: chrono::Utc::now(),
        });

        store.handle_bill_scheduled(&BillScheduled {
            bill_id: BillID::new(),
            name: "Bill B".into(),
            amount: Some(Money::from_cents(200_00, Currency::BRL)),
            due_date: chrono::NaiveDate::from_ymd_opt(2026, 4, 5).unwrap(),
            timestamp: chrono::Utc::now(),
        });

        let result = handler
            .handle(GetDailyBillTotalsQuery {
                year: 2026,
                month: 4,
            })
            .await;
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].total, Money::from_cents(300_00, Currency::BRL));
    }

    #[tokio::test]
    async fn test_get_daily_bill_totals_empty() {
        let store = Arc::new(BillCalendarStore::new());
        let handler = GetDailyBillTotalsHandler::new(store);

        let result = handler
            .handle(GetDailyBillTotalsQuery {
                year: 2026,
                month: 7,
            })
            .await;
        assert_eq!(result.len(), 0);
    }
}
