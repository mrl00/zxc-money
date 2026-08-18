use chrono::NaiveDate;

use crate::ledger::domain::events::TransactionUpdated;
use crate::ledger::domain::repository::TransactionRepository;
use crate::ledger::domain::transaction::TransactionType;
use crate::shared::errors::LedgerError;
use crate::shared::events::EventPublisher;
use crate::shared::ids::{CategoryID, TransactionID};
use crate::shared::money::Money;
use std::sync::Arc;

/// Command to update fields of an existing transaction.
pub struct UpdateTransactionCommand {
    pub transaction_id: TransactionID,
    pub amount: Option<Money>,
    pub description: Option<String>,
    pub date: Option<NaiveDate>,
    pub category_id: Option<Option<CategoryID>>,
}

/// Handler that processes [`UpdateTransactionCommand`] requests.
pub struct UpdateTransactionHandler<T: TransactionRepository, P: EventPublisher> {
    transaction_repository: Arc<T>,
    event_publisher: Arc<P>,
}

impl<T: TransactionRepository, P: EventPublisher> UpdateTransactionHandler<T, P> {
    pub fn new(transaction_repository: Arc<T>, event_publisher: Arc<P>) -> Self {
        Self {
            transaction_repository,
            event_publisher,
        }
    }

    /// Updates the transaction fields, validates invariants, persists, and publishes
    /// [`TransactionUpdated`].
    ///
    /// # Errors
    /// Fails if the transaction is reconciled, derived from a purchase, or any
    /// invariant is violated.
    pub async fn handle(&self, cmd: UpdateTransactionCommand) -> Result<(), LedgerError> {
        let mut transaction = self
            .transaction_repository
            .find_by_id(cmd.transaction_id)
            .await?
            .ok_or_else(|| LedgerError::TransactionNotFound(cmd.transaction_id.to_string()))?;

        if transaction.reconciled {
            return Err(LedgerError::InvariantViolation(
                "cannot edit reconciled transaction".into(),
            ));
        }

        if transaction.source_purchase_id.is_some() {
            return Err(LedgerError::InvariantViolation(
                "cannot edit transaction derived from credit card purchase".into(),
            ));
        }

        if let Some(amount) = cmd.amount {
            if !amount.is_positive() {
                return Err(LedgerError::InvalidAmount(
                    "transaction amount must be positive".into(),
                ));
            }
            transaction.amount = amount;
        }

        if let Some(description) = cmd.description {
            if description.is_empty() {
                return Err(LedgerError::InvariantViolation(
                    "transaction description must not be empty".into(),
                ));
            }
            transaction.description = description;
        }

        if let Some(date) = cmd.date {
            transaction.date = date;
        }

        if let Some(category_result) = cmd.category_id {
            match category_result {
                Some(category_id) => {
                    transaction = transaction.with_category(category_id)?;
                }
                None => {
                    if transaction.tx_type == TransactionType::Transfer {
                        return Err(LedgerError::InvariantViolation(
                            "transfer must have counterpart account".into(),
                        ));
                    }
                    transaction.category_id = None;
                }
            }
        }

        transaction.validate()?;

        self.transaction_repository.save(&transaction).await?;

        let event = TransactionUpdated {
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
    use crate::ledger::domain::transaction::Transaction;
    use crate::shared::events::InMemoryEventDispatcher;
    use crate::shared::ids::{AccountID, PurchaseID};
    use crate::shared::mock::MockTransactionRepository;
    use crate::shared::money::Currency;

    fn make_tx(id: TransactionID) -> Transaction {
        Transaction::new(
            id,
            AccountID::new(),
            TransactionType::Income,
            Money::new(100, Currency::BRL),
            "Salary".into(),
            chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
        )
        .unwrap()
        .with_category(CategoryID::new())
        .unwrap()
    }

    #[tokio::test]
    async fn test_update_description() {
        let repo = Arc::new(MockTransactionRepository::new());
        let publisher = Arc::new(InMemoryEventDispatcher::new());
        let tx_id = TransactionID::new();
        repo.save(&make_tx(tx_id)).await.unwrap();

        let handler = UpdateTransactionHandler::new(repo, publisher);
        let cmd = UpdateTransactionCommand {
            transaction_id: tx_id,
            amount: None,
            description: Some("Updated salary".into()),
            date: None,
            category_id: None,
        };
        assert!(handler.handle(cmd).await.is_ok());
    }

    #[tokio::test]
    async fn test_update_reconciled_blocked() {
        let repo = Arc::new(MockTransactionRepository::new());
        let publisher = Arc::new(InMemoryEventDispatcher::new());
        let tx_id = TransactionID::new();
        let mut tx = make_tx(tx_id);
        tx.mark_reconciled();
        repo.save(&tx).await.unwrap();

        let handler = UpdateTransactionHandler::new(repo, publisher);
        let cmd = UpdateTransactionCommand {
            transaction_id: tx_id,
            amount: Some(Money::new(200, Currency::BRL)),
            description: None,
            date: None,
            category_id: None,
        };
        let result = handler.handle(cmd).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            LedgerError::InvariantViolation(_)
        ));
    }

    #[tokio::test]
    async fn test_update_derived_blocked() {
        let repo = Arc::new(MockTransactionRepository::new());
        let publisher = Arc::new(InMemoryEventDispatcher::new());
        let tx_id = TransactionID::new();
        let tx = make_tx(tx_id).with_source_purchase(PurchaseID::new());
        repo.save(&tx).await.unwrap();

        let handler = UpdateTransactionHandler::new(repo, publisher);
        let cmd = UpdateTransactionCommand {
            transaction_id: tx_id,
            amount: None,
            description: Some("Hack".into()),
            date: None,
            category_id: None,
        };
        let result = handler.handle(cmd).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update_empty_description_rejected() {
        let repo = Arc::new(MockTransactionRepository::new());
        let publisher = Arc::new(InMemoryEventDispatcher::new());
        let tx_id = TransactionID::new();
        repo.save(&make_tx(tx_id)).await.unwrap();

        let handler = UpdateTransactionHandler::new(repo, publisher);
        let cmd = UpdateTransactionCommand {
            transaction_id: tx_id,
            amount: None,
            description: Some("".into()),
            date: None,
            category_id: None,
        };
        let result = handler.handle(cmd).await;
        assert!(result.is_err());
    }
}
