use std::sync::Arc;

use crate::importing::application::confirm::{ConfirmCommand, ConfirmHandler};
use crate::importing::application::match_candidates::{
    MatchCandidate, MatchCandidatesCommand, MatchCandidatesHandler,
};
use crate::importing::application::preview::{PreviewCandidate, PreviewCommand, PreviewHandler};
use crate::ledger::domain::repository::TransactionRepository;
use crate::provider::id::IdGenerator;
use crate::shared::errors::ImportError;
use crate::shared::events::EventPublisher;

/// Facade for the Importing bounded context.
///
/// Aggregates preview, match-candidates, and confirm handlers behind a
/// single entry point. Frontends use this to orchestrate the import flow:
///
/// ```text
/// 1. preview()      → detect duplicates
/// 2. match_candidates() → find fuzzy matches
/// 3. confirm()      → create transactions in the ledger
/// ```
pub struct ImportingFacade<T: TransactionRepository, P: EventPublisher, I: IdGenerator> {
    preview: PreviewHandler<T>,
    match_candidates: MatchCandidatesHandler<T>,
    confirm: ConfirmHandler<T, P, I>,
}

impl<T: TransactionRepository, P: EventPublisher, I: IdGenerator> ImportingFacade<T, P, I> {
    /// Creates a new facade with shared dependencies.
    pub fn new(
        transaction_repository: Arc<T>,
        event_publisher: Arc<P>,
        id_generator: Arc<I>,
    ) -> Self {
        Self {
            preview: PreviewHandler::new(transaction_repository.clone()),
            match_candidates: MatchCandidatesHandler::new(transaction_repository.clone()),
            confirm: ConfirmHandler::new(transaction_repository, event_publisher, id_generator),
        }
    }

    /// Previews a batch of raw transactions, flagging duplicates.
    pub async fn preview(&self, cmd: PreviewCommand) -> Result<Vec<PreviewCandidate>, ImportError> {
        self.preview.handle(cmd).await
    }

    /// Finds candidate matches between raw and existing transactions.
    pub async fn match_candidates(
        &self,
        cmd: MatchCandidatesCommand,
    ) -> Result<Vec<MatchCandidate>, ImportError> {
        self.match_candidates.handle(cmd).await
    }

    /// Confirms and imports raw transactions into the ledger.
    pub async fn confirm(&self, cmd: ConfirmCommand) -> Result<usize, ImportError> {
        self.confirm.handle(cmd).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::importing::domain::raw_transaction::RawTransaction;
    use crate::provider::id::MockIdGenerator;
    use crate::shared::events::InMemoryEventDispatcher;
    use crate::shared::ids::AccountID;
    use crate::shared::mock::MockTransactionRepository;
    use crate::shared::money::{Currency, Money};

    fn sample_raw(date: &str, amount_cents: i64) -> RawTransaction {
        RawTransaction {
            date: chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap(),
            amount: Money::from_cents(amount_cents, Currency::BRL),
            description: "test".into(),
            raw_line: format!("{date},{amount_cents},test"),
        }
    }

    fn setup_facade()
    -> ImportingFacade<MockTransactionRepository, InMemoryEventDispatcher, MockIdGenerator> {
        let repo = Arc::new(MockTransactionRepository::new());
        let publisher = Arc::new(InMemoryEventDispatcher::new());
        let id_gen = Arc::new(MockIdGenerator::new(uuid::Uuid::new_v4()));
        ImportingFacade::new(repo, publisher, id_gen)
    }

    #[tokio::test]
    async fn test_facade_preview_empty() {
        let facade = setup_facade();
        let result = facade
            .preview(PreviewCommand {
                account_id: AccountID::new(),
                transactions: Vec::new(),
            })
            .await
            .unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_facade_full_flow() {
        let facade = setup_facade();
        let account_id = AccountID::new();
        let txs = vec![
            sample_raw("2026-03-15", -2500),
            sample_raw("2026-03-16", 50000),
        ];

        // 1. Preview
        let preview = facade
            .preview(PreviewCommand {
                account_id,
                transactions: txs.clone(),
            })
            .await
            .unwrap();
        assert_eq!(preview.len(), 2);
        assert!(!preview[0].is_duplicate);
        assert!(!preview[1].is_duplicate);

        // 2. Match candidates
        let matches = facade
            .match_candidates(MatchCandidatesCommand {
                account_id,
                transactions: txs.clone(),
                date_tolerance_days: 3,
                exact_amount: true,
            })
            .await
            .unwrap();
        assert_eq!(matches.len(), 2);

        // 3. Confirm
        let count = facade
            .confirm(ConfirmCommand {
                account_id,
                transactions: txs,
            })
            .await
            .unwrap();
        assert_eq!(count, 2);
    }
}
