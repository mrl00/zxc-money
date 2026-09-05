use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::shared::errors::InvestmentError;
use crate::shared::ids::AssetID;
use crate::shared::money::Currency;

/// Classification of a tradeable asset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetClass {
    /// Equity share in a company.
    Stock,
    /// Mutual fund or ETF.
    Fund,
    /// Bond or other fixed-income instrument.
    FixedIncome,
    /// Cryptocurrency.
    Crypto,
}

/// A tradeable financial instrument (e.g. stock, fund, bond, crypto).
///
/// Assets form a shared catalog — they are not scoped to any single user.
/// A [`Portfolio`](super::portfolio::Portfolio) references assets by their
/// [`AssetID`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    /// Unique identifier.
    pub id: AssetID,
    /// Ticker symbol (e.g. "PETR4", "BOVA11").
    pub ticker: String,
    /// Full name of the asset.
    pub name: String,
    /// Asset class (stock, fund, etc.).
    pub class: AssetClass,
    /// Trading currency.
    pub currency: Currency,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
}

impl Asset {
    /// Creates a new [`Asset`] after validating invariants.
    ///
    /// # Errors
    ///
    /// Returns [`InvestmentError::InvariantViolation`] if `ticker` or `name`
    /// is empty.
    pub fn new(
        id: AssetID,
        ticker: String,
        name: String,
        class: AssetClass,
        currency: Currency,
    ) -> Result<Self, InvestmentError> {
        if ticker.trim().is_empty() {
            return Err(InvestmentError::InvariantViolation(
                "ticker must not be empty".into(),
            ));
        }
        if name.trim().is_empty() {
            return Err(InvestmentError::InvariantViolation(
                "name must not be empty".into(),
            ));
        }
        Ok(Self {
            id,
            ticker,
            name,
            class,
            currency,
            created_at: Utc::now(),
        })
    }

    /// Calculates the total cost for a given quantity at the given unit price.
    ///
    /// `price` is the per-unit price in the asset's currency. Returns the
    /// total cost as [`Money`](crate::shared::money::Money).
    pub fn total_cost(
        &self,
        price: crate::shared::money::Money,
        quantity: Decimal,
    ) -> crate::shared::money::Money {
        price * quantity
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::ids::AssetID;

    #[test]
    fn test_asset_new_valid() {
        let asset = Asset::new(
            AssetID::new(),
            "PETR4".into(),
            "Petrobras".into(),
            AssetClass::Stock,
            Currency::BRL,
        );
        assert!(asset.is_ok());
        let asset = asset.unwrap();
        assert_eq!(asset.ticker, "PETR4");
        assert_eq!(asset.name, "Petrobras");
    }

    #[test]
    fn test_asset_new_empty_ticker_fails() {
        let result = Asset::new(
            AssetID::new(),
            "".into(),
            "Petrobras".into(),
            AssetClass::Stock,
            Currency::BRL,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_asset_new_whitespace_ticker_fails() {
        let result = Asset::new(
            AssetID::new(),
            "   ".into(),
            "Petrobras".into(),
            AssetClass::Stock,
            Currency::BRL,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_asset_new_empty_name_fails() {
        let result = Asset::new(
            AssetID::new(),
            "PETR4".into(),
            "".into(),
            AssetClass::Stock,
            Currency::BRL,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_asset_total_cost() {
        let asset = Asset::new(
            AssetID::new(),
            "PETR4".into(),
            "Petrobras".into(),
            AssetClass::Stock,
            Currency::BRL,
        )
        .unwrap();
        let price = crate::shared::money::Money::from_cents(2500, Currency::BRL); // R$ 25.00
        let quantity = Decimal::from(10);
        let total = asset.total_cost(price, quantity);
        assert_eq!(total.amount(), Decimal::from(250)); // R$ 250.00
    }
}
