use chrono::NaiveDate;

use crate::ledger::domain::events::RecurringTransactionCreated;
use crate::ledger::domain::recurring_transaction::{Frequency, RecurringTransaction};
use crate::ledger::domain::repository::RecurringTransactionRepository;
use crate::ledger::domain::transaction::TransactionType;
use crate::provider::id::IdGenerator;
use crate::shared::errors::LedgerError;
use crate::shared::events::EventPublisher;
use crate::shared::ids::{AccountID, CategoryID, RecurringTransactionID, UserID};
use crate::shared::money::Money;
use std::sync::Arc;

/// Command to create a new recurring transaction.
pub struct CreateRecurringTransactionCommand {
    pub owner_id: UserID,
    pub account_id: AccountID,
    pub tx_type: TransactionType,
    pub amount: Money,
    pub description: String,
    pub category_id: Option<CategoryID>,
    pub frequency: Frequency,
    pub next_date: NaiveDate,
}

/// Handler that processes [`CreateRecurringTransactionCommand`] requests.
pub struct CreateRecurringTransactionHandler<
    R: RecurringTransactionRepository,
    P: EventPublisher,
    I: IdGenerator,
> {
    repository: Arc<R>,
    event_publisher: Arc<P>,
    id_generator: Arc<I>,
}

impl<R: RecurringTransactionRepository, P: EventPublisher, I: IdGenerator>
    CreateRecurringTransactionHandler<R, P, I>
{
    pub fn new(repository: Arc<R>, event_publisher: Arc<P>, id_generator: Arc<I>) -> Self {
        Self {
            repository,
            event_publisher,
            id_generator,
        }
    }

    /// Executes the command: creates the recurring transaction, persists it,
    /// and publishes [`RecurringTransactionCreated`].
    ///
    /// Returns the new [`RecurringTransactionID`] on success.
    pub async fn handle(
        &self,
        cmd: CreateRecurringTransactionCommand,
    ) -> Result<RecurringTransactionID, LedgerError> {
        let id = RecurringTransactionID::from_uuid(self.id_generator.new_id());

        let recurring = RecurringTransaction::new(
            id,
            cmd.owner_id,
            cmd.account_id,
            cmd.tx_type,
            cmd.amount,
            cmd.description,
            cmd.category_id,
            cmd.frequency,
            cmd.next_date,
        )?;

        self.repository.save(&recurring).await?;

        let event = RecurringTransactionCreated {
            recurring_transaction_id: id,
            owner_id: cmd.owner_id,
            account_id: cmd.account_id,
            amount: cmd.amount,
            frequency: cmd.frequency,
            next_date: cmd.next_date,
            timestamp: chrono::Utc::now(),
        };
        self.event_publisher.publish(vec![&event]).await?;

        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::id::MockIdGenerator;
    use crate::shared::events::InMemoryEventDispatcher;
    use crate::shared::mock::MockRecurringTransactionRepository;
    use crate::shared::money::Currency;

    #[tokio::test]
    async fn test_create_recurring() {
        let repo = Arc::new(MockRecurringTransactionRepository::new());
        let publisher = Arc::new(InMemoryEventDispatcher::new());
        let id_gen = Arc::new(MockIdGenerator::new(uuid::Uuid::new_v4()));

        let handler = CreateRecurringTransactionHandler::new(repo, publisher, id_gen);

        let cmd = CreateRecurringTransactionCommand {
            owner_id: UserID::new(),
            account_id: AccountID::new(),
            tx_type: TransactionType::Expense,
            amount: Money::from_cents(5000, Currency::BRL),
            description: "Netflix".into(),
            category_id: Some(CategoryID::new()),
            frequency: Frequency::Monthly,
            next_date: chrono::NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
        };

        let id = handler.handle(cmd).await.unwrap();
        assert!(!id.as_uuid().is_nil());
    }
}
