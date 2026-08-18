use crate::shared::errors::RepositoryError;
use crate::shared::ids::{AccountID, CategoryID, RecurringTransactionID, TagID, UserID};
use crate::shared::period::Period;
use async_trait::async_trait;

use super::account::Account;
use super::category::{Category, Tag};
use super::recurring_transaction::RecurringTransaction;
use super::transaction::{Transaction, TransactionType};

/// Persistence operations for [`Account`] entities.
#[async_trait]
pub trait AccountRepository: Send + Sync {
    async fn save(&self, account: &Account) -> Result<(), RepositoryError>;
    async fn find_by_id(&self, id: AccountID) -> Result<Option<Account>, RepositoryError>;
    async fn find_by_owner(&self, owner: UserID) -> Result<Vec<Account>, RepositoryError>;
    async fn delete(&self, id: AccountID) -> Result<(), RepositoryError>;
}

/// Filter criteria for querying transactions.
pub struct TransactionFilter {
    pub tx_type: Option<TransactionType>,
    pub category_id: Option<CategoryID>,
    pub tags: Option<Vec<TagID>>,
    pub reconciled: Option<bool>,
}

/// Persistence operations for [`Transaction`] entities.
#[async_trait]
pub trait TransactionRepository: Send + Sync {
    async fn save(&self, transaction: &Transaction) -> Result<(), RepositoryError>;
    async fn find_by_id(
        &self,
        id: crate::shared::ids::TransactionID,
    ) -> Result<Option<Transaction>, RepositoryError>;
    async fn find_by_account(
        &self,
        account_id: AccountID,
        period: Period,
    ) -> Result<Vec<Transaction>, RepositoryError>;
    async fn find_by_account_filtered(
        &self,
        account_id: AccountID,
        period: Period,
        filter: &TransactionFilter,
    ) -> Result<Vec<Transaction>, RepositoryError>;
    async fn has_transactions(&self, account_id: AccountID) -> Result<bool, RepositoryError>;
    async fn delete(&self, id: crate::shared::ids::TransactionID) -> Result<(), RepositoryError>;
}

/// Persistence operations for [`Category`] entities.
#[async_trait]
pub trait CategoryRepository: Send + Sync {
    async fn save(&self, category: &Category) -> Result<(), RepositoryError>;
    async fn find_by_id(&self, id: CategoryID) -> Result<Option<Category>, RepositoryError>;
    async fn delete(&self, id: CategoryID) -> Result<(), RepositoryError>;
}

/// Persistence operations for [`Tag`] entities.
#[async_trait]
pub trait TagRepository: Send + Sync {
    async fn save(&self, tag: &Tag) -> Result<(), RepositoryError>;
    async fn find_by_id(&self, id: TagID) -> Result<Option<Tag>, RepositoryError>;
    async fn find_or_create(&self, name: String) -> Result<Tag, RepositoryError>;
    async fn delete(&self, id: TagID) -> Result<(), RepositoryError>;
}

/// Persistence operations for [`RecurringTransaction`] entities.
#[async_trait]
pub trait RecurringTransactionRepository: Send + Sync {
    async fn save(&self, recurring: &RecurringTransaction) -> Result<(), RepositoryError>;
    async fn find_by_id(
        &self,
        id: RecurringTransactionID,
    ) -> Result<Option<RecurringTransaction>, RepositoryError>;
    async fn find_by_owner(
        &self,
        owner: UserID,
    ) -> Result<Vec<RecurringTransaction>, RepositoryError>;
    async fn find_due(
        &self,
        today: chrono::NaiveDate,
    ) -> Result<Vec<RecurringTransaction>, RepositoryError>;
    async fn delete(&self, id: RecurringTransactionID) -> Result<(), RepositoryError>;
}
