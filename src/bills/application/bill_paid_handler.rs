use std::sync::Arc;

use crate::bills::domain::events::BillPaid;
use crate::ledger::domain::repository::{AccountRepository, TransactionRepository};
use crate::ledger::domain::transaction::{Transaction, TransactionType};
use crate::provider::id::IdGenerator;
use crate::shared::errors::BillsError;
use crate::shared::ids::TransactionID;

/// Cross-context handler that reacts to [`BillPaid`] events by creating
/// a corresponding expense [`Transaction`]
/// in the Ledger.
pub struct BillPaidHandler<A: AccountRepository, T: TransactionRepository, I: IdGenerator> {
    account_repository: Arc<A>,
    transaction_repository: Arc<T>,
    id_generator: Arc<I>,
}

impl<A: AccountRepository, T: TransactionRepository, I: IdGenerator> BillPaidHandler<A, T, I> {
    /// Creates a new handler with the given dependencies.
    pub fn new(
        account_repository: Arc<A>,
        transaction_repository: Arc<T>,
        id_generator: Arc<I>,
    ) -> Self {
        Self {
            account_repository,
            transaction_repository,
            id_generator,
        }
    }

    /// Handles a [`BillPaid`] event by creating an expense transaction.
    ///
    /// # Validation
    ///
    /// - The event must carry a non-`None` amount.
    /// - The target account must exist.
    /// - The bill amount currency must match the account currency.
    pub async fn handle(&self, event: &BillPaid) -> Result<TransactionID, BillsError> {
        let amount = match event.amount {
            Some(a) => a,
            None => {
                return Err(BillsError::InvariantViolation(
                    "cannot create transaction for bill with no amount".into(),
                ));
            }
        };

        let account = self
            .account_repository
            .find_by_id(event.account_id)
            .await?
            .ok_or_else(|| {
                BillsError::InvariantViolation(format!(
                    "account {} not found for bill payment",
                    event.account_id
                ))
            })?;

        if amount.currency() != account.currency() {
            return Err(BillsError::InvariantViolation(format!(
                "currency mismatch: bill amount is {} but account is {}",
                amount.currency().code(),
                account.currency().code(),
            )));
        }

        let tx_id = TransactionID::from_uuid(self.id_generator.new_id());

        let mut tx = Transaction::new(
            tx_id,
            event.account_id,
            TransactionType::Expense,
            amount,
            format!("Bill payment - {}", event.bill_id),
            chrono::Utc::now().date_naive(),
        )?;

        tx = tx.with_category(event.category_id)?;
        tx.validate()?;

        self.transaction_repository.save(&tx).await?;

        Ok(tx_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bills::domain::events::BillPaid;
    use crate::ledger::domain::account::{Account, AccountType};
    use crate::provider::id::MockIdGenerator;
    use crate::shared::errors::BillsError;
    use crate::shared::ids::{AccountID, BillID, CategoryID, UserID};
    use crate::shared::mock::{MockAccountRepository, MockTransactionRepository};
    use crate::shared::money::{Currency, Money};
    use crate::shared::period::Period;

    fn setup() -> (
        Arc<MockAccountRepository>,
        Arc<MockTransactionRepository>,
        Arc<MockIdGenerator>,
    ) {
        (
            Arc::new(MockAccountRepository::new()),
            Arc::new(MockTransactionRepository::new()),
            Arc::new(MockIdGenerator::new(uuid::Uuid::new_v4())),
        )
    }

    async fn create_account(repo: &MockAccountRepository, currency: Currency) -> AccountID {
        let id = AccountID::new();
        let account = Account::new(
            id,
            UserID::new(),
            "Checking".into(),
            AccountType::Checking,
            currency,
            Money::from_cents(500000, currency),
        )
        .unwrap();
        repo.save(&account).await.unwrap();
        id
    }

    #[tokio::test]
    async fn test_bill_paid_creates_expense() {
        let (account_repo, tx_repo, id_gen) = setup();
        let account_id = create_account(&account_repo, Currency::BRL).await;

        let handler = BillPaidHandler::new(account_repo, tx_repo.clone(), id_gen);

        let event = BillPaid {
            bill_id: BillID::new(),
            owner_id: UserID::new(),
            amount: Some(Money::from_cents(99_90, Currency::BRL)),
            account_id,
            category_id: CategoryID::new(),
            timestamp: chrono::Utc::now(),
        };

        let tx_id = handler.handle(&event).await.unwrap();

        let period = Period::new(
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        );
        let txns = tx_repo.find_by_account(account_id, period).await.unwrap();
        assert_eq!(txns.len(), 1);
        assert_eq!(txns[0].tx_type, TransactionType::Expense);
        assert_eq!(txns[0].amount, Money::from_cents(99_90, Currency::BRL));
        assert_eq!(txns[0].id, tx_id);
    }

    #[tokio::test]
    async fn test_bill_paid_no_amount_fails() {
        let (account_repo, tx_repo, id_gen) = setup();
        let account_id = create_account(&account_repo, Currency::BRL).await;

        let handler = BillPaidHandler::new(account_repo, tx_repo, id_gen);

        let event = BillPaid {
            bill_id: BillID::new(),
            owner_id: UserID::new(),
            amount: None,
            account_id,
            category_id: CategoryID::new(),
            timestamp: chrono::Utc::now(),
        };

        let result = handler.handle(&event).await;
        assert!(matches!(result, Err(BillsError::InvariantViolation(_))));
    }

    #[tokio::test]
    async fn test_bill_paid_account_not_found() {
        let (account_repo, tx_repo, id_gen) = setup();

        let handler = BillPaidHandler::new(account_repo, tx_repo, id_gen);

        let event = BillPaid {
            bill_id: BillID::new(),
            owner_id: UserID::new(),
            amount: Some(Money::from_cents(5000, Currency::BRL)),
            account_id: AccountID::new(),
            category_id: CategoryID::new(),
            timestamp: chrono::Utc::now(),
        };

        let result = handler.handle(&event).await;
        assert!(matches!(result, Err(BillsError::InvariantViolation(_))));
    }

    #[tokio::test]
    async fn test_bill_paid_currency_mismatch() {
        let (account_repo, tx_repo, id_gen) = setup();
        let account_id = create_account(&account_repo, Currency::BRL).await;

        let handler = BillPaidHandler::new(account_repo, tx_repo, id_gen);

        let event = BillPaid {
            bill_id: BillID::new(),
            owner_id: UserID::new(),
            amount: Some(Money::from_cents(5000, Currency::USD)),
            account_id,
            category_id: CategoryID::new(),
            timestamp: chrono::Utc::now(),
        };

        let result = handler.handle(&event).await;
        assert!(matches!(result, Err(BillsError::InvariantViolation(_))));
    }

    #[tokio::test]
    async fn test_bill_paid_preserves_category() {
        let (account_repo, tx_repo, id_gen) = setup();
        let account_id = create_account(&account_repo, Currency::BRL).await;

        let handler = BillPaidHandler::new(account_repo, tx_repo.clone(), id_gen);
        let category = CategoryID::new();

        let event = BillPaid {
            bill_id: BillID::new(),
            owner_id: UserID::new(),
            amount: Some(Money::from_cents(20000, Currency::BRL)),
            account_id,
            category_id: category,
            timestamp: chrono::Utc::now(),
        };

        handler.handle(&event).await.unwrap();

        let period = Period::new(
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        );
        let txns = tx_repo.find_by_account(account_id, period).await.unwrap();
        assert_eq!(txns[0].category_id, Some(category));
    }
}
