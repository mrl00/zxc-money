use chrono::{DateTime, Utc};

use crate::shared::ids::{AccountID, CategoryID, TransactionID, UserID};
use crate::shared::money::Money;

#[derive(Debug)]
pub struct AccountOpened {
    pub account_id: AccountID,
    pub owner_id: UserID,
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

#[derive(Debug)]
pub struct AccountRenamed {
    pub account_id: AccountID,
    pub new_name: String,
    pub timestamp: DateTime<Utc>,
}

impl crate::shared::events::DomainEvent for AccountRenamed {
    fn event_type(&self) -> &'static str {
        "AccountRenamed"
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug)]
pub struct AccountTypeChanged {
    pub account_id: AccountID,
    pub new_type: crate::ledger::domain::account::AccountType,
    pub timestamp: DateTime<Utc>,
}

impl crate::shared::events::DomainEvent for AccountTypeChanged {
    fn event_type(&self) -> &'static str {
        "AccountTypeChanged"
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug)]
pub struct AccountDeleted {
    pub account_id: AccountID,
    pub timestamp: DateTime<Utc>,
}

impl crate::shared::events::DomainEvent for AccountDeleted {
    fn event_type(&self) -> &'static str {
        "AccountDeleted"
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug)]
pub struct TransactionRecorded {
    pub transaction_id: TransactionID,
    pub account_id: AccountID,
    pub tx_type: crate::ledger::domain::transaction::TransactionType,
    pub amount: Money,
    pub category_id: Option<CategoryID>,
    pub description: String,
    pub date: chrono::NaiveDate,
    pub timestamp: DateTime<Utc>,
}

impl crate::shared::events::DomainEvent for TransactionRecorded {
    fn event_type(&self) -> &'static str {
        "TransactionRecorded"
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug)]
pub struct TransactionDeleted {
    pub transaction_id: crate::shared::ids::TransactionID,
    pub account_id: AccountID,
    pub timestamp: DateTime<Utc>,
}

impl crate::shared::events::DomainEvent for TransactionDeleted {
    fn event_type(&self) -> &'static str {
        "TransactionDeleted"
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug)]
pub struct TransactionReconciled {
    pub transaction_id: TransactionID,
    pub account_id: AccountID,
    pub reconciled: bool,
    pub timestamp: DateTime<Utc>,
}

impl crate::shared::events::DomainEvent for TransactionReconciled {
    fn event_type(&self) -> &'static str {
        "TransactionReconciled"
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug)]
pub struct TransactionUpdated {
    pub transaction_id: TransactionID,
    pub account_id: AccountID,
    pub timestamp: DateTime<Utc>,
}

impl crate::shared::events::DomainEvent for TransactionUpdated {
    fn event_type(&self) -> &'static str {
        "TransactionUpdated"
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug)]
pub struct TransferCompleted {
    pub from_account_id: AccountID,
    pub to_account_id: AccountID,
    pub amount: Money,
    pub timestamp: DateTime<Utc>,
}

impl crate::shared::events::DomainEvent for TransferCompleted {
    fn event_type(&self) -> &'static str {
        "TransferCompleted"
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
