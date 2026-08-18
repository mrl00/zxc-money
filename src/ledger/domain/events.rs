use chrono::{DateTime, NaiveDate, Utc};

use crate::shared::ids::{AccountID, CategoryID, RecurringTransactionID, TransactionID, UserID};
use crate::shared::money::Money;

/// Emitted when a new account is opened.
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

/// Emitted when an account is renamed.
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

/// Emitted when an account's type is changed.
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

/// Emitted when an account is deleted.
#[derive(Debug)]
pub struct AccountDeleted {
    pub account_id: AccountID,
    pub owner_id: UserID,
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

/// Emitted when a transaction is recorded.
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

/// Emitted when a transaction is deleted.
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

/// Emitted when a transaction is reconciled.
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

/// Emitted when a transaction is updated.
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

/// Emitted when a recurring transaction is created.
#[derive(Debug)]
pub struct RecurringTransactionCreated {
    pub recurring_transaction_id: RecurringTransactionID,
    pub owner_id: UserID,
    pub account_id: AccountID,
    pub amount: Money,
    pub frequency: crate::ledger::domain::recurring_transaction::Frequency,
    pub next_date: NaiveDate,
    pub timestamp: DateTime<Utc>,
}

impl crate::shared::events::DomainEvent for RecurringTransactionCreated {
    fn event_type(&self) -> &'static str {
        "RecurringTransactionCreated"
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Emitted when a pending recurring transaction generates a concrete transaction.
#[derive(Debug)]
pub struct RecurringTransactionGenerated {
    pub recurring_transaction_id: RecurringTransactionID,
    pub transaction_id: TransactionID,
    pub account_id: AccountID,
    pub next_date: NaiveDate,
    pub timestamp: DateTime<Utc>,
}

impl crate::shared::events::DomainEvent for RecurringTransactionGenerated {
    fn event_type(&self) -> &'static str {
        "RecurringTransactionGenerated"
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Emitted when funds are transferred between two accounts.
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
