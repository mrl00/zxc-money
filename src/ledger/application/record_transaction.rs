use crate::ledger::domain::events::TransactionRecorded;
use crate::ledger::domain::repository::{AccountRepository, TransactionRepository};
use crate::ledger::domain::transaction::{Transaction, TransactionType};
use crate::provider::id::IdGenerator;
use crate::shared::errors::LedgerError;
use crate::shared::events::EventPublisher;
use crate::shared::ids::{AccountID, CategoryID, TransactionID};
use crate::shared::money::Money;
use chrono::NaiveDate;
use std::sync::Arc;

/// Command to record a new transaction on an account.
pub struct RecordTransactionCommand {
    pub account_id: AccountID,
    pub tx_type: TransactionType,
    pub amount: Money,
    pub description: String,
    pub date: NaiveDate,
    pub category_id: Option<CategoryID>,
}

/// Handler that processes [`RecordTransactionCommand`] requests.
pub struct RecordTransactionHandler<
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
    RecordTransactionHandler<A, T, P, I>
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

    /// Executes the command: validates currency, creates the transaction, persists it,
    /// and publishes [`TransactionRecorded`].
    ///
    /// Returns the new [`TransactionID`] on success.
    pub async fn handle(
        &self,
        cmd: RecordTransactionCommand,
    ) -> Result<TransactionID, LedgerError> {
        let account = self
            .account_repository
            .find_by_id(cmd.account_id)
            .await?
            .ok_or_else(|| LedgerError::AccountNotFound(cmd.account_id.to_string()))?;

        if cmd.amount.currency() != account.currency() {
            return Err(LedgerError::CurrencyMismatch {
                expected: account.currency().code().to_string(),
                received: cmd.amount.currency().code().to_string(),
            });
        }

        let id = TransactionID::from_uuid(self.id_generator.new_id());

        let category_id = cmd.category_id;
        let description = cmd.description;
        let date = cmd.date;

        let mut transaction = Transaction::new(
            id,
            cmd.account_id,
            cmd.tx_type,
            cmd.amount,
            description.clone(),
            date,
        )?;

        if let Some(cid) = category_id {
            transaction = transaction.with_category(cid)?;
        }

        transaction.validate()?;

        self.transaction_repository.save(&transaction).await?;

        let event = TransactionRecorded {
            transaction_id: id,
            account_id: cmd.account_id,
            tx_type: cmd.tx_type,
            amount: cmd.amount,
            category_id,
            description,
            date,
            timestamp: chrono::Utc::now(),
        };
        self.event_publisher.publish(vec![&event]).await?;

        Ok(id)
    }

    /// Validates the command fields without persisting.
    pub fn validate(&self, cmd: &RecordTransactionCommand) -> Result<(), LedgerError> {
        if !cmd.amount.is_positive() {
            return Err(LedgerError::InvalidAmount("amount must be positive".into()));
        }

        if cmd.description.is_empty() {
            return Err(LedgerError::InvariantViolation(
                "description must not be empty".into(),
            ));
        }

        match cmd.tx_type {
            TransactionType::Transfer => {
                if cmd.category_id.is_some() {
                    return Err(LedgerError::InvariantViolation(
                        "transfer must not have category".into(),
                    ));
                }
            }
            TransactionType::Income | TransactionType::Expense => {
                if cmd.category_id.is_none() {
                    return Err(LedgerError::InvariantViolation(
                        "income/expense must have category".into(),
                    ));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::id::MockIdGenerator;
    use crate::shared::events::InMemoryEventDispatcher;
    use crate::shared::ids::UserID;
    use crate::shared::mock::{MockAccountRepository, MockTransactionRepository};
    use crate::shared::money::Currency;

    #[tokio::test]
    async fn test_record_income() {
        let account_repo = Arc::new(MockAccountRepository::new());
        let tx_repo = Arc::new(MockTransactionRepository::new());
        let publisher = Arc::new(InMemoryEventDispatcher::new());
        let id_gen = Arc::new(MockIdGenerator::new(uuid::Uuid::new_v4()));

        let account_id = AccountID::new();
        let account = crate::ledger::domain::account::Account::new(
            account_id,
            UserID::new(),
            "Test".into(),
            crate::ledger::domain::account::AccountType::Checking,
            Currency::BRL,
            Money::new(0, Currency::BRL),
        )
        .unwrap();
        account_repo.save(&account).await.unwrap();

        let handler = RecordTransactionHandler::new(account_repo, tx_repo, publisher, id_gen);

        let category_id = CategoryID::new();
        let cmd = RecordTransactionCommand {
            account_id,
            tx_type: TransactionType::Income,
            amount: Money::new(50000, Currency::BRL),
            description: "Salary".into(),
            date: chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            category_id: Some(category_id),
        };

        let tx_id = handler.handle(cmd).await.unwrap();
        assert!(!tx_id.as_uuid().is_nil());
    }

    #[tokio::test]
    async fn test_validate_income_requires_category() {
        let account_repo = Arc::new(MockAccountRepository::new());
        let tx_repo = Arc::new(MockTransactionRepository::new());
        let publisher = Arc::new(InMemoryEventDispatcher::new());
        let id_gen = Arc::new(MockIdGenerator::new(uuid::Uuid::new_v4()));

        let handler = RecordTransactionHandler::new(account_repo, tx_repo, publisher, id_gen);

        let cmd = RecordTransactionCommand {
            account_id: AccountID::new(),
            tx_type: TransactionType::Income,
            amount: Money::new(50000, Currency::BRL),
            description: "Salary".into(),
            date: chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            category_id: None,
        };

        let result = handler.validate(&cmd);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_transfer_no_category() {
        let account_repo = Arc::new(MockAccountRepository::new());
        let tx_repo = Arc::new(MockTransactionRepository::new());
        let publisher = Arc::new(InMemoryEventDispatcher::new());
        let id_gen = Arc::new(MockIdGenerator::new(uuid::Uuid::new_v4()));

        let handler = RecordTransactionHandler::new(account_repo, tx_repo, publisher, id_gen);

        let cmd = RecordTransactionCommand {
            account_id: AccountID::new(),
            tx_type: TransactionType::Transfer,
            amount: Money::new(50000, Currency::BRL),
            description: "Transfer".into(),
            date: chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            category_id: Some(CategoryID::new()),
        };

        let result = handler.validate(&cmd);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_currency_mismatch_rejected() {
        let account_repo = Arc::new(MockAccountRepository::new());
        let tx_repo = Arc::new(MockTransactionRepository::new());
        let publisher = Arc::new(InMemoryEventDispatcher::new());
        let id_gen = Arc::new(MockIdGenerator::new(uuid::Uuid::new_v4()));

        let account_id = AccountID::new();
        let account = crate::ledger::domain::account::Account::new(
            account_id,
            UserID::new(),
            "BRL Account".into(),
            crate::ledger::domain::account::AccountType::Checking,
            Currency::BRL,
            Money::new(0, Currency::BRL),
        )
        .unwrap();
        account_repo.save(&account).await.unwrap();

        let handler = RecordTransactionHandler::new(account_repo, tx_repo, publisher, id_gen);

        let cmd = RecordTransactionCommand {
            account_id,
            tx_type: TransactionType::Income,
            amount: Money::new(100, Currency::USD),
            description: "USD income".into(),
            date: chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            category_id: Some(CategoryID::new()),
        };

        let result = handler.handle(cmd).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            LedgerError::CurrencyMismatch { .. }
        ));
    }
}
