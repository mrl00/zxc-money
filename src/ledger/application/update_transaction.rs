use chrono::NaiveDate;

use crate::ledger::domain::events::TransactionUpdated;
use crate::ledger::domain::repository::{AccountRepository, TransactionRepository};
use crate::ledger::domain::transaction::TransactionType;
use crate::shared::errors::LedgerError;
use crate::shared::events::EventPublisher;
use crate::shared::ids::{CategoryID, Principal, TransactionID};
use crate::shared::money::Money;
use std::sync::Arc;

/// Command to update fields of an existing transaction.
pub struct UpdateTransactionCommand {
    pub principal: Principal,
    pub transaction_id: TransactionID,
    pub amount: Option<Money>,
    pub description: Option<String>,
    pub date: Option<NaiveDate>,
    pub category_id: Option<Option<CategoryID>>,
}

/// Handler that processes [`UpdateTransactionCommand`] requests.
pub struct UpdateTransactionHandler<
    A: AccountRepository,
    T: TransactionRepository,
    P: EventPublisher,
> {
    account_repository: Arc<A>,
    transaction_repository: Arc<T>,
    event_publisher: Arc<P>,
}

impl<A: AccountRepository, T: TransactionRepository, P: EventPublisher>
    UpdateTransactionHandler<A, T, P>
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

    /// Updates the transaction fields, validates invariants, persists, and publishes
    /// [`TransactionUpdated`].
    ///
    /// # Errors
    /// Fails if the transaction is reconciled, derived from a purchase, does not
    /// belong to the authenticated user, or any invariant is violated.
    pub async fn handle(&self, cmd: UpdateTransactionCommand) -> Result<(), LedgerError> {
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
    use crate::ledger::domain::account::Account;
    use crate::ledger::domain::transaction::Transaction;
    use crate::shared::events::InMemoryEventDispatcher;
    use crate::shared::ids::{AccountID, Principal, PurchaseID, UserID};
    use crate::shared::mock::{MockAccountRepository, MockTransactionRepository};
    use crate::shared::money::Currency;

    fn make_tx(id: TransactionID, account_id: AccountID) -> Transaction {
        Transaction::new(
            id,
            account_id,
            TransactionType::Income,
            Money::from_cents(100, Currency::BRL),
            "Salary".into(),
            chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
        )
        .unwrap()
        .with_category(CategoryID::new())
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
    async fn test_update_description() {
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

        let handler = UpdateTransactionHandler::new(account_repo, tx_repo, publisher);
        let cmd = UpdateTransactionCommand {
            principal: Principal::new(owner),
            transaction_id: tx_id,
            amount: None,
            description: Some("Updated salary".into()),
            date: None,
            category_id: None,
        };
        assert!(handler.handle(cmd).await.is_ok());
    }

    #[tokio::test]
    async fn test_update_wrong_owner_blocked() {
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

        let handler = UpdateTransactionHandler::new(account_repo, tx_repo, publisher);
        let cmd = UpdateTransactionCommand {
            principal: Principal::new(UserID::new()),
            transaction_id: tx_id,
            amount: None,
            description: Some("Hack".into()),
            date: None,
            category_id: None,
        };
        let result = handler.handle(cmd).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LedgerError::Forbidden(_)));
    }

    #[tokio::test]
    async fn test_update_reconciled_blocked() {
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

        let handler = UpdateTransactionHandler::new(account_repo, tx_repo, publisher);
        let cmd = UpdateTransactionCommand {
            principal: Principal::new(owner),
            transaction_id: tx_id,
            amount: Some(Money::from_cents(200, Currency::BRL)),
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

        let handler = UpdateTransactionHandler::new(account_repo, tx_repo, publisher);
        let cmd = UpdateTransactionCommand {
            principal: Principal::new(owner),
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

        let handler = UpdateTransactionHandler::new(account_repo, tx_repo, publisher);
        let cmd = UpdateTransactionCommand {
            principal: Principal::new(owner),
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
