use crate::ledger::domain::account::{Account, AccountType};
use crate::ledger::domain::events::AccountOpened;
use crate::ledger::domain::repository::AccountRepository;
use crate::provider::id::IdGenerator;
use crate::shared::errors::DomainError;
use crate::shared::events::EventPublisher;
use crate::shared::ids::AccountID;
use crate::shared::money::{Currency, Money};
use std::sync::Arc;

pub struct OpenAccountCommand {
    pub name: String,
    pub account_type: AccountType,
    pub currency: Currency,
    pub opening_balance: Money,
}

pub struct OpenAccountHandler<R: AccountRepository, P: EventPublisher, I: IdGenerator> {
    repository: Arc<R>,
    event_publisher: Arc<P>,
    id_generator: Arc<I>,
}

impl<R: AccountRepository, P: EventPublisher, I: IdGenerator> OpenAccountHandler<R, P, I> {
    pub fn new(repository: Arc<R>, event_publisher: Arc<P>, id_generator: Arc<I>) -> Self {
        Self {
            repository,
            event_publisher,
            id_generator,
        }
    }

    pub async fn handle(&self, cmd: OpenAccountCommand) -> Result<AccountID, DomainError> {
        let id = AccountID::from_uuid(self.id_generator.new_id());

        let account = Account::new(
            id,
            cmd.name,
            cmd.account_type,
            cmd.currency,
            cmd.opening_balance,
        );

        self.repository.save(&account).await?;

        let event = AccountOpened {
            account_id: id,
            name: account.name.clone(),
            currency: account.currency,
            opening_balance: account.opening_balance,
            timestamp: chrono::Utc::now(),
        };
        self.event_publisher.publish(&event);

        Ok(id)
    }
}
