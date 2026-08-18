use chrono::NaiveDate;

use crate::ledger::domain::recurring_transaction::RecurringTransaction;
use crate::ledger::domain::repository::RecurringTransactionRepository;
use crate::shared::errors::LedgerError;
use std::sync::Arc;

/// A recurring transaction that is due for generation.
pub struct PendingRecurring {
    pub recurring_transaction_id: crate::shared::ids::RecurringTransactionID,
    pub account_id: crate::shared::ids::AccountID,
    pub tx_type: crate::ledger::domain::transaction::TransactionType,
    pub amount: crate::shared::money::Money,
    pub description: String,
    pub next_date: NaiveDate,
}

/// Query that finds all recurring transactions due on or before a given date.
pub struct GeneratePendingRecurringQuery<R: RecurringTransactionRepository> {
    repository: Arc<R>,
}

impl<R: RecurringTransactionRepository> GeneratePendingRecurringQuery<R> {
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    /// Returns all due recurring transactions mapped to [`PendingRecurring`] items.
    pub async fn execute(&self, today: NaiveDate) -> Result<Vec<PendingRecurring>, LedgerError> {
        let due = self.repository.find_due(today).await?;

        Ok(due
            .into_iter()
            .map(|r| PendingRecurring {
                recurring_transaction_id: r.id,
                account_id: r.account_id,
                tx_type: r.tx_type,
                amount: r.amount,
                description: r.description,
                next_date: r.next_date,
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::domain::recurring_transaction::{Frequency, RecurringTransaction};
    use crate::ledger::domain::transaction::TransactionType;
    use crate::shared::ids::{AccountID, CategoryID, RecurringTransactionID, UserID};
    use crate::shared::mock::MockRecurringTransactionRepository;
    use crate::shared::money::{Currency, Money};

    #[tokio::test]
    async fn test_generate_pending() {
        let repo = Arc::new(MockRecurringTransactionRepository::new());

        let r1 = RecurringTransaction::new(
            RecurringTransactionID::new(),
            UserID::new(),
            AccountID::new(),
            TransactionType::Expense,
            Money::new(5000, Currency::BRL),
            "Netflix".into(),
            Some(CategoryID::new()),
            Frequency::Monthly,
            chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
        )
        .unwrap();
        let r2 = RecurringTransaction::new(
            RecurringTransactionID::new(),
            UserID::new(),
            AccountID::new(),
            TransactionType::Income,
            Money::new(300000, Currency::BRL),
            "Salary".into(),
            Some(CategoryID::new()),
            Frequency::Monthly,
            chrono::NaiveDate::from_ymd_opt(2026, 2, 5).unwrap(),
        )
        .unwrap();
        repo.save(&r1).await.unwrap();
        repo.save(&r2).await.unwrap();

        let query = GeneratePendingRecurringQuery::new(repo);
        let today = chrono::NaiveDate::from_ymd_opt(2026, 1, 20).unwrap();
        let pending = query.execute(today).await.unwrap();

        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].description, "Netflix");
    }

    #[tokio::test]
    async fn test_generate_pending_empty() {
        let repo = Arc::new(MockRecurringTransactionRepository::new());

        let r = RecurringTransaction::new(
            RecurringTransactionID::new(),
            UserID::new(),
            AccountID::new(),
            TransactionType::Expense,
            Money::new(5000, Currency::BRL),
            "Netflix".into(),
            Some(CategoryID::new()),
            Frequency::Monthly,
            chrono::NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
        )
        .unwrap();
        repo.save(&r).await.unwrap();

        let query = GeneratePendingRecurringQuery::new(repo);
        let today = chrono::NaiveDate::from_ymd_opt(2026, 1, 20).unwrap();
        let pending = query.execute(today).await.unwrap();

        assert_eq!(pending.len(), 0);
    }
}
