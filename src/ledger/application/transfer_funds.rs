use crate::ledger::domain::events::TransferCompleted;
use crate::ledger::domain::repository::{AccountRepository, TransactionRepository};
use crate::ledger::domain::transaction::{Transaction, TransactionType};
use crate::provider::id::IdGenerator;
use crate::shared::errors::LedgerError;
use crate::shared::events::EventPublisher;
use crate::shared::ids::{AccountID, TransactionID};
use crate::shared::money::Money;
use chrono::NaiveDate;
use std::sync::Arc;

/// Command to transfer funds between two accounts.
pub struct TransferFundsCommand {
    pub from_account_id: AccountID,
    pub to_account_id: AccountID,
    pub amount: Money,
    pub description: String,
    pub date: NaiveDate,
}

/// Handler that processes [`TransferFundsCommand`] requests.
pub struct TransferFundsHandler<
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
    TransferFundsHandler<A, T, P, I>
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

    /// Executes the transfer: creates outgoing and incoming transactions, persists them,
    /// and publishes [`TransferCompleted`].
    ///
    /// # Errors
    /// Fails if accounts have different currencies or different owners.
    pub async fn handle(&self, cmd: TransferFundsCommand) -> Result<(), LedgerError> {
        self.validate(&cmd)?;

        let from_account = self
            .account_repository
            .find_by_id(cmd.from_account_id)
            .await?
            .ok_or_else(|| LedgerError::AccountNotFound(cmd.from_account_id.to_string()))?;

        let to_account = self
            .account_repository
            .find_by_id(cmd.to_account_id)
            .await?
            .ok_or_else(|| LedgerError::AccountNotFound(cmd.to_account_id.to_string()))?;

        if from_account.currency() != to_account.currency() {
            return Err(LedgerError::CurrencyMismatch {
                expected: from_account.currency().code().to_string(),
                received: to_account.currency().code().to_string(),
            });
        }

        if from_account.owner_id != to_account.owner_id {
            return Err(LedgerError::Forbidden(
                "transfer between accounts of different owners is not allowed".into(),
            ));
        }

        let from_id = TransactionID::from_uuid(self.id_generator.new_id());
        let to_id = TransactionID::from_uuid(self.id_generator.new_id());

        let outgoing = Transaction::new(
            from_id,
            cmd.from_account_id,
            TransactionType::Transfer,
            cmd.amount,
            format!("Transfer to {}", to_account.name),
            cmd.date,
        )?
        .with_counterpart(cmd.to_account_id)?;

        let incoming = Transaction::new(
            to_id,
            cmd.to_account_id,
            TransactionType::Transfer,
            cmd.amount,
            format!("Transfer from {}", from_account.name),
            cmd.date,
        )?
        .with_counterpart(cmd.from_account_id)?;

        self.transaction_repository.save(&outgoing).await?;
        self.transaction_repository.save(&incoming).await?;

        let event = TransferCompleted {
            from_account_id: cmd.from_account_id,
            to_account_id: cmd.to_account_id,
            amount: cmd.amount,
            timestamp: chrono::Utc::now(),
        };
        self.event_publisher.publish(vec![&event]).await?;

        Ok(())
    }

    /// Validates basic command constraints (different accounts, positive amount).
    pub fn validate(&self, cmd: &TransferFundsCommand) -> Result<(), LedgerError> {
        if cmd.from_account_id == cmd.to_account_id {
            return Err(LedgerError::InvariantViolation(
                "source and destination accounts must be different".into(),
            ));
        }

        if !cmd.amount.is_positive() {
            return Err(LedgerError::InvalidAmount("amount must be positive".into()));
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
    async fn test_transfer_funds() {
        let account_repo = Arc::new(MockAccountRepository::new());
        let tx_repo = Arc::new(MockTransactionRepository::new());
        let publisher = Arc::new(InMemoryEventDispatcher::new());
        let id_gen = Arc::new(MockIdGenerator::new(uuid::Uuid::new_v4()));

        let from_id = AccountID::new();
        let to_id = AccountID::new();
        let owner = UserID::new();

        let from_account = crate::ledger::domain::account::Account::new(
            from_id,
            owner,
            "From".into(),
            crate::ledger::domain::account::AccountType::Checking,
            Currency::BRL,
            Money::new(100000, Currency::BRL),
        )
        .unwrap();
        let to_account = crate::ledger::domain::account::Account::new(
            to_id,
            owner,
            "To".into(),
            crate::ledger::domain::account::AccountType::Checking,
            Currency::BRL,
            Money::new(50000, Currency::BRL),
        )
        .unwrap();
        account_repo.save(&from_account).await.unwrap();
        account_repo.save(&to_account).await.unwrap();

        let handler = TransferFundsHandler::new(account_repo, tx_repo, publisher, id_gen);

        let cmd = TransferFundsCommand {
            from_account_id: from_id,
            to_account_id: to_id,
            amount: Money::new(25000, Currency::BRL),
            description: "Transfer".into(),
            date: chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
        };

        let result = handler.handle(cmd).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_different_accounts() {
        let account_repo = Arc::new(MockAccountRepository::new());
        let tx_repo = Arc::new(MockTransactionRepository::new());
        let publisher = Arc::new(InMemoryEventDispatcher::new());
        let id_gen = Arc::new(MockIdGenerator::new(uuid::Uuid::new_v4()));

        let handler = TransferFundsHandler::new(account_repo, tx_repo, publisher, id_gen);

        let same_id = AccountID::new();
        let cmd = TransferFundsCommand {
            from_account_id: same_id,
            to_account_id: same_id,
            amount: Money::new(25000, Currency::BRL),
            description: "Transfer".into(),
            date: chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
        };

        let result = handler.validate(&cmd);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_transfer_cross_owner_blocked() {
        let account_repo = Arc::new(MockAccountRepository::new());
        let tx_repo = Arc::new(MockTransactionRepository::new());
        let publisher = Arc::new(InMemoryEventDispatcher::new());
        let id_gen = Arc::new(MockIdGenerator::new(uuid::Uuid::new_v4()));

        let from_id = AccountID::new();
        let to_id = AccountID::new();

        let from_account = crate::ledger::domain::account::Account::new(
            from_id,
            UserID::new(),
            "Owner A".into(),
            crate::ledger::domain::account::AccountType::Checking,
            Currency::BRL,
            Money::new(100000, Currency::BRL),
        )
        .unwrap();
        let to_account = crate::ledger::domain::account::Account::new(
            to_id,
            UserID::new(),
            "Owner B".into(),
            crate::ledger::domain::account::AccountType::Checking,
            Currency::BRL,
            Money::new(50000, Currency::BRL),
        )
        .unwrap();
        account_repo.save(&from_account).await.unwrap();
        account_repo.save(&to_account).await.unwrap();

        let handler = TransferFundsHandler::new(account_repo, tx_repo, publisher, id_gen);

        let cmd = TransferFundsCommand {
            from_account_id: from_id,
            to_account_id: to_id,
            amount: Money::new(25000, Currency::BRL),
            description: "Transfer".into(),
            date: chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
        };

        let result = handler.handle(cmd).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LedgerError::Forbidden(_)));
    }
}
