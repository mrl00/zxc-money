use std::sync::Arc;

use crate::credit_card::application::check_limit::{
    CreditCardService, CreditCardSummary, LimitAlert,
};
use crate::credit_card::application::close_invoice::{CloseInvoiceCommand, CloseInvoiceHandler};
use crate::credit_card::application::pay_invoice::{PayInvoiceCommand, PayInvoiceHandler};
use crate::credit_card::application::register_card::{RegisterCardCommand, RegisterCardHandler};
use crate::credit_card::application::register_purchase::{
    RegisterPurchaseCommand, RegisterPurchaseHandler,
};
use crate::credit_card::domain::card::CreditCard;
use crate::credit_card::domain::invoice::Invoice;
use crate::credit_card::domain::repository::{CreditCardRepository, InvoiceRepository};
use crate::provider::id::IdGenerator;
use crate::shared::errors::CreditCardError;
use crate::shared::events::EventPublisher;
use crate::shared::ids::{CreditCardID, InvoiceID, PurchaseID};

/// Facade for the CreditCard bounded context.
///
/// Aggregates all command and query handlers behind a single entry point.
pub struct CreditCardFacade<
    C: CreditCardRepository,
    I: InvoiceRepository,
    P: EventPublisher,
    Id: IdGenerator,
> {
    register_card: RegisterCardHandler<C, Id>,
    register_purchase: RegisterPurchaseHandler<C, I, P, Id>,
    close_invoice: CloseInvoiceHandler<C, I, P>,
    pay_invoice: PayInvoiceHandler<C, I, P>,
    check_limit: CreditCardService<I>,
    invoice_repository: Arc<I>,
}

impl<C: CreditCardRepository, I: InvoiceRepository, P: EventPublisher, Id: IdGenerator>
    CreditCardFacade<C, I, P, Id>
{
    /// Creates a new facade with shared dependencies.
    pub fn new(
        credit_card_repository: Arc<C>,
        invoice_repository: Arc<I>,
        event_publisher: Arc<P>,
        id_generator: Arc<Id>,
    ) -> Self {
        Self {
            register_card: RegisterCardHandler::new(
                credit_card_repository.clone(),
                id_generator.clone(),
            ),
            register_purchase: RegisterPurchaseHandler::new(
                credit_card_repository.clone(),
                invoice_repository.clone(),
                event_publisher.clone(),
                id_generator,
            ),
            close_invoice: CloseInvoiceHandler::new(
                credit_card_repository.clone(),
                invoice_repository.clone(),
                event_publisher.clone(),
            ),
            pay_invoice: PayInvoiceHandler::new(
                credit_card_repository,
                invoice_repository.clone(),
                event_publisher,
            ),
            check_limit: CreditCardService::new(invoice_repository.clone()),
            invoice_repository,
        }
    }

    // ── Commands ──────────────────────────────────────────────

    /// Registers a new credit card. See [`RegisterCardHandler`].
    pub async fn register_card(
        &self,
        cmd: RegisterCardCommand,
    ) -> Result<CreditCardID, CreditCardError> {
        self.register_card.handle(cmd).await
    }

    /// Registers a purchase on a credit card. See [`RegisterPurchaseHandler`].
    pub async fn register_purchase(
        &self,
        cmd: RegisterPurchaseCommand,
    ) -> Result<Vec<PurchaseID>, CreditCardError> {
        self.register_purchase.handle(cmd).await
    }

    /// Closes the current invoice for a credit card. See [`CloseInvoiceHandler`].
    pub async fn close_invoice(
        &self,
        cmd: CloseInvoiceCommand,
    ) -> Result<InvoiceID, CreditCardError> {
        self.close_invoice.handle(cmd).await
    }

    /// Pays an invoice. See [`PayInvoiceHandler`].
    pub async fn pay_invoice(&self, cmd: PayInvoiceCommand) -> Result<(), CreditCardError> {
        self.pay_invoice.handle(cmd).await
    }

    // ── Queries ───────────────────────────────────────────────

    /// Computes a credit card summary (limit usage). See [`CreditCardService::summary`].
    pub async fn summary(&self, card: &CreditCard) -> Result<CreditCardSummary, CreditCardError> {
        self.check_limit.summary(card).await
    }

    /// Checks whether utilization exceeds a threshold. See [`CreditCardService::check_limit_alert`].
    pub async fn check_limit_alert(
        &self,
        card: &CreditCard,
        threshold_pct: f64,
    ) -> Result<Option<LimitAlert>, CreditCardError> {
        self.check_limit
            .check_limit_alert(card, threshold_pct)
            .await
    }

    /// Finds the currently open invoice for a credit card.
    pub async fn get_open_invoice(
        &self,
        credit_card_id: CreditCardID,
    ) -> Result<Option<Invoice>, CreditCardError> {
        Ok(self.invoice_repository.find_open(credit_card_id).await?)
    }
}
