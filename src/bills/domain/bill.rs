use chrono::{DateTime, Datelike, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use crate::shared::ids::{BillID, CategoryID, UserID};
use crate::shared::money::Money;

/// Status of a bill in its lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BillStatus {
    /// Bill is awaiting payment.
    Pending,
    /// Bill has been paid.
    Paid,
    /// Bill was not paid by its due date.
    Overdue,
}

/// Recurrence pattern for a repeating bill.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecurrenceRule {
    /// Repeats every month.
    Monthly,
    /// Repeats every week.
    Weekly,
    /// Repeats every year.
    Yearly,
}

/// A scheduled financial obligation (recurring or one-time).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bill {
    pub id: BillID,
    pub owner_id: UserID,
    pub name: String,
    pub amount: Option<Money>,
    pub due_date: NaiveDate,
    pub recurrence: Option<RecurrenceRule>,
    pub category_id: CategoryID,
    pub status: BillStatus,
    pub created_at: DateTime<Utc>,
}

impl Bill {
    /// Creates a new [`Bill`] with [`BillStatus::Pending`] and the current timestamp.
    pub fn new(
        id: BillID,
        owner_id: UserID,
        name: String,
        amount: Option<Money>,
        due_date: NaiveDate,
        recurrence: Option<RecurrenceRule>,
        category_id: CategoryID,
    ) -> Self {
        Self {
            id,
            owner_id,
            name,
            amount,
            due_date,
            recurrence,
            category_id,
            status: BillStatus::Pending,
            created_at: Utc::now(),
        }
    }

    /// Marks the bill as paid.
    ///
    /// Returns an error if the bill is not in [`BillStatus::Pending`].
    pub fn mark_paid(&mut self) -> Result<(), crate::shared::errors::BillsError> {
        if self.status != BillStatus::Pending {
            return Err(crate::shared::errors::BillsError::InvariantViolation(
                "only pending bills can be marked as paid".into(),
            ));
        }
        self.status = BillStatus::Paid;
        Ok(())
    }

    /// Marks the bill as overdue.
    ///
    /// Returns an error if the bill is not in [`BillStatus::Pending`].
    pub fn mark_overdue(&mut self) -> Result<(), crate::shared::errors::BillsError> {
        if self.status != BillStatus::Pending {
            return Err(crate::shared::errors::BillsError::InvariantViolation(
                "only pending bills can be marked as overdue".into(),
            ));
        }
        self.status = BillStatus::Overdue;
        Ok(())
    }

    /// Returns the next due date based on the recurrence rule.
    ///
    /// Returns `None` if the bill has no recurrence.
    pub fn next_due_date(&self) -> Option<NaiveDate> {
        match self.recurrence? {
            RecurrenceRule::Monthly => {
                if self.due_date.month() == 12 {
                    NaiveDate::from_ymd_opt(self.due_date.year() + 1, 1, self.due_date.day())
                } else {
                    NaiveDate::from_ymd_opt(
                        self.due_date.year(),
                        self.due_date.month() + 1,
                        self.due_date.day(),
                    )
                }
            }
            RecurrenceRule::Weekly => self.due_date.checked_add_signed(chrono::Duration::weeks(1)),
            RecurrenceRule::Yearly => NaiveDate::from_ymd_opt(
                self.due_date.year() + 1,
                self.due_date.month(),
                self.due_date.day(),
            ),
        }
    }
}
