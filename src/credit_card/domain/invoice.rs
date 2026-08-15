use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::shared::ids::{CreditCardID, InvoiceID};
use crate::shared::money::Money;
use crate::shared::period::YearMonth;

use super::purchase::Purchase;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvoiceStatus {
    Open,
    Closed,
    Paid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    pub id: InvoiceID,
    pub credit_card_id: CreditCardID,
    pub reference_month: YearMonth,
    pub purchases: Vec<Purchase>,
    pub status: InvoiceStatus,
    pub closed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl Invoice {
    pub fn new(id: InvoiceID, credit_card_id: CreditCardID, reference_month: YearMonth) -> Self {
        Self {
            id,
            credit_card_id,
            reference_month,
            purchases: Vec::new(),
            status: InvoiceStatus::Open,
            closed_at: None,
            created_at: Utc::now(),
        }
    }

    pub fn add_purchase(
        &mut self,
        purchase: Purchase,
    ) -> Result<(), crate::shared::errors::CreditCardError> {
        if self.status != InvoiceStatus::Open {
            return Err(crate::shared::errors::CreditCardError::InvoiceNotOpen);
        }
        self.purchases.push(purchase);
        Ok(())
    }

    pub fn close(&mut self) -> Result<(), crate::shared::errors::CreditCardError> {
        if self.status != InvoiceStatus::Open {
            return Err(crate::shared::errors::CreditCardError::InvoiceNotOpen);
        }
        self.status = InvoiceStatus::Closed;
        self.closed_at = Some(Utc::now());
        Ok(())
    }

    pub fn pay(&mut self) -> Result<(), crate::shared::errors::CreditCardError> {
        if self.status != InvoiceStatus::Closed {
            return Err(crate::shared::errors::CreditCardError::InvariantViolation(
                "only closed invoices can be paid".into(),
            ));
        }
        self.status = InvoiceStatus::Paid;
        Ok(())
    }

    pub fn total(&self) -> Money {
        let mut total = Money::zero(
            self.purchases
                .first()
                .map(|p| p.total_amount.currency())
                .unwrap_or(crate::shared::money::Currency::BRL),
        );
        for purchase in &self.purchases {
            total = total + purchase.total_amount;
        }
        total
    }

    pub fn is_open(&self) -> bool {
        self.status == InvoiceStatus::Open
    }
}
