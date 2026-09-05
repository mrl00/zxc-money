use std::sync::Arc;

use crate::bills::application::get_bills_by_month::{GetBillsByMonthHandler, GetBillsByMonthQuery};
use crate::bills::application::get_daily_bill_totals::{
    GetDailyBillTotalsHandler, GetDailyBillTotalsQuery,
};
use crate::bills::application::get_upcoming_bills::{
    GetUpcomingBillsHandler, GetUpcomingBillsQuery,
};
use crate::bills::application::mark_bill_paid::{MarkBillPaidCommand, MarkBillPaidHandler};
use crate::bills::application::schedule_bill::{ScheduleBillCommand, ScheduleBillHandler};
use crate::bills::domain::bill::Bill;
use crate::bills::domain::repository::BillRepository;
use crate::bills::projections::bill_calendar::DayBillTotal;
use crate::bills::projections::bill_calendar::{BillCalendarEntry, BillCalendarStore};
use crate::provider::id::IdGenerator;
use crate::shared::errors::BillsError;
use crate::shared::events::EventPublisher;
use crate::shared::ids::BillID;

/// Facade for the BillsReminder bounded context.
///
/// Aggregates all command and query handlers behind a single entry point.
///
/// # Example
///
/// ```ignore
/// let facade = BillsFacade::new(bill_repo, event_publisher, id_gen, calendar_store);
///
/// let bill_id = facade.schedule_bill(ScheduleBillCommand { ... }).await?;
/// let upcoming = facade.get_upcoming_bills(GetUpcomingBillsQuery { days: 7 }).await;
/// ```
pub struct BillsFacade<B: BillRepository, P: EventPublisher, I: IdGenerator> {
    schedule_bill: ScheduleBillHandler<B, P, I>,
    mark_bill_paid: MarkBillPaidHandler<B, P, I>,
    get_bills_by_month: GetBillsByMonthHandler,
    get_daily_bill_totals: GetDailyBillTotalsHandler,
    get_upcoming_bills: GetUpcomingBillsHandler,
    bill_repository: Arc<B>,
}

impl<B: BillRepository, P: EventPublisher, I: IdGenerator> BillsFacade<B, P, I> {
    /// Creates a new facade with shared dependencies.
    pub fn new(
        bill_repository: Arc<B>,
        event_publisher: Arc<P>,
        id_generator: Arc<I>,
        calendar_store: Arc<BillCalendarStore>,
    ) -> Self {
        Self {
            schedule_bill: ScheduleBillHandler::new(
                bill_repository.clone(),
                event_publisher.clone(),
                id_generator.clone(),
            ),
            mark_bill_paid: MarkBillPaidHandler::new(
                bill_repository.clone(),
                event_publisher,
                id_generator,
            ),
            get_bills_by_month: GetBillsByMonthHandler::new(calendar_store.clone()),
            get_daily_bill_totals: GetDailyBillTotalsHandler::new(calendar_store.clone()),
            get_upcoming_bills: GetUpcomingBillsHandler::new(calendar_store),
            bill_repository,
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

    /// Gets all bills due in a specific month. See [`GetBillsByMonthHandler`].
    pub async fn get_bills_by_month(&self, query: GetBillsByMonthQuery) -> Vec<BillCalendarEntry> {
        self.get_bills_by_month.handle(query).await
    }

    /// Gets daily aggregated bill totals for a month. See [`GetDailyBillTotalsHandler`].
    pub async fn get_daily_bill_totals(&self, query: GetDailyBillTotalsQuery) -> Vec<DayBillTotal> {
        self.get_daily_bill_totals.handle(query).await
    }

    /// Gets upcoming pending bills. See [`GetUpcomingBillsHandler`].
    pub async fn get_upcoming_bills(&self, query: GetUpcomingBillsQuery) -> Vec<BillCalendarEntry> {
        self.get_upcoming_bills.handle(query).await
    }

    /// Lists all pending bills from the repository.
    pub async fn get_pending_bills(&self) -> Result<Vec<Bill>, BillsError> {
        Ok(self.bill_repository.find_pending().await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bills::domain::bill::{BillStatus, RecurrenceRule};
    use crate::provider::id::MockIdGenerator;
    use crate::shared::events::InMemoryEventDispatcher;
    use crate::shared::ids::{AccountID, CategoryID, Principal, UserID};
    use crate::shared::mock::MockBillRepository;
    use crate::shared::money::{Currency, Money};

    fn setup() -> (
        Arc<MockBillRepository>,
        Arc<InMemoryEventDispatcher>,
        Arc<MockIdGenerator>,
        Arc<BillCalendarStore>,
    ) {
        (
            Arc::new(MockBillRepository::new()),
            Arc::new(InMemoryEventDispatcher::new()),
            Arc::new(MockIdGenerator::new(uuid::Uuid::new_v4())),
            Arc::new(BillCalendarStore::new()),
        )
    }

    #[tokio::test]
    async fn test_facade_schedule_bill() {
        let (repo, publisher, id_gen, calendar) = setup();
        let facade = BillsFacade::new(repo.clone(), publisher, id_gen, calendar);

        let cmd = ScheduleBillCommand {
            principal: Principal::new(UserID::new()),
            name: "Electricity".into(),
            amount: Some(Money::from_cents(25000, Currency::BRL)),
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
        let (repo, publisher, id_gen, calendar) = setup();
        let facade = BillsFacade::new(repo.clone(), publisher.clone(), id_gen.clone(), calendar);

        let owner = UserID::new();
        let schedule_cmd = ScheduleBillCommand {
            principal: Principal::new(owner),
            name: "Water".into(),
            amount: Some(Money::from_cents(8000, Currency::BRL)),
            due_date: chrono::NaiveDate::from_ymd_opt(2026, 4, 10).unwrap(),
            recurrence: None,
            category_id: CategoryID::new(),
        };

        let bill_id = facade.schedule_bill(schedule_cmd).await.unwrap();

        let paid_cmd = MarkBillPaidCommand {
            principal: Principal::new(owner),
            bill_id,
            account_id: AccountID::new(),
        };

        facade.mark_bill_paid(paid_cmd).await.unwrap();

        let bill = repo.find_by_id(bill_id).await.unwrap().unwrap();
        assert_eq!(bill.status, BillStatus::Paid);
    }

    #[tokio::test]
    async fn test_facade_get_bills_by_month() {
        let (_repo, publisher, id_gen, calendar) = setup();
        let facade = BillsFacade::new(
            Arc::new(MockBillRepository::new()),
            publisher,
            id_gen,
            calendar.clone(),
        );

        // Seed the calendar store
        let owner = UserID::new();
        calendar.handle_bill_scheduled(&crate::bills::domain::events::BillScheduled {
            bill_id: crate::shared::ids::BillID::new(),
            owner_id: owner,
            name: "Rent".into(),
            amount: Some(Money::from_cents(150000, Currency::BRL)),
            due_date: chrono::NaiveDate::from_ymd_opt(2026, 3, 5).unwrap(),
            timestamp: chrono::Utc::now(),
        });

        let result = facade
            .get_bills_by_month(GetBillsByMonthQuery {
                principal: Principal::new(owner),
                year: 2026,
                month: 3,
            })
            .await;
        assert_eq!(result.len(), 1);
    }
}
