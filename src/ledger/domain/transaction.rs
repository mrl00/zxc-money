use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use crate::shared::errors::LedgerError;
use crate::shared::ids::{AccountID, CategoryID, PurchaseID, TagID, TransactionID};
use crate::shared::money::Money;

/// Type of a financial transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionType {
    /// Money received into an account.
    Income,
    /// Money spent from an account.
    Expense,
    /// Money moved between two accounts.
    Transfer,
}

/// A single financial transaction recorded against an account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: TransactionID,
    pub account_id: AccountID,
    pub tx_type: TransactionType,
    pub amount: Money,
    pub description: String,
    pub date: NaiveDate,
    pub category_id: Option<CategoryID>,
    pub tags: Vec<TagID>,
    pub counterpart_account_id: Option<AccountID>,
    pub source_purchase_id: Option<PurchaseID>,
    pub reconciled: bool,
    pub created_at: DateTime<Utc>,
}

impl Transaction {
    /// Creates a new transaction with the required fields.
    ///
    /// # Errors
    /// Returns an error if `amount` is not positive or `description` is empty.
    pub fn new(
        id: TransactionID,
        account_id: AccountID,
        tx_type: TransactionType,
        amount: Money,
        description: String,
        date: NaiveDate,
    ) -> Result<Self, LedgerError> {
        if !amount.is_positive() {
            return Err(LedgerError::InvalidAmount(
                "transaction amount must be positive".into(),
            ));
        }

        if description.is_empty() {
            return Err(LedgerError::InvariantViolation(
                "transaction description must not be empty".into(),
            ));
        }

        Ok(Self {
            id,
            account_id,
            tx_type,
            amount,
            description,
            date,
            category_id: None,
            tags: Vec::new(),
            counterpart_account_id: None,
            source_purchase_id: None,
            reconciled: false,
            created_at: Utc::now(),
        })
    }

    /// Attaches a category to an income or expense transaction.
    ///
    /// # Errors
    /// Returns an error if the transaction is a transfer (transfers cannot have categories).
    pub fn with_category(mut self, category_id: CategoryID) -> Result<Self, LedgerError> {
        match self.tx_type {
            TransactionType::Transfer => Err(LedgerError::InvariantViolation(
                "transfer must not have category".into(),
            )),
            TransactionType::Income | TransactionType::Expense => {
                self.category_id = Some(category_id);
                Ok(self)
            }
        }
    }

    /// Sets the counterpart account for a transfer transaction.
    ///
    /// # Errors
    /// Returns an error if the transaction is not a transfer.
    pub fn with_counterpart(
        mut self,
        counterpart_account_id: AccountID,
    ) -> Result<Self, LedgerError> {
        if self.tx_type != TransactionType::Transfer {
            return Err(LedgerError::InvariantViolation(
                "only transfer transactions can have counterpart".into(),
            ));
        }
        self.counterpart_account_id = Some(counterpart_account_id);
        Ok(self)
    }

    /// Sets the tags on this transaction.
    pub fn with_tags(mut self, tags: Vec<TagID>) -> Self {
        self.tags = tags;
        self
    }

    /// Records the source credit card purchase ID.
    pub fn with_source_purchase(mut self, purchase_id: PurchaseID) -> Self {
        self.source_purchase_id = Some(purchase_id);
        self
    }

    /// Marks this transaction as reconciled.
    pub fn mark_reconciled(&mut self) {
        self.reconciled = true;
    }

    /// Validates all business invariants on this transaction.
    ///
    /// # Errors
    /// Returns an error if any invariant is violated (e.g. transfer without counterpart,
    /// income/expense without category).
    pub fn validate(&self) -> Result<(), LedgerError> {
        if !self.amount.is_positive() {
            return Err(LedgerError::InvalidAmount(
                "transaction amount must be positive".into(),
            ));
        }

        if self.description.is_empty() {
            return Err(LedgerError::InvariantViolation(
                "transaction description must not be empty".into(),
            ));
        }

        match self.tx_type {
            TransactionType::Transfer => {
                if self.counterpart_account_id.is_none() {
                    return Err(LedgerError::InvariantViolation(
                        "transfer must have counterpart account".into(),
                    ));
                }
                if self.category_id.is_some() {
                    return Err(LedgerError::InvariantViolation(
                        "transfer must not have category".into(),
                    ));
                }
            }
            TransactionType::Income | TransactionType::Expense => {
                if self.category_id.is_none() {
                    return Err(LedgerError::InvariantViolation(
                        "income/expense must have category".into(),
                    ));
                }
                if self.counterpart_account_id.is_some() {
                    return Err(LedgerError::InvariantViolation(
                        "income/expense must not have counterpart".into(),
                    ));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::money::Currency;

    fn income_tx() -> Transaction {
        Transaction::new(
            TransactionID::new(),
            AccountID::new(),
            TransactionType::Income,
            Money::new(1000, Currency::BRL),
            "Salary".into(),
            NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
        )
        .unwrap()
    }

    fn transfer_tx() -> Transaction {
        Transaction::new(
            TransactionID::new(),
            AccountID::new(),
            TransactionType::Transfer,
            Money::new(500, Currency::BRL),
            "Transfer".into(),
            NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
        )
        .unwrap()
        .with_counterpart(AccountID::new())
        .unwrap()
    }

    #[test]
    fn test_income_requires_category() {
        let tx = income_tx();
        assert!(tx.category_id.is_none());
        let tx = tx.with_category(CategoryID::new()).unwrap();
        assert!(tx.category_id.is_some());
    }

    #[test]
    fn test_transfer_rejects_category() {
        let tx = transfer_tx();
        let result = tx.with_category(CategoryID::new());
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            LedgerError::InvariantViolation(_)
        ));
    }

    #[test]
    fn test_transfer_requires_counterpart() {
        let tx = Transaction::new(
            TransactionID::new(),
            AccountID::new(),
            TransactionType::Transfer,
            Money::new(500, Currency::BRL),
            "Transfer".into(),
            NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
        )
        .unwrap();
        assert!(tx.counterpart_account_id.is_none());
        let result = tx.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_income_rejects_counterpart() {
        let tx = income_tx();
        let result = tx.with_counterpart(AccountID::new());
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            LedgerError::InvariantViolation(_)
        ));
    }

    #[test]
    fn test_amount_must_be_positive() {
        let result = Transaction::new(
            TransactionID::new(),
            AccountID::new(),
            TransactionType::Income,
            Money::new(0, Currency::BRL),
            "Zero".into(),
            NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
        );
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LedgerError::InvalidAmount(_)));
    }

    #[test]
    fn test_negative_amount_rejected() {
        let result = Transaction::new(
            TransactionID::new(),
            AccountID::new(),
            TransactionType::Expense,
            Money::new(-100, Currency::BRL),
            "Negative".into(),
            NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_description_must_not_be_empty() {
        let result = Transaction::new(
            TransactionID::new(),
            AccountID::new(),
            TransactionType::Income,
            Money::new(1000, Currency::BRL),
            "".into(),
            NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
        );
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            LedgerError::InvariantViolation(_)
        ));
    }

    #[test]
    fn test_validate_income_without_category() {
        let tx = Transaction::new(
            TransactionID::new(),
            AccountID::new(),
            TransactionType::Income,
            Money::new(1000, Currency::BRL),
            "Salary".into(),
            NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
        )
        .unwrap();
        let result = tx.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_transfer_with_category() {
        let mut tx = Transaction::new(
            TransactionID::new(),
            AccountID::new(),
            TransactionType::Transfer,
            Money::new(500, Currency::BRL),
            "Transfer".into(),
            NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
        )
        .unwrap()
        .with_counterpart(AccountID::new())
        .unwrap();
        tx.category_id = Some(CategoryID::new());
        let result = tx.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_transfer() {
        let tx = transfer_tx();
        assert!(tx.validate().is_ok());
    }

    #[test]
    fn test_valid_income() {
        let tx = income_tx().with_category(CategoryID::new()).unwrap();
        assert!(tx.validate().is_ok());
    }
}
