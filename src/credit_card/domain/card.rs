use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::shared::ids::{CreditCardID, UserID};
use crate::shared::money::Money;

/// A credit card owned by a user, with a spending limit and billing cycle configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditCard {
    pub id: CreditCardID,
    pub owner_id: UserID,
    pub name: String,
    pub brand: String,
    pub limit: Money,
    pub closing_day: u32,
    pub due_day: u32,
    pub created_at: DateTime<Utc>,
}

impl CreditCard {
    /// Creates a new [`CreditCard`] with the given parameters.
    /// Sets [`created_at`](Self::created_at) to the current UTC time.
    pub fn new(
        id: CreditCardID,
        owner_id: UserID,
        name: String,
        brand: String,
        limit: Money,
        closing_day: u32,
        due_day: u32,
    ) -> Self {
        Self {
            id,
            owner_id,
            name,
            brand,
            limit,
            closing_day,
            due_day,
            created_at: Utc::now(),
        }
    }

    /// Returns the remaining available credit after subtracting `used` from [`limit`](Self::limit).
    ///
    /// # Errors
    ///
    /// Returns [`CreditCardError::InvariantViolation`](crate::shared::errors::CreditCardError::InvariantViolation) if the used amount exceeds the limit.
    pub fn available_limit(
        &self,
        used: Money,
    ) -> Result<Money, crate::shared::errors::CreditCardError> {
        self.limit.checked_sub(used).map_err(|_| {
            crate::shared::errors::CreditCardError::InvariantViolation(
                "used amount exceeds limit".into(),
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::money::Currency;

    fn make_card(limit_cents: i64) -> CreditCard {
        CreditCard::new(
            CreditCardID::new(),
            UserID::new(),
            "Nubank".into(),
            "Mastercard".into(),
            Money::new(limit_cents, Currency::BRL),
            20,
            27,
        )
    }

    #[test]
    fn test_available_limit_within_bound() {
        let card = make_card(1000000);
        let available = card
            .available_limit(Money::new(300000, Currency::BRL))
            .unwrap();
        assert_eq!(available, Money::new(700000, Currency::BRL));
    }

    #[test]
    fn test_available_limit_exact() {
        let card = make_card(1000000);
        let available = card
            .available_limit(Money::new(1000000, Currency::BRL))
            .unwrap();
        assert_eq!(available, Money::new(0, Currency::BRL));
    }

    #[test]
    fn test_available_limit_exceeds() {
        let card = make_card(1000000);
        let available = card
            .available_limit(Money::new(1500000, Currency::BRL))
            .unwrap();
        assert!(available.amount() < 0);
    }

    #[test]
    fn test_available_limit_zero_used() {
        let card = make_card(500000);
        let available = card.available_limit(Money::new(0, Currency::BRL)).unwrap();
        assert_eq!(available, Money::new(500000, Currency::BRL));
    }
}
