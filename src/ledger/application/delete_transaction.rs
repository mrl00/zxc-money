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

        if transaction.reconciled {
            return Err(LedgerError::InvariantViolation(
                "cannot delete reconciled transaction".into(),
            ));
        }

        if transaction.source_purchase_id.is_some() {
            return Err(LedgerError::InvariantViolation(
                "cannot delete transaction derived from credit card purchase".into(),
            ));
        }

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::domain::transaction::{Transaction, TransactionType};
    use crate::shared::events::InMemoryEventDispatcher;
    use crate::shared::ids::{AccountID, PurchaseID};
    use crate::shared::mock::MockTransactionRepository;
    use crate::shared::money::{Currency, Money};

    fn make_tx(id: TransactionID) -> Transaction {
        Transaction::new(
            id,
            AccountID::new(),
            TransactionType::Income,
            Money::new(100, Currency::BRL),
            "Test".into(),
            chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
        )
        .unwrap()
        .with_category(crate::shared::ids::CategoryID::new())
        .unwrap()
    }

    #[tokio::test]
    async fn test_delete_normal_transaction() {
        let repo = Arc::new(MockTransactionRepository::new());
        let publisher = Arc::new(InMemoryEventDispatcher::new());
        let tx_id = TransactionID::new();
        repo.save(&make_tx(tx_id)).await.unwrap();

        let handler = DeleteTransactionHandler::new(repo, publisher);
        let result = handler
            .handle(DeleteTransactionCommand {
                transaction_id: tx_id,
            })
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_reconciled_transaction_blocked() {
        let repo = Arc::new(MockTransactionRepository::new());
        let publisher = Arc::new(InMemoryEventDispatcher::new());
        let tx_id = TransactionID::new();
        let mut tx = make_tx(tx_id);
        tx.mark_reconciled();
        repo.save(&tx).await.unwrap();

        let handler = DeleteTransactionHandler::new(repo, publisher);
        let result = handler
            .handle(DeleteTransactionCommand {
                transaction_id: tx_id,
            })
            .await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            LedgerError::InvariantViolation(_)
        ));
    }

    #[tokio::test]
    async fn test_delete_derived_transaction_blocked() {
        let repo = Arc::new(MockTransactionRepository::new());
        let publisher = Arc::new(InMemoryEventDispatcher::new());
        let tx_id = TransactionID::new();
        let tx = make_tx(tx_id).with_source_purchase(PurchaseID::new());
        repo.save(&tx).await.unwrap();

        let handler = DeleteTransactionHandler::new(repo, publisher);
        let result = handler
            .handle(DeleteTransactionCommand {
                transaction_id: tx_id,
            })
            .await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            LedgerError::InvariantViolation(_)
        ));
    }
}
