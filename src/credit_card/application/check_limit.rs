use crate::credit_card::domain::card::CreditCard;
use crate::credit_card::domain::repository::InvoiceRepository;
use crate::shared::errors::CreditCardError;
use crate::shared::ids::CreditCardID;
use crate::shared::money::Money;
use std::sync::Arc;

/// Summary of a credit card's current limit usage.
#[derive(Debug, Clone)]
pub struct CreditCardSummary {
    pub credit_card_id: CreditCardID,
    pub name: String,
    pub limit: Money,
    pub used: Money,
    pub available: Money,
    pub utilization_pct: f64,
}

/// Alerts emitted when credit card utilization crosses configured thresholds.
#[derive(Debug, Clone, PartialEq)]
pub enum LimitAlert {
    /// Utilization exceeded the threshold (e.g. 80%)
    ThresholdExceeded {
        credit_card_id: CreditCardID,
        name: String,
        utilization_pct: f64,
        threshold_pct: f64,
    },
    /// Utilization is at 100% or beyond
    LimitReached {
        credit_card_id: CreditCardID,
        name: String,
    },
}

/// Service for computing credit card summaries and checking limit alerts.
pub struct CreditCardService<I: InvoiceRepository> {
    invoice_repository: Arc<I>,
}

impl<I: InvoiceRepository> CreditCardService<I> {
    /// Creates a new [`CreditCardService`] with the given invoice repository.
    pub fn new(invoice_repository: Arc<I>) -> Self {
        Self { invoice_repository }
    }

    /// Computes a [`CreditCardSummary`] for the given card using its open invoice total.
    pub async fn summary(&self, card: &CreditCard) -> Result<CreditCardSummary, CreditCardError> {
        let used = match self.invoice_repository.find_open(card.id).await? {
            Some(invoice) => invoice.total(),
            None => Money::zero(card.limit.currency()),
        };

        let available = card.available_limit(used)?;
        let limit_amount = card.limit.amount();
        let utilization_pct = if limit_amount > 0 {
            (used.amount() as f64 / limit_amount as f64) * 100.0
        } else {
            0.0
        };

        Ok(CreditCardSummary {
            credit_card_id: card.id,
            name: card.name.clone(),
            limit: card.limit,
            used,
            available,
            utilization_pct,
        })
    }

    /// Checks whether the card's utilization exceeds `threshold_pct` or has reached 100%.
    pub async fn check_limit_alert(
        &self,
        card: &CreditCard,
        threshold_pct: f64,
    ) -> Result<Option<LimitAlert>, CreditCardError> {
        let summary = self.summary(card).await?;

        if summary.utilization_pct >= 100.0 {
            return Ok(Some(LimitAlert::LimitReached {
                credit_card_id: card.id,
                name: card.name.clone(),
            }));
        }

        if summary.utilization_pct >= threshold_pct {
            return Ok(Some(LimitAlert::ThresholdExceeded {
                credit_card_id: card.id,
                name: card.name.clone(),
                utilization_pct: summary.utilization_pct,
                threshold_pct,
            }));
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::credit_card::domain::invoice::Invoice;
    use crate::credit_card::domain::purchase::Purchase;
    use crate::shared::ids::{CategoryID, CreditCardID, InvoiceID, PurchaseID, UserID};
    use crate::shared::mock::MockInvoiceRepository;
    use crate::shared::money::Currency;
    use crate::shared::period::YearMonth;

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

    #[tokio::test]
    async fn test_summary_no_open_invoice() {
        let inv_repo = Arc::new(MockInvoiceRepository::new());
        let service = CreditCardService::new(inv_repo);

        let card = make_card(1000000);
        let summary = service.summary(&card).await.unwrap();

        assert_eq!(summary.used, Money::new(0, Currency::BRL));
        assert_eq!(summary.available, Money::new(1000000, Currency::BRL));
        assert_eq!(summary.utilization_pct, 0.0);
    }

    #[tokio::test]
    async fn test_summary_with_open_invoice() {
        let inv_repo = Arc::new(MockInvoiceRepository::new());
        let card_id = CreditCardID::new();

        let mut invoice = Invoice::new(InvoiceID::new(), card_id, YearMonth::new(2026, 1));
        invoice
            .add_purchase(Purchase::new(
                PurchaseID::new(),
                "Netflix".into(),
                Money::new(5000, Currency::BRL),
                1,
                CategoryID::new(),
                chrono::NaiveDate::from_ymd_opt(2026, 1, 10).unwrap(),
            ))
            .unwrap();
        invoice
            .add_purchase(Purchase::new(
                PurchaseID::new(),
                "Spotify".into(),
                Money::new(3000, Currency::BRL),
                1,
                CategoryID::new(),
                chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            ))
            .unwrap();
        inv_repo.save(&invoice).await.unwrap();

        let service = CreditCardService::new(inv_repo);

        let card = CreditCard::new(
            card_id,
            UserID::new(),
            "Nubank".into(),
            "Mastercard".into(),
            Money::new(1000000, Currency::BRL),
            20,
            27,
        );
        let summary = service.summary(&card).await.unwrap();

        assert_eq!(summary.used, Money::new(8000, Currency::BRL));
        assert_eq!(summary.available, Money::new(992000, Currency::BRL));
        assert!((summary.utilization_pct - 0.8).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_check_limit_alert_below_threshold() {
        let inv_repo = Arc::new(MockInvoiceRepository::new());
        let card_id = CreditCardID::new();

        let mut invoice = Invoice::new(InvoiceID::new(), card_id, YearMonth::new(2026, 1));
        invoice
            .add_purchase(Purchase::new(
                PurchaseID::new(),
                "Coffee".into(),
                Money::new(5000, Currency::BRL),
                1,
                CategoryID::new(),
                chrono::NaiveDate::from_ymd_opt(2026, 1, 10).unwrap(),
            ))
            .unwrap();
        inv_repo.save(&invoice).await.unwrap();

        let service = CreditCardService::new(inv_repo);

        let card = CreditCard::new(
            card_id,
            UserID::new(),
            "Nubank".into(),
            "Mastercard".into(),
            Money::new(1000000, Currency::BRL),
            20,
            27,
        );
        let alert = service.check_limit_alert(&card, 80.0).await.unwrap();
        assert!(alert.is_none());
    }

    #[tokio::test]
    async fn test_check_limit_alert_exceeded() {
        let inv_repo = Arc::new(MockInvoiceRepository::new());
        let card_id = CreditCardID::new();

        let mut invoice = Invoice::new(InvoiceID::new(), card_id, YearMonth::new(2026, 1));
        invoice
            .add_purchase(Purchase::new(
                PurchaseID::new(),
                "TV".into(),
                Money::new(900000, Currency::BRL),
                1,
                CategoryID::new(),
                chrono::NaiveDate::from_ymd_opt(2026, 1, 10).unwrap(),
            ))
            .unwrap();
        inv_repo.save(&invoice).await.unwrap();

        let service = CreditCardService::new(inv_repo);

        let card = CreditCard::new(
            card_id,
            UserID::new(),
            "Nubank".into(),
            "Mastercard".into(),
            Money::new(1000000, Currency::BRL),
            20,
            27,
        );
        let alert = service.check_limit_alert(&card, 80.0).await.unwrap();
        assert!(matches!(alert, Some(LimitAlert::ThresholdExceeded { .. })));
    }

    #[tokio::test]
    async fn test_check_limit_alert_reached() {
        let inv_repo = Arc::new(MockInvoiceRepository::new());
        let card_id = CreditCardID::new();

        let mut invoice = Invoice::new(InvoiceID::new(), card_id, YearMonth::new(2026, 1));
        invoice
            .add_purchase(Purchase::new(
                PurchaseID::new(),
                "Everything".into(),
                Money::new(1000000, Currency::BRL),
                1,
                CategoryID::new(),
                chrono::NaiveDate::from_ymd_opt(2026, 1, 10).unwrap(),
            ))
            .unwrap();
        inv_repo.save(&invoice).await.unwrap();

        let service = CreditCardService::new(inv_repo);

        let card = CreditCard::new(
            card_id,
            UserID::new(),
            "Nubank".into(),
            "Mastercard".into(),
            Money::new(1000000, Currency::BRL),
            20,
            27,
        );
        let alert = service.check_limit_alert(&card, 80.0).await.unwrap();
        assert!(matches!(alert, Some(LimitAlert::LimitReached { .. })));
    }
}
