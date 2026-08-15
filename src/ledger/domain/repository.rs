use crate::shared::errors::RepositoryError;
use crate::shared::ids::{AccountID, UserID};
use crate::shared::period::Period;
use async_trait::async_trait;

use super::account::Account;
use super::category::{Category, Tag};

#[async_trait]
pub trait AccountRepository: Send + Sync {
    async fn save(&self, account: &Account) -> Result<(), RepositoryError>;
    async fn find_by_id(&self, id: AccountID) -> Result<Option<Account>, RepositoryError>;
    async fn find_by_owner(&self, owner: UserID) -> Result<Vec<Account>, RepositoryError>;
    async fn delete(&self, id: AccountID) -> Result<(), RepositoryError>;
}

use super::transaction::Transaction;

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
    async fn delete(&self, id: crate::shared::ids::TransactionID) -> Result<(), RepositoryError>;
}

use crate::shared::ids::CategoryID;

#[async_trait]
pub trait CategoryRepository: Send + Sync {
    async fn save(&self, category: &Category) -> Result<(), RepositoryError>;
    async fn find_by_id(&self, id: CategoryID) -> Result<Option<Category>, RepositoryError>;
    async fn delete(&self, id: CategoryID) -> Result<(), RepositoryError>;
}

use crate::shared::ids::TagID;

#[async_trait]
pub trait TagRepository: Send + Sync {
    async fn save(&self, tag: &Tag) -> Result<(), RepositoryError>;
    async fn find_by_id(&self, id: TagID) -> Result<Option<Tag>, RepositoryError>;
    async fn find_or_create(&self, name: String) -> Result<Tag, RepositoryError>;
    async fn delete(&self, id: TagID) -> Result<(), RepositoryError>;
}
