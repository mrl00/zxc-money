use crate::shared::errors::RepositoryError;
use crate::shared::ids::{CreditCardID, InvoiceID, UserID};
use crate::shared::period::YearMonth;
use async_trait::async_trait;

use super::card::CreditCard;
use super::invoice::Invoice;

/// Persistence interface for [`CreditCard`] aggregates.
#[async_trait]
pub trait CreditCardRepository: Send + Sync {
    async fn save(&self, card: &CreditCard) -> Result<(), RepositoryError>;
    async fn find_by_id(&self, id: CreditCardID) -> Result<Option<CreditCard>, RepositoryError>;
    async fn find_by_owner(&self, owner: UserID) -> Result<Vec<CreditCard>, RepositoryError>;
    async fn delete(&self, id: CreditCardID) -> Result<(), RepositoryError>;
}

/// Persistence interface for [`Invoice`] aggregates.
#[async_trait]
pub trait InvoiceRepository: Send + Sync {
    async fn save(&self, invoice: &Invoice) -> Result<(), RepositoryError>;
    async fn find_by_id(&self, id: InvoiceID) -> Result<Option<Invoice>, RepositoryError>;
    async fn find_open(
        &self,
        credit_card_id: CreditCardID,
    ) -> Result<Option<Invoice>, RepositoryError>;
    /// Find an invoice for a specific card and reference month.
    async fn find_by_card_and_month(
        &self,
        credit_card_id: CreditCardID,
        reference_month: YearMonth,
    ) -> Result<Option<Invoice>, RepositoryError>;
}
