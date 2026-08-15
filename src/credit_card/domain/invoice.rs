use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::shared::ids::InvoiceID;
use crate::shared::money::Money;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvoiceStatus {
    Open,
    Closed,
    Paid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    pub id: InvoiceID,
    pub credit_card_id: crate::shared::ids::CreditCardID,
    pub reference_year: i32,
    pub reference_month: u32,
    pub total: Money,
    pub status: InvoiceStatus,
    pub created_at: DateTime<Utc>,
}
