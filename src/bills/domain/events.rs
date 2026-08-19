use chrono::{DateTime, Utc};

use crate::shared::ids::{BillID, CategoryID};
use crate::shared::money::Money;

/// Event emitted when a new bill is scheduled.
#[derive(Debug)]
pub struct BillScheduled {
    pub bill_id: BillID,
    pub name: String,
    pub amount: Option<Money>,
    pub due_date: chrono::NaiveDate,
    pub timestamp: DateTime<Utc>,
}

impl crate::shared::events::DomainEvent for BillScheduled {
    fn event_type(&self) -> &'static str {
        "BillScheduled"
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Event emitted when a bill is marked as paid.
#[derive(Debug)]
pub struct BillPaid {
    pub bill_id: BillID,
    pub amount: Option<Money>,
    pub account_id: crate::shared::ids::AccountID,
    pub category_id: CategoryID,
    pub timestamp: DateTime<Utc>,
}

impl crate::shared::events::DomainEvent for BillPaid {
    fn event_type(&self) -> &'static str {
        "BillPaid"
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Event emitted when a bill becomes overdue.
#[derive(Debug)]
pub struct BillOverdue {
    pub bill_id: BillID,
    pub due_date: chrono::NaiveDate,
    pub timestamp: DateTime<Utc>,
}

impl crate::shared::events::DomainEvent for BillOverdue {
    fn event_type(&self) -> &'static str {
        "BillOverdue"
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Event emitted when a bill is approaching its due date.
#[derive(Debug)]
pub struct BillDueSoon {
    pub bill_id: BillID,
    pub due_date: chrono::NaiveDate,
    pub days_until_due: i64,
    pub timestamp: DateTime<Utc>,
}

impl crate::shared::events::DomainEvent for BillDueSoon {
    fn event_type(&self) -> &'static str {
        "BillDueSoon"
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
