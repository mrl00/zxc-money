use crate::credit_card::domain::events::InvoiceClosed;
use crate::credit_card::domain::repository::{CreditCardRepository, InvoiceRepository};
use crate::shared::errors::CreditCardError;
use crate::shared::events::EventPublisher;
use crate::shared::ids::{CreditCardID, InvoiceID, UserID};
use std::sync::Arc;

/// Command to close the open invoice for a credit card.
pub struct CloseInvoiceCommand {
    pub owner_id: UserID,
    pub credit_card_id: CreditCardID,
}

/// Handles [`CloseInvoiceCommand`] by closing the open invoice and publishing
/// an [`InvoiceClosed`](crate::credit_card::domain::events::InvoiceClosed) event.
pub struct CloseInvoiceHandler<
    C: CreditCardRepository,
    I: InvoiceRepository,
    P: EventPublisher,
> {
    credit_card_repository: Arc<C>,
    invoice_repository: Arc<I>,
    event_publisher: Arc<P>,
}

impl<C: CreditCardRepository, I: InvoiceRepository, P: EventPublisher>
    CloseInvoiceHandler<C, I, P>
{
    /// Creates a new [`CloseInvoiceHandler`] with the given dependencies.
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

    /// Executes the close-invoice use case.
    ///
    /// Validates ownership, closes the open invoice, persists it, and publishes
    /// an [`InvoiceClosed`](crate::credit_card::domain::events::InvoiceClosed) event.
    pub async fn handle(&self, cmd: CloseInvoiceCommand) -> Result<InvoiceID, CreditCardError> {
        let card = self
            .credit_card_repository
            .find_by_id(cmd.credit_card_id)
            .await?
            .ok_or_else(|| {
                CreditCardError::CreditCardNotFound(cmd.credit_card_id.to_string())
            })?;

        if card.owner_id != cmd.owner_id {
            return Err(CreditCardError::InvariantViolation(
                "credit card does not belong to owner".into(),
            ));
        }

        let mut invoice = self
            .invoice_repository
            .find_open(cmd.credit_card_id)
            .await?
            .ok_or_else(|| {
                CreditCardError::InvariantViolation("no open invoice found".into())
            })?;

        let total = invoice.total();
        invoice.close()?;
        self.invoice_repository.save(&invoice).await?;

        let event = InvoiceClosed {
            invoice_id: invoice.id,
            credit_card_id: cmd.credit_card_id,
            total,
            timestamp: chrono::Utc::now(),
        };
        self.event_publisher.publish(vec![&event]).await?;

        Ok(invoice.id)
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
    async fn test_close_invoice() {
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
        inv_repo.save(&invoice).await.unwrap();

        let handler = CloseInvoiceHandler::new(cc_repo, inv_repo.clone(), publisher);

        let closed_id = handler
            .handle(CloseInvoiceCommand {
                owner_id: owner,
                credit_card_id: card_id,
            })
            .await
            .unwrap();

        assert_eq!(closed_id, inv_id);

        let updated = inv_repo.find_by_id(inv_id).await.unwrap().unwrap();
        assert_eq!(updated.status, InvoiceStatus::Closed);
    }

    #[tokio::test]
    async fn test_close_no_open_invoice() {
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

        let handler = CloseInvoiceHandler::new(cc_repo, inv_repo, publisher);

        let result = handler
            .handle(CloseInvoiceCommand {
                owner_id: owner,
                credit_card_id: card_id,
            })
            .await;

        assert!(matches!(
            result,
            Err(CreditCardError::InvariantViolation(_))
        ));
    }

    #[tokio::test]
    async fn test_close_wrong_owner() {
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

        let handler = CloseInvoiceHandler::new(cc_repo, inv_repo, publisher);

        let result = handler
            .handle(CloseInvoiceCommand {
                owner_id: UserID::new(),
                credit_card_id: card_id,
            })
            .await;

        assert!(matches!(
            result,
            Err(CreditCardError::InvariantViolation(_))
        ));
    }
}
