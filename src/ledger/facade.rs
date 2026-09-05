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
use crate::ledger::application::update_account::{UpdateAccountCommand, UpdateAccountHandler};
use crate::ledger::application::update_recurring::{
    CancelRecurringCommand, PauseRecurringCommand, ResumeRecurringCommand, UpdateRecurringHandler,
};
use crate::ledger::application::update_transaction::{
    UpdateTransactionCommand, UpdateTransactionHandler,
};
use crate::ledger::domain::recurring_transaction::RecurringTransaction;
use crate::ledger::domain::repository::{
    AccountRepository, RecurringTransactionRepository, TransactionRepository,
};
use crate::provider::id::IdGenerator;
use crate::shared::errors::LedgerError;
use crate::shared::events::EventPublisher;
use crate::shared::ids::{AccountID, RecurringTransactionID, TransactionID, UserID};
use crate::shared::period::Period;
use crate::shared::repository::IdempotencyRepository;

use crate::ledger::domain::account::Account;
use crate::ledger::domain::transaction::Transaction;

/// Facade for the Ledger bounded context.
///
/// Aggregates all command and query handlers behind a single entry point.
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
    ID: IdempotencyRepository,
> {
    open_account: OpenAccountHandler<A, P, I>,
    update_account: UpdateAccountHandler<A, P>,
    delete_account: DeleteAccountHandler<A, T, P>,
    record_transaction: RecordTransactionHandler<A, T, P, I, ID>,
    update_transaction: UpdateTransactionHandler<T, P>,
    delete_transaction: DeleteTransactionHandler<T, P>,
    transfer_funds: TransferFundsHandler<A, T, P, I, ID>,
    reconcile_transaction: ReconcileTransactionHandler<T, P>,
    create_recurring: CreateRecurringTransactionHandler<R, P, I>,
    update_recurring: UpdateRecurringHandler<R, P>,
    confirm_recurring: ConfirmRecurringHandler<R, T, P, I>,
    account_repository: Arc<A>,
    transaction_repository: Arc<T>,
    recurring_repository: Arc<R>,
}

impl<
    A: AccountRepository,
    T: TransactionRepository,
    R: RecurringTransactionRepository,
    P: EventPublisher,
    I: IdGenerator,
    ID: IdempotencyRepository,
> LedgerFacade<A, T, R, P, I, ID>
{
    /// Creates a new facade with shared dependencies.
    pub fn new(
        account_repository: Arc<A>,
        transaction_repository: Arc<T>,
        recurring_repository: Arc<R>,
        event_publisher: Arc<P>,
        id_generator: Arc<I>,
        idempotency_repository: Arc<ID>,
    ) -> Self {
        Self {
            open_account: OpenAccountHandler::new(
                account_repository.clone(),
                event_publisher.clone(),
                id_generator.clone(),
            ),
            update_account: UpdateAccountHandler::new(
                account_repository.clone(),
                event_publisher.clone(),
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
                idempotency_repository.clone(),
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
                account_repository.clone(),
                transaction_repository.clone(),
                event_publisher.clone(),
                id_generator.clone(),
                idempotency_repository,
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
            update_recurring: UpdateRecurringHandler::new(
                recurring_repository.clone(),
                event_publisher.clone(),
            ),
            confirm_recurring: ConfirmRecurringHandler::new(
                recurring_repository.clone(),
                transaction_repository.clone(),
                event_publisher,
                id_generator,
            ),
            account_repository,
            transaction_repository,
            recurring_repository,
        }
    }

    // ── Commands ──────────────────────────────────────────────

    /// Opens a new account. See [`OpenAccountHandler`].
    pub async fn open_account(&self, cmd: OpenAccountCommand) -> Result<AccountID, LedgerError> {
        self.open_account.handle(cmd).await
    }

    /// Updates an account's name and/or type. See [`UpdateAccountHandler`].
    pub async fn update_account(&self, cmd: UpdateAccountCommand) -> Result<(), LedgerError> {
        self.update_account.handle(cmd).await
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

    /// Pauses a recurring transaction. See [`UpdateRecurringHandler::pause`].
    pub async fn pause_recurring(&self, cmd: PauseRecurringCommand) -> Result<(), LedgerError> {
        self.update_recurring.pause(cmd).await
    }

    /// Resumes a paused recurring transaction. See [`UpdateRecurringHandler::resume`].
    pub async fn resume_recurring(&self, cmd: ResumeRecurringCommand) -> Result<(), LedgerError> {
        self.update_recurring.resume(cmd).await
    }

    /// Cancels a recurring transaction. See [`UpdateRecurringHandler::cancel`].
    pub async fn cancel_recurring(&self, cmd: CancelRecurringCommand) -> Result<(), LedgerError> {
        self.update_recurring.cancel(cmd).await
    }

    /// Confirms a recurring transaction. See [`ConfirmRecurringHandler`].
    pub async fn confirm_recurring(
        &self,
        cmd: ConfirmRecurringCommand,
    ) -> Result<TransactionID, LedgerError> {
        self.confirm_recurring.handle(cmd).await
    }

    // ── Queries ───────────────────────────────────────────────

    /// Lists all accounts for a given owner.
    pub async fn list_accounts(&self, owner_id: UserID) -> Result<Vec<Account>, LedgerError> {
        Ok(self.account_repository.find_by_owner(owner_id).await?)
    }

    /// Lists transactions for an account within a period.
    pub async fn list_transactions(
        &self,
        account_id: AccountID,
        period: Period,
    ) -> Result<Vec<Transaction>, LedgerError> {
        Ok(self
            .transaction_repository
            .find_by_account(account_id, period)
            .await?)
    }

    /// Lists transactions for an account within a period, with filters.
    pub async fn filter_transactions(
        &self,
        account_id: AccountID,
        period: Period,
        filter: &crate::ledger::domain::repository::TransactionFilter,
    ) -> Result<Vec<Transaction>, LedgerError> {
        Ok(self
            .transaction_repository
            .find_by_account_filtered(account_id, period, filter)
            .await?)
    }

    /// Generates pending recurring transactions due on or before `today`.
    pub async fn generate_pending(
        &self,
        today: chrono::NaiveDate,
    ) -> Result<Vec<RecurringTransaction>, LedgerError> {
        Ok(self.recurring_repository.find_due(today).await?)
    }
}
