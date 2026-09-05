use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::shared::errors::LedgerError;
use crate::shared::ids::{AccountID, UserID};
use crate::shared::money::Money;

/// Type of financial account.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountType {
    /// Standard checking (current) account.
    Checking,
    /// Savings account with potential interest.
    Savings,
    /// Digital wallet / cash account.
    Wallet,
    /// Investment or brokerage account.
    Investment,
}

/// A financial account belonging to a user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    /// Unique identifier.
    pub id: AccountID,
    /// Owner of this account.
    pub owner_id: UserID,
    /// User-defined display name.
    pub name: String,
    /// Type of account (checking, savings, etc.).
    pub account_type: AccountType,
    /// Currency of this account.
    pub currency: crate::shared::money::Currency,
    /// Balance at account creation.
    pub opening_balance: Money,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
}

impl Account {
    /// Creates a new account with the given parameters.
    ///
    /// # Errors
    /// Returns [`LedgerError::InvariantViolation`] if `name` is empty, or
    /// [`LedgerError::CurrencyMismatch`] if the opening balance currency differs
    /// from `currency`.
    pub fn new(
        id: AccountID,
        owner_id: UserID,
        name: String,
        account_type: AccountType,
        currency: crate::shared::money::Currency,
        opening_balance: Money,
    ) -> Result<Self, LedgerError> {
        if name.is_empty() {
            return Err(LedgerError::InvariantViolation(
                "account name must not be empty".into(),
            ));
        }

        if opening_balance.currency() != currency {
            return Err(LedgerError::CurrencyMismatch {
                expected: currency.code().to_string(),
                received: opening_balance.currency().code().to_string(),
            });
        }

        Ok(Self {
            id,
            owner_id,
            name,
            account_type,
            currency,
            opening_balance,
            created_at: Utc::now(),
        })
    }

    /// Renames the account.
    pub fn rename(&mut self, new_name: String) {
        self.name = new_name;
    }

    /// Changes the account type.
    pub fn change_type(&mut self, new_type: AccountType) {
        self.account_type = new_type;
    }

    /// Returns the account's currency.
    pub fn currency(&self) -> crate::shared::money::Currency {
        self.currency
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::money::Currency;

    #[test]
    fn test_valid_account() {
        let account = Account::new(
            AccountID::new(),
            UserID::new(),
            "My Account".into(),
            AccountType::Checking,
            Currency::BRL,
            Money::from_cents(1000, Currency::BRL),
        );
        assert!(account.is_ok());
    }

    #[test]
    fn test_empty_name_rejected() {
        let result = Account::new(
            AccountID::new(),
            UserID::new(),
            "".into(),
            AccountType::Checking,
            Currency::BRL,
            Money::from_cents(1000, Currency::BRL),
        );
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            LedgerError::InvariantViolation(_)
        ));
    }

    #[test]
    fn test_currency_mismatch_rejected() {
        let result = Account::new(
            AccountID::new(),
            UserID::new(),
            "Account".into(),
            AccountType::Checking,
            Currency::BRL,
            Money::from_cents(1000, Currency::USD),
        );
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            LedgerError::CurrencyMismatch { .. }
        ));
    }

    #[test]
    fn test_rename() {
        let mut account = Account::new(
            AccountID::new(),
            UserID::new(),
            "Old Name".into(),
            AccountType::Checking,
            Currency::BRL,
            Money::from_cents(0, Currency::BRL),
        )
        .unwrap();
        account.rename("New Name".into());
        assert_eq!(account.name, "New Name");
    }

    #[test]
    fn test_change_type() {
        let mut account = Account::new(
            AccountID::new(),
            UserID::new(),
            "Account".into(),
            AccountType::Checking,
            Currency::BRL,
            Money::from_cents(0, Currency::BRL),
        )
        .unwrap();
        account.change_type(AccountType::Savings);
        assert_eq!(account.account_type, AccountType::Savings);
    }
}
