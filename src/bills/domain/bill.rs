use chrono::{DateTime, Datelike, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use crate::shared::ids::{BillID, CategoryID, UserID};
use crate::shared::money::Money;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BillStatus {
    Pending,
    Paid,
    Overdue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecurrenceRule {
    Monthly,
    Weekly,
    Yearly,
}

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

    pub fn mark_paid(&mut self) -> Result<(), crate::shared::errors::BillsError> {
        if self.status != BillStatus::Pending {
            return Err(crate::shared::errors::BillsError::InvariantViolation(
                "only pending bills can be marked as paid".into(),
            ));
        }
        self.status = BillStatus::Paid;
        Ok(())
    }

    pub fn mark_overdue(&mut self) -> Result<(), crate::shared::errors::BillsError> {
        if self.status != BillStatus::Pending {
            return Err(crate::shared::errors::BillsError::InvariantViolation(
                "only pending bills can be marked as overdue".into(),
            ));
        }
        self.status = BillStatus::Overdue;
        Ok(())
    }

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
