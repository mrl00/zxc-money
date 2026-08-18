use std::sync::Arc;

use crate::budgeting::domain::budget::Budget;
use crate::budgeting::domain::events::BudgetDefined;
use crate::budgeting::domain::repository::BudgetRepository;
use crate::provider::id::IdGenerator;
use crate::shared::errors::BudgetingError;
use crate::shared::events::EventPublisher;
use crate::shared::ids::{BudgetID, CategoryID, UserID};
use crate::shared::money::Money;
use crate::shared::period::Period;

/// Command to define a budget for a category within a period.
pub struct DefineBudgetCommand {
    pub owner_id: UserID,
    pub category_id: CategoryID,
    pub period: Period,
    pub planned_amount: Money,
}

/// Handles [`DefineBudgetCommand`] by creating a [`Budget`] and publishing [`BudgetDefined`].
///
/// Enforces the invariant that only one budget may exist per `(category_id, period)`.
pub struct DefineBudgetHandler<B: BudgetRepository, P: EventPublisher, Id: IdGenerator> {
    budget_repository: Arc<B>,
    event_publisher: Arc<P>,
    id_generator: Arc<Id>,
}

impl<B: BudgetRepository, P: EventPublisher, Id: IdGenerator>
    DefineBudgetHandler<B, P, Id>
{
    /// Creates a new [`DefineBudgetHandler`] with the given dependencies.
    pub fn new(
        budget_repository: Arc<B>,
        event_publisher: Arc<P>,
        id_generator: Arc<Id>,
    ) -> Self {
        Self {
            budget_repository,
            event_publisher,
            id_generator,
        }
    }

    /// Executes the define-budget use case.
    ///
    /// # Errors
    ///
    /// Returns [`BudgetingError::InvariantViolation`] if a budget already exists
    /// for the given `(category_id, period)`.
    pub async fn handle(&self, cmd: DefineBudgetCommand) -> Result<BudgetID, BudgetingError> {
        let existing = self
            .budget_repository
            .find_by_category_and_period(cmd.category_id, cmd.period)
            .await?;

        if existing.is_some() {
            return Err(BudgetingError::InvariantViolation(
                "a budget already exists for this category and period".into(),
            ));
        }

        let id = BudgetID::from_uuid(self.id_generator.new_id());
        let budget = Budget::new(id, cmd.owner_id, cmd.category_id, cmd.period, cmd.planned_amount)?;

        let event = BudgetDefined {
            budget_id: budget.id,
            category_id: budget.category_id,
            planned_amount: budget.planned_amount,
            timestamp: chrono::Utc::now(),
        };

        self.budget_repository.save(&budget).await?;
        self.event_publisher.publish(vec![&event]).await?;

        Ok(budget.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::id::MockIdGenerator;
    use crate::shared::events::InMemoryEventDispatcher;
    use crate::shared::mock::MockBudgetRepository;
    use crate::shared::money::{Currency, Money};
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
    async fn test_define_budget_happy_path() {
        let (budget_repo, publisher, id_gen) = setup();
        let handler = DefineBudgetHandler::new(budget_repo.clone(), publisher.clone(), id_gen);

        let cmd = DefineBudgetCommand {
            owner_id: UserID::new(),
            category_id: CategoryID::new(),
            period: Period::new(
                chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                chrono::NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            ),
            planned_amount: Money::new(500_00, Currency::BRL),
        };

        let budget_id = handler.handle(cmd).await.unwrap();
        let saved = budget_repo.find_by_id(budget_id).await.unwrap();
        assert!(saved.is_some());
        assert_eq!(saved.unwrap().planned_amount.amount(), 500_00);
    }

    #[tokio::test]
    async fn test_define_budget_duplicate_category_period() {
        let (budget_repo, publisher, id_gen) = setup();
        let handler = DefineBudgetHandler::new(budget_repo.clone(), publisher.clone(), id_gen.clone());

        let owner_id = UserID::new();
        let category_id = CategoryID::new();
        let period = Period::new(
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
        );

        let cmd1 = DefineBudgetCommand {
            owner_id,
            category_id,
            period,
            planned_amount: Money::new(500_00, Currency::BRL),
        };
        handler.handle(cmd1).await.unwrap();

        let cmd2 = DefineBudgetCommand {
            owner_id,
            category_id,
            period,
            planned_amount: Money::new(600_00, Currency::BRL),
        };
        let result = handler.handle(cmd2).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_define_budget_zero_amount() {
        let (budget_repo, publisher, id_gen) = setup();
        let handler = DefineBudgetHandler::new(budget_repo, publisher, id_gen);

        let cmd = DefineBudgetCommand {
            owner_id: UserID::new(),
            category_id: CategoryID::new(),
            period: Period::new(
                chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                chrono::NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            ),
            planned_amount: Money::new(0, Currency::BRL),
        };

        let result = handler.handle(cmd).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_define_budget_different_categories() {
        let budget_repo = Arc::new(MockBudgetRepository::new());
        let publisher = Arc::new(InMemoryEventDispatcher::new());
        let id_gen = Arc::new(crate::provider::id::UuidGenerator);
        let handler =
            DefineBudgetHandler::new(budget_repo.clone(), publisher, id_gen);

        let period = Period::new(
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
        );

        let id1 = handler
            .handle(DefineBudgetCommand {
                owner_id: UserID::new(),
                category_id: CategoryID::new(),
                period,
                planned_amount: Money::new(500_00, Currency::BRL),
            })
            .await
            .unwrap();

        let id2 = handler
            .handle(DefineBudgetCommand {
                owner_id: UserID::new(),
                category_id: CategoryID::new(),
                period,
                planned_amount: Money::new(300_00, Currency::BRL),
            })
            .await
            .unwrap();

        assert_ne!(id1, id2);
        assert_eq!(budget_repo.find_by_id(id1).await.unwrap().unwrap().planned_amount.amount(), 500_00);
        assert_eq!(budget_repo.find_by_id(id2).await.unwrap().unwrap().planned_amount.amount(), 300_00);
    }
}
