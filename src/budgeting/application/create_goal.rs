use chrono::NaiveDate;
use std::sync::Arc;

use crate::budgeting::domain::goal::FinancialGoal;
use crate::budgeting::domain::repository::GoalRepository;
use crate::provider::id::IdGenerator;
use crate::shared::errors::BudgetingError;
use crate::shared::ids::{GoalID, UserID};
use crate::shared::money::Money;

/// Command to create a new financial goal.
pub struct CreateGoalCommand {
    pub owner_id: UserID,
    pub name: String,
    pub target_amount: Money,
    pub target_date: NaiveDate,
}

/// Handles [`CreateGoalCommand`] by creating a [`FinancialGoal`].
pub struct CreateGoalHandler<G: GoalRepository, Id: IdGenerator> {
    goal_repository: Arc<G>,
    id_generator: Arc<Id>,
}

impl<G: GoalRepository, Id: IdGenerator> CreateGoalHandler<G, Id> {
    /// Creates a new [`CreateGoalHandler`] with the given dependencies.
    pub fn new(goal_repository: Arc<G>, id_generator: Arc<Id>) -> Self {
        Self {
            goal_repository,
            id_generator,
        }
    }

    /// Executes the create-goal use case.
    pub async fn handle(&self, cmd: CreateGoalCommand) -> Result<GoalID, BudgetingError> {
        let id = GoalID::from_uuid(self.id_generator.new_id());
        let goal = FinancialGoal::new(
            id,
            cmd.owner_id,
            cmd.name,
            cmd.target_amount,
            cmd.target_date,
        );

        let goal_id = goal.id;
        self.goal_repository.save(&goal).await?;

        Ok(goal_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::id::MockIdGenerator;
    use crate::shared::mock::MockGoalRepository;
    use crate::shared::money::{Currency, Money};
    use std::sync::Arc;

    fn setup() -> (Arc<MockGoalRepository>, Arc<MockIdGenerator>) {
        let goal_repo = Arc::new(MockGoalRepository::new());
        let id_gen = Arc::new(MockIdGenerator::new(uuid::Uuid::new_v4()));
        (goal_repo, id_gen)
    }

    #[tokio::test]
    async fn test_create_goal_happy_path() {
        let (goal_repo, id_gen) = setup();
        let handler = CreateGoalHandler::new(goal_repo.clone(), id_gen);

        let cmd = CreateGoalCommand {
            owner_id: UserID::new(),
            name: "Emergency Fund".into(),
            target_amount: Money::new(10_000_00, Currency::BRL),
            target_date: NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        };

        let goal_id = handler.handle(cmd).await.unwrap();
        let goal = goal_repo.find_by_id(goal_id).await.unwrap().unwrap();
        assert_eq!(goal.name, "Emergency Fund");
        assert_eq!(goal.target_amount.amount(), 10_000_00);
    }

    #[tokio::test]
    async fn test_create_goal_with_linked_account() {
        let (goal_repo, id_gen) = setup();
        let handler = CreateGoalHandler::new(goal_repo.clone(), id_gen);

        let cmd = CreateGoalCommand {
            owner_id: UserID::new(),
            name: "Vacation".into(),
            target_amount: Money::new(5_000_00, Currency::BRL),
            target_date: NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
        };

        let goal_id = handler.handle(cmd).await.unwrap();
        let goal = goal_repo.find_by_id(goal_id).await.unwrap().unwrap();
        assert_eq!(goal.linked_account_id, None);
    }
}
