use chrono::NaiveDate;

use crate::credit_card::domain::events::PurchaseAdded;
use crate::credit_card::domain::invoice::Invoice;
use crate::credit_card::domain::purchase::Purchase;
use crate::credit_card::domain::repository::{CreditCardRepository, InvoiceRepository};
use crate::provider::id::IdGenerator;
use crate::shared::errors::CreditCardError;
use crate::shared::events::EventPublisher;
use crate::shared::ids::{CategoryID, CreditCardID, InvoiceID, PurchaseID, UserID};
use crate::shared::money::Money;
use crate::shared::period::YearMonth;
use std::sync::Arc;

/// Command to register a new purchase on a credit card.
pub struct RegisterPurchaseCommand {
    pub owner_id: UserID,
    pub credit_card_id: CreditCardID,
    pub description: String,
    pub total_amount: Money,
    pub installments_count: u32,
    pub category_id: CategoryID,
    pub purchased_at: NaiveDate,
}

/// Handles [`RegisterPurchaseCommand`] by creating or appending to an open invoice
/// and publishing a [`PurchaseAdded`](crate::credit_card::domain::events::PurchaseAdded) event.
pub struct RegisterPurchaseHandler<
    C: CreditCardRepository,
    I: InvoiceRepository,
    P: EventPublisher,
    Id: IdGenerator,
> {
    credit_card_repository: Arc<C>,
    invoice_repository: Arc<I>,
    event_publisher: Arc<P>,
    id_generator: Arc<Id>,
}

impl<C: CreditCardRepository, I: InvoiceRepository, P: EventPublisher, Id: IdGenerator>
    RegisterPurchaseHandler<C, I, P, Id>
{
    /// Creates a new [`RegisterPurchaseHandler`] with the given dependencies.
    pub fn new(
        credit_card_repository: Arc<C>,
        invoice_repository: Arc<I>,
        event_publisher: Arc<P>,
        id_generator: Arc<Id>,
    ) -> Self {
        Self {
            credit_card_repository,
            invoice_repository,
            event_publisher,
            id_generator,
        }
    }

    /// Executes the register-purchase use case.
    ///
    /// Validates ownership, finds or creates the open invoice, adds the purchase,
    /// and publishes a [`PurchaseAdded`](crate::credit_card::domain::events::PurchaseAdded) event.
    pub async fn handle(
        &self,
        cmd: RegisterPurchaseCommand,
    ) -> Result<PurchaseID, CreditCardError> {
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

        let purchase_id = PurchaseID::from_uuid(self.id_generator.new_id());
        let purchase = Purchase::new(
            purchase_id,
            cmd.description,
            cmd.total_amount,
            cmd.installments_count,
            cmd.category_id,
            cmd.purchased_at,
        );

        let reference_month = YearMonth::from_date(cmd.purchased_at);

        let mut invoice = match self
            .invoice_repository
            .find_open(cmd.credit_card_id)
            .await?
        {
            Some(inv) => inv,
            None => {
                let new_id = InvoiceID::from_uuid(self.id_generator.new_id());
                Invoice::new(new_id, cmd.credit_card_id, reference_month)
            }
        };

        invoice.add_purchase(purchase)?;
        self.invoice_repository.save(&invoice).await?;

        let event = PurchaseAdded {
            purchase_id,
            invoice_id: invoice.id,
            credit_card_id: cmd.credit_card_id,
            amount: cmd.total_amount,
            timestamp: chrono::Utc::now(),
        };
        self.event_publisher.publish(vec![&event]).await?;

        Ok(purchase_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::id::MockIdGenerator;
    use crate::shared::events::InMemoryEventDispatcher;
    use crate::shared::mock::{MockCreditCardRepository, MockInvoiceRepository};
    use crate::shared::money::Currency;

    fn setup() -> (
        Arc<MockCreditCardRepository>,
        Arc<MockInvoiceRepository>,
        Arc<InMemoryEventDispatcher>,
        Arc<MockIdGenerator>,
    ) {
        let cc_repo = Arc::new(MockCreditCardRepository::new());
        let inv_repo = Arc::new(MockInvoiceRepository::new());
        let publisher = Arc::new(InMemoryEventDispatcher::new());
        let id_gen = Arc::new(MockIdGenerator::new(uuid::Uuid::new_v4()));
        (cc_repo, inv_repo, publisher, id_gen)
    }

    #[tokio::test]
    async fn test_register_purchase_creates_invoice() {
        let (cc_repo, inv_repo, publisher, id_gen) = setup();
        let owner = UserID::new();
        let card_id = CreditCardID::new();

        let card = crate::credit_card::domain::card::CreditCard::new(
            card_id,
            owner,
            "Nubank".into(),
            "Mastercard".into(),
            Money::new(500000, Currency::BRL),
            20,
            27,
        );
        cc_repo.save(&card).await.unwrap();

        let handler = RegisterPurchaseHandler::new(cc_repo, inv_repo.clone(), publisher, id_gen);

        let purchase_id = handler
            .handle(RegisterPurchaseCommand {
                owner_id: owner,
                credit_card_id: card_id,
                description: "Netflix".into(),
                total_amount: Money::new(5000, Currency::BRL),
                installments_count: 1,
                category_id: CategoryID::new(),
                purchased_at: chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            })
            .await
            .unwrap();

        assert!(!purchase_id.as_uuid().is_nil());

        let invoice = inv_repo.find_open(card_id).await.unwrap().unwrap();
        assert_eq!(invoice.purchases.len(), 1);
    }

    #[tokio::test]
    async fn test_register_purchase_adds_to_existing_invoice() {
        let (cc_repo, inv_repo, publisher, id_gen) = setup();
        let owner = UserID::new();
        let card_id = CreditCardID::new();

        let card = crate::credit_card::domain::card::CreditCard::new(
            card_id,
            owner,
            "Nubank".into(),
            "Mastercard".into(),
            Money::new(500000, Currency::BRL),
            20,
            27,
        );
        cc_repo.save(&card).await.unwrap();

        let handler = RegisterPurchaseHandler::new(cc_repo, inv_repo.clone(), publisher, id_gen);

        handler
            .handle(RegisterPurchaseCommand {
                owner_id: owner,
                credit_card_id: card_id,
                description: "Netflix".into(),
                total_amount: Money::new(5000, Currency::BRL),
                installments_count: 1,
                category_id: CategoryID::new(),
                purchased_at: chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            })
            .await
            .unwrap();

        handler
            .handle(RegisterPurchaseCommand {
                owner_id: owner,
                credit_card_id: card_id,
                description: "Spotify".into(),
                total_amount: Money::new(3000, Currency::BRL),
                installments_count: 1,
                category_id: CategoryID::new(),
                purchased_at: chrono::NaiveDate::from_ymd_opt(2026, 1, 20).unwrap(),
            })
            .await
            .unwrap();

        let invoice = inv_repo.find_open(card_id).await.unwrap().unwrap();
        assert_eq!(invoice.purchases.len(), 2);
    }

    #[tokio::test]
    async fn test_register_purchase_wrong_owner() {
        let (cc_repo, inv_repo, publisher, id_gen) = setup();
        let owner = UserID::new();
        let card_id = CreditCardID::new();

        let card = crate::credit_card::domain::card::CreditCard::new(
            card_id,
            owner,
            "Nubank".into(),
            "Mastercard".into(),
            Money::new(500000, Currency::BRL),
            20,
            27,
        );
        cc_repo.save(&card).await.unwrap();

        let handler = RegisterPurchaseHandler::new(cc_repo, inv_repo, publisher, id_gen);

        let result = handler
            .handle(RegisterPurchaseCommand {
                owner_id: UserID::new(),
                credit_card_id: card_id,
                description: "Hack".into(),
                total_amount: Money::new(5000, Currency::BRL),
                installments_count: 1,
                category_id: CategoryID::new(),
                purchased_at: chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            })
            .await;

        assert!(matches!(
            result,
            Err(CreditCardError::InvariantViolation(_))
        ));
    }

    #[tokio::test]
    async fn test_register_purchase_card_not_found() {
        let (cc_repo, inv_repo, publisher, id_gen) = setup();

        let handler = RegisterPurchaseHandler::new(cc_repo, inv_repo, publisher, id_gen);

        let result = handler
            .handle(RegisterPurchaseCommand {
                owner_id: UserID::new(),
                credit_card_id: CreditCardID::new(),
                description: "Netflix".into(),
                total_amount: Money::new(5000, Currency::BRL),
                installments_count: 1,
                category_id: CategoryID::new(),
                purchased_at: chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            })
            .await;

        assert!(matches!(
            result,
            Err(CreditCardError::CreditCardNotFound(_))
        ));
    }
}
