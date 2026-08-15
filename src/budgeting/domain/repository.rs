use crate::shared::errors::RepositoryError;
use crate::shared::ids::BudgetID;
use crate::shared::ids::GoalID;
use crate::shared::period::Period;
use async_trait::async_trait;

use super::budget::Budget;
use super::goal::FinancialGoal;

#[async_trait]
pub trait BudgetRepository: Send + Sync {
    async fn save(&self, budget: &Budget) -> Result<(), RepositoryError>;
    async fn find_by_id(&self, id: BudgetID) -> Result<Option<Budget>, RepositoryError>;
    async fn find_by_category_and_period(
        &self,
        category_id: crate::shared::ids::CategoryID,
        period: Period,
    ) -> Result<Option<Budget>, RepositoryError>;
    async fn delete(&self, id: BudgetID) -> Result<(), RepositoryError>;
}

#[async_trait]
pub trait GoalRepository: Send + Sync {
    async fn save(&self, goal: &FinancialGoal) -> Result<(), RepositoryError>;
    async fn find_by_id(&self, id: GoalID) -> Result<Option<FinancialGoal>, RepositoryError>;
    async fn delete(&self, id: GoalID) -> Result<(), RepositoryError>;
}
