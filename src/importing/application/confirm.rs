use std::sync::Arc;

use chrono::Utc;

use crate::importing::domain::events::TransactionsImported;
use crate::importing::domain::raw_transaction::RawTransaction;
use crate::ledger::domain::repository::TransactionRepository;
use crate::ledger::domain::transaction::{Transaction, TransactionType};
use crate::provider::id::IdGenerator;
use crate::shared::errors::ImportError;
use crate::shared::events::EventPublisher;
use crate::shared::ids::{AccountID, TransactionID};

/// Command to confirm and import a batch of raw transactions into the ledger.
///
/// Creates a [`Transaction`] record for each raw transaction and publishes
/// a [`TransactionsImported`] event.
pub struct ConfirmCommand {
    /// Target account to import into.
    pub account_id: AccountID,
    /// Raw transactions to import (must have been previewed/matched first).
    pub transactions: Vec<RawTransaction>,
}

/// Handler that confirms and creates transactions from raw imports.
pub struct ConfirmHandler<T: TransactionRepository, P: EventPublisher, I: IdGenerator> {
    transaction_repository: Arc<T>,
    event_publisher: Arc<P>,
    id_generator: Arc<I>,
}

impl<T: TransactionRepository, P: EventPublisher, I: IdGenerator> ConfirmHandler<T, P, I> {
    pub fn new(
        transaction_repository: Arc<T>,
        event_publisher: Arc<P>,
        id_generator: Arc<I>,
    ) -> Self {
        Self {
            transaction_repository,
            event_publisher,
            id_generator,
        }
    }

    /// Creates transactions for each raw record and publishes [`TransactionsImported`].
    ///
    /// Returns the number of transactions successfully imported.
    pub async fn handle(&self, cmd: ConfirmCommand) -> Result<usize, ImportError> {
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
    use crate::provider::id::MockIdGenerator;
    use crate::shared::events::InMemoryEventDispatcher;
    use crate::shared::ids::AccountID;
    use crate::shared::mock::MockTransactionRepository;
    use crate::shared::money::{Currency, Money};
    use crate::shared::period::Period;

    fn sample_raw(date: &str, amount_cents: i64, desc: &str) -> RawTransaction {
        RawTransaction {
            date: chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap(),
            amount: Money::new(amount_cents, Currency::BRL),
            description: desc.into(),
            raw_line: format!("{date},{amount_cents},{desc}"),
        }
    }

    #[tokio::test]
    async fn test_confirm_empty() {
        let repo = Arc::new(MockTransactionRepository::new());
        let publisher = Arc::new(InMemoryEventDispatcher::new());
        let id_gen = Arc::new(MockIdGenerator::new(uuid::Uuid::new_v4()));
        let handler = ConfirmHandler::new(repo, publisher, id_gen);

        let count = handler
            .handle(ConfirmCommand {
                account_id: AccountID::new(),
                transactions: Vec::new(),
            })
            .await
            .unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_confirm_creates_transactions() {
        let repo = Arc::new(MockTransactionRepository::new());
        let publisher = Arc::new(InMemoryEventDispatcher::new());
        let id_gen = Arc::new(MockIdGenerator::new(uuid::Uuid::new_v4()));
        let handler = ConfirmHandler::new(repo, publisher, id_gen);

        let txs = vec![
            sample_raw("2026-03-15", -2500, "Supermarket"),
            sample_raw("2026-03-16", 50000, "Salary"),
        ];

        let count = handler
            .handle(ConfirmCommand {
                account_id: AccountID::new(),
                transactions: txs,
            })
            .await
            .unwrap();
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_confirm_negative_becomes_expense() {
        let repo = Arc::new(MockTransactionRepository::new());
        let publisher = Arc::new(InMemoryEventDispatcher::new());
        let id_gen = Arc::new(MockIdGenerator::new(uuid::Uuid::new_v4()));
        let handler = ConfirmHandler::new(repo.clone(), publisher, id_gen);

        let txs = vec![sample_raw("2026-01-01", -1000, "Coffee")];
        let account_id = AccountID::new();

        handler
            .handle(ConfirmCommand {
                account_id,
                transactions: txs,
            })
            .await
            .unwrap();

        // Verify the transaction was saved
        let start = chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let end = chrono::NaiveDate::from_ymd_opt(2027, 12, 31).unwrap();
        let txs = repo
            .find_by_account(account_id, Period::new(start, end))
            .await
            .unwrap();
        assert_eq!(txs.len(), 1);
        assert_eq!(txs[0].tx_type, TransactionType::Expense);
        assert_eq!(txs[0].amount, Money::new(1000, Currency::BRL));
    }
}
