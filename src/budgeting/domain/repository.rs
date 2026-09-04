use crate::shared::errors::RepositoryError;
use crate::shared::ids::{BudgetID, GoalID, UserID};
use crate::shared::period::Period;
use async_trait::async_trait;

use super::budget::Budget;
use super::goal::FinancialGoal;

/// Persistence trait for [`Budget`] entities.
#[async_trait]
pub trait BudgetRepository: Send + Sync {
    /// Persists a budget.
    async fn save(&self, budget: &Budget) -> Result<(), RepositoryError>;
    /// Retrieves a budget by its unique identifier.
    async fn find_by_id(&self, id: BudgetID) -> Result<Option<Budget>, RepositoryError>;
    /// Retrieves all budgets belonging to a specific user.
    async fn find_by_owner(&self, owner: UserID) -> Result<Vec<Budget>, RepositoryError>;
    /// Retrieves the budget for a specific category and period, if any.
    async fn find_by_category_and_period(
        &self,
        category_id: crate::shared::ids::CategoryID,
        period: Period,
    ) -> Result<Option<Budget>, RepositoryError>;
    /// Deletes a budget by its unique identifier.
    async fn delete(&self, id: BudgetID) -> Result<(), RepositoryError>;
}

/// Persistence trait for [`FinancialGoal`] entities.
#[async_trait]
pub trait GoalRepository: Send + Sync {
    /// Persists a financial goal.
    async fn save(&self, goal: &FinancialGoal) -> Result<(), RepositoryError>;
    /// Retrieves a goal by its unique identifier.
    async fn find_by_id(&self, id: GoalID) -> Result<Option<FinancialGoal>, RepositoryError>;
    /// Retrieves all goals belonging to a specific user.
    async fn find_by_owner(&self, owner: UserID) -> Result<Vec<FinancialGoal>, RepositoryError>;
    /// Retrieves all in-progress goals linked to a specific account.
    async fn find_by_linked_account(
        &self,
        account_id: crate::shared::ids::AccountID,
    ) -> Result<Vec<FinancialGoal>, RepositoryError>;
    /// Deletes a goal by its unique identifier.
    async fn delete(&self, id: GoalID) -> Result<(), RepositoryError>;
}
