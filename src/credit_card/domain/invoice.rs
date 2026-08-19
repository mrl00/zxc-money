use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::shared::ids::{CreditCardID, InvoiceID};
use crate::shared::money::Money;
use crate::shared::period::YearMonth;

use super::purchase::Purchase;

/// Lifecycle status of an invoice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvoiceStatus {
    Open,
    Closed,
    Paid,
}

/// A monthly invoice for a [`CreditCard`](super::card::CreditCard), holding its purchases and status.
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
    /// Creates a new open invoice for the given credit card and reference month.
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

    /// Appends a [`Purchase`] to this invoice.
    ///
    /// # Errors
    ///
    /// Returns [`CreditCardError::InvoiceNotOpen`](crate::shared::errors::CreditCardError::InvoiceNotOpen) if the invoice is not in `Open` status.
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

    /// Closes this invoice, transitioning to [`InvoiceStatus::Closed`].
    ///
    /// # Errors
    ///
    /// Returns [`CreditCardError::InvoiceNotOpen`](crate::shared::errors::CreditCardError::InvoiceNotOpen) if the invoice is not open.
    pub fn close(&mut self) -> Result<(), crate::shared::errors::CreditCardError> {
        if self.status != InvoiceStatus::Open {
            return Err(crate::shared::errors::CreditCardError::InvoiceNotOpen);
        }
        self.status = InvoiceStatus::Closed;
        self.closed_at = Some(Utc::now());
        Ok(())
    }

    /// Pays this invoice, transitioning to [`InvoiceStatus::Paid`].
    ///
    /// # Errors
    ///
    /// Returns [`CreditCardError::InvariantViolation`](crate::shared::errors::CreditCardError::InvariantViolation) if the invoice is not closed.
    pub fn pay(&mut self) -> Result<(), crate::shared::errors::CreditCardError> {
        if self.status != InvoiceStatus::Closed {
            return Err(crate::shared::errors::CreditCardError::InvariantViolation(
                "only closed invoices can be paid".into(),
            ));
        }
        self.status = InvoiceStatus::Paid;
        Ok(())
    }

    /// Returns the sum of all purchase amounts in this invoice.
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

    /// Returns `true` if this invoice is in [`InvoiceStatus::Open`].
    pub fn is_open(&self) -> bool {
        self.status == InvoiceStatus::Open
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::errors::CreditCardError;
    use crate::shared::ids::{CategoryID, PurchaseID};
    use crate::shared::money::Currency;

    fn make_purchase(cents: i64) -> Purchase {
        Purchase::new(
            PurchaseID::new(),
            "Netflix".into(),
            Money::new(cents, Currency::BRL),
            1,
            CategoryID::new(),
            chrono::NaiveDate::from_ymd_opt(2026, 1, 10).unwrap(),
        )
    }

    fn make_invoice() -> Invoice {
        Invoice::new(
            InvoiceID::new(),
            crate::shared::ids::CreditCardID::new(),
            YearMonth::new(2026, 1),
        )
    }

    #[test]
    fn test_new_invoice_is_open() {
        let invoice = make_invoice();
        assert!(invoice.is_open());
        assert_eq!(invoice.purchases.len(), 0);
        assert_eq!(invoice.status, InvoiceStatus::Open);
    }

    #[test]
    fn test_add_purchase_to_open_invoice() {
        let mut invoice = make_invoice();
        let purchase = make_purchase(5000);
        invoice.add_purchase(purchase).unwrap();
        assert_eq!(invoice.purchases.len(), 1);
    }

    #[test]
    fn test_add_purchase_to_closed_invoice_fails() {
        let mut invoice = make_invoice();
        invoice.close().unwrap();
        let purchase = make_purchase(5000);
        let result = invoice.add_purchase(purchase);
        assert!(matches!(result, Err(CreditCardError::InvoiceNotOpen)));
    }

    #[test]
    fn test_close_open_invoice() {
        let mut invoice = make_invoice();
        invoice.close().unwrap();
        assert_eq!(invoice.status, InvoiceStatus::Closed);
        assert!(invoice.closed_at.is_some());
    }

    #[test]
    fn test_close_already_closed_fails() {
        let mut invoice = make_invoice();
        invoice.close().unwrap();
        let result = invoice.close();
        assert!(matches!(result, Err(CreditCardError::InvoiceNotOpen)));
    }

    #[test]
    fn test_pay_closed_invoice() {
        let mut invoice = make_invoice();
        invoice.close().unwrap();
        invoice.pay().unwrap();
        assert_eq!(invoice.status, InvoiceStatus::Paid);
    }

    #[test]
    fn test_pay_open_invoice_fails() {
        let mut invoice = make_invoice();
        let result = invoice.pay();
        assert!(matches!(
            result,
            Err(CreditCardError::InvariantViolation(_))
        ));
    }

    #[test]
    fn test_total_empty_invoice() {
        let invoice = make_invoice();
        assert_eq!(invoice.total(), Money::new(0, Currency::BRL));
    }

    #[test]
    fn test_total_with_purchases() {
        let mut invoice = make_invoice();
        invoice.add_purchase(make_purchase(5000)).unwrap();
        invoice.add_purchase(make_purchase(3000)).unwrap();
        invoice.add_purchase(make_purchase(2000)).unwrap();
        assert_eq!(invoice.total(), Money::new(10000, Currency::BRL));
    }
}
