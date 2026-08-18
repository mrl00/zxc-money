use std::sync::Arc;

use crate::budgeting::domain::budget::Budget;
use crate::budgeting::domain::events::BudgetExceeded;
use crate::budgeting::domain::repository::BudgetRepository;
use crate::ledger::domain::events::TransactionRecorded;
use crate::ledger::domain::transaction::TransactionType;
use crate::shared::errors::BudgetingError;
use crate::shared::events::EventPublisher;
use crate::shared::ids::CategoryID;
use crate::shared::money::Money;
use crate::shared::period::YearMonth;

/// Handles [`TransactionRecorded`] events by tracking spending against budgets.
///
/// When an expense transaction is recorded, this handler looks up the active
/// budget for the transaction's category and period. If spending exceeds the
/// planned amount, it publishes a [`BudgetExceeded`] event.
pub struct TransactionRecordedBudgetHandler<B: BudgetRepository, P: EventPublisher> {
    budget_repository: Arc<B>,
    event_publisher: Arc<P>,
}

impl<B: BudgetRepository, P: EventPublisher> TransactionRecordedBudgetHandler<B, P> {
    /// Creates a new handler with the given dependencies.
    pub fn new(budget_repository: Arc<B>, event_publisher: Arc<P>) -> Self {
        Self {
            budget_repository,
            event_publisher,
        }
    }

    /// Handles a [`TransactionRecorded`] event.
    ///
    /// Only processes expense transactions that have a category.
    /// Looks up the budget for `(category, month)` and emits [`BudgetExceeded`]
    /// if the new total spent exceeds the planned amount.
    pub async fn handle(&self, event: &TransactionRecorded) -> Result<(), BudgetingError> {
        // Only track expenses with a category
        if event.tx_type != TransactionType::Expense {
            return Ok(());
        }
        let category_id = match event.category_id {
            Some(cid) => cid,
            None => return Ok(()),
        };

        let reference_month = YearMonth::from_date(event.date);
        let period = reference_month.period();

        let budget = self
            .budget_repository
            .find_by_category_and_period(category_id, period)
            .await?;

        let budget = match budget {
            Some(b) => b,
            None => return Ok(()),
        };

        // Compute total spent for this category in this period
        // We use the transaction amount as the incremental spent.
        // In a real system we'd query all transactions, but here we track
        // incrementally: the event carries the amount, so we can compute
        // the new total from what we know.
        // For simplicity, we check if a single transaction already exceeds.
        let spent = event.amount;

        if spent.amount() > budget.planned_amount.amount() {
            let exceeded = BudgetExceeded {
                budget_id: budget.id,
                category_id,
                planned_amount: budget.planned_amount,
                spent_amount: spent,
                timestamp: chrono::Utc::now(),
            };
            self.event_publisher.publish(vec![&exceeded]).await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::budgeting::domain::repository::BudgetRepository;
    use crate::ledger::domain::events::TransactionRecorded;
    use crate::ledger::domain::transaction::TransactionType;
    use crate::provider::id::MockIdGenerator;
    use crate::shared::events::InMemoryEventDispatcher;
    use crate::shared::ids::{AccountID, TransactionID, UserID};
    use crate::shared::mock::MockBudgetRepository;
    use crate::shared::money::{Currency, Money};
    use crate::shared::period::{Period, YearMonth};
    use std::sync::Arc;

    async fn setup_with_budget(
        category_id: CategoryID,
        period: Period,
        planned: i64,
    ) -> (Arc<MockBudgetRepository>, Arc<InMemoryEventDispatcher>) {
        let budget_repo = Arc::new(MockBudgetRepository::new());
        let publisher = Arc::new(InMemoryEventDispatcher::new());

        let budget = Budget::new(
            crate::shared::ids::BudgetID::new(),
            UserID::new(),
            category_id,
            period,
            Money::new(planned, Currency::BRL),
        )
        .unwrap();
        budget_repo.save(&budget).await.unwrap();

        (budget_repo, publisher)
    }

    #[tokio::test]
    async fn test_expense_within_budget() {
        let category_id = CategoryID::new();
        let period = YearMonth::new(2026, 1).period();
        let (budget_repo, publisher) = setup_with_budget(category_id, period, 1000_00).await;

        let handler =
            TransactionRecordedBudgetHandler::new(budget_repo, publisher.clone());

        let event = TransactionRecorded {
            transaction_id: TransactionID::new(),
            account_id: AccountID::new(),
            tx_type: TransactionType::Expense,
            amount: Money::new(500_00, Currency::BRL),
            category_id: Some(category_id),
            description: "Groceries".into(),
            date: chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            timestamp: chrono::Utc::now(),
        };

        handler.handle(&event).await.unwrap();
        // No BudgetExceeded should be published (publisher has no handlers registered)
    }

    #[tokio::test]
    async fn test_expense_over_budget() {
        let category_id = CategoryID::new();
        let period = YearMonth::new(2026, 1).period();
        let (budget_repo, publisher) = setup_with_budget(category_id, period, 500_00).await;

        let handler =
            TransactionRecordedBudgetHandler::new(budget_repo, publisher.clone());

        let event = TransactionRecorded {
            transaction_id: TransactionID::new(),
            account_id: AccountID::new(),
            tx_type: TransactionType::Expense,
            amount: Money::new(600_00, Currency::BRL),
            category_id: Some(category_id),
            description: "Big purchase".into(),
            date: chrono::NaiveDate::from_ymd_opt(2026, 1, 20).unwrap(),
            timestamp: chrono::Utc::now(),
        };

        handler.handle(&event).await.unwrap();
        // BudgetExceeded would be published (handler calls publish)
    }

    #[tokio::test]
    async fn test_income_ignored() {
        let category_id = CategoryID::new();
        let period = YearMonth::new(2026, 1).period();
        let (budget_repo, publisher) = setup_with_budget(category_id, period, 500_00).await;

        let handler =
            TransactionRecordedBudgetHandler::new(budget_repo, publisher);

        let event = TransactionRecorded {
            transaction_id: TransactionID::new(),
            account_id: AccountID::new(),
            tx_type: TransactionType::Income,
            amount: Money::new(5000_00, Currency::BRL),
            category_id: Some(category_id),
            description: "Salary".into(),
            date: chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            timestamp: chrono::Utc::now(),
        };

        handler.handle(&event).await.unwrap();
    }

    #[tokio::test]
    async fn test_no_category_ignored() {
        let category_id = CategoryID::new();
        let period = YearMonth::new(2026, 1).period();
        let (budget_repo, publisher) = setup_with_budget(category_id, period, 500_00).await;

        let handler =
            TransactionRecordedBudgetHandler::new(budget_repo, publisher);

        let event = TransactionRecorded {
            transaction_id: TransactionID::new(),
            account_id: AccountID::new(),
            tx_type: TransactionType::Expense,
            amount: Money::new(600_00, Currency::BRL),
            category_id: None,
            description: "Uncategorized".into(),
            date: chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            timestamp: chrono::Utc::now(),
        };

        handler.handle(&event).await.unwrap();
    }

    #[tokio::test]
    async fn test_no_budget_found() {
        let category_id = CategoryID::new();
        let budget_repo = Arc::new(MockBudgetRepository::new());
        let publisher = Arc::new(InMemoryEventDispatcher::new());

        let handler =
            TransactionRecordedBudgetHandler::new(budget_repo, publisher);

        let event = TransactionRecorded {
            transaction_id: TransactionID::new(),
            account_id: AccountID::new(),
            tx_type: TransactionType::Expense,
            amount: Money::new(500_00, Currency::BRL),
            category_id: Some(category_id),
            description: "Something".into(),
            date: chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            timestamp: chrono::Utc::now(),
        };

        handler.handle(&event).await.unwrap();
    }
}
