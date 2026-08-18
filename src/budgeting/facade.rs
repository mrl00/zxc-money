use std::sync::Arc;

use crate::budgeting::application::contribute_to_goal::{
    ContributeToGoalCommand, ContributeToGoalHandler,
};
use crate::budgeting::application::create_goal::{CreateGoalCommand, CreateGoalHandler};
use crate::budgeting::application::define_budget::{DefineBudgetCommand, DefineBudgetHandler};
use crate::budgeting::domain::repository::{BudgetRepository, GoalRepository};
use crate::provider::id::IdGenerator;
use crate::shared::errors::BudgetingError;
use crate::shared::events::EventPublisher;

/// Facade aggregating all budgeting use cases for consumption by front-ends.
///
/// Follows the Facade pattern from the DDD architecture spec (section 9.3).
pub struct Facade<B: BudgetRepository, G: GoalRepository, P: EventPublisher, Id: IdGenerator> {
    define_budget: DefineBudgetHandler<B, P, Id>,
    create_goal: CreateGoalHandler<G, Id>,
    contribute_to_goal: ContributeToGoalHandler<G, P>,
}

impl<B: BudgetRepository, G: GoalRepository, P: EventPublisher, Id: IdGenerator>
    Facade<B, G, P, Id>
{
    /// Creates a new [`Facade`] with the given dependency arcs.
    pub fn new(
        budget_repository: Arc<B>,
        goal_repository: Arc<G>,
        event_publisher: Arc<P>,
        id_generator: Arc<Id>,
    ) -> Self {
        Self {
            define_budget: DefineBudgetHandler::new(
                budget_repository,
                event_publisher.clone(),
                id_generator.clone(),
            ),
            create_goal: CreateGoalHandler::new(goal_repository.clone(), id_generator),
            contribute_to_goal: ContributeToGoalHandler::new(goal_repository, event_publisher),
        }
    }

    /// Defines a budget for a category within a period.
    pub async fn define_budget(
        &self,
        cmd: DefineBudgetCommand,
    ) -> Result<crate::shared::ids::BudgetID, BudgetingError> {
        self.define_budget.handle(cmd).await
    }

    /// Creates a new financial goal.
    pub async fn create_goal(
        &self,
        cmd: CreateGoalCommand,
    ) -> Result<crate::shared::ids::GoalID, BudgetingError> {
        self.create_goal.handle(cmd).await
    }

    /// Contributes toward a financial goal.
    pub async fn contribute_to_goal(
        &self,
        cmd: ContributeToGoalCommand,
    ) -> Result<(), BudgetingError> {
        self.contribute_to_goal.handle(cmd).await
    }
}
