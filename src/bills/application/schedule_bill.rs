use std::sync::Arc;

use chrono::NaiveDate;

use crate::bills::domain::bill::Bill;
use crate::bills::domain::bill::RecurrenceRule;
use crate::bills::domain::events::BillScheduled;
use crate::bills::domain::repository::BillRepository;
use crate::provider::id::IdGenerator;
use crate::shared::errors::BillsError;
use crate::shared::events::EventPublisher;
use crate::shared::ids::{BillID, CategoryID, Principal};
use crate::shared::money::Money;

/// Command to schedule a new bill (recurring or one-time).
pub struct ScheduleBillCommand {
    /// The user who owns this bill.
    pub principal: Principal,
    /// Human-readable bill name (e.g. "Internet", "Rent").
    pub name: String,
    /// Monetary amount, or `None` for variable-amount bills.
    pub amount: Option<Money>,
    /// When the bill is due.
    pub due_date: NaiveDate,
    /// Recurrence pattern, or `None` for one-time bills.
    pub recurrence: Option<RecurrenceRule>,
    /// Category for budgeting/tracking purposes.
    pub category_id: CategoryID,
}

/// Handler that processes [`ScheduleBillCommand`] requests.
///
/// Creates a new [`Bill`] with `Pending` status, publishes
/// [`BillScheduled`], and persists the entity via [`BillRepository`].
pub struct ScheduleBillHandler<B: BillRepository, P: EventPublisher, I: IdGenerator> {
    bill_repository: Arc<B>,
    event_publisher: Arc<P>,
    id_generator: Arc<I>,
}

impl<B: BillRepository, P: EventPublisher, I: IdGenerator> ScheduleBillHandler<B, P, I> {
    /// Creates a new handler with the given dependencies.
    pub fn new(bill_repository: Arc<B>, event_publisher: Arc<P>, id_generator: Arc<I>) -> Self {
        Self {
            bill_repository,
            event_publisher,
            id_generator,
        }
    }

    /// Executes the command: creates the bill, publishes [`BillScheduled`],
    /// and persists it.
    ///
    /// Returns the new [`BillID`] on success.
    pub async fn handle(&self, cmd: ScheduleBillCommand) -> Result<BillID, BillsError> {
        let id = BillID::from_uuid(self.id_generator.new_id());

        let bill = Bill::new(
            id,
            cmd.principal.user_id,
            cmd.name.clone(),
            cmd.amount,
            cmd.due_date,
            cmd.recurrence,
            cmd.category_id,
        );

        let event = BillScheduled {
            bill_id: id,
            owner_id: cmd.principal.user_id,
            name: cmd.name,
            amount: cmd.amount,
            due_date: cmd.due_date,
            timestamp: chrono::Utc::now(),
        };

        self.event_publisher.publish(vec![&event]).await?;
        self.bill_repository.save(&bill).await?;

        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bills::domain::bill::{BillStatus, RecurrenceRule};
    use crate::provider::id::MockIdGenerator;
    use crate::shared::events::InMemoryEventDispatcher;
    use crate::shared::ids::UserID;
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
    async fn test_schedule_bill() {
        let (repo, publisher, id_gen) = setup();
        let handler = ScheduleBillHandler::new(repo.clone(), publisher.clone(), id_gen);

        let cmd = ScheduleBillCommand {
            principal: Principal::new(UserID::new()),
            name: "Internet".into(),
            amount: Some(Money::from_cents(99_90, Currency::BRL)),
            due_date: chrono::NaiveDate::from_ymd_opt(2026, 2, 10).unwrap(),
            recurrence: Some(RecurrenceRule::Monthly),
            category_id: CategoryID::new(),
        };

        let bill_id = handler.handle(cmd).await.unwrap();

        let bill = repo.find_by_id(bill_id).await.unwrap().unwrap();
        assert_eq!(bill.name, "Internet");
        assert_eq!(bill.amount, Some(Money::from_cents(99_90, Currency::BRL)));
        assert_eq!(bill.status, BillStatus::Pending);
        assert_eq!(bill.recurrence, Some(RecurrenceRule::Monthly));
    }

    #[tokio::test]
    async fn test_schedule_bill_no_amount() {
        let (repo, publisher, id_gen) = setup();
        let handler = ScheduleBillHandler::new(repo.clone(), publisher, id_gen);

        let cmd = ScheduleBillCommand {
            principal: Principal::new(UserID::new()),
            name: "Variable bill".into(),
            amount: None,
            due_date: chrono::NaiveDate::from_ymd_opt(2026, 3, 15).unwrap(),
            recurrence: None,
            category_id: CategoryID::new(),
        };

        let bill_id = handler.handle(cmd).await.unwrap();
        let bill = repo.find_by_id(bill_id).await.unwrap().unwrap();
        assert_eq!(bill.amount, None);
        assert_eq!(bill.recurrence, None);
    }

    #[tokio::test]
    async fn test_schedule_bill_one_time() {
        let (repo, publisher, id_gen) = setup();
        let handler = ScheduleBillHandler::new(repo.clone(), publisher, id_gen);

        let cmd = ScheduleBillCommand {
            principal: Principal::new(UserID::new()),
            name: "One-time fee".into(),
            amount: Some(Money::from_cents(50_00, Currency::BRL)),
            due_date: chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            recurrence: None,
            category_id: CategoryID::new(),
        };

        let bill_id = handler.handle(cmd).await.unwrap();
        let bill = repo.find_by_id(bill_id).await.unwrap().unwrap();
        assert_eq!(bill.recurrence, None);
    }

    #[tokio::test]
    async fn test_schedule_bill_weekly_recurrence() {
        let (repo, publisher, id_gen) = setup();
        let handler = ScheduleBillHandler::new(repo.clone(), publisher, id_gen);

        let cmd = ScheduleBillCommand {
            principal: Principal::new(UserID::new()),
            name: "Weekly cleaning".into(),
            amount: Some(Money::from_cents(30_00, Currency::BRL)),
            due_date: chrono::NaiveDate::from_ymd_opt(2026, 4, 7).unwrap(),
            recurrence: Some(RecurrenceRule::Weekly),
            category_id: CategoryID::new(),
        };

        let bill_id = handler.handle(cmd).await.unwrap();
        let bill = repo.find_by_id(bill_id).await.unwrap().unwrap();
        assert_eq!(bill.recurrence, Some(RecurrenceRule::Weekly));
    }
}
