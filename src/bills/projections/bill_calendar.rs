use std::collections::HashMap;
use std::sync::Mutex;

use chrono::{Datelike, NaiveDate};

use crate::bills::domain::bill::BillStatus;
use crate::bills::domain::events::{BillPaid, BillScheduled};
use crate::shared::ids::BillID;
use crate::shared::money::Money;

/// A single bill entry in the calendar view.
#[derive(Debug, Clone)]
pub struct BillCalendarEntry {
    /// The bill's unique identifier.
    pub bill_id: BillID,
    /// Human-readable bill name.
    pub name: String,
    /// Monetary amount, or `None` for variable-amount bills.
    pub amount: Option<Money>,
    /// When the bill is due.
    pub due_date: NaiveDate,
    /// Current payment status.
    pub status: BillStatus,
    /// Category for budgeting/tracking purposes.
    pub category_id: crate::shared::ids::CategoryID,
}

/// Aggregated bill total for a single day.
#[derive(Debug, Clone)]
pub struct DayBillTotal {
    /// The calendar date.
    pub date: NaiveDate,
    /// Sum of all pending bills due on this date.
    pub total: Money,
}

/// In-memory projection store for the bill calendar.
///
/// Consumes [`BillScheduled`] and [`BillPaid`] events to maintain a
/// read-optimized view of bills by date. Queries are O(n) over the
/// stored entries, which is acceptable for personal finance volumes.
pub struct BillCalendarStore {
    entries: Mutex<HashMap<BillID, BillCalendarEntry>>,
}

impl Default for BillCalendarStore {
    fn default() -> Self {
        Self::new()
    }
}

impl BillCalendarStore {
    /// Creates an empty calendar store.
    pub fn new() -> Self {
        Self {
            entries: Mutex::new(HashMap::new()),
        }
    }

    /// Updates the store when a new bill is scheduled.
    ///
    /// Inserts a [`BillCalendarEntry`] with [`BillStatus::Pending`].
    pub fn handle_bill_scheduled(&self, event: &BillScheduled) {
        let mut entries = self.entries.lock().unwrap();
        entries.insert(
            event.bill_id,
            BillCalendarEntry {
                bill_id: event.bill_id,
                name: event.name.clone(),
                amount: event.amount,
                due_date: event.due_date,
                status: BillStatus::Pending,
                category_id: crate::shared::ids::CategoryID::new(),
            },
        );
    }

    /// Updates the store when a bill is paid.
    ///
    /// Transitions the matching entry to [`BillStatus::Paid`].
    pub fn handle_bill_paid(&self, event: &BillPaid) {
        let mut entries = self.entries.lock().unwrap();
        if let Some(entry) = entries.get_mut(&event.bill_id) {
            entry.status = BillStatus::Paid;
        }
    }

    /// Returns all bills due in the given month, regardless of status.
    pub fn find_by_month(&self, year: i32, month: u32) -> Vec<BillCalendarEntry> {
        let entries = self.entries.lock().unwrap();
        entries
            .values()
            .filter(|e| e.due_date.year() == year && e.due_date.month() == month)
            .cloned()
            .collect()
    }

    /// Returns the total pending amount per day for the given month.
    ///
    /// Only includes bills with `Pending` status that have a
    /// non-`None` amount. Results are sorted chronologically.
    pub fn daily_totals(&self, year: i32, month: u32) -> Vec<DayBillTotal> {
        let entries = self.entries.lock().unwrap();
        let mut totals: HashMap<NaiveDate, i64> = HashMap::new();

        for entry in entries.values() {
            if entry.due_date.year() == year
                && entry.due_date.month() == month
                && entry.status == BillStatus::Pending
                && let Some(amount) = entry.amount
            {
                *totals.entry(entry.due_date).or_insert(0) += amount.amount();
            }
        }

        let mut result: Vec<DayBillTotal> = totals
            .into_iter()
            .map(|(date, total)| DayBillTotal {
                date,
                total: Money::new(total, crate::shared::money::Currency::BRL),
            })
            .collect();
        result.sort_by_key(|d| d.date);
        result
    }

    /// Returns pending bills due within the next `days` days from today.
    ///
    /// Excludes bills that have already been paid or are overdue.
    pub fn find_upcoming(&self, days: i64) -> Vec<BillCalendarEntry> {
        let entries = self.entries.lock().unwrap();
        let today = chrono::Utc::now().date_naive();
        let limit = today + chrono::Duration::days(days);

        entries
            .values()
            .filter(|e| {
                e.status == BillStatus::Pending && e.due_date >= today && e.due_date <= limit
            })
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::ids::CategoryID;
    use crate::shared::money::{Currency, Money};

    fn make_event(
        bill_id: BillID,
        name: &str,
        amount: Option<Money>,
        due_date: NaiveDate,
    ) -> BillScheduled {
        BillScheduled {
            bill_id,
            name: name.into(),
            amount,
            due_date,
            timestamp: chrono::Utc::now(),
        }
    }

    #[test]
    fn test_handle_bill_scheduled() {
        let store = BillCalendarStore::new();
        let id = BillID::new();
        let event = make_event(
            id,
            "Rent",
            Some(Money::new(1500_00, Currency::BRL)),
            chrono::NaiveDate::from_ymd_opt(2026, 3, 5).unwrap(),
        );

        store.handle_bill_scheduled(&event);

        let entries = store.find_by_month(2026, 3);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "Rent");
        assert_eq!(entries[0].status, BillStatus::Pending);
    }

    #[test]
    fn test_handle_bill_paid() {
        let store = BillCalendarStore::new();
        let id = BillID::new();
        let event = make_event(
            id,
            "Internet",
            Some(Money::new(99_90, Currency::BRL)),
            chrono::NaiveDate::from_ymd_opt(2026, 2, 10).unwrap(),
        );

        store.handle_bill_scheduled(&event);

        let paid = BillPaid {
            bill_id: id,
            amount: Some(Money::new(99_90, Currency::BRL)),
            account_id: crate::shared::ids::AccountID::new(),
            category_id: CategoryID::new(),
            timestamp: chrono::Utc::now(),
        };
        store.handle_bill_paid(&paid);

        let entries = store.find_by_month(2026, 2);
        assert_eq!(entries[0].status, BillStatus::Paid);
    }

    #[test]
    fn test_find_by_month_filters_correctly() {
        let store = BillCalendarStore::new();
        store.handle_bill_scheduled(&make_event(
            BillID::new(),
            "Jan bill",
            None,
            chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
        ));
        store.handle_bill_scheduled(&make_event(
            BillID::new(),
            "Feb bill",
            None,
            chrono::NaiveDate::from_ymd_opt(2026, 2, 10).unwrap(),
        ));
        store.handle_bill_scheduled(&make_event(
            BillID::new(),
            "Jan bill 2",
            None,
            chrono::NaiveDate::from_ymd_opt(2026, 1, 20).unwrap(),
        ));

        let jan = store.find_by_month(2026, 1);
        assert_eq!(jan.len(), 2);
        let feb = store.find_by_month(2026, 2);
        assert_eq!(feb.len(), 1);
    }

    #[test]
    fn test_daily_totals_only_pending() {
        let store = BillCalendarStore::new();
        let id1 = BillID::new();
        let id2 = BillID::new();

        store.handle_bill_scheduled(&make_event(
            id1,
            "Paid bill",
            Some(Money::new(100_00, Currency::BRL)),
            chrono::NaiveDate::from_ymd_opt(2026, 4, 5).unwrap(),
        ));
        store.handle_bill_scheduled(&make_event(
            id2,
            "Pending bill",
            Some(Money::new(200_00, Currency::BRL)),
            chrono::NaiveDate::from_ymd_opt(2026, 4, 5).unwrap(),
        ));

        let paid = BillPaid {
            bill_id: id1,
            amount: Some(Money::new(100_00, Currency::BRL)),
            account_id: crate::shared::ids::AccountID::new(),
            category_id: CategoryID::new(),
            timestamp: chrono::Utc::now(),
        };
        store.handle_bill_paid(&paid);

        let totals = store.daily_totals(2026, 4);
        assert_eq!(totals.len(), 1);
        assert_eq!(totals[0].total, Money::new(200_00, Currency::BRL));
    }

    #[test]
    fn test_daily_totals_groups_by_date() {
        let store = BillCalendarStore::new();
        store.handle_bill_scheduled(&make_event(
            BillID::new(),
            "Bill A",
            Some(Money::new(50_00, Currency::BRL)),
            chrono::NaiveDate::from_ymd_opt(2026, 5, 10).unwrap(),
        ));
        store.handle_bill_scheduled(&make_event(
            BillID::new(),
            "Bill B",
            Some(Money::new(75_00, Currency::BRL)),
            chrono::NaiveDate::from_ymd_opt(2026, 5, 10).unwrap(),
        ));
        store.handle_bill_scheduled(&make_event(
            BillID::new(),
            "Bill C",
            Some(Money::new(30_00, Currency::BRL)),
            chrono::NaiveDate::from_ymd_opt(2026, 5, 15).unwrap(),
        ));

        let totals = store.daily_totals(2026, 5);
        assert_eq!(totals.len(), 2);
        assert_eq!(totals[0].total, Money::new(125_00, Currency::BRL));
        assert_eq!(totals[1].total, Money::new(30_00, Currency::BRL));
    }

    #[test]
    fn test_find_upcoming() {
        let store = BillCalendarStore::new();
        let today = chrono::Utc::now().date_naive();

        store.handle_bill_scheduled(&make_event(
            BillID::new(),
            "Due tomorrow",
            None,
            today + chrono::Duration::days(1),
        ));
        store.handle_bill_scheduled(&make_event(
            BillID::new(),
            "Due in 10 days",
            None,
            today + chrono::Duration::days(10),
        ));
        store.handle_bill_scheduled(&make_event(
            BillID::new(),
            "Due in 30 days",
            None,
            today + chrono::Duration::days(30),
        ));

        let upcoming = store.find_upcoming(7);
        assert_eq!(upcoming.len(), 1);
        assert_eq!(upcoming[0].name, "Due tomorrow");
    }

    #[test]
    fn test_find_upcoming_excludes_paid() {
        let store = BillCalendarStore::new();
        let today = chrono::Utc::now().date_naive();
        let id = BillID::new();

        store.handle_bill_scheduled(&make_event(
            id,
            "Paid bill",
            None,
            today + chrono::Duration::days(2),
        ));

        let paid = BillPaid {
            bill_id: id,
            amount: None,
            account_id: crate::shared::ids::AccountID::new(),
            category_id: CategoryID::new(),
            timestamp: chrono::Utc::now(),
        };
        store.handle_bill_paid(&paid);

        let upcoming = store.find_upcoming(7);
        assert_eq!(upcoming.len(), 0);
    }

    #[test]
    fn test_daily_totals_sorted_by_date() {
        let store = BillCalendarStore::new();
        store.handle_bill_scheduled(&make_event(
            BillID::new(),
            "Later",
            Some(Money::new(100_00, Currency::BRL)),
            chrono::NaiveDate::from_ymd_opt(2026, 6, 20).unwrap(),
        ));
        store.handle_bill_scheduled(&make_event(
            BillID::new(),
            "Earlier",
            Some(Money::new(50_00, Currency::BRL)),
            chrono::NaiveDate::from_ymd_opt(2026, 6, 5).unwrap(),
        ));

        let totals = store.daily_totals(2026, 6);
        assert!(totals[0].date < totals[1].date);
    }
}
