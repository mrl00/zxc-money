use chrono::{DateTime, Utc};

use crate::shared::ids::AccountID;
use crate::shared::money::Money;

#[derive(Debug)]
pub struct AccountOpened {
    pub account_id: AccountID,
    pub name: String,
    pub currency: crate::shared::money::Currency,
    pub opening_balance: Money,
    pub timestamp: DateTime<Utc>,
}

impl crate::shared::events::DomainEvent for AccountOpened {
    fn event_type(&self) -> &'static str {
        "AccountOpened"
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
