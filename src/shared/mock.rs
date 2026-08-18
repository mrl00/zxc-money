use crate::credit_card::domain::card::CreditCard;
use crate::credit_card::domain::invoice::Invoice;
use crate::credit_card::domain::repository::{CreditCardRepository, InvoiceRepository};
use crate::shared::ids::{CreditCardID, InvoiceID};
use std::collections::HashMap;
use std::sync::Mutex;

use crate::shared::errors::RepositoryError;
use crate::shared::ids::{AccountID, TransactionID, UserID};
use crate::shared::period::Period;

use crate::ledger::domain::account::Account;
use crate::ledger::domain::repository::{
    AccountRepository, RecurringTransactionRepository, TransactionFilter, TransactionRepository,
};
use crate::ledger::domain::recurring_transaction::RecurringTransaction;
use crate::ledger::domain::transaction::Transaction;
use crate::shared::ids::RecurringTransactionID;

pub struct MockAccountRepository {
    accounts: Mutex<HashMap<AccountID, Account>>,
}

impl MockAccountRepository {
    pub fn new() -> Self {
        Self {
            accounts: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for MockAccountRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl AccountRepository for MockAccountRepository {
    async fn save(&self, account: &Account) -> Result<(), RepositoryError> {
        let mut accounts = self.accounts.lock().unwrap();
        accounts.insert(account.id, account.clone());
        Ok(())
    }

    async fn find_by_id(&self, id: AccountID) -> Result<Option<Account>, RepositoryError> {
        let accounts = self.accounts.lock().unwrap();
        Ok(accounts.get(&id).cloned())
    }

    async fn find_by_owner(&self, owner: UserID) -> Result<Vec<Account>, RepositoryError> {
        let accounts = self.accounts.lock().unwrap();
        let result: Vec<Account> = accounts
            .values()
            .filter(|a| a.owner_id == owner)
            .cloned()
            .collect();
        Ok(result)
    }

    async fn delete(&self, id: AccountID) -> Result<(), RepositoryError> {
        let mut accounts = self.accounts.lock().unwrap();
        accounts.remove(&id);
        Ok(())
    }
}

pub struct MockTransactionRepository {
    transactions: Mutex<HashMap<TransactionID, Transaction>>,
}

impl MockTransactionRepository {
    pub fn new() -> Self {
        Self {
            transactions: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for MockTransactionRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl TransactionRepository for MockTransactionRepository {
    async fn save(&self, transaction: &Transaction) -> Result<(), RepositoryError> {
        let mut transactions = self.transactions.lock().unwrap();
        transactions.insert(transaction.id, transaction.clone());
        Ok(())
    }

    async fn find_by_id(&self, id: TransactionID) -> Result<Option<Transaction>, RepositoryError> {
        let transactions = self.transactions.lock().unwrap();
        Ok(transactions.get(&id).cloned())
    }

    async fn find_by_account(
        &self,
        account_id: AccountID,
        period: Period,
    ) -> Result<Vec<Transaction>, RepositoryError> {
        let transactions = self.transactions.lock().unwrap();
        let result: Vec<Transaction> = transactions
            .values()
            .filter(|t| t.account_id == account_id && period.contains(t.date))
            .cloned()
            .collect();
        Ok(result)
    }

    async fn find_by_account_filtered(
        &self,
        account_id: AccountID,
        period: Period,
        filter: &TransactionFilter,
    ) -> Result<Vec<Transaction>, RepositoryError> {
        let transactions = self.transactions.lock().unwrap();
        let result: Vec<Transaction> = transactions
            .values()
            .filter(|t| {
                t.account_id == account_id
                    && period.contains(t.date)
                    && filter.tx_type.map_or(true, |ty| t.tx_type == ty)
                    && filter
                        .category_id
                        .map_or(true, |cid| t.category_id == Some(cid))
                    && filter.reconciled.map_or(true, |r| t.reconciled == r)
                    && filter.tags.as_ref().map_or(true, |required_tags| {
                        required_tags.iter().any(|tag| t.tags.contains(tag))
                    })
            })
            .cloned()
            .collect();
        Ok(result)
    }

    async fn has_transactions(&self, account_id: AccountID) -> Result<bool, RepositoryError> {
        let transactions = self.transactions.lock().unwrap();
        Ok(transactions.values().any(|t| t.account_id == account_id))
    }

    async fn delete(&self, id: TransactionID) -> Result<(), RepositoryError> {
        let mut transactions = self.transactions.lock().unwrap();
        transactions.remove(&id);
        Ok(())
    }
}

pub struct MockRecurringTransactionRepository {
    recurring: Mutex<HashMap<RecurringTransactionID, RecurringTransaction>>,
}

impl MockRecurringTransactionRepository {
    pub fn new() -> Self {
        Self {
            recurring: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for MockRecurringTransactionRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl RecurringTransactionRepository for MockRecurringTransactionRepository {
    async fn save(&self, recurring: &RecurringTransaction) -> Result<(), RepositoryError> {
        let mut map = self.recurring.lock().unwrap();
        map.insert(recurring.id, recurring.clone());
        Ok(())
    }

    async fn find_by_id(
        &self,
        id: RecurringTransactionID,
    ) -> Result<Option<RecurringTransaction>, RepositoryError> {
        let recurring = self.recurring.lock().unwrap();
        Ok(recurring.get(&id).cloned())
    }

    async fn find_by_owner(
        &self,
        owner: UserID,
    ) -> Result<Vec<RecurringTransaction>, RepositoryError> {
        let recurring = self.recurring.lock().unwrap();
        let result: Vec<RecurringTransaction> = recurring
            .values()
            .filter(|r| r.owner_id == owner)
            .cloned()
            .collect();
        Ok(result)
    }

    async fn find_due(
        &self,
        today: chrono::NaiveDate,
    ) -> Result<Vec<RecurringTransaction>, RepositoryError> {
        let recurring = self.recurring.lock().unwrap();
        let result: Vec<RecurringTransaction> = recurring
            .values()
            .filter(|r| r.is_due(today))
            .cloned()
            .collect();
        Ok(result)
    }

    async fn delete(&self, id: RecurringTransactionID) -> Result<(), RepositoryError> {
        let mut recurring = self.recurring.lock().unwrap();
        recurring.remove(&id);
        Ok(())
    }
}

pub struct MockCreditCardRepository {
    cards: Mutex<HashMap<CreditCardID, CreditCard>>,
}

impl MockCreditCardRepository {
    pub fn new() -> Self {
        Self {
            cards: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for MockCreditCardRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl CreditCardRepository for MockCreditCardRepository {
    async fn save(&self, card: &CreditCard) -> Result<(), RepositoryError> {
        let mut cards = self.cards.lock().unwrap();
        cards.insert(card.id, card.clone());
        Ok(())
    }

    async fn find_by_id(&self, id: CreditCardID) -> Result<Option<CreditCard>, RepositoryError> {
        let cards = self.cards.lock().unwrap();
        Ok(cards.get(&id).cloned())
    }

    async fn find_by_owner(&self, owner: UserID) -> Result<Vec<CreditCard>, RepositoryError> {
        let cards = self.cards.lock().unwrap();
        let result: Vec<CreditCard> = cards
            .values()
            .filter(|c| c.owner_id == owner)
            .cloned()
            .collect();
        Ok(result)
    }

    async fn delete(&self, id: CreditCardID) -> Result<(), RepositoryError> {
        let mut cards = self.cards.lock().unwrap();
        cards.remove(&id);
        Ok(())
    }
}

pub struct MockInvoiceRepository {
    invoices: Mutex<HashMap<InvoiceID, Invoice>>,
}

impl MockInvoiceRepository {
    pub fn new() -> Self {
        Self {
            invoices: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for MockInvoiceRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl InvoiceRepository for MockInvoiceRepository {
    async fn save(&self, invoice: &Invoice) -> Result<(), RepositoryError> {
        let mut invoices = self.invoices.lock().unwrap();
        invoices.insert(invoice.id, invoice.clone());
        Ok(())
    }

    async fn find_by_id(&self, id: InvoiceID) -> Result<Option<Invoice>, RepositoryError> {
        let invoices = self.invoices.lock().unwrap();
        Ok(invoices.get(&id).cloned())
    }

    async fn find_open(
        &self,
        credit_card_id: CreditCardID,
    ) -> Result<Option<Invoice>, RepositoryError> {
        let invoices = self.invoices.lock().unwrap();
        let result = invoices
            .values()
            .find(|i| i.credit_card_id == credit_card_id && i.is_open())
            .cloned();
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::money::{Currency, Money};

    #[tokio::test]
    async fn test_mock_account_repository() {
        let repo = MockAccountRepository::new();
        let owner_id = UserID::new();
        let account = Account::new(
            AccountID::new(),
            owner_id,
            "Test Account".into(),
            crate::ledger::domain::account::AccountType::Checking,
            Currency::BRL,
            Money::new(1000, Currency::BRL),
        )
        .unwrap();

        repo.save(&account).await.unwrap();
        let found = repo.find_by_id(account.id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Test Account");

        repo.delete(account.id).await.unwrap();
        let found = repo.find_by_id(account.id).await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_mock_account_find_by_owner() {
        let repo = MockAccountRepository::new();
        let owner1 = UserID::new();
        let owner2 = UserID::new();

        let account1 = Account::new(
            AccountID::new(),
            owner1,
            "Account 1".into(),
            crate::ledger::domain::account::AccountType::Checking,
            Currency::BRL,
            Money::new(1000, Currency::BRL),
        )
        .unwrap();
        let account2 = Account::new(
            AccountID::new(),
            owner1,
            "Account 2".into(),
            crate::ledger::domain::account::AccountType::Savings,
            Currency::BRL,
            Money::new(2000, Currency::BRL),
        )
        .unwrap();
        let account3 = Account::new(
            AccountID::new(),
            owner2,
            "Account 3".into(),
            crate::ledger::domain::account::AccountType::Checking,
            Currency::BRL,
            Money::new(3000, Currency::BRL),
        )
        .unwrap();

        repo.save(&account1).await.unwrap();
        repo.save(&account2).await.unwrap();
        repo.save(&account3).await.unwrap();

        let owner1_accounts = repo.find_by_owner(owner1).await.unwrap();
        assert_eq!(owner1_accounts.len(), 2);

        let owner2_accounts = repo.find_by_owner(owner2).await.unwrap();
        assert_eq!(owner2_accounts.len(), 1);
    }

    #[tokio::test]
    async fn test_mock_transaction_repository() {
        let repo = MockTransactionRepository::new();
        let account_id = AccountID::new();
        let transaction = Transaction::new(
            TransactionID::new(),
            account_id,
            crate::ledger::domain::transaction::TransactionType::Income,
            Money::new(500, Currency::BRL),
            "Salary".into(),
            chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
        )
        .unwrap();

        repo.save(&transaction).await.unwrap();
        let found = repo.find_by_id(transaction.id).await.unwrap();
        assert!(found.is_some());

        let period = Period::new(
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
        );
        let txns = repo.find_by_account(account_id, period).await.unwrap();
        assert_eq!(txns.len(), 1);

        repo.delete(transaction.id).await.unwrap();
        let found = repo.find_by_id(transaction.id).await.unwrap();
        assert!(found.is_none());
    }
}
