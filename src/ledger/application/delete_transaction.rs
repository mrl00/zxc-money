use crate::ledger::domain::events::TransactionDeleted;
use crate::ledger::domain::repository::{AccountRepository, TransactionRepository};
use crate::shared::errors::LedgerError;
use crate::shared::events::EventPublisher;
use crate::shared::ids::{Principal, TransactionID};
use std::sync::Arc;

/// Command to delete a transaction.
pub struct DeleteTransactionCommand {
    pub principal: Principal,
    pub transaction_id: TransactionID,
}

/// Handler that processes [`DeleteTransactionCommand`] requests.
pub struct DeleteTransactionHandler<
    A: AccountRepository,
    T: TransactionRepository,
    P: EventPublisher,
> {
    account_repository: Arc<A>,
    transaction_repository: Arc<T>,
    event_publisher: Arc<P>,
}

impl<A: AccountRepository, T: TransactionRepository, P: EventPublisher>
    DeleteTransactionHandler<A, T, P>
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

    /// Deletes the transaction and publishes [`TransactionDeleted`].
    ///
    /// # Errors
    /// Fails if the transaction is reconciled, derived from a credit card purchase,
    /// or does not belong to the authenticated user.
    pub async fn handle(&self, cmd: DeleteTransactionCommand) -> Result<(), LedgerError> {
        let transaction = self
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
    use crate::ledger::domain::account::Account;
    use crate::ledger::domain::transaction::{Transaction, TransactionType};
    use crate::shared::events::InMemoryEventDispatcher;
    use crate::shared::ids::{AccountID, Principal, PurchaseID, UserID};
    use crate::shared::mock::{MockAccountRepository, MockTransactionRepository};
    use crate::shared::money::{Currency, Money};

    fn make_tx(id: TransactionID, account_id: AccountID) -> Transaction {
        Transaction::new(
            id,
            account_id,
            TransactionType::Income,
            Money::from_cents(100, Currency::BRL),
            "Test".into(),
            chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
        )
        .unwrap()
        .with_category(crate::shared::ids::CategoryID::new())
        .unwrap()
    }

    fn make_account(id: AccountID, owner: UserID) -> Account {
        Account::new(
            id,
            owner,
            "Test".into(),
            crate::ledger::domain::account::AccountType::Checking,
            Currency::BRL,
            Money::from_cents(0, Currency::BRL),
        )
        .unwrap()
    }

    #[tokio::test]
    async fn test_delete_normal_transaction() {
        let account_repo = Arc::new(MockAccountRepository::new());
        let tx_repo = Arc::new(MockTransactionRepository::new());
        let publisher = Arc::new(InMemoryEventDispatcher::new());
        let owner = UserID::new();
        let account_id = AccountID::new();
        let tx_id = TransactionID::new();

        account_repo
            .save(&make_account(account_id, owner))
            .await
            .unwrap();
        tx_repo.save(&make_tx(tx_id, account_id)).await.unwrap();

        let handler = DeleteTransactionHandler::new(account_repo, tx_repo, publisher);
        let result = handler
            .handle(DeleteTransactionCommand {
                principal: Principal::new(owner),
                transaction_id: tx_id,
            })
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_wrong_owner_blocked() {
        let account_repo = Arc::new(MockAccountRepository::new());
        let tx_repo = Arc::new(MockTransactionRepository::new());
        let publisher = Arc::new(InMemoryEventDispatcher::new());
        let account_id = AccountID::new();
        let tx_id = TransactionID::new();

        account_repo
            .save(&make_account(account_id, UserID::new()))
            .await
            .unwrap();
        tx_repo.save(&make_tx(tx_id, account_id)).await.unwrap();

        let handler = DeleteTransactionHandler::new(account_repo, tx_repo, publisher);
        let result = handler
            .handle(DeleteTransactionCommand {
                principal: Principal::new(UserID::new()),
                transaction_id: tx_id,
            })
            .await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LedgerError::Forbidden(_)));
    }

    #[tokio::test]
    async fn test_delete_reconciled_transaction_blocked() {
        let account_repo = Arc::new(MockAccountRepository::new());
        let tx_repo = Arc::new(MockTransactionRepository::new());
        let publisher = Arc::new(InMemoryEventDispatcher::new());
        let owner = UserID::new();
        let account_id = AccountID::new();
        let tx_id = TransactionID::new();

        account_repo
            .save(&make_account(account_id, owner))
            .await
            .unwrap();
        let mut tx = make_tx(tx_id, account_id);
        tx.mark_reconciled();
        tx_repo.save(&tx).await.unwrap();

        let handler = DeleteTransactionHandler::new(account_repo, tx_repo, publisher);
        let result = handler
            .handle(DeleteTransactionCommand {
                principal: Principal::new(owner),
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
        let account_repo = Arc::new(MockAccountRepository::new());
        let tx_repo = Arc::new(MockTransactionRepository::new());
        let publisher = Arc::new(InMemoryEventDispatcher::new());
        let owner = UserID::new();
        let account_id = AccountID::new();
        let tx_id = TransactionID::new();

        account_repo
            .save(&make_account(account_id, owner))
            .await
            .unwrap();
        let tx = make_tx(tx_id, account_id).with_source_purchase(PurchaseID::new());
        tx_repo.save(&tx).await.unwrap();

        let handler = DeleteTransactionHandler::new(account_repo, tx_repo, publisher);
        let result = handler
            .handle(DeleteTransactionCommand {
                principal: Principal::new(owner),
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
