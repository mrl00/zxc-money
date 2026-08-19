//! Monetary value object with currency-safe arithmetic.
//!
//! All amounts are stored as `i64` cents to avoid floating-point precision issues.
//! The [`Currency`] enum enforces that operations only combine matching currencies.
//!
//! # Example
//!
//! ```ignore
//! use zxc_money::shared::money::{Money, Currency};
//!
//! let price = Money::new(49_90, Currency::BRL);  // R$ 49.90
//! let tax   = Money::new(9_98, Currency::BRL);   // R$  9.98
//! let total = price.checked_add(tax).unwrap();    // R$ 59.88
//! assert_eq!(format!("{total}"), "59 88");
//! ```

use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, Sub};

/// A monetary value with an associated currency.
///
/// Amounts are stored in **cents** (minor currency units) as `i64`.
/// This avoids floating-point rounding errors common in financial calculations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Serialize, Deserialize)]
pub struct Money {
    amount: i64,
    currency: Currency,
}

/// Supported currencies (ISO 4217 subset).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Serialize, Deserialize)]
pub enum Currency {
    /// Brazilian Real
    BRL,
    /// United States Dollar
    USD,
    /// Euro
    EUR,
}

impl Currency {
    /// Return the currency symbol (e.g. `"R$"`, `"$"`, `"€"`).
    pub fn symbol(&self) -> &'static str {
        match self {
            Currency::BRL => "R$",
            Currency::USD => "$",
            Currency::EUR => "€",
        }
    }

    /// Return the ISO 4217 currency code (e.g. `"BRL"`, `"USD"`, `"EUR"`).
    pub fn code(&self) -> &'static str {
        match self {
            Currency::BRL => "BRL",
            Currency::USD => "USD",
            Currency::EUR => "EUR",
        }
    }
}

impl Money {
    /// Create a new `Money` from an amount in cents and a currency.
    pub fn new(amount: i64, currency: Currency) -> Self {
        Self { amount, currency }
    }

    /// Alias for [`Money::new`]. Creates a `Money` from cents.
    pub fn from_decimal(cents: i64, currency: Currency) -> Self {
        Self {
            amount: cents,
            currency,
        }
    }

    /// Create a zero-amount `Money` in the given currency.
    pub fn zero(currency: Currency) -> Self {
        Self {
            amount: 0,
            currency,
        }
    }

    /// Return the amount in cents.
    pub fn amount(&self) -> i64 {
        self.amount
    }

    /// Return the currency.
    pub fn currency(&self) -> Currency {
        self.currency
    }

    /// Returns `true` if the amount is positive.
    pub fn is_positive(&self) -> bool {
        self.amount > 0
    }

    /// Returns `true` if the amount is negative.
    pub fn is_negative(&self) -> bool {
        self.amount < 0
    }

    /// Returns `true` if the amount is zero.
    pub fn is_zero(&self) -> bool {
        self.amount == 0
    }

    /// Return the absolute value of the amount.
    pub fn abs(&self) -> Self {
        Self {
            amount: self.amount.abs(),
            currency: self.currency,
        }
    }

    /// Safely add two `Money` values.
    ///
    /// Returns [`LedgerError::CurrencyMismatch`](crate::shared::errors::LedgerError::CurrencyMismatch) if currencies differ,
    /// or [`LedgerError::InvalidAmount`](crate::shared::errors::LedgerError::InvalidAmount) on integer overflow.
    pub fn checked_add(self, other: Money) -> Result<Money, crate::shared::errors::LedgerError> {
        if self.currency != other.currency {
            return Err(crate::shared::errors::LedgerError::CurrencyMismatch {
                expected: self.currency.code().to_string(),
                received: other.currency.code().to_string(),
            });
        }
        Ok(Money {
            amount: self.amount.checked_add(other.amount).ok_or_else(|| {
                crate::shared::errors::LedgerError::InvalidAmount("overflow on addition".into())
            })?,
            currency: self.currency,
        })
    }

    /// Safely subtract two `Money` values.
    ///
    /// Returns [`LedgerError::CurrencyMismatch`](crate::shared::errors::LedgerError::CurrencyMismatch) if currencies differ,
    /// or [`LedgerError::InvalidAmount`](crate::shared::errors::LedgerError::InvalidAmount) on integer overflow.
    pub fn checked_sub(self, other: Money) -> Result<Money, crate::shared::errors::LedgerError> {
        if self.currency != other.currency {
            return Err(crate::shared::errors::LedgerError::CurrencyMismatch {
                expected: self.currency.code().to_string(),
                received: other.currency.code().to_string(),
            });
        }
        Ok(Money {
            amount: self.amount.checked_sub(other.amount).ok_or_else(|| {
                crate::shared::errors::LedgerError::InvalidAmount("overflow on subtraction".into())
            })?,
            currency: self.currency,
        })
    }
}

impl Add for Money {
    type Output = Money;

    fn add(self, rhs: Money) -> Self::Output {
        self.checked_add(rhs)
            .expect("moeda incompatível ou overflow")
    }
}

impl Sub for Money {
    type Output = Money;

    fn sub(self, rhs: Money) -> Self::Output {
        self.checked_sub(rhs)
            .expect("moeda incompatível ou overflow")
    }
}

impl std::ops::Mul<rust_decimal::Decimal> for Money {
    type Output = Money;

    fn mul(self, rhs: rust_decimal::Decimal) -> Self::Output {
        let result = (self.amount as f64 * rhs.to_string().parse::<f64>().unwrap()) as i64;
        Money {
            amount: result,
            currency: self.currency,
        }
    }
}

impl std::ops::Mul<i64> for Money {
    type Output = Money;

    fn mul(self, rhs: i64) -> Self::Output {
        Money {
            amount: self.amount * rhs,
            currency: self.currency,
        }
    }
}

impl std::ops::Div<rust_decimal::Decimal> for Money {
    type Output = Money;

    fn div(self, rhs: rust_decimal::Decimal) -> Self::Output {
        let result = (self.amount as f64 / rhs.to_string().parse::<f64>().unwrap()) as i64;
        Money {
            amount: result,
            currency: self.currency,
        }
    }
}

impl std::ops::Div<i64> for Money {
    type Output = Money;

    fn div(self, rhs: i64) -> Self::Output {
        Money {
            amount: self.amount / rhs,
            currency: self.currency,
        }
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let units = self.amount / 100;
        let cents = (self.amount % 100).unsigned_abs();
        write!(f, "{} {:02}", units, cents)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_money_creation() {
        let m = Money::new(1500, Currency::BRL);
        assert_eq!(m.amount(), 1500);
        assert_eq!(m.currency(), Currency::BRL);
    }

    #[test]
    fn test_money_zero() {
        let m = Money::zero(Currency::USD);
        assert!(m.is_zero());
        assert!(!m.is_positive());
        assert!(!m.is_negative());
    }

    #[test]
    fn test_add_same_currency() {
        let a = Money::new(1000, Currency::BRL);
        let b = Money::new(500, Currency::BRL);
        let result = a.checked_add(b).unwrap();
        assert_eq!(result.amount(), 1500);
    }

    #[test]
    fn test_add_different_currency_fails() {
        let a = Money::new(1000, Currency::BRL);
        let b = Money::new(500, Currency::USD);
        assert!(a.checked_add(b).is_err());
    }

    #[test]
    fn test_sub() {
        let a = Money::new(1000, Currency::BRL);
        let b = Money::new(300, Currency::BRL);
        let result = a.checked_sub(b).unwrap();
        assert_eq!(result.amount(), 700);
    }

    #[test]
    fn test_display() {
        let m = Money::new(1550, Currency::BRL);
        assert_eq!(format!("{m}"), "15 50");
    }

    #[test]
    fn test_abs() {
        let m = Money::new(-500, Currency::EUR);
        assert_eq!(m.abs().amount(), 500);
    }

    #[test]
    fn test_currency_symbol() {
        assert_eq!(Currency::BRL.symbol(), "R$");
        assert_eq!(Currency::USD.symbol(), "$");
        assert_eq!(Currency::EUR.symbol(), "€");
    }

    #[test]
    fn test_serde_roundtrip() {
        let m = Money::new(12345, Currency::BRL);
        let json = serde_json::to_string(&m).unwrap();
        let deserialized: Money = serde_json::from_str(&json).unwrap();
        assert_eq!(m, deserialized);
    }
}
