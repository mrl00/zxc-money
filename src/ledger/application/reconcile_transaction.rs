use crate::ledger::domain::events::TransactionReconciled;
use crate::ledger::domain::repository::TransactionRepository;
use crate::shared::errors::LedgerError;
use crate::shared::events::EventPublisher;
use crate::shared::ids::TransactionID;
use std::sync::Arc;

/// Command to mark a transaction as reconciled or unreconciled.
pub struct ReconcileTransactionCommand {
    pub transaction_id: TransactionID,
    pub reconciled: bool,
}

/// Handler that processes [`ReconcileTransactionCommand`] requests.
pub struct ReconcileTransactionHandler<T: TransactionRepository, P: EventPublisher> {
    transaction_repository: Arc<T>,
    event_publisher: Arc<P>,
}

impl<T: TransactionRepository, P: EventPublisher> ReconcileTransactionHandler<T, P> {
    pub fn new(transaction_repository: Arc<T>, event_publisher: Arc<P>) -> Self {
        Self {
            transaction_repository,
            event_publisher,
        }
    }

    /// Updates the reconciliation status and publishes [`TransactionReconciled`].
    pub async fn handle(&self, cmd: ReconcileTransactionCommand) -> Result<(), LedgerError> {
        let mut transaction = self
            .transaction_repository
            .find_by_id(cmd.transaction_id)
            .await?
            .ok_or_else(|| LedgerError::TransactionNotFound(cmd.transaction_id.to_string()))?;

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
