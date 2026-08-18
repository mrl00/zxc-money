use crate::credit_card::domain::events::InvoicePaid;
use crate::credit_card::domain::repository::{CreditCardRepository, InvoiceRepository};
use crate::shared::errors::CreditCardError;
use crate::shared::events::EventPublisher;
use crate::shared::ids::{CreditCardID, InvoiceID, UserID};
use std::sync::Arc;

/// Command to pay a closed invoice on a credit card.
pub struct PayInvoiceCommand {
    pub owner_id: UserID,
    pub credit_card_id: CreditCardID,
    pub invoice_id: InvoiceID,
}

/// Handles [`PayInvoiceCommand`] by marking the invoice as paid and publishing
/// an [`InvoicePaid`](crate::credit_card::domain::events::InvoicePaid) event.
pub struct PayInvoiceHandler<C: CreditCardRepository, I: InvoiceRepository, P: EventPublisher> {
    credit_card_repository: Arc<C>,
    invoice_repository: Arc<I>,
    event_publisher: Arc<P>,
}

impl<C: CreditCardRepository, I: InvoiceRepository, P: EventPublisher> PayInvoiceHandler<C, I, P> {
    /// Creates a new [`PayInvoiceHandler`] with the given dependencies.
    pub fn new(
        credit_card_repository: Arc<C>,
        invoice_repository: Arc<I>,
        event_publisher: Arc<P>,
    ) -> Self {
        Self {
            credit_card_repository,
            invoice_repository,
            event_publisher,
        }
    }

    /// Executes the pay-invoice use case.
    ///
    /// Validates ownership, verifies the invoice belongs to the credit card,
    /// marks it as paid, persists it, and publishes an
    /// [`InvoicePaid`](crate::credit_card::domain::events::InvoicePaid) event.
    pub async fn handle(&self, cmd: PayInvoiceCommand) -> Result<(), CreditCardError> {
        let card = self
            .credit_card_repository
            .find_by_id(cmd.credit_card_id)
            .await?
            .ok_or_else(|| CreditCardError::CreditCardNotFound(cmd.credit_card_id.to_string()))?;

        if card.owner_id != cmd.owner_id {
            return Err(CreditCardError::InvariantViolation(
                "credit card does not belong to owner".into(),
            ));
        }

        let mut invoice = self
            .invoice_repository
            .find_by_id(cmd.invoice_id)
            .await?
            .ok_or_else(|| CreditCardError::InvoiceNotFound(cmd.invoice_id.to_string()))?;

        if invoice.credit_card_id != cmd.credit_card_id {
            return Err(CreditCardError::InvariantViolation(
                "invoice does not belong to this credit card".into(),
            ));
        }

        let total = invoice.total();
        invoice.pay()?;
        self.invoice_repository.save(&invoice).await?;

        let event = InvoicePaid {
            invoice_id: cmd.invoice_id,
            credit_card_id: cmd.credit_card_id,
            total,
            timestamp: chrono::Utc::now(),
        };
        self.event_publisher.publish(vec![&event]).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::credit_card::domain::card::CreditCard;
    use crate::credit_card::domain::invoice::{Invoice, InvoiceStatus};
    use crate::credit_card::domain::purchase::Purchase;
    use crate::shared::events::InMemoryEventDispatcher;
    use crate::shared::ids::{CategoryID, PurchaseID};
    use crate::shared::mock::{MockCreditCardRepository, MockInvoiceRepository};
    use crate::shared::money::{Currency, Money};
    use crate::shared::period::YearMonth;

    fn setup() -> (
        Arc<MockCreditCardRepository>,
        Arc<MockInvoiceRepository>,
        Arc<InMemoryEventDispatcher>,
    ) {
        let cc_repo = Arc::new(MockCreditCardRepository::new());
        let inv_repo = Arc::new(MockInvoiceRepository::new());
        let publisher = Arc::new(InMemoryEventDispatcher::new());
        (cc_repo, inv_repo, publisher)
    }

    #[tokio::test]
    async fn test_pay_closed_invoice() {
        let (cc_repo, inv_repo, publisher) = setup();
        let owner = UserID::new();
        let card_id = CreditCardID::new();

        let card = CreditCard::new(
            card_id,
            owner,
            "Nubank".into(),
            "Mastercard".into(),
            Money::new(500000, Currency::BRL),
            20,
            27,
        );
        cc_repo.save(&card).await.unwrap();

        let inv_id = InvoiceID::new();
        let mut invoice = Invoice::new(inv_id, card_id, YearMonth::new(2026, 1));
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
        invoice.close().unwrap();
        inv_repo.save(&invoice).await.unwrap();

        let handler = PayInvoiceHandler::new(cc_repo, inv_repo.clone(), publisher);

        handler
            .handle(PayInvoiceCommand {
                owner_id: owner,
                credit_card_id: card_id,
                invoice_id: inv_id,
            })
            .await
            .unwrap();

        let updated = inv_repo.find_by_id(inv_id).await.unwrap().unwrap();
        assert_eq!(updated.status, InvoiceStatus::Paid);
    }

    #[tokio::test]
    async fn test_pay_open_invoice_fails() {
        let (cc_repo, inv_repo, publisher) = setup();
        let owner = UserID::new();
        let card_id = CreditCardID::new();

        let card = CreditCard::new(
            card_id,
            owner,
            "Nubank".into(),
            "Mastercard".into(),
            Money::new(500000, Currency::BRL),
            20,
            27,
        );
        cc_repo.save(&card).await.unwrap();

        let inv_id = InvoiceID::new();
        let invoice = Invoice::new(inv_id, card_id, YearMonth::new(2026, 1));
        inv_repo.save(&invoice).await.unwrap();

        let handler = PayInvoiceHandler::new(cc_repo, inv_repo, publisher);

        let result = handler
            .handle(PayInvoiceCommand {
                owner_id: owner,
                credit_card_id: card_id,
                invoice_id: inv_id,
            })
            .await;

        assert!(matches!(
            result,
            Err(CreditCardError::InvariantViolation(_))
        ));
    }

    #[tokio::test]
    async fn test_pay_wrong_invoice() {
        let (cc_repo, inv_repo, publisher) = setup();
        let owner = UserID::new();
        let card_id = CreditCardID::new();
        let other_card_id = CreditCardID::new();

        let card = CreditCard::new(
            card_id,
            owner,
            "Nubank".into(),
            "Mastercard".into(),
            Money::new(500000, Currency::BRL),
            20,
            27,
        );
        cc_repo.save(&card).await.unwrap();

        let inv_id = InvoiceID::new();
        let mut invoice = Invoice::new(inv_id, other_card_id, YearMonth::new(2026, 1));
        invoice.close().unwrap();
        inv_repo.save(&invoice).await.unwrap();

        let handler = PayInvoiceHandler::new(cc_repo, inv_repo, publisher);

        let result = handler
            .handle(PayInvoiceCommand {
                owner_id: owner,
                credit_card_id: card_id,
                invoice_id: inv_id,
            })
            .await;

        assert!(matches!(
            result,
            Err(CreditCardError::InvariantViolation(_))
        ));
    }

    #[tokio::test]
    async fn test_pay_invoice_not_found() {
        let (cc_repo, inv_repo, publisher) = setup();
        let owner = UserID::new();
        let card_id = CreditCardID::new();

        let card = CreditCard::new(
            card_id,
            owner,
            "Nubank".into(),
            "Mastercard".into(),
            Money::new(500000, Currency::BRL),
            20,
            27,
        );
        cc_repo.save(&card).await.unwrap();

        let handler = PayInvoiceHandler::new(cc_repo, inv_repo, publisher);

        let result = handler
            .handle(PayInvoiceCommand {
                owner_id: owner,
                credit_card_id: card_id,
                invoice_id: InvoiceID::new(),
            })
            .await;

        assert!(matches!(result, Err(CreditCardError::InvoiceNotFound(_))));
    }
}
