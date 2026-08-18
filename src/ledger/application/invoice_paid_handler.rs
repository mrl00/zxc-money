use crate::credit_card::domain::events::InvoicePaid;
use crate::ledger::domain::repository::TransactionRepository;
use crate::ledger::domain::transaction::{Transaction, TransactionType};
use crate::provider::id::IdGenerator;
use crate::shared::errors::LedgerError;
use crate::shared::events::EventPublisher;
use crate::shared::ids::{AccountID, CategoryID, TransactionID};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Handles [`InvoicePaid`] events by creating an expense transaction on the
/// configured payment account.
pub struct InvoicePaidHandler<T: TransactionRepository, P: EventPublisher, I: IdGenerator> {
    transaction_repository: Arc<T>,
    event_publisher: Arc<P>,
    id_generator: Arc<I>,
    /// Maps CreditCardID → AccountID for payment
    payment_accounts: Arc<Mutex<HashMap<crate::shared::ids::CreditCardID, AccountID>>>,
    /// Category used for credit card payment transactions
    pub payment_category_id: Option<CategoryID>,
}

impl<T: TransactionRepository, P: EventPublisher, I: IdGenerator> InvoicePaidHandler<T, P, I> {
    pub fn new(
        transaction_repository: Arc<T>,
        event_publisher: Arc<P>,
        id_generator: Arc<I>,
    ) -> Self {
        Self {
            transaction_repository,
            event_publisher,
            id_generator,
            payment_accounts: Arc::new(Mutex::new(HashMap::new())),
            payment_category_id: None,
        }
    }

    /// Maps a credit card to the bank account used to pay its invoices.
    pub fn set_payment_account(
        &self,
        credit_card_id: crate::shared::ids::CreditCardID,
        account_id: AccountID,
    ) {
        let mut accounts = self.payment_accounts.lock().unwrap();
        accounts.insert(credit_card_id, account_id);
    }

    /// Creates an expense transaction for the invoice total on the mapped payment account.
    pub async fn handle(&self, event: &InvoicePaid) -> Result<(), LedgerError> {
        let payment_account_id = {
            let accounts = self.payment_accounts.lock().unwrap();
            accounts
                .get(&event.credit_card_id)
                .cloned()
                .ok_or_else(|| {
                    LedgerError::InvariantViolation(format!(
                        "no payment account configured for credit card {}",
                        event.credit_card_id
                    ))
                })?
        };

        let tx_id = TransactionID::from_uuid(self.id_generator.new_id());

        let mut tx = Transaction::new(
            tx_id,
            payment_account_id,
            TransactionType::Expense,
            event.total,
            format!("Invoice payment - {}", event.invoice_id),
            chrono::Utc::now().date_naive(),
        )?;

        if let Some(category_id) = self.payment_category_id {
            tx = tx.with_category(category_id)?;
        }

        tx.validate()?;

        self.transaction_repository.save(&tx).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::id::MockIdGenerator;
    use crate::shared::events::InMemoryEventDispatcher;
    use crate::shared::ids::{CreditCardID, InvoiceID};
    use crate::shared::mock::MockTransactionRepository;
    use crate::shared::money::{Currency, Money};
    use crate::shared::period::Period;

    #[tokio::test]
    async fn test_invoice_paid_creates_expense() {
        let tx_repo = Arc::new(MockTransactionRepository::new());
        let publisher = Arc::new(InMemoryEventDispatcher::new());
        let id_gen = Arc::new(MockIdGenerator::new(uuid::Uuid::new_v4()));

        let mut handler = InvoicePaidHandler::new(tx_repo.clone(), publisher, id_gen);
        handler.payment_category_id = Some(CategoryID::new());

        let card_id = CreditCardID::new();
        let account_id = AccountID::new();
        handler.set_payment_account(card_id, account_id);

        let event = InvoicePaid {
            invoice_id: InvoiceID::new(),
            credit_card_id: card_id,
            total: Money::new(15000, Currency::BRL),
            timestamp: chrono::Utc::now(),
        };

        handler.handle(&event).await.unwrap();

        let period = Period::new(
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        );
        let txns = tx_repo.find_by_account(account_id, period).await.unwrap();
        assert_eq!(txns.len(), 1);
        assert_eq!(txns[0].tx_type, TransactionType::Expense);
        assert_eq!(txns[0].amount, Money::new(15000, Currency::BRL));
    }

    #[tokio::test]
    async fn test_invoice_paid_no_payment_account() {
        let tx_repo = Arc::new(MockTransactionRepository::new());
        let publisher = Arc::new(InMemoryEventDispatcher::new());
        let id_gen = Arc::new(MockIdGenerator::new(uuid::Uuid::new_v4()));

        let handler = InvoicePaidHandler::new(tx_repo, publisher, id_gen);

        let event = InvoicePaid {
            invoice_id: InvoiceID::new(),
            credit_card_id: CreditCardID::new(),
            total: Money::new(15000, Currency::BRL),
            timestamp: chrono::Utc::now(),
        };

        let result = handler.handle(&event).await;
        assert!(matches!(result, Err(LedgerError::InvariantViolation(_))));
    }
}
