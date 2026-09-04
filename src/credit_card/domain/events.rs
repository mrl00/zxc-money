use chrono::{DateTime, Utc};

use crate::shared::ids::{CreditCardID, InvoiceID, PurchaseID};
use crate::shared::money::Money;

/// Domain event emitted when a purchase is added to an invoice.
#[derive(Debug)]
pub struct PurchaseAdded {
    pub purchase_id: PurchaseID,
    pub invoice_id: InvoiceID,
    pub credit_card_id: CreditCardID,
    pub amount: Money,
    pub timestamp: DateTime<Utc>,
}

impl crate::shared::events::DomainEvent for PurchaseAdded {
    fn event_type(&self) -> &'static str {
        "PurchaseAdded"
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Domain event emitted when an invoice is closed.
#[derive(Debug)]
pub struct InvoiceClosed {
    pub invoice_id: InvoiceID,
    pub credit_card_id: CreditCardID,
    pub total: Money,
    pub timestamp: DateTime<Utc>,
}

impl crate::shared::events::DomainEvent for InvoiceClosed {
    fn event_type(&self) -> &'static str {
        "InvoiceClosed"
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Domain event emitted when a closed invoice is paid.
#[derive(Debug, Clone)]
pub struct InvoicePaid {
    pub invoice_id: InvoiceID,
    pub credit_card_id: CreditCardID,
    pub total: Money,
    pub timestamp: DateTime<Utc>,
}

impl crate::shared::events::DomainEvent for InvoicePaid {
    fn event_type(&self) -> &'static str {
        "InvoicePaid"
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
