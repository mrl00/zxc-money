use std::sync::Arc;

use crate::ledger::domain::account::AccountType;
use crate::ledger::domain::events::{AccountRenamed, AccountTypeChanged};
use crate::ledger::domain::repository::AccountRepository;
use crate::shared::errors::LedgerError;
use crate::shared::events::EventPublisher;
use crate::shared::ids::{AccountID, Principal};

/// Command to update an account's name and/or type.
pub struct UpdateAccountCommand {
    pub principal: Principal,
    pub account_id: AccountID,
    pub new_name: Option<String>,
    pub new_type: Option<AccountType>,
}

/// Handler that processes [`UpdateAccountCommand`] requests.
pub struct UpdateAccountHandler<A: AccountRepository, P: EventPublisher> {
    account_repository: Arc<A>,
    event_publisher: Arc<P>,
}

impl<A: AccountRepository, P: EventPublisher> UpdateAccountHandler<A, P> {
    pub fn new(account_repository: Arc<A>, event_publisher: Arc<P>) -> Self {
        Self {
            account_repository,
            event_publisher,
        }
    }

    /// Validates ownership, applies changes, persists, and publishes events.
    pub async fn handle(&self, cmd: UpdateAccountCommand) -> Result<(), LedgerError> {
        let mut account = self
            .account_repository
            .find_by_id(cmd.account_id)
            .await?
            .ok_or_else(|| LedgerError::AccountNotFound(cmd.account_id.to_string()))?;

        if account.owner_id != cmd.principal.user_id {
            return Err(LedgerError::Forbidden(
                "not the owner of this account".into(),
            ));
        }

        let mut events: Vec<&dyn crate::shared::events::DomainEvent> = Vec::new();
        let now = chrono::Utc::now();

        // We need to hold event structs alive while references exist
        let rename_event;
        let type_event;

        if let Some(new_name) = cmd.new_name {
            account.rename(new_name.clone());
            rename_event = AccountRenamed {
                account_id: cmd.account_id,
                new_name,
                timestamp: now,
            };
            events.push(&rename_event);
        }

        if let Some(new_type) = cmd.new_type {
            account.change_type(new_type);
            type_event = AccountTypeChanged {
                account_id: cmd.account_id,
                new_type,
                timestamp: now,
            };
            events.push(&type_event);
        }

        if events.is_empty() {
            return Ok(());
        }

        self.account_repository.save(&account).await?;
        self.event_publisher
            .publish(events)
            .await
            .map_err(LedgerError::Publish)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::domain::account::Account;
    use crate::shared::events::InMemoryEventDispatcher;
    use crate::shared::ids::{Principal, UserID};
    use crate::shared::mock::MockAccountRepository;
    use crate::shared::money::{Currency, Money};

    async fn setup() -> (
        Arc<MockAccountRepository>,
        Arc<InMemoryEventDispatcher>,
        AccountID,
        Principal,
    ) {
        let repo = Arc::new(MockAccountRepository::new());
        let publisher = Arc::new(InMemoryEventDispatcher::new());
        let owner = Principal::new(UserID::new());
        let account = Account::new(
            AccountID::new(),
            owner.user_id,
            "Checking".into(),
            AccountType::Checking,
            Currency::BRL,
            Money::from_cents(0, Currency::BRL),
        )
        .unwrap();
        let id = account.id;
        repo.save(&account).await.unwrap();
        (repo, publisher, id, owner)
    }

    #[tokio::test]
    async fn test_update_account_rename() {
        let (repo, publisher, id, owner) = setup().await;
        let handler = UpdateAccountHandler::new(repo, publisher);
        handler
            .handle(UpdateAccountCommand {
                account_id: id,
                principal: owner,
                new_name: Some("New Name".into()),
                new_type: None,
            })
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_update_account_wrong_owner() {
        let (repo, publisher, id, _owner) = setup().await;
        let handler = UpdateAccountHandler::new(repo, publisher);
        let result = handler
            .handle(UpdateAccountCommand {
                account_id: id,
                principal: Principal::new(UserID::new()),
                new_name: Some("Hacked".into()),
                new_type: None,
            })
            .await;
        assert!(matches!(result, Err(LedgerError::Forbidden(_))));
    }
}
