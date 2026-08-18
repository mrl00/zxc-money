use std::sync::Arc;

use crate::budgeting::domain::events::{GoalAchieved, GoalContributed};
use crate::budgeting::domain::goal::GoalStatus;
use crate::budgeting::domain::repository::GoalRepository;
use crate::ledger::domain::events::TransactionRecorded;
use crate::shared::errors::BudgetingError;
use crate::shared::events::EventPublisher;
use crate::shared::ids::AccountID;
use crate::shared::money::Money;

/// Handles [`TransactionRecorded`] events by auto-contributing to linked goals.
///
/// When a transaction is recorded on an account that is linked to a goal,
/// this handler contributes the transaction amount toward the goal.
pub struct TransactionRecordedGoalHandler<G: GoalRepository, P: EventPublisher> {
    goal_repository: Arc<G>,
    event_publisher: Arc<P>,
}

impl<G: GoalRepository, P: EventPublisher> TransactionRecordedGoalHandler<G, P> {
    /// Creates a new handler with the given dependencies.
    pub fn new(goal_repository: Arc<G>, event_publisher: Arc<P>) -> Self {
        Self {
            goal_repository,
            event_publisher,
        }
    }

    /// Handles a [`TransactionRecorded`] event.
    ///
    /// Finds all in-progress goals linked to the transaction's account
    /// and contributes the transaction amount. Only income transactions
    /// trigger auto-contribution (savings come from income, not expenses).
    ///
    /// Publishes [`GoalContributed`] for each contribution and [`GoalAchieved`]
    /// if any goal reaches its target.
    pub async fn handle(&self, event: &TransactionRecorded) -> Result<(), BudgetingError> {
        // Only auto-contribute from income (savings from income)
        if !matches!(
            event.tx_type,
            crate::ledger::domain::transaction::TransactionType::Income
        ) {
            return Ok(());
        }

        let goals = self
            .goal_repository
            .find_by_linked_account(event.account_id)
            .await?;

        for mut goal in goals {
            if goal.status != GoalStatus::InProgress {
                continue;
            }

            let was_in_progress = goal.status == GoalStatus::InProgress;

            goal.contribute(event.amount)?;

            let contributed = GoalContributed {
                goal_id: goal.id,
                amount: event.amount,
                timestamp: chrono::Utc::now(),
            };

            let achieved = GoalAchieved {
                goal_id: goal.id,
                timestamp: chrono::Utc::now(),
            };

            let mut events: Vec<&dyn crate::shared::events::DomainEvent> =
                vec![&contributed];

            if was_in_progress && goal.status == GoalStatus::Achieved {
                events.push(&achieved);
            }

            self.goal_repository.save(&goal).await?;
            self.event_publisher.publish(events).await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::budgeting::domain::goal::FinancialGoal;
    use crate::ledger::domain::events::TransactionRecorded;
    use crate::ledger::domain::transaction::TransactionType;
    use crate::shared::events::InMemoryEventDispatcher;
    use crate::shared::ids::{GoalID, TransactionID, UserID};
    use crate::shared::mock::MockGoalRepository;
    use crate::shared::money::{Currency, Money};
    use std::sync::Arc;

    async fn setup_with_linked_goal(
        target: i64,
        account_id: AccountID,
    ) -> (Arc<MockGoalRepository>, Arc<InMemoryEventDispatcher>, GoalID) {
        let goal_repo = Arc::new(MockGoalRepository::new());
        let publisher = Arc::new(InMemoryEventDispatcher::new());
        let mut goal = FinancialGoal::new(
            GoalID::new(),
            UserID::new(),
            "Savings Goal".into(),
            Money::new(target, Currency::BRL),
            chrono::NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        )
        .with_linked_account(account_id);
        let goal_id = goal.id;
        goal_repo.save(&goal).await.unwrap();
        (goal_repo, publisher, goal_id)
    }

    #[tokio::test]
    async fn test_income_auto_contributes() {
        let account_id = AccountID::new();
        let (goal_repo, publisher, goal_id) = setup_with_linked_goal(10_000_00, account_id).await;

        let handler = TransactionRecordedGoalHandler::new(goal_repo.clone(), publisher);

        let event = TransactionRecorded {
            transaction_id: TransactionID::new(),
            account_id,
            tx_type: TransactionType::Income,
            amount: Money::new(3_000_00, Currency::BRL),
            category_id: None,
            description: "Salary".into(),
            date: chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            timestamp: chrono::Utc::now(),
        };

        handler.handle(&event).await.unwrap();

        let goal = goal_repo.find_by_id(goal_id).await.unwrap().unwrap();
        assert_eq!(goal.current_amount.amount(), 3_000_00);
    }

    #[tokio::test]
    async fn test_expense_does_not_contribute() {
        let account_id = AccountID::new();
        let (goal_repo, publisher, goal_id) = setup_with_linked_goal(10_000_00, account_id).await;

        let handler = TransactionRecordedGoalHandler::new(goal_repo.clone(), publisher);

        let event = TransactionRecorded {
            transaction_id: TransactionID::new(),
            account_id,
            tx_type: TransactionType::Expense,
            amount: Money::new(500_00, Currency::BRL),
            category_id: None,
            description: "Groceries".into(),
            date: chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            timestamp: chrono::Utc::now(),
        };

        handler.handle(&event).await.unwrap();

        let goal = goal_repo.find_by_id(goal_id).await.unwrap().unwrap();
        assert!(goal.current_amount.is_zero());
    }

    #[tokio::test]
    async fn test_no_linked_goals() {
        let goal_repo = Arc::new(MockGoalRepository::new());
        let publisher = Arc::new(InMemoryEventDispatcher::new());

        let handler = TransactionRecordedGoalHandler::new(goal_repo, publisher);

        let event = TransactionRecorded {
            transaction_id: TransactionID::new(),
            account_id: AccountID::new(),
            tx_type: TransactionType::Income,
            amount: Money::new(5000_00, Currency::BRL),
            category_id: None,
            description: "Salary".into(),
            date: chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            timestamp: chrono::Utc::now(),
        };

        handler.handle(&event).await.unwrap();
    }

    #[tokio::test]
    async fn test_income_achieves_goal() {
        let account_id = AccountID::new();
        let (goal_repo, publisher, goal_id) = setup_with_linked_goal(5_000_00, account_id).await;

        let handler = TransactionRecordedGoalHandler::new(goal_repo.clone(), publisher);

        let event = TransactionRecorded {
            transaction_id: TransactionID::new(),
            account_id,
            tx_type: TransactionType::Income,
            amount: Money::new(5_000_00, Currency::BRL),
            category_id: None,
            description: "Bonus".into(),
            date: chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            timestamp: chrono::Utc::now(),
        };

        handler.handle(&event).await.unwrap();

        let goal = goal_repo.find_by_id(goal_id).await.unwrap().unwrap();
        assert_eq!(goal.status, GoalStatus::Achieved);
    }
}
