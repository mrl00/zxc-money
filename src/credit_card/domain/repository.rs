use crate::shared::errors::RepositoryError;
use crate::shared::ids::{CreditCardID, InvoiceID, UserID};
use async_trait::async_trait;

use super::card::CreditCard;
use super::invoice::Invoice;

#[async_trait]
pub trait CreditCardRepository: Send + Sync {
    async fn save(&self, card: &CreditCard) -> Result<(), RepositoryError>;
    async fn find_by_id(&self, id: CreditCardID) -> Result<Option<CreditCard>, RepositoryError>;
    async fn find_by_owner(&self, owner: UserID) -> Result<Vec<CreditCard>, RepositoryError>;
    async fn delete(&self, id: CreditCardID) -> Result<(), RepositoryError>;
}

#[async_trait]
pub trait InvoiceRepository: Send + Sync {
    async fn save(&self, invoice: &Invoice) -> Result<(), RepositoryError>;
    async fn find_by_id(&self, id: InvoiceID) -> Result<Option<Invoice>, RepositoryError>;
    async fn find_open(
        &self,
        credit_card_id: CreditCardID,
    ) -> Result<Option<Invoice>, RepositoryError>;
}
