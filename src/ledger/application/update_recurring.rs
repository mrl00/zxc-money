use std::sync::Arc;

use crate::ledger::domain::events::{
    RecurringTransactionCancelled, RecurringTransactionPaused, RecurringTransactionResumed,
};
use crate::ledger::domain::repository::RecurringTransactionRepository;
use crate::shared::errors::LedgerError;
use crate::shared::events::EventPublisher;
use crate::shared::ids::{Principal, RecurringTransactionID};

/// Command to pause a recurring transaction.
pub struct PauseRecurringCommand {
    pub recurring_id: RecurringTransactionID,
    pub principal: Principal,
}

/// Command to resume a paused recurring transaction.
pub struct ResumeRecurringCommand {
    pub recurring_id: RecurringTransactionID,
    pub principal: Principal,
}

/// Command to cancel a recurring transaction.
pub struct CancelRecurringCommand {
    pub recurring_id: RecurringTransactionID,
    pub principal: Principal,
}

/// Handler for pause/resume/cancel operations on recurring transactions.
pub struct UpdateRecurringHandler<R: RecurringTransactionRepository, P: EventPublisher> {
    recurring_repository: Arc<R>,
    event_publisher: Arc<P>,
}

impl<R: RecurringTransactionRepository, P: EventPublisher> UpdateRecurringHandler<R, P> {
    pub fn new(recurring_repository: Arc<R>, event_publisher: Arc<P>) -> Self {
        Self {
            recurring_repository,
            event_publisher,
        }
    }

    /// Pauses a recurring transaction.
    pub async fn pause(&self, cmd: PauseRecurringCommand) -> Result<(), LedgerError> {
        let mut recurring = self
            .recurring_repository
            .find_by_id(cmd.recurring_id)
            .await?
            .ok_or_else(|| {
                LedgerError::RecurringTransactionNotFound(cmd.recurring_id.to_string())
            })?;

        if recurring.owner_id != cmd.principal.user_id {
            return Err(LedgerError::Forbidden(
                "not the owner of this recurring transaction".into(),
            ));
        }

        recurring.pause();
        self.recurring_repository.save(&recurring).await?;

        let event = RecurringTransactionPaused {
            recurring_transaction_id: cmd.recurring_id,
            timestamp: chrono::Utc::now(),
        };
        self.event_publisher
            .publish(vec![&event])
            .await
            .map_err(LedgerError::Publish)?;

        Ok(())
    }

    /// Resumes a paused recurring transaction.
    pub async fn resume(&self, cmd: ResumeRecurringCommand) -> Result<(), LedgerError> {
        let mut recurring = self
            .recurring_repository
            .find_by_id(cmd.recurring_id)
            .await?
            .ok_or_else(|| {
                LedgerError::RecurringTransactionNotFound(cmd.recurring_id.to_string())
            })?;

        if recurring.owner_id != cmd.principal.user_id {
            return Err(LedgerError::Forbidden(
                "not the owner of this recurring transaction".into(),
            ));
        }

        recurring.resume();
        self.recurring_repository.save(&recurring).await?;

        let event = RecurringTransactionResumed {
            recurring_transaction_id: cmd.recurring_id,
            timestamp: chrono::Utc::now(),
        };
        self.event_publisher
            .publish(vec![&event])
            .await
            .map_err(LedgerError::Publish)?;

        Ok(())
    }

    /// Cancels a recurring transaction.
    pub async fn cancel(&self, cmd: CancelRecurringCommand) -> Result<(), LedgerError> {
        let mut recurring = self
            .recurring_repository
            .find_by_id(cmd.recurring_id)
            .await?
            .ok_or_else(|| {
                LedgerError::RecurringTransactionNotFound(cmd.recurring_id.to_string())
            })?;

        if recurring.owner_id != cmd.principal.user_id {
            return Err(LedgerError::Forbidden(
                "not the owner of this recurring transaction".into(),
            ));
        }

        recurring.cancel();
        self.recurring_repository.save(&recurring).await?;

        let event = RecurringTransactionCancelled {
            recurring_transaction_id: cmd.recurring_id,
            timestamp: chrono::Utc::now(),
        };
        self.event_publisher
            .publish(vec![&event])
            .await
            .map_err(LedgerError::Publish)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::domain::recurring_transaction::{Frequency, RecurringTransaction};
    use crate::ledger::domain::transaction::TransactionType;
    use crate::shared::events::InMemoryEventDispatcher;
    use crate::shared::ids::{AccountID, Principal, UserID};
    use crate::shared::mock::MockRecurringTransactionRepository;
    use crate::shared::money::{Currency, Money};

    fn make_recurring(owner: UserID) -> RecurringTransaction {
        RecurringTransaction::new(
            RecurringTransactionID::new(),
            owner,
            AccountID::new(),
            TransactionType::Expense,
            Money::from_cents(1000, Currency::BRL),
            "Netflix".into(),
            Some(crate::shared::ids::CategoryID::new()),
            Frequency::Monthly,
            chrono::NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
        )
        .unwrap()
    }

    #[tokio::test]
    async fn test_pause_resume_cancel() {
        let repo = Arc::new(MockRecurringTransactionRepository::new());
        let publisher = Arc::new(InMemoryEventDispatcher::new());
        let owner = UserID::new();
        let _principal = Principal::new(owner);

        let recurring = make_recurring(owner);
        let id = recurring.id;
        repo.save(&recurring).await.unwrap();

        let handler = UpdateRecurringHandler::new(repo.clone(), publisher);

        // Pause
        handler
            .pause(PauseRecurringCommand {
                recurring_id: id,
                principal: Principal::new(owner),
            })
            .await
            .unwrap();
        let r = repo.find_by_id(id).await.unwrap().unwrap();
        assert!(!r.active);

        // Resume
        handler
            .resume(ResumeRecurringCommand {
                recurring_id: id,
                principal: Principal::new(owner),
            })
            .await
            .unwrap();
        let r = repo.find_by_id(id).await.unwrap().unwrap();
        assert!(r.active);

        // Cancel
        handler
            .cancel(CancelRecurringCommand {
                recurring_id: id,
                principal: Principal::new(owner),
            })
            .await
            .unwrap();
        let r = repo.find_by_id(id).await.unwrap().unwrap();
        assert!(!r.active);
    }

    #[tokio::test]
    async fn test_wrong_owner_rejected() {
        let repo = Arc::new(MockRecurringTransactionRepository::new());
        let publisher = Arc::new(InMemoryEventDispatcher::new());
        let owner = UserID::new();

        let recurring = make_recurring(owner);
        let id = recurring.id;
        repo.save(&recurring).await.unwrap();

        let handler = UpdateRecurringHandler::new(repo, publisher);
        let result = handler
            .pause(PauseRecurringCommand {
                recurring_id: id,
                principal: Principal::new(UserID::new()),
            })
            .await;
        assert!(matches!(result, Err(LedgerError::Forbidden(_))));
    }
}
