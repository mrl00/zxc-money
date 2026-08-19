use chrono::{DateTime, Utc};

use crate::shared::ids::AccountID;

/// Event emitted when a batch of raw transactions is confirmed and imported.
#[derive(Debug)]
pub struct TransactionsImported {
    /// The target account that received the imported transactions.
    pub account_id: AccountID,
    /// Number of transactions imported in this batch.
    pub count: usize,
    /// Timestamp of the import.
    pub timestamp: DateTime<Utc>,
}

impl crate::shared::events::DomainEvent for TransactionsImported {
    fn event_type(&self) -> &'static str {
        "TransactionsImported"
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::events::DomainEvent;
    use crate::shared::ids::AccountID;

    #[test]
    fn test_event_type() {
        let event = TransactionsImported {
            account_id: AccountID::new(),
            count: 5,
            timestamp: Utc::now(),
        };
        assert_eq!(event.event_type(), "TransactionsImported");
    }

    #[test]
    fn test_as_any() {
        let event = TransactionsImported {
            account_id: AccountID::new(),
            count: 3,
            timestamp: Utc::now(),
        };
        let any_ref = event.as_any();
        assert!(any_ref.downcast_ref::<TransactionsImported>().is_some());
    }
}
