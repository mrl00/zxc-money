use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use crate::shared::ids::BillID;
use crate::shared::money::Money;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BillStatus {
    Pending,
    Paid,
    Overdue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bill {
    pub id: BillID,
    pub name: String,
    pub amount: Option<Money>,
    pub due_date: NaiveDate,
    pub status: BillStatus,
    pub created_at: DateTime<Utc>,
}
