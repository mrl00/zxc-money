use crate::ledger::domain::events::AccountDeleted;
use crate::ledger::domain::repository::{AccountRepository, TransactionRepository};
use crate::shared::errors::LedgerError;
use crate::shared::events::EventPublisher;
use crate::shared::ids::AccountID;
use std::sync::Arc;

/// Command to delete an existing account.
pub struct DeleteAccountCommand {
    pub account_id: AccountID,
}

/// Handler that processes [`DeleteAccountCommand`] requests.
pub struct DeleteAccountHandler<A: AccountRepository, T: TransactionRepository, P: EventPublisher> {
    account_repository: Arc<A>,
    transaction_repository: Arc<T>,
    event_publisher: Arc<P>,
}

impl<A: AccountRepository, T: TransactionRepository, P: EventPublisher>
    DeleteAccountHandler<A, T, P>
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

    /// Deletes the account if it has no linked transactions and publishes [`AccountDeleted`].
    pub async fn handle(&self, cmd: DeleteAccountCommand) -> Result<(), LedgerError> {
        let account = self
            .account_repository
            .find_by_id(cmd.account_id)
            .await?
            .ok_or_else(|| LedgerError::AccountNotFound(cmd.account_id.to_string()))?;

        let has_txns = self
            .transaction_repository
            .has_transactions(cmd.account_id)
            .await?;

        if has_txns {
            return Err(LedgerError::InvariantViolation(
                "cannot delete account with linked transactions".into(),
            ));
        }

        self.account_repository.delete(cmd.account_id).await?;

        let event = AccountDeleted {
            account_id: cmd.account_id,
            owner_id: account.owner_id,
            timestamp: chrono::Utc::now(),
        };
        self.event_publisher.publish(vec![&event]).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::domain::account::{Account, AccountType};
    use crate::ledger::domain::transaction::{Transaction, TransactionType};
    use crate::shared::events::InMemoryEventDispatcher;
    use crate::shared::ids::UserID;
    use crate::shared::mock::{MockAccountRepository, MockTransactionRepository};
    use crate::shared::money::{Currency, Money};

    #[tokio::test]
    async fn test_delete_empty_account() {
        let account_repo = Arc::new(MockAccountRepository::new());
        let tx_repo = Arc::new(MockTransactionRepository::new());
        let publisher = Arc::new(InMemoryEventDispatcher::new());

        let account_id = AccountID::new();
        let account = Account::new(
            account_id,
            UserID::new(),
            "To Delete".into(),
            AccountType::Checking,
            Currency::BRL,
            Money::from_cents(0, Currency::BRL),
        )
        .unwrap();
        account_repo.save(&account).await.unwrap();

        let handler = DeleteAccountHandler::new(account_repo.clone(), tx_repo, publisher);
        let result = handler.handle(DeleteAccountCommand { account_id }).await;
        assert!(result.is_ok());
        assert!(account_repo.find_by_id(account_id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_delete_account_with_transactions_blocked() {
        let account_repo = Arc::new(MockAccountRepository::new());
        let tx_repo = Arc::new(MockTransactionRepository::new());
        let publisher = Arc::new(InMemoryEventDispatcher::new());

        let account_id = AccountID::new();
        let account = Account::new(
            account_id,
            UserID::new(),
            "Has Transactions".into(),
            AccountType::Checking,
            Currency::BRL,
            Money::from_cents(0, Currency::BRL),
        )
        .unwrap();
        account_repo.save(&account).await.unwrap();

        let tx = Transaction::new(
            crate::shared::ids::TransactionID::new(),
            account_id,
            TransactionType::Income,
            Money::from_cents(100, Currency::BRL),
            "Salary".into(),
            chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
        )
        .unwrap()
        .with_category(crate::shared::ids::CategoryID::new())
        .unwrap();
        tx_repo.save(&tx).await.unwrap();

        let handler = DeleteAccountHandler::new(account_repo, tx_repo, publisher);
        let result = handler.handle(DeleteAccountCommand { account_id }).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            LedgerError::InvariantViolation(_)
        ));
    }
}
