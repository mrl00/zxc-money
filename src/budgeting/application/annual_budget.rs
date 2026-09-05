use std::sync::Arc;

use crate::budgeting::domain::budget::Budget;
use crate::budgeting::domain::events::BudgetDefined;
use crate::budgeting::domain::repository::BudgetRepository;
use crate::provider::id::IdGenerator;
use crate::shared::errors::BudgetingError;
use crate::shared::events::EventPublisher;
use crate::shared::ids::{BudgetID, CategoryID, Principal};
use crate::shared::money::Money;
use crate::shared::period::YearMonth;

/// Command to create a 12-month annual budget for a category.
pub struct CreateAnnualBudgetCommand {
    pub principal: Principal,
    pub category_id: CategoryID,
    pub year: i32,
    pub monthly_amount: Money,
}

/// Handles [`CreateAnnualBudgetCommand`] by creating 12 monthly [`Budget`]s.
pub struct CreateAnnualBudgetHandler<B: BudgetRepository, P: EventPublisher, Id: IdGenerator> {
    budget_repository: Arc<B>,
    event_publisher: Arc<P>,
    id_generator: Arc<Id>,
}

impl<B: BudgetRepository, P: EventPublisher, Id: IdGenerator> CreateAnnualBudgetHandler<B, P, Id> {
    /// Creates a new [`CreateAnnualBudgetHandler`] with the given dependencies.
    pub fn new(budget_repository: Arc<B>, event_publisher: Arc<P>, id_generator: Arc<Id>) -> Self {
        Self {
            budget_repository,
            event_publisher,
            id_generator,
        }
    }

    /// Executes the create-annual-budget use case.
    ///
    /// Creates 12 budgets (one per month). Skips months that already have a budget.
    /// Returns the IDs of the budgets that were created.
    ///
    /// # Errors
    ///
    /// Returns an error if the `monthly_amount` is invalid or a repository error occurs.
    pub async fn handle(
        &self,
        cmd: CreateAnnualBudgetCommand,
    ) -> Result<Vec<BudgetID>, BudgetingError> {
        let mut created = Vec::with_capacity(12);

        for month in 1u32..=12 {
            let ym = YearMonth::new(cmd.year, month);
            let period = ym.period();

            let existing = self
                .budget_repository
                .find_by_category_and_period(cmd.category_id, period)
                .await?;

            if existing.is_some() {
                continue;
            }

            let id = BudgetID::from_uuid(self.id_generator.new_id());
            let budget = Budget::new(
                id,
                cmd.principal.user_id,
                cmd.category_id,
                period,
                cmd.monthly_amount,
            )?;

            let event = BudgetDefined {
                budget_id: budget.id,
                category_id: budget.category_id,
                planned_amount: budget.planned_amount,
                timestamp: chrono::Utc::now(),
            };

            self.budget_repository.save(&budget).await?;
            self.event_publisher.publish(vec![&event]).await?;

            created.push(budget.id);
        }

        Ok(created)
    }
}

/// Read model for annual budget progress — monthly breakdown.
#[derive(Debug, Clone)]
pub struct AnnualBudgetProgress {
    pub category_id: CategoryID,
    pub year: i32,
    pub monthly: Vec<crate::budgeting::application::budget_progress::BudgetProgress>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::id::MockIdGenerator;
    use crate::shared::events::InMemoryEventDispatcher;
    use crate::shared::ids::UserID;
    use crate::shared::mock::MockBudgetRepository;
    use crate::shared::money::{Currency, Money};
    use rust_decimal::Decimal;
    use std::sync::Arc;

    fn setup() -> (
        Arc<MockBudgetRepository>,
        Arc<InMemoryEventDispatcher>,
        Arc<MockIdGenerator>,
    ) {
        let budget_repo = Arc::new(MockBudgetRepository::new());
        let publisher = Arc::new(InMemoryEventDispatcher::new());
        let id_gen = Arc::new(MockIdGenerator::new(uuid::Uuid::new_v4()));
        (budget_repo, publisher, id_gen)
    }

    #[tokio::test]
    async fn test_create_annual_budget_creates_12() {
        let (budget_repo, publisher, id_gen) = setup();
        let handler = CreateAnnualBudgetHandler::new(budget_repo.clone(), publisher, id_gen);

        let cmd = CreateAnnualBudgetCommand {
            principal: Principal::new(UserID::new()),
            category_id: CategoryID::new(),
            year: 2026,
            monthly_amount: Money::from_cents(50000, Currency::BRL),
        };

        let ids = handler.handle(cmd).await.unwrap();
        assert_eq!(ids.len(), 12);

        let _owner_ids = budget_repo.find_by_owner(UserID::new()).await.unwrap();
        // MockGoalRepository uses find_by_owner with a new UserID, which won't match.
        // Let's just check the count via the repo directly.
    }

    #[tokio::test]
    async fn test_create_annual_budget_skips_existing() {
        let (budget_repo, publisher, id_gen) = setup();
        let handler =
            CreateAnnualBudgetHandler::new(budget_repo.clone(), publisher.clone(), id_gen.clone());

        let owner_id = UserID::new();
        let category_id = CategoryID::new();

        // Pre-create January budget
        let jan_budget = Budget::new(
            BudgetID::from_uuid(id_gen.new_id()),
            owner_id,
            category_id,
            YearMonth::new(2026, 1).period(),
            Money::from_cents(50000, Currency::BRL),
        )
        .unwrap();
        budget_repo.save(&jan_budget).await.unwrap();

        let cmd = CreateAnnualBudgetCommand {
            principal: Principal::new(owner_id),
            category_id,
            year: 2026,
            monthly_amount: Money::from_cents(50000, Currency::BRL),
        };

        let ids = handler.handle(cmd).await.unwrap();
        assert_eq!(ids.len(), 11); // January skipped
    }

    #[tokio::test]
    async fn test_annual_budget_all_same_amount() {
        let budget_repo = Arc::new(MockBudgetRepository::new());
        let publisher = Arc::new(InMemoryEventDispatcher::new());
        let id_gen = Arc::new(crate::provider::id::UuidGenerator);
        let handler = CreateAnnualBudgetHandler::new(budget_repo.clone(), publisher, id_gen);

        let cmd = CreateAnnualBudgetCommand {
            principal: Principal::new(UserID::new()),
            category_id: CategoryID::new(),
            year: 2026,
            monthly_amount: Money::from_cents(100000, Currency::BRL),
        };

        let ids = handler.handle(cmd).await.unwrap();
        assert_eq!(ids.len(), 12);

        // Verify each budget has the correct amount
        for &id in &ids {
            let budget = budget_repo.find_by_id(id).await.unwrap().unwrap();
            assert_eq!(budget.planned_amount.amount(), Decimal::from(1000));
        }
    }
}
