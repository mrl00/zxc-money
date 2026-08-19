use std::sync::Arc;

use crate::bills::application::mark_bill_paid::{MarkBillPaidCommand, MarkBillPaidHandler};
use crate::bills::application::schedule_bill::{ScheduleBillCommand, ScheduleBillHandler};
use crate::bills::domain::repository::BillRepository;
use crate::provider::id::IdGenerator;
use crate::shared::errors::BillsError;
use crate::shared::events::EventPublisher;
use crate::shared::ids::BillID;

/// Facade for the BillsReminder bounded context.
///
/// Aggregates [`ScheduleBillHandler`] and [`MarkBillPaidHandler`] behind a
/// single entry point, following the facade pattern used across all bounded
/// contexts in this codebase.
///
/// # Example
///
/// ```ignore
/// let facade = Facade::new(bill_repo, event_publisher, id_gen);
///
/// let bill_id = facade.schedule_bill(ScheduleBillCommand { ... }).await?;
/// facade.mark_bill_paid(MarkBillPaidCommand { bill_id, account_id }).await?;
/// ```
pub struct Facade<B: BillRepository, P: EventPublisher, I: IdGenerator> {
    schedule_bill: ScheduleBillHandler<B, P, I>,
    mark_bill_paid: MarkBillPaidHandler<B, P, I>,
}

impl<B: BillRepository, P: EventPublisher, I: IdGenerator> Facade<B, P, I> {
    /// Creates a new facade with shared dependencies.
    pub fn new(bill_repository: Arc<B>, event_publisher: Arc<P>, id_generator: Arc<I>) -> Self {
        Self {
            schedule_bill: ScheduleBillHandler::new(
                bill_repository.clone(),
                event_publisher.clone(),
                id_generator.clone(),
            ),
            mark_bill_paid: MarkBillPaidHandler::new(
                bill_repository,
                event_publisher,
                id_generator,
            ),
        }
    }

    /// Schedules a new bill. See [`ScheduleBillHandler`].
    pub async fn schedule_bill(&self, cmd: ScheduleBillCommand) -> Result<BillID, BillsError> {
        self.schedule_bill.handle(cmd).await
    }

    /// Marks a bill as paid. See [`MarkBillPaidHandler`].
    pub async fn mark_bill_paid(&self, cmd: MarkBillPaidCommand) -> Result<(), BillsError> {
        self.mark_bill_paid.handle(cmd).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bills::domain::bill::{BillStatus, RecurrenceRule};
    use crate::provider::id::MockIdGenerator;
    use crate::shared::events::InMemoryEventDispatcher;
    use crate::shared::ids::{AccountID, CategoryID, UserID};
    use crate::shared::mock::MockBillRepository;
    use crate::shared::money::{Currency, Money};

    fn setup() -> (
        Arc<MockBillRepository>,
        Arc<InMemoryEventDispatcher>,
        Arc<MockIdGenerator>,
    ) {
        (
            Arc::new(MockBillRepository::new()),
            Arc::new(InMemoryEventDispatcher::new()),
            Arc::new(MockIdGenerator::new(uuid::Uuid::new_v4())),
        )
    }

    #[tokio::test]
    async fn test_facade_schedule_bill() {
        let (repo, publisher, id_gen) = setup();
        let facade = Facade::new(repo.clone(), publisher, id_gen);

        let cmd = ScheduleBillCommand {
            owner_id: UserID::new(),
            name: "Electricity".into(),
            amount: Some(Money::new(250_00, Currency::BRL)),
            due_date: chrono::NaiveDate::from_ymd_opt(2026, 3, 5).unwrap(),
            recurrence: Some(RecurrenceRule::Monthly),
            category_id: CategoryID::new(),
        };

        let bill_id = facade.schedule_bill(cmd).await.unwrap();
        let bill = repo.find_by_id(bill_id).await.unwrap().unwrap();
        assert_eq!(bill.name, "Electricity");
        assert_eq!(bill.status, BillStatus::Pending);
    }

    #[tokio::test]
    async fn test_facade_mark_bill_paid() {
        let (repo, publisher, id_gen) = setup();
        let facade = Facade::new(repo.clone(), publisher.clone(), id_gen.clone());

        let schedule_cmd = ScheduleBillCommand {
            owner_id: UserID::new(),
            name: "Water".into(),
            amount: Some(Money::new(80_00, Currency::BRL)),
            due_date: chrono::NaiveDate::from_ymd_opt(2026, 4, 10).unwrap(),
            recurrence: None,
            category_id: CategoryID::new(),
        };

        let bill_id = facade.schedule_bill(schedule_cmd).await.unwrap();

        let paid_cmd = MarkBillPaidCommand {
            bill_id,
            account_id: AccountID::new(),
        };

        facade.mark_bill_paid(paid_cmd).await.unwrap();

        let bill = repo.find_by_id(bill_id).await.unwrap().unwrap();
        assert_eq!(bill.status, BillStatus::Paid);
    }
}
