use std::sync::Arc;

use rust_decimal::prelude::ToPrimitive;

use crate::budgeting::application::annual_budget::{
    CreateAnnualBudgetCommand, CreateAnnualBudgetHandler,
};
use crate::budgeting::application::contribute_to_goal::{
    ContributeToGoalCommand, ContributeToGoalHandler,
};
use crate::budgeting::application::create_goal::{CreateGoalCommand, CreateGoalHandler};
use crate::budgeting::application::define_budget::{DefineBudgetCommand, DefineBudgetHandler};
use crate::budgeting::domain::repository::{BudgetRepository, GoalRepository};
use crate::provider::id::IdGenerator;
use crate::shared::errors::BudgetingError;
use crate::shared::events::EventPublisher;
use crate::shared::ids::{BudgetID, GoalID};
use crate::shared::money::Money;
use crate::shared::period::Period;

/// Result of a goal progress query.
#[derive(Debug, Clone)]
pub struct GoalProgress {
    pub goal_id: GoalID,
    pub name: String,
    pub target_amount: Money,
    pub current_amount: Money,
    pub pct_complete: f64,
    pub remaining: Money,
}

/// Facade aggregating all budgeting use cases for consumption by front-ends.
pub struct BudgetingFacade<
    B: BudgetRepository,
    G: GoalRepository,
    P: EventPublisher,
    Id: IdGenerator,
> {
    define_budget: DefineBudgetHandler<B, P, Id>,
    create_goal: CreateGoalHandler<G, Id>,
    contribute_to_goal: ContributeToGoalHandler<G, P>,
    create_annual_budgets: CreateAnnualBudgetHandler<B, P, Id>,
    budget_repository: Arc<B>,
    goal_repository: Arc<G>,
}

impl<B: BudgetRepository, G: GoalRepository, P: EventPublisher, Id: IdGenerator>
    BudgetingFacade<B, G, P, Id>
{
    /// Creates a new [`BudgetingFacade`] with the given dependency arcs.
    pub fn new(
        budget_repository: Arc<B>,
        goal_repository: Arc<G>,
        event_publisher: Arc<P>,
        id_generator: Arc<Id>,
    ) -> Self {
        Self {
            define_budget: DefineBudgetHandler::new(
                budget_repository.clone(),
                event_publisher.clone(),
                id_generator.clone(),
            ),
            create_goal: CreateGoalHandler::new(goal_repository.clone(), id_generator.clone()),
            contribute_to_goal: ContributeToGoalHandler::new(
                goal_repository.clone(),
                event_publisher.clone(),
            ),
            create_annual_budgets: CreateAnnualBudgetHandler::new(
                budget_repository.clone(),
                event_publisher,
                id_generator,
            ),
            budget_repository,
            goal_repository,
        }
    }

    // ── Commands ──────────────────────────────────────────────

    /// Defines a budget for a category within a period.
    pub async fn define_budget(
        &self,
        cmd: DefineBudgetCommand,
    ) -> Result<BudgetID, BudgetingError> {
        self.define_budget.handle(cmd).await
    }

    /// Creates 12 monthly budgets for a category in a given year.
    pub async fn create_annual_budgets(
        &self,
        cmd: CreateAnnualBudgetCommand,
    ) -> Result<Vec<BudgetID>, BudgetingError> {
        self.create_annual_budgets.handle(cmd).await
    }

    /// Creates a new financial goal.
    pub async fn create_goal(&self, cmd: CreateGoalCommand) -> Result<GoalID, BudgetingError> {
        self.create_goal.handle(cmd).await
    }

    /// Contributes toward a financial goal.
    pub async fn contribute_to_goal(
        &self,
        cmd: ContributeToGoalCommand,
    ) -> Result<(), BudgetingError> {
        self.contribute_to_goal.handle(cmd).await
    }

    // ── Queries ───────────────────────────────────────────────

    /// Returns the budget progress for a specific category and period.
    pub async fn get_budget_progress(
        &self,
        owner: crate::shared::ids::UserID,
        category_id: crate::shared::ids::CategoryID,
        period: Period,
    ) -> Result<
        Option<crate::budgeting::application::budget_progress::BudgetProgress>,
        BudgetingError,
    > {
        let budget = self
            .budget_repository
            .find_by_category_and_period(owner, category_id, period)
            .await?;

        match budget {
            Some(b) => Ok(Some(
                crate::budgeting::application::budget_progress::BudgetProgress::new(
                    b.id,
                    b.category_id,
                    b.period,
                    b.planned_amount,
                    Money::zero(b.planned_amount.currency()),
                ),
            )),
            None => Ok(None),
        }
    }

    /// Returns progress info for a specific financial goal.
    pub async fn get_goal_progress(
        &self,
        goal_id: GoalID,
    ) -> Result<Option<GoalProgress>, BudgetingError> {
        let goal = self.goal_repository.find_by_id(goal_id).await?;

        Ok(goal.map(|g| {
            let pct = if g.target_amount.is_zero() {
                0.0
            } else {
                (g.current_amount.amount().to_f64().unwrap_or(0.0)
                    / g.target_amount.amount().to_f64().unwrap_or(1.0))
                    * 100.0
            };
            let remaining = g
                .target_amount
                .checked_sub(g.current_amount)
                .unwrap_or_else(|_| Money::zero(g.target_amount.currency()));
            GoalProgress {
                goal_id: g.id,
                name: g.name,
                target_amount: g.target_amount,
                current_amount: g.current_amount,
                pct_complete: pct,
                remaining,
            }
        }))
    }
}
