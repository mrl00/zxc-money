use std::sync::Arc;

use crate::bills::domain::bill::Bill;
use crate::bills::domain::events::BillPaid;
use crate::bills::domain::events::BillScheduled;
use crate::bills::domain::repository::BillRepository;
use crate::provider::id::IdGenerator;
use crate::shared::errors::BillsError;
use crate::shared::events::EventPublisher;
use crate::shared::ids::{AccountID, BillID};

/// Command to mark a bill as paid.
pub struct MarkBillPaidCommand {
    /// The bill to mark as paid.
    pub bill_id: BillID,
    /// The account from which the bill was paid (used by the Ledger integration
    /// to create the corresponding expense transaction).
    pub account_id: AccountID,
}

/// Handler that processes [`MarkBillPaidCommand`] requests.
///
/// Transitions the bill from `Pending` to `Paid`,
/// publishes [`BillPaid`], and — if the bill is recurring — automatically
/// creates the next instance with an advanced due date, publishing
/// [`BillScheduled`] for it.
pub struct MarkBillPaidHandler<B: BillRepository, P: EventPublisher, I: IdGenerator> {
    bill_repository: Arc<B>,
    event_publisher: Arc<P>,
    id_generator: Arc<I>,
}

impl<B: BillRepository, P: EventPublisher, I: IdGenerator> MarkBillPaidHandler<B, P, I> {
    /// Creates a new handler with the given dependencies.
    pub fn new(bill_repository: Arc<B>, event_publisher: Arc<P>, id_generator: Arc<I>) -> Self {
        Self {
            bill_repository,
            event_publisher,
            id_generator,
        }
    }

    /// Executes the command: looks up the bill, validates the state transition,
    /// marks it paid, publishes events, and persists the changes.
    ///
    /// For recurring bills, a new [`Bill`] is created with the next due date
    /// computed via [`Bill::next_due_date`].
    pub async fn handle(&self, cmd: MarkBillPaidCommand) -> Result<(), BillsError> {
        let mut bill = self
            .bill_repository
            .find_by_id(cmd.bill_id)
            .await?
            .ok_or_else(|| BillsError::BillNotFound(cmd.bill_id.to_string()))?;

        bill.mark_paid()?;

        let paid_event = BillPaid {
            bill_id: cmd.bill_id,
            amount: bill.amount,
            account_id: cmd.account_id,
            category_id: bill.category_id,
            timestamp: chrono::Utc::now(),
        };
        self.event_publisher.publish(vec![&paid_event]).await?;

        if bill.recurrence.is_some()
            && let Some(next_date) = bill.next_due_date()
        {
            let new_id = BillID::from_uuid(self.id_generator.new_id());
            let new_bill = Bill::new(
                new_id,
                bill.owner_id,
                bill.name.clone(),
                bill.amount,
                next_date,
                bill.recurrence,
                bill.category_id,
            );

            let scheduled_event = BillScheduled {
                bill_id: new_id,
                name: new_bill.name.clone(),
                amount: new_bill.amount,
                due_date: next_date,
                timestamp: chrono::Utc::now(),
            };
            self.event_publisher.publish(vec![&scheduled_event]).await?;
            self.bill_repository.save(&new_bill).await?;
        }

        self.bill_repository.save(&bill).await?;
        Ok(())
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

    async fn schedule_bill(
        repo: &MockBillRepository,
        recurrence: Option<RecurrenceRule>,
    ) -> BillID {
        let bill = Bill::new(
            BillID::new(),
            UserID::new(),
            "Internet".into(),
            Some(Money::from_cents(99_90, Currency::BRL)),
            chrono::NaiveDate::from_ymd_opt(2026, 2, 10).unwrap(),
            recurrence,
            CategoryID::new(),
        );
        let id = bill.id;
        repo.save(&bill).await.unwrap();
        id
    }

    #[tokio::test]
    async fn test_mark_bill_paid_one_time() {
        let (repo, publisher, id_gen) = setup();
        let bill_id = schedule_bill(&repo, None).await;
        let handler = MarkBillPaidHandler::new(repo.clone(), publisher, id_gen);

        handler
            .handle(MarkBillPaidCommand {
                bill_id,
                account_id: AccountID::new(),
            })
            .await
            .unwrap();

        let bill = repo.find_by_id(bill_id).await.unwrap().unwrap();
        assert_eq!(bill.status, BillStatus::Paid);
    }

    #[tokio::test]
    async fn test_mark_bill_paid_recurring_creates_next() {
        let (repo, publisher, id_gen) = setup();
        let bill_id = schedule_bill(&repo, Some(RecurrenceRule::Monthly)).await;
        let handler = MarkBillPaidHandler::new(repo.clone(), publisher, id_gen);

        handler
            .handle(MarkBillPaidCommand {
                bill_id,
                account_id: AccountID::new(),
            })
            .await
            .unwrap();

        let original = repo.find_by_id(bill_id).await.unwrap().unwrap();
        assert_eq!(original.status, BillStatus::Paid);

        let all_bills = repo.find_by_owner(original.owner_id).await.unwrap();
        assert_eq!(all_bills.len(), 2);

        let next_bill = all_bills.iter().find(|b| b.id != bill_id).unwrap();
        assert_eq!(
            next_bill.due_date,
            chrono::NaiveDate::from_ymd_opt(2026, 3, 10).unwrap()
        );
        assert_eq!(next_bill.status, BillStatus::Pending);
    }

    #[tokio::test]
    async fn test_mark_bill_paid_not_found() {
        let (repo, publisher, id_gen) = setup();
        let handler = MarkBillPaidHandler::new(repo, publisher, id_gen);

        let result = handler
            .handle(MarkBillPaidCommand {
                bill_id: BillID::new(),
                account_id: AccountID::new(),
            })
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mark_bill_paid_already_paid() {
        let (repo, publisher, id_gen) = setup();
        let bill_id = schedule_bill(&repo, None).await;
        let handler = MarkBillPaidHandler::new(repo.clone(), publisher.clone(), id_gen.clone());

        handler
            .handle(MarkBillPaidCommand {
                bill_id,
                account_id: AccountID::new(),
            })
            .await
            .unwrap();

        let handler2 = MarkBillPaidHandler::new(repo, publisher, id_gen);
        let result = handler2
            .handle(MarkBillPaidCommand {
                bill_id,
                account_id: AccountID::new(),
            })
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mark_bill_paid_recurring_preserves_fields() {
        let (repo, publisher, id_gen) = setup();
        let owner = UserID::new();
        let bill = Bill::new(
            BillID::new(),
            owner,
            "Gym".into(),
            Some(Money::from_cents(15000, Currency::BRL)),
            chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(),
            Some(RecurrenceRule::Monthly),
            CategoryID::new(),
        );
        let id = bill.id;
        repo.save(&bill).await.unwrap();

        let handler = MarkBillPaidHandler::new(repo.clone(), publisher, id_gen);
        handler
            .handle(MarkBillPaidCommand {
                bill_id: id,
                account_id: AccountID::new(),
            })
            .await
            .unwrap();

        let all = repo.find_by_owner(owner).await.unwrap();
        let next = all.iter().find(|b| b.id != id).unwrap();
        assert_eq!(next.name, "Gym");
        assert_eq!(next.amount, Some(Money::from_cents(15000, Currency::BRL)));
        assert_eq!(next.recurrence, Some(RecurrenceRule::Monthly));
    }
}
