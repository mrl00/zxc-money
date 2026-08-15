use crate::ledger::domain::events::TransactionDeleted;
use crate::ledger::domain::repository::TransactionRepository;
use crate::shared::errors::LedgerError;
use crate::shared::events::EventPublisher;
use crate::shared::ids::TransactionID;
use std::sync::Arc;

pub struct DeleteTransactionCommand {
    pub transaction_id: TransactionID,
}

pub struct DeleteTransactionHandler<T: TransactionRepository, P: EventPublisher> {
    transaction_repository: Arc<T>,
    event_publisher: Arc<P>,
}

impl<T: TransactionRepository, P: EventPublisher> DeleteTransactionHandler<T, P> {
    pub fn new(transaction_repository: Arc<T>, event_publisher: Arc<P>) -> Self {
        Self {
            transaction_repository,
            event_publisher,
        }
    }

    pub async fn handle(&self, cmd: DeleteTransactionCommand) -> Result<(), LedgerError> {
        let transaction = self
            .transaction_repository
            .find_by_id(cmd.transaction_id)
            .await?
            .ok_or_else(|| LedgerError::TransactionNotFound(cmd.transaction_id.to_string()))?;

        self.transaction_repository
            .delete(cmd.transaction_id)
            .await?;

        let event = TransactionDeleted {
            transaction_id: cmd.transaction_id,
            account_id: transaction.account_id,
            timestamp: chrono::Utc::now(),
        };
        self.event_publisher.publish(vec![&event]).await?;

        Ok(())
    }
}
