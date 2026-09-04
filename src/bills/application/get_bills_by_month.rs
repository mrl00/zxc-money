use std::sync::Arc;

use crate::bills::projections::bill_calendar::{BillCalendarEntry, BillCalendarStore};

/// Query to retrieve all bills due in a specific month.
pub struct GetBillsByMonthQuery {
    /// Calendar year (e.g. `2026`).
    pub year: i32,
    /// Calendar month (1–12).
    pub month: u32,
}

/// Handler that returns all bills due in the given month.
///
/// Reads from the [`BillCalendarStore`] projection. Results include both
/// pending and paid bills — use the `status` field to filter in the UI.
pub struct GetBillsByMonthHandler {
    calendar_store: Arc<BillCalendarStore>,
}

impl GetBillsByMonthHandler {
    /// Creates a new handler backed by the given calendar store.
    pub fn new(calendar_store: Arc<BillCalendarStore>) -> Self {
        Self { calendar_store }
    }

    /// Executes the query and returns matching [`BillCalendarEntry`] items.
    pub async fn handle(&self, query: GetBillsByMonthQuery) -> Vec<BillCalendarEntry> {
        self.calendar_store.find_by_month(query.year, query.month)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bills::domain::events::BillScheduled;
    use crate::shared::ids::BillID;
    use crate::shared::money::{Currency, Money};

    #[tokio::test]
    async fn test_get_bills_by_month() {
        let store = Arc::new(BillCalendarStore::new());
        let handler = GetBillsByMonthHandler::new(store.clone());

        store.handle_bill_scheduled(&BillScheduled {
            bill_id: BillID::new(),
            name: "Rent".into(),
            amount: Some(Money::new(1500_00, Currency::BRL)),
            due_date: chrono::NaiveDate::from_ymd_opt(2026, 3, 5).unwrap(),
            timestamp: chrono::Utc::now(),
        });

        store.handle_bill_scheduled(&BillScheduled {
            bill_id: BillID::new(),
            name: "Internet".into(),
            amount: Some(Money::new(99_90, Currency::BRL)),
            due_date: chrono::NaiveDate::from_ymd_opt(2026, 3, 10).unwrap(),
            timestamp: chrono::Utc::now(),
        });

        let result = handler
            .handle(GetBillsByMonthQuery {
                year: 2026,
                month: 3,
            })
            .await;
        assert_eq!(result.len(), 2);
    }

    #[tokio::test]
    async fn test_get_bills_by_month_empty() {
        let store = Arc::new(BillCalendarStore::new());
        let handler = GetBillsByMonthHandler::new(store);

        let result = handler
            .handle(GetBillsByMonthQuery {
                year: 2026,
                month: 12,
            })
            .await;
        assert_eq!(result.len(), 0);
    }
}
