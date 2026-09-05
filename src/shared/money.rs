//! Monetary value object with currency-safe arithmetic.
//!
//! Amounts are stored as [`rust_decimal::Decimal`] in major currency units
//! (e.g. `15.50` for R$ 15,50). The [`Currency`] enum enforces that operations
//! only combine matching currencies.
//!
//! # Example
//!
//! ```ignore
//! use zxc_money::shared::money::{Money, Currency};
//!
//! let price = Money::new(dec!(49.90), Currency::BRL);  // R$ 49,90
//! let tax   = Money::new(dec!(9.98),  Currency::BRL);  // R$  9,98
//! let total = price.checked_add(tax).unwrap();           // R$ 59,88
//! assert_eq!(format!("{total}"), "49.90 + 9.98");
//! ```

use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, Sub};

/// A monetary value with an associated currency.
///
/// Amounts are stored as [`Decimal`] in major currency units (e.g. `15.50`).
/// This avoids floating-point rounding errors common in financial calculations
/// and provides arbitrary precision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Serialize, Deserialize)]
pub struct Money {
    amount: Decimal,
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
    /// Create a new `Money` from a decimal amount and a currency.
    pub fn new(amount: Decimal, currency: Currency) -> Self {
        Self { amount, currency }
    }

    /// Create a `Money` from an amount in minor currency units (cents).
    ///
    /// This is a convenience for migrating from the old `i64`-based API.
    /// `Money::from_cents(1500, BRL)` creates R$ 15,00.
    pub fn from_cents(cents: i64, currency: Currency) -> Self {
        Self {
            amount: Decimal::from(cents) / Decimal::from(100),
            currency,
        }
    }

    /// Create a zero-amount `Money` in the given currency.
    pub fn zero(currency: Currency) -> Self {
        Self {
            amount: Decimal::ZERO,
            currency,
        }
    }

    /// Return the amount as a [`Decimal`].
    pub fn amount(&self) -> Decimal {
        self.amount
    }

    /// Return the amount truncated to an integer (no fractional part).
    pub fn amount_trunc(&self) -> Decimal {
        self.amount.trunc()
    }

    /// Return the amount in minor currency units (cents) as `i64`.
    ///
    /// Returns `None` if the amount cannot be exactly represented as `i64` cents.
    pub fn to_cents(&self) -> Option<i64> {
        (self.amount * Decimal::from(100)).to_i64()
    }

    /// Return the currency.
    pub fn currency(&self) -> Currency {
        self.currency
    }

    /// Returns `true` if the amount is positive.
    pub fn is_positive(&self) -> bool {
        self.amount > Decimal::ZERO
    }

    /// Returns `true` if the amount is negative.
    pub fn is_negative(&self) -> bool {
        self.amount < Decimal::ZERO
    }

    /// Returns `true` if the amount is zero.
    pub fn is_zero(&self) -> bool {
        self.amount == Decimal::ZERO
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
    /// Returns [`LedgerError::CurrencyMismatch`] if currencies differ.
    pub fn checked_add(self, other: Money) -> Result<Money, crate::shared::errors::LedgerError> {
        if self.currency != other.currency {
            return Err(crate::shared::errors::LedgerError::CurrencyMismatch {
                expected: self.currency.code().to_string(),
                received: other.currency.code().to_string(),
            });
        }
        Ok(Money {
            amount: self.amount + other.amount,
            currency: self.currency,
        })
    }

    /// Safely subtract two `Money` values.
    ///
    /// Returns [`LedgerError::CurrencyMismatch`] if currencies differ.
    pub fn checked_sub(self, other: Money) -> Result<Money, crate::shared::errors::LedgerError> {
        if self.currency != other.currency {
            return Err(crate::shared::errors::LedgerError::CurrencyMismatch {
                expected: self.currency.code().to_string(),
                received: other.currency.code().to_string(),
            });
        }
        Ok(Money {
            amount: self.amount - other.amount,
            currency: self.currency,
        })
    }
}

impl Add for Money {
    type Output = Result<Money, crate::shared::errors::LedgerError>;

    fn add(self, rhs: Money) -> Self::Output {
        self.checked_add(rhs)
    }
}

impl Sub for Money {
    type Output = Result<Money, crate::shared::errors::LedgerError>;

    fn sub(self, rhs: Money) -> Self::Output {
        self.checked_sub(rhs)
    }
}

impl std::ops::Mul<Decimal> for Money {
    type Output = Money;

    fn mul(self, rhs: Decimal) -> Self::Output {
        Money {
            amount: self.amount * rhs,
            currency: self.currency,
        }
    }
}

impl std::ops::Mul<i64> for Money {
    type Output = Money;

    fn mul(self, rhs: i64) -> Self::Output {
        Money {
            amount: self.amount * Decimal::from(rhs),
            currency: self.currency,
        }
    }
}

impl std::ops::Div<Decimal> for Money {
    type Output = Money;

    fn div(self, rhs: Decimal) -> Self::Output {
        Money {
            amount: self.amount / rhs,
            currency: self.currency,
        }
    }
}

impl std::ops::Div<i64> for Money {
    type Output = Money;

    fn div(self, rhs: i64) -> Self::Output {
        Money {
            amount: self.amount / Decimal::from(rhs),
            currency: self.currency,
        }
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.amount)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_money_creation() {
        let m = Money::new(dec!(15.00), Currency::BRL);
        assert_eq!(m.amount(), dec!(15.00));
        assert_eq!(m.currency(), Currency::BRL);
    }

    #[test]
    fn test_from_cents() {
        let m = Money::from_cents(1500, Currency::BRL);
        assert_eq!(m.amount(), dec!(15.00));
        assert_eq!(m.to_cents(), Some(1500));
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
        let a = Money::new(dec!(10.00), Currency::BRL);
        let b = Money::new(dec!(5.00), Currency::BRL);
        let result = (a + b).unwrap();
        assert_eq!(result.amount(), dec!(15.00));
    }

    #[test]
    fn test_add_different_currency_fails() {
        let a = Money::new(dec!(10.00), Currency::BRL);
        let b = Money::new(dec!(5.00), Currency::USD);
        assert!((a + b).is_err());
    }

    #[test]
    fn test_sub() {
        let a = Money::new(dec!(10.00), Currency::BRL);
        let b = Money::new(dec!(3.00), Currency::BRL);
        let result = (a - b).unwrap();
        assert_eq!(result.amount(), dec!(7.00));
    }

    #[test]
    fn test_mul_decimal() {
        let m = Money::new(dec!(100.00), Currency::BRL);
        let result = m * dec!(1.5);
        assert_eq!(result.amount(), dec!(150.00));
    }

    #[test]
    fn test_mul_i64() {
        let m = Money::new(dec!(50.00), Currency::BRL);
        let result = m * 3;
        assert_eq!(result.amount(), dec!(150.00));
    }

    #[test]
    fn test_div_decimal() {
        let m = Money::new(dec!(150.00), Currency::BRL);
        let result = m / dec!(3);
        assert_eq!(result.amount(), dec!(50.00));
    }

    #[test]
    fn test_div_i64() {
        let m = Money::new(dec!(150.00), Currency::BRL);
        let result = m / 3;
        assert_eq!(result.amount(), dec!(50.00));
    }

    #[test]
    fn test_display() {
        let m = Money::new(dec!(15.50), Currency::BRL);
        assert_eq!(format!("{m}"), "15.50");
    }

    #[test]
    fn test_abs() {
        let m = Money::new(dec!(-5.00), Currency::EUR);
        assert_eq!(m.abs().amount(), dec!(5.00));
    }

    #[test]
    fn test_currency_symbol() {
        assert_eq!(Currency::BRL.symbol(), "R$");
        assert_eq!(Currency::USD.symbol(), "$");
        assert_eq!(Currency::EUR.symbol(), "€");
    }

    #[test]
    fn test_serde_roundtrip() {
        let m = Money::new(dec!(123.45), Currency::BRL);
        let json = serde_json::to_string(&m).unwrap();
        let deserialized: Money = serde_json::from_str(&json).unwrap();
        assert_eq!(m, deserialized);
    }

    #[test]
    fn test_to_cents() {
        let m = Money::new(dec!(15.50), Currency::BRL);
        assert_eq!(m.to_cents(), Some(1550));
    }

    #[test]
    fn test_amount_trunc() {
        let m = Money::new(dec!(15.99), Currency::BRL);
        assert_eq!(m.amount_trunc(), dec!(15));
    }
}
