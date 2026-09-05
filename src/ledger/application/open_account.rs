use crate::ledger::domain::account::{Account, AccountType};
use crate::ledger::domain::events::AccountOpened;
use crate::ledger::domain::repository::AccountRepository;
use crate::provider::id::IdGenerator;
use crate::shared::errors::LedgerError;
use crate::shared::events::EventPublisher;
use crate::shared::ids::{AccountID, Principal};
use crate::shared::money::{Currency, Money};
use std::sync::Arc;

/// Command to open a new account.
pub struct OpenAccountCommand {
    pub principal: Principal,
    pub name: String,
    pub account_type: AccountType,
    pub currency: Currency,
    pub opening_balance: Money,
}

/// Handler that processes [`OpenAccountCommand`] requests.
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

    /// Executes the command: creates the account, persists it, and publishes [`AccountOpened`].
    ///
    /// Returns the new [`AccountID`] on success.
    pub async fn handle(&self, cmd: OpenAccountCommand) -> Result<AccountID, LedgerError> {
        let id = AccountID::from_uuid(self.id_generator.new_id());

        let account = Account::new(
            id,
            cmd.principal.user_id,
            cmd.name,
            cmd.account_type,
            cmd.currency,
            cmd.opening_balance,
        )?;

        self.repository.save(&account).await?;

        let event = AccountOpened {
            account_id: id,
            owner_id: cmd.principal.user_id,
            name: account.name.clone(),
            currency: account.currency,
            opening_balance: account.opening_balance,
            timestamp: chrono::Utc::now(),
        };
        self.event_publisher.publish(vec![&event]).await?;

        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::id::MockIdGenerator;
    use crate::shared::events::InMemoryEventDispatcher;
    use crate::shared::ids::{Principal, UserID};
    use crate::shared::mock::MockAccountRepository;

    #[tokio::test]
    async fn test_open_account() {
        let repo = Arc::new(MockAccountRepository::new());
        let publisher = Arc::new(InMemoryEventDispatcher::new());
        let id_gen = Arc::new(MockIdGenerator::new(uuid::Uuid::new_v4()));

        let handler = OpenAccountHandler::new(repo.clone(), publisher, id_gen);

        let cmd = OpenAccountCommand {
            principal: Principal::new(UserID::new()),
            name: "My Account".into(),
            account_type: AccountType::Checking,
            currency: Currency::BRL,
            opening_balance: Money::from_cents(10000, Currency::BRL),
        };

        let id = handler.handle(cmd).await.unwrap();
        let account = repo.find_by_id(id).await.unwrap();
        assert!(account.is_some());
        assert_eq!(account.unwrap().name, "My Account");
    }
}
