use crate::ledger::domain::events::TransactionReconciled;
use crate::ledger::domain::repository::{AccountRepository, TransactionRepository};
use crate::shared::errors::LedgerError;
use crate::shared::events::EventPublisher;
use crate::shared::ids::{Principal, TransactionID};
use std::sync::Arc;

/// Command to mark a transaction as reconciled or unreconciled.
pub struct ReconcileTransactionCommand {
    pub principal: Principal,
    pub transaction_id: TransactionID,
    pub reconciled: bool,
}

/// Handler that processes [`ReconcileTransactionCommand`] requests.
pub struct ReconcileTransactionHandler<
    A: AccountRepository,
    T: TransactionRepository,
    P: EventPublisher,
> {
    account_repository: Arc<A>,
    transaction_repository: Arc<T>,
    event_publisher: Arc<P>,
}

impl<A: AccountRepository, T: TransactionRepository, P: EventPublisher>
    ReconcileTransactionHandler<A, T, P>
{
    pub fn new(
        account_repository: Arc<A>,
        transaction_repository: Arc<T>,
        event_publisher: Arc<P>,
    ) -> Self {
        Self {
            account_repository,
            transaction_repository,
            event_publisher,
        }
    }

    /// Updates the reconciliation status and publishes [`TransactionReconciled`].
    ///
    /// # Errors
    /// Fails if the transaction does not belong to the authenticated user.
    pub async fn handle(&self, cmd: ReconcileTransactionCommand) -> Result<(), LedgerError> {
        let mut transaction = self
            .transaction_repository
            .find_by_id(cmd.transaction_id)
            .await?
            .ok_or_else(|| LedgerError::TransactionNotFound(cmd.transaction_id.to_string()))?;

        let account = self
            .account_repository
            .find_by_id(transaction.account_id)
            .await?
            .ok_or_else(|| LedgerError::AccountNotFound(transaction.account_id.to_string()))?;

        if account.owner_id != cmd.principal.user_id {
            return Err(LedgerError::Forbidden(
                "not the owner of this account".into(),
            ));
        }

        if cmd.reconciled {
            transaction.mark_reconciled();
        }

        self.transaction_repository.save(&transaction).await?;

        let event = TransactionReconciled {
            transaction_id: cmd.transaction_id,
            account_id: transaction.account_id,
            reconciled: cmd.reconciled,
            timestamp: chrono::Utc::now(),
        };
        self.event_publisher.publish(vec![&event]).await?;

        Ok(())
    }
}
