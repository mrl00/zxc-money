use std::sync::Arc;

use crate::bills::projections::bill_calendar::{BillCalendarEntry, BillCalendarStore};

/// Query to retrieve bills due within the next N days.
pub struct GetUpcomingBillsQuery {
    /// Number of days to look ahead from today.
    pub days: i64,
}

/// Handler that returns upcoming pending bills.
///
/// Only includes bills with `Pending` status
/// whose due date falls between today and `today + days`.
pub struct GetUpcomingBillsHandler {
    calendar_store: Arc<BillCalendarStore>,
}

impl GetUpcomingBillsHandler {
    /// Creates a new handler backed by the given calendar store.
    pub fn new(calendar_store: Arc<BillCalendarStore>) -> Self {
        Self { calendar_store }
    }

    /// Executes the query and returns matching [`BillCalendarEntry`] items.
    pub async fn handle(&self, query: GetUpcomingBillsQuery) -> Vec<BillCalendarEntry> {
        self.calendar_store.find_upcoming(query.days)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bills::domain::events::BillScheduled;
    use crate::shared::ids::BillID;
    use crate::shared::money::{Currency, Money};

    #[tokio::test]
    async fn test_get_upcoming_bills() {
        let store = Arc::new(BillCalendarStore::new());
        let handler = GetUpcomingBillsHandler::new(store.clone());
        let today = chrono::Utc::now().date_naive();

        store.handle_bill_scheduled(&BillScheduled {
            bill_id: BillID::new(),
            name: "Due tomorrow".into(),
            amount: Some(Money::from_cents(50_00, Currency::BRL)),
            due_date: today + chrono::Duration::days(1),
            timestamp: chrono::Utc::now(),
        });

        store.handle_bill_scheduled(&BillScheduled {
            bill_id: BillID::new(),
            name: "Due in 30 days".into(),
            amount: Some(Money::from_cents(100_00, Currency::BRL)),
            due_date: today + chrono::Duration::days(30),
            timestamp: chrono::Utc::now(),
        });

        let result = handler.handle(GetUpcomingBillsQuery { days: 7 }).await;
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "Due tomorrow");
    }

    #[tokio::test]
    async fn test_get_upcoming_bills_none() {
        let store = Arc::new(BillCalendarStore::new());
        let handler = GetUpcomingBillsHandler::new(store);

        let result = handler.handle(GetUpcomingBillsQuery { days: 3 }).await;
        assert_eq!(result.len(), 0);
    }
}
