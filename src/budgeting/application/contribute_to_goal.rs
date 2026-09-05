use std::sync::Arc;

use crate::budgeting::domain::events::{GoalAchieved, GoalContributed};
use crate::budgeting::domain::goal::GoalStatus;
use crate::budgeting::domain::repository::GoalRepository;
use crate::shared::errors::BudgetingError;
use crate::shared::events::EventPublisher;
use crate::shared::ids::GoalID;
use crate::shared::money::Money;

/// Command to contribute toward a financial goal.
pub struct ContributeToGoalCommand {
    pub goal_id: GoalID,
    pub amount: Money,
}

/// Handles [`ContributeToGoalCommand`] by contributing to a goal
/// and publishing [`GoalContributed`] / [`GoalAchieved`] events.
pub struct ContributeToGoalHandler<G: GoalRepository, P: EventPublisher> {
    goal_repository: Arc<G>,
    event_publisher: Arc<P>,
}

impl<G: GoalRepository, P: EventPublisher> ContributeToGoalHandler<G, P> {
    /// Creates a new [`ContributeToGoalHandler`] with the given dependencies.
    pub fn new(goal_repository: Arc<G>, event_publisher: Arc<P>) -> Self {
        Self {
            goal_repository,
            event_publisher,
        }
    }

    /// Executes the contribute-to-goal use case.
    ///
    /// # Errors
    ///
    /// Returns [`BudgetingError::GoalNotFound`] if the goal does not exist,
    /// or [`BudgetingError::InvariantViolation`] if the goal is not in progress.
    pub async fn handle(&self, cmd: ContributeToGoalCommand) -> Result<(), BudgetingError> {
        let mut goal = self
            .goal_repository
            .find_by_id(cmd.goal_id)
            .await?
            .ok_or_else(|| BudgetingError::GoalNotFound(cmd.goal_id.to_string()))?;

        let was_in_progress = goal.status == GoalStatus::InProgress;

        goal.contribute(cmd.amount)?;

        let contributed = GoalContributed {
            goal_id: cmd.goal_id,
            amount: cmd.amount,
            timestamp: chrono::Utc::now(),
        };

        let achieved = GoalAchieved {
            goal_id: cmd.goal_id,
            timestamp: chrono::Utc::now(),
        };

        let mut events: Vec<&dyn crate::shared::events::DomainEvent> = vec![&contributed];
        if was_in_progress && goal.status == GoalStatus::Achieved {
            events.push(&achieved);
        }

        self.goal_repository.save(&goal).await?;
        self.event_publisher.publish(events).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::budgeting::domain::goal::FinancialGoal;
    use crate::shared::events::InMemoryEventDispatcher;
    use crate::shared::mock::MockGoalRepository;
    use crate::shared::money::{Currency, Money};
    use rust_decimal::Decimal;
    use std::sync::Arc;

    async fn setup_with_goal(
        target: i64,
    ) -> (
        Arc<MockGoalRepository>,
        Arc<InMemoryEventDispatcher>,
        GoalID,
    ) {
        let goal_repo = Arc::new(MockGoalRepository::new());
        let publisher = Arc::new(InMemoryEventDispatcher::new());
        let goal = FinancialGoal::new(
            GoalID::new(),
            crate::shared::ids::UserID::new(),
            "Test Goal".into(),
            Money::from_cents(target, Currency::BRL),
            chrono::NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        );
        let goal_id = goal.id;
        goal_repo.save(&goal).await.unwrap();
        (goal_repo, publisher, goal_id)
    }

    #[tokio::test]
    async fn test_contribute_partial() {
        let (goal_repo, publisher, goal_id) = setup_with_goal(1000000).await;
        let handler = ContributeToGoalHandler::new(goal_repo.clone(), publisher);

        handler
            .handle(ContributeToGoalCommand {
                goal_id,
                amount: Money::from_cents(300000, Currency::BRL),
            })
            .await
            .unwrap();

        let goal = goal_repo.find_by_id(goal_id).await.unwrap().unwrap();
        assert_eq!(goal.current_amount.amount(), Decimal::from(3000));
        assert_eq!(goal.status, GoalStatus::InProgress);
    }

    #[tokio::test]
    async fn test_contribute_achieves_goal() {
        let (goal_repo, publisher, goal_id) = setup_with_goal(500000).await;
        let handler = ContributeToGoalHandler::new(goal_repo.clone(), publisher);

        handler
            .handle(ContributeToGoalCommand {
                goal_id,
                amount: Money::from_cents(500000, Currency::BRL),
            })
            .await
            .unwrap();

        let goal = goal_repo.find_by_id(goal_id).await.unwrap().unwrap();
        assert_eq!(goal.status, GoalStatus::Achieved);
    }

    #[tokio::test]
    async fn test_contribute_to_achieved_goal_fails() {
        let (goal_repo, publisher, goal_id) = setup_with_goal(500000).await;
        let handler = ContributeToGoalHandler::new(goal_repo.clone(), publisher);

        handler
            .handle(ContributeToGoalCommand {
                goal_id,
                amount: Money::from_cents(500000, Currency::BRL),
            })
            .await
            .unwrap();

        let result = handler
            .handle(ContributeToGoalCommand {
                goal_id,
                amount: Money::from_cents(100000, Currency::BRL),
            })
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_contribute_to_nonexistent_goal() {
        let goal_repo = Arc::new(MockGoalRepository::new());
        let publisher = Arc::new(InMemoryEventDispatcher::new());
        let handler = ContributeToGoalHandler::new(goal_repo, publisher);

        let result = handler
            .handle(ContributeToGoalCommand {
                goal_id: GoalID::new(),
                amount: Money::from_cents(100000, Currency::BRL),
            })
            .await;
        assert!(result.is_err());
    }
}
