use std::sync::Arc;

use chrono::Utc;

use crate::importing::domain::events::TransactionsImported;
use crate::importing::domain::raw_transaction::RawTransaction;
use crate::ledger::domain::repository::{AccountRepository, TransactionRepository};
use crate::ledger::domain::transaction::{Transaction, TransactionType};
use crate::provider::id::IdGenerator;
use crate::shared::errors::ImportError;
use crate::shared::events::EventPublisher;
use crate::shared::ids::{AccountID, Principal, TransactionID};

/// Command to confirm and import a batch of raw transactions into the ledger.
///
/// Creates a [`Transaction`] record for each raw transaction and publishes
/// a [`TransactionsImported`] event.
pub struct ConfirmCommand {
    pub principal: Principal,
    /// Target account to import into.
    pub account_id: AccountID,
    /// Raw transactions to import (must have been previewed/matched first).
    pub transactions: Vec<RawTransaction>,
}

/// Handler that confirms and creates transactions from raw imports.
pub struct ConfirmHandler<
    A: AccountRepository,
    T: TransactionRepository,
    P: EventPublisher,
    I: IdGenerator,
> {
    account_repository: Arc<A>,
    transaction_repository: Arc<T>,
    event_publisher: Arc<P>,
    id_generator: Arc<I>,
}

impl<A: AccountRepository, T: TransactionRepository, P: EventPublisher, I: IdGenerator>
    ConfirmHandler<A, T, P, I>
{
    pub fn new(
        account_repository: Arc<A>,
        transaction_repository: Arc<T>,
        event_publisher: Arc<P>,
        id_generator: Arc<I>,
    ) -> Self {
        Self {
            account_repository,
            transaction_repository,
            event_publisher,
            id_generator,
        }
    }

    /// Creates transactions for each raw record and publishes [`TransactionsImported`].
    ///
    /// Returns the number of transactions successfully imported.
    ///
    /// # Errors
    /// Fails if the account does not belong to the authenticated user.
    pub async fn handle(&self, cmd: ConfirmCommand) -> Result<usize, ImportError> {
        let account = self
            .account_repository
            .find_by_id(cmd.account_id)
            .await?
            .ok_or_else(|| {
                ImportError::NotFound(format!("account not found: {}", cmd.account_id))
            })?;

        if account.owner_id != cmd.principal.user_id {
            return Err(ImportError::Forbidden(
                "not the owner of this account".into(),
            ));
        }

        if cmd.transactions.is_empty() {
            return Ok(0);
        }

        let mut count = 0usize;

        for raw in &cmd.transactions {
            let id = TransactionID::from_uuid(self.id_generator.new_id());

            let tx_type = if raw.amount.is_positive() {
                TransactionType::Income
            } else {
                TransactionType::Expense
            };

            let amount = raw.amount.abs();

            let mut transaction = Transaction::new(
                id,
                cmd.account_id,
                tx_type,
                amount,
                raw.description.clone(),
                raw.date,
            )
            .map_err(|e| ImportError::InvariantViolation(e.to_string()))?;

            transaction.created_at = Utc::now();

            self.transaction_repository.save(&transaction).await?;
            count += 1;
        }

        let event = TransactionsImported {
            account_id: cmd.account_id,
            count,
            timestamp: Utc::now(),
        };

        self.event_publisher
            .publish(vec![&event])
            .await
            .map_err(ImportError::Publish)?;

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::domain::account::{Account, AccountType};
    use crate::provider::id::MockIdGenerator;
    use crate::shared::events::InMemoryEventDispatcher;
    use crate::shared::ids::{AccountID, Principal, UserID};
    use crate::shared::mock::{MockAccountRepository, MockTransactionRepository};
    use crate::shared::money::{Currency, Money};
    use crate::shared::period::Period;

    fn sample_raw(date: &str, amount_cents: i64, desc: &str) -> RawTransaction {
        RawTransaction {
            date: chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap(),
            amount: Money::from_cents(amount_cents, Currency::BRL),
            description: desc.into(),
            raw_line: format!("{date},{amount_cents},{desc}"),
        }
    }

    #[tokio::test]
    async fn test_confirm_empty() {
        let account_repo = Arc::new(MockAccountRepository::new());
        let tx_repo = Arc::new(MockTransactionRepository::new());
        let publisher = Arc::new(InMemoryEventDispatcher::new());
        let id_gen = Arc::new(MockIdGenerator::new(uuid::Uuid::new_v4()));
        let owner = UserID::new();
        let account_id = AccountID::new();

        let account = Account::new(
            account_id,
            owner,
            "Test".into(),
            AccountType::Checking,
            Currency::BRL,
            Money::from_cents(0, Currency::BRL),
        )
        .unwrap();
        account_repo.save(&account).await.unwrap();

        let handler = ConfirmHandler::new(account_repo, tx_repo, publisher, id_gen);

        let count = handler
            .handle(ConfirmCommand {
                principal: Principal::new(owner),
                account_id,
                transactions: Vec::new(),
            })
            .await
            .unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_confirm_creates_transactions() {
        let account_repo = Arc::new(MockAccountRepository::new());
        let tx_repo = Arc::new(MockTransactionRepository::new());
        let publisher = Arc::new(InMemoryEventDispatcher::new());
        let id_gen = Arc::new(MockIdGenerator::new(uuid::Uuid::new_v4()));
        let owner = UserID::new();
        let account_id = AccountID::new();

        let account = Account::new(
            account_id,
            owner,
            "Test".into(),
            AccountType::Checking,
            Currency::BRL,
            Money::from_cents(0, Currency::BRL),
        )
        .unwrap();
        account_repo.save(&account).await.unwrap();

        let handler = ConfirmHandler::new(account_repo, tx_repo, publisher, id_gen);

        let txs = vec![
            sample_raw("2026-03-15", -2500, "Supermarket"),
            sample_raw("2026-03-16", 50000, "Salary"),
        ];

        let count = handler
            .handle(ConfirmCommand {
                principal: Principal::new(owner),
                account_id,
                transactions: txs,
            })
            .await
            .unwrap();
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_confirm_negative_becomes_expense() {
        let account_repo = Arc::new(MockAccountRepository::new());
        let tx_repo = Arc::new(MockTransactionRepository::new());
        let publisher = Arc::new(InMemoryEventDispatcher::new());
        let id_gen = Arc::new(MockIdGenerator::new(uuid::Uuid::new_v4()));
        let owner = UserID::new();
        let account_id = AccountID::new();

        let account = Account::new(
            account_id,
            owner,
            "Test".into(),
            AccountType::Checking,
            Currency::BRL,
            Money::from_cents(0, Currency::BRL),
        )
        .unwrap();
        account_repo.save(&account).await.unwrap();

        let handler = ConfirmHandler::new(account_repo, tx_repo.clone(), publisher, id_gen);

        let txs = vec![sample_raw("2026-01-01", -1000, "Coffee")];

        handler
            .handle(ConfirmCommand {
                principal: Principal::new(owner),
                account_id,
                transactions: txs,
            })
            .await
            .unwrap();

        // Verify the transaction was saved
        let start = chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let end = chrono::NaiveDate::from_ymd_opt(2027, 12, 31).unwrap();
        let txs = tx_repo
            .find_by_account(account_id, Period::new(start, end))
            .await
            .unwrap();
        assert_eq!(txs.len(), 1);
        assert_eq!(txs[0].tx_type, TransactionType::Expense);
        assert_eq!(txs[0].amount, Money::from_cents(1000, Currency::BRL));
    }

    #[tokio::test]
    async fn test_confirm_wrong_owner_blocked() {
        let account_repo = Arc::new(MockAccountRepository::new());
        let tx_repo = Arc::new(MockTransactionRepository::new());
        let publisher = Arc::new(InMemoryEventDispatcher::new());
        let id_gen = Arc::new(MockIdGenerator::new(uuid::Uuid::new_v4()));
        let account_id = AccountID::new();

        let account = Account::new(
            account_id,
            UserID::new(),
            "Test".into(),
            AccountType::Checking,
            Currency::BRL,
            Money::from_cents(0, Currency::BRL),
        )
        .unwrap();
        account_repo.save(&account).await.unwrap();

        let handler = ConfirmHandler::new(account_repo, tx_repo, publisher, id_gen);

        let txs = vec![sample_raw("2026-01-01", -1000, "Coffee")];

        let result = handler
            .handle(ConfirmCommand {
                principal: Principal::new(UserID::new()),
                account_id,
                transactions: txs,
            })
            .await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ImportError::Forbidden(_)));
    }
}
