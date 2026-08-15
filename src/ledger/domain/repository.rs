use crate::shared::errors::DomainError;
use crate::shared::ids::AccountID;
use async_trait::async_trait;

use super::account::Account;

#[async_trait]
pub trait AccountRepository: Send + Sync {
    async fn save(&self, account: &Account) -> Result<(), DomainError>;
    async fn find_by_id(&self, id: AccountID) -> Result<Option<Account>, DomainError>;
    async fn delete(&self, id: AccountID) -> Result<(), DomainError>;
}
