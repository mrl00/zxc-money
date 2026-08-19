use std::sync::Arc;

use crate::ledger::application::confirm_recurring::{
    ConfirmRecurringCommand, ConfirmRecurringHandler,
};
use crate::ledger::application::create_recurring::{
    CreateRecurringTransactionCommand, CreateRecurringTransactionHandler,
};
use crate::ledger::application::delete_account::{DeleteAccountCommand, DeleteAccountHandler};
use crate::ledger::application::delete_transaction::{
    DeleteTransactionCommand, DeleteTransactionHandler,
};
use crate::ledger::application::open_account::{OpenAccountCommand, OpenAccountHandler};
use crate::ledger::application::reconcile_transaction::{
    ReconcileTransactionCommand, ReconcileTransactionHandler,
};
use crate::ledger::application::record_transaction::{
    RecordTransactionCommand, RecordTransactionHandler,
};
use crate::ledger::application::transfer_funds::{TransferFundsCommand, TransferFundsHandler};
use crate::ledger::application::update_transaction::{
    UpdateTransactionCommand, UpdateTransactionHandler,
};
use crate::ledger::domain::repository::{
    AccountRepository, RecurringTransactionRepository, TransactionRepository,
};
use crate::provider::id::IdGenerator;
use crate::shared::errors::LedgerError;
use crate::shared::events::EventPublisher;
use crate::shared::ids::{AccountID, RecurringTransactionID, TransactionID};

/// Facade for the Ledger bounded context.
///
/// Aggregates all command handlers behind a single entry point. Query
/// handlers (`generate_pending`) and cross-context handlers
/// (`invoice_paid_handler`) are wired separately via event subscription.
///
/// # Example
///
/// ```ignore
/// let facade = LedgerFacade::new(account_repo, tx_repo, recurring_repo, event_pub, id_gen);
///
/// let account_id = facade.open_account(OpenAccountCommand { ... }).await?;
/// facade.record_transaction(RecordTransactionCommand { ... }).await?;
/// ```
pub struct LedgerFacade<
    A: AccountRepository,
    T: TransactionRepository,
    R: RecurringTransactionRepository,
    P: EventPublisher,
    I: IdGenerator,
> {
    open_account: OpenAccountHandler<A, P, I>,
    delete_account: DeleteAccountHandler<A, T, P>,
    record_transaction: RecordTransactionHandler<A, T, P, I>,
    update_transaction: UpdateTransactionHandler<T, P>,
    delete_transaction: DeleteTransactionHandler<T, P>,
    transfer_funds: TransferFundsHandler<A, T, P, I>,
    reconcile_transaction: ReconcileTransactionHandler<T, P>,
    create_recurring: CreateRecurringTransactionHandler<R, P, I>,
    confirm_recurring: ConfirmRecurringHandler<R, T, P, I>,
}

impl<
    A: AccountRepository,
    T: TransactionRepository,
    R: RecurringTransactionRepository,
    P: EventPublisher,
    I: IdGenerator,
> LedgerFacade<A, T, R, P, I>
{
    /// Creates a new facade with shared dependencies.
    pub fn new(
        account_repository: Arc<A>,
        transaction_repository: Arc<T>,
        recurring_repository: Arc<R>,
        event_publisher: Arc<P>,
        id_generator: Arc<I>,
    ) -> Self {
        Self {
            open_account: OpenAccountHandler::new(
                account_repository.clone(),
                event_publisher.clone(),
                id_generator.clone(),
            ),
            delete_account: DeleteAccountHandler::new(
                account_repository.clone(),
                transaction_repository.clone(),
                event_publisher.clone(),
            ),
            record_transaction: RecordTransactionHandler::new(
                account_repository.clone(),
                transaction_repository.clone(),
                event_publisher.clone(),
                id_generator.clone(),
            ),
            update_transaction: UpdateTransactionHandler::new(
                transaction_repository.clone(),
                event_publisher.clone(),
            ),
            delete_transaction: DeleteTransactionHandler::new(
                transaction_repository.clone(),
                event_publisher.clone(),
            ),
            transfer_funds: TransferFundsHandler::new(
                account_repository,
                transaction_repository.clone(),
                event_publisher.clone(),
                id_generator.clone(),
            ),
            reconcile_transaction: ReconcileTransactionHandler::new(
                transaction_repository.clone(),
                event_publisher.clone(),
            ),
            create_recurring: CreateRecurringTransactionHandler::new(
                recurring_repository.clone(),
                event_publisher.clone(),
                id_generator.clone(),
            ),
            confirm_recurring: ConfirmRecurringHandler::new(
                recurring_repository,
                transaction_repository,
                event_publisher,
                id_generator,
            ),
        }
    }

    /// Opens a new account. See [`OpenAccountHandler`].
    pub async fn open_account(&self, cmd: OpenAccountCommand) -> Result<AccountID, LedgerError> {
        self.open_account.handle(cmd).await
    }

    /// Deletes an account. See [`DeleteAccountHandler`].
    pub async fn delete_account(&self, cmd: DeleteAccountCommand) -> Result<(), LedgerError> {
        self.delete_account.handle(cmd).await
    }

    /// Records a transaction. See [`RecordTransactionHandler`].
    pub async fn record_transaction(
        &self,
        cmd: RecordTransactionCommand,
    ) -> Result<TransactionID, LedgerError> {
        self.record_transaction.handle(cmd).await
    }

    /// Updates a transaction. See [`UpdateTransactionHandler`].
    pub async fn update_transaction(
        &self,
        cmd: UpdateTransactionCommand,
    ) -> Result<(), LedgerError> {
        self.update_transaction.handle(cmd).await
    }

    /// Deletes a transaction. See [`DeleteTransactionHandler`].
    pub async fn delete_transaction(
        &self,
        cmd: DeleteTransactionCommand,
    ) -> Result<(), LedgerError> {
        self.delete_transaction.handle(cmd).await
    }

    /// Transfers funds between accounts. See [`TransferFundsHandler`].
    pub async fn transfer_funds(&self, cmd: TransferFundsCommand) -> Result<(), LedgerError> {
        self.transfer_funds.handle(cmd).await
    }

    /// Reconciles a transaction. See [`ReconcileTransactionHandler`].
    pub async fn reconcile_transaction(
        &self,
        cmd: ReconcileTransactionCommand,
    ) -> Result<(), LedgerError> {
        self.reconcile_transaction.handle(cmd).await
    }

    /// Creates a recurring transaction. See [`CreateRecurringTransactionHandler`].
    pub async fn create_recurring(
        &self,
        cmd: CreateRecurringTransactionCommand,
    ) -> Result<RecurringTransactionID, LedgerError> {
        self.create_recurring.handle(cmd).await
    }

    /// Confirms a recurring transaction. See [`ConfirmRecurringHandler`].
    pub async fn confirm_recurring(
        &self,
        cmd: ConfirmRecurringCommand,
    ) -> Result<TransactionID, LedgerError> {
        self.confirm_recurring.handle(cmd).await
    }
}
