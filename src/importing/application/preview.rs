use std::sync::Arc;

use crate::importing::domain::raw_transaction::RawTransaction;
use crate::ledger::domain::repository::TransactionRepository;
use crate::shared::errors::ImportError;
use crate::shared::ids::AccountID;
use crate::shared::period::Period;

/// Command to preview a batch of raw transactions before import.
///
/// Scans the target account for exact date+amount matches and flags them
/// as duplicates. The frontend uses this to let the user review before confirming.
pub struct PreviewCommand {
    /// Target account to import into.
    pub account_id: AccountID,
    /// Parsed raw transactions to preview.
    pub transactions: Vec<RawTransaction>,
}

/// A raw transaction with duplicate-detection metadata.
#[derive(Debug, Clone)]
pub struct PreviewCandidate {
    /// The original raw transaction.
    pub raw: RawTransaction,
    /// `true` if an exact date+amount match was found in the target account.
    pub is_duplicate: bool,
}

/// Handler that previews a batch of raw transactions for duplicate detection.
///
/// Queries the target account for transactions in the date range covered by the
/// raw batch and flags exact date+amount matches as duplicates.
pub struct PreviewHandler<T: TransactionRepository> {
    transaction_repository: Arc<T>,
}

impl<T: TransactionRepository> PreviewHandler<T> {
    pub fn new(transaction_repository: Arc<T>) -> Self {
        Self {
            transaction_repository,
        }
    }

    /// Runs the preview: scans for duplicates and returns candidates with flags.
    pub async fn handle(&self, cmd: PreviewCommand) -> Result<Vec<PreviewCandidate>, ImportError> {
        if cmd.transactions.is_empty() {
            return Ok(Vec::new());
        }

        let min_date = cmd
            .transactions
            .iter()
            .map(|r| r.date)
            .min()
            .expect("non-empty");
        let max_date = cmd
            .transactions
            .iter()
            .map(|r| r.date)
            .max()
            .expect("non-empty");

        let period = Period::new(min_date, max_date);
        let existing = self
            .transaction_repository
            .find_by_account(cmd.account_id, period)
            .await?;

        let candidates = cmd
            .transactions
            .into_iter()
            .map(|raw| {
                let is_duplicate = existing
                    .iter()
                    .any(|t| t.date == raw.date && t.amount == raw.amount);
                PreviewCandidate { raw, is_duplicate }
            })
            .collect();

        Ok(candidates)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

    #[tokio::test]
    async fn test_preview_empty() {
        let repo = Arc::new(MockTransactionRepository::new());
        let handler = PreviewHandler::new(repo);
        let result = handler
            .handle(PreviewCommand {
                account_id: AccountID::new(),
                transactions: Vec::new(),
            })
            .await
            .unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_preview_no_duplicates() {
        let repo = Arc::new(MockTransactionRepository::new());
        let handler = PreviewHandler::new(repo);
        let txs = vec![sample_raw("2026-03-15", 1000)];
        let result = handler
            .handle(PreviewCommand {
                account_id: AccountID::new(),
                transactions: txs,
            })
            .await
            .unwrap();
        assert_eq!(result.len(), 1);
        assert!(!result[0].is_duplicate);
    }
}
