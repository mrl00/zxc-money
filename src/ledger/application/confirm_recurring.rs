use crate::ledger::domain::events::{RecurringTransactionGenerated, TransactionRecorded};
use crate::ledger::domain::repository::{RecurringTransactionRepository, TransactionRepository};
use crate::ledger::domain::transaction::Transaction;
use crate::provider::id::IdGenerator;
use crate::shared::errors::LedgerError;
use crate::shared::events::EventPublisher;
use crate::shared::ids::{RecurringTransactionID, TransactionID};
use std::sync::Arc;

/// Command to confirm a due recurring transaction and generate a concrete transaction.
pub struct ConfirmRecurringCommand {
    pub recurring_transaction_id: RecurringTransactionID,
}

/// Handler that processes [`ConfirmRecurringCommand`] requests.
pub struct ConfirmRecurringHandler<
    R: RecurringTransactionRepository,
    T: TransactionRepository,
    P: EventPublisher,
    I: IdGenerator,
> {
    recurring_repository: Arc<R>,
    transaction_repository: Arc<T>,
    event_publisher: Arc<P>,
    id_generator: Arc<I>,
}

impl<R: RecurringTransactionRepository, T: TransactionRepository, P: EventPublisher, I: IdGenerator>
    ConfirmRecurringHandler<R, T, P, I>
{
    pub fn new(
        recurring_repository: Arc<R>,
        transaction_repository: Arc<T>,
        event_publisher: Arc<P>,
        id_generator: Arc<I>,
    ) -> Self {
        Self {
            recurring_repository,
            transaction_repository,
            event_publisher,
            id_generator,
        }
    }

    /// Creates a [`Transaction`] from the recurring template, advances the schedule,
    /// and publishes [`TransactionRecorded`] and [`RecurringTransactionGenerated`].
    ///
    /// Returns the new [`TransactionID`] on success.
    pub async fn handle(&self, cmd: ConfirmRecurringCommand) -> Result<TransactionID, LedgerError> {
        let mut recurring = self
            .recurring_repository
            .find_by_id(cmd.recurring_transaction_id)
            .await?
            .ok_or_else(|| {
                LedgerError::RecurringTransactionNotFound(cmd.recurring_transaction_id.to_string())
            })?;

        if !recurring.active {
            return Err(LedgerError::InvariantViolation(
                "recurring transaction is not active".into(),
            ));
        }

        let tx_id = TransactionID::from_uuid(self.id_generator.new_id());

        let mut transaction = Transaction::new(
            tx_id,
            recurring.account_id,
            recurring.tx_type,
            recurring.amount,
            recurring.description.clone(),
            recurring.next_date,
        )?;

        if let Some(category_id) = recurring.category_id {
            transaction = transaction.with_category(category_id)?;
        }

        transaction.validate()?;

        self.transaction_repository.save(&transaction).await?;

        recurring.advance();
        self.recurring_repository.save(&recurring).await?;

        let tx_event = TransactionRecorded {
            transaction_id: tx_id,
            account_id: recurring.account_id,
            tx_type: recurring.tx_type,
            amount: recurring.amount,
            category_id: recurring.category_id,
            description: recurring.description,
            date: recurring.next_date,
            timestamp: chrono::Utc::now(),
        };

        let gen_event = RecurringTransactionGenerated {
            recurring_transaction_id: cmd.recurring_transaction_id,
            transaction_id: tx_id,
            account_id: recurring.account_id,
            next_date: recurring.next_date,
            timestamp: chrono::Utc::now(),
        };

        self.event_publisher
            .publish(vec![&tx_event, &gen_event])
            .await?;

        Ok(tx_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::ledger::domain::recurring_transaction::Frequency;
    use crate::ledger::domain::recurring_transaction::RecurringTransaction;
    use crate::ledger::domain::transaction::TransactionType;
    use crate::provider::id::MockIdGenerator;
    use crate::shared::events::InMemoryEventDispatcher;
    use crate::shared::ids::{AccountID, CategoryID, UserID};
    use crate::shared::mock::{MockRecurringTransactionRepository, MockTransactionRepository};
    use crate::shared::money::{Currency, Money};

    #[tokio::test]
    async fn test_confirm_recurring() {
        let recurring_repo = Arc::new(MockRecurringTransactionRepository::new());
        let tx_repo = Arc::new(MockTransactionRepository::new());
        let publisher = Arc::new(InMemoryEventDispatcher::new());
        let id_gen = Arc::new(MockIdGenerator::new(uuid::Uuid::new_v4()));

        let recurring_id = RecurringTransactionID::new();
        let r = RecurringTransaction::new(
            recurring_id,
            UserID::new(),
            AccountID::new(),
            TransactionType::Expense,
            Money::from_cents(5000, Currency::BRL),
            "Netflix".into(),
            Some(CategoryID::new()),
            Frequency::Monthly,
            chrono::NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
        )
        .unwrap();
        recurring_repo.save(&r).await.unwrap();

        let handler =
            ConfirmRecurringHandler::new(recurring_repo.clone(), tx_repo, publisher, id_gen);

        let tx_id = handler
            .handle(ConfirmRecurringCommand {
                recurring_transaction_id: recurring_id,
            })
            .await
            .unwrap();
        assert!(!tx_id.as_uuid().is_nil());

        let updated = recurring_repo
            .find_by_id(recurring_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            updated.next_date,
            chrono::NaiveDate::from_ymd_opt(2026, 3, 1).unwrap()
        );
    }

    #[tokio::test]
    async fn test_confirm_inactive_recurring_blocked() {
        let recurring_repo = Arc::new(MockRecurringTransactionRepository::new());
        let tx_repo = Arc::new(MockTransactionRepository::new());
        let publisher = Arc::new(InMemoryEventDispatcher::new());
        let id_gen = Arc::new(MockIdGenerator::new(uuid::Uuid::new_v4()));

        let recurring_id = RecurringTransactionID::new();
        let mut r = RecurringTransaction::new(
            recurring_id,
            UserID::new(),
            AccountID::new(),
            TransactionType::Expense,
            Money::from_cents(5000, Currency::BRL),
            "Netflix".into(),
            Some(CategoryID::new()),
            Frequency::Monthly,
            chrono::NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
        )
        .unwrap();
        r.pause();
        recurring_repo.save(&r).await.unwrap();

        let handler = ConfirmRecurringHandler::new(recurring_repo, tx_repo, publisher, id_gen);

        let result = handler
            .handle(ConfirmRecurringCommand {
                recurring_transaction_id: recurring_id,
            })
            .await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            LedgerError::InvariantViolation(_)
        ));
    }
}
