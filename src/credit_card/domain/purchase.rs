use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use crate::shared::ids::{CategoryID, PurchaseID};
use crate::shared::money::Money;

/// A single purchase made with a credit card, potentially split into installments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Purchase {
    pub id: PurchaseID,
    pub description: String,
    pub total_amount: Money,
    pub installments_count: u32,
    pub category_id: CategoryID,
    pub purchased_at: NaiveDate,
    pub created_at: DateTime<Utc>,
}

impl Purchase {
    /// Creates a new [`Purchase`] with the given details.
    /// Sets [`created_at`](Self::created_at) to the current UTC time.
    pub fn new(
        id: PurchaseID,
        description: String,
        total_amount: Money,
        installments_count: u32,
        category_id: CategoryID,
        purchased_at: NaiveDate,
    ) -> Self {
        Self {
            id,
            description,
            total_amount,
            installments_count,
            category_id,
            purchased_at,
            created_at: Utc::now(),
        }
    }

    /// Returns the amount of a single installment (integer division, truncating).
    pub fn installment_amount(&self) -> Money {
        let per_installment = self.total_amount.amount() / self.installments_count as i64;
        Money::new(per_installment, self.total_amount.currency())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::money::Currency;

    #[test]
    fn test_single_installment() {
        let p = Purchase::new(
            PurchaseID::new(),
            "Netflix".into(),
            Money::new(5000, Currency::BRL),
            1,
            CategoryID::new(),
            chrono::NaiveDate::from_ymd_opt(2026, 1, 10).unwrap(),
        );
        assert_eq!(p.installment_amount(), Money::new(5000, Currency::BRL));
    }

    #[test]
    fn test_three_installments() {
        let p = Purchase::new(
            PurchaseID::new(),
            "TV".into(),
            Money::new(9000, Currency::BRL),
            3,
            CategoryID::new(),
            chrono::NaiveDate::from_ymd_opt(2026, 1, 10).unwrap(),
        );
        assert_eq!(p.installment_amount(), Money::new(3000, Currency::BRL));
    }

    #[test]
    fn test_ten_installments() {
        let p = Purchase::new(
            PurchaseID::new(),
            "Notebook".into(),
            Money::new(50000, Currency::BRL),
            10,
            CategoryID::new(),
            chrono::NaiveDate::from_ymd_opt(2026, 1, 10).unwrap(),
        );
        assert_eq!(p.installment_amount(), Money::new(5000, Currency::BRL));
    }

    #[test]
    fn test_integer_division_truncates() {
        let p = Purchase::new(
            PurchaseID::new(),
            "Something".into(),
            Money::new(1000, Currency::BRL),
            3,
            CategoryID::new(),
            chrono::NaiveDate::from_ymd_opt(2026, 1, 10).unwrap(),
        );
        assert_eq!(p.installment_amount(), Money::new(333, Currency::BRL));
    }

    #[test]
    fn test_division_by_one() {
        let p = Purchase::new(
            PurchaseID::new(),
            "Coffee".into(),
            Money::new(1500, Currency::BRL),
            1,
            CategoryID::new(),
            chrono::NaiveDate::from_ymd_opt(2026, 1, 10).unwrap(),
        );
        assert_eq!(p.installment_amount(), Money::new(1500, Currency::BRL));
    }
}
