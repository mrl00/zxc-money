use std::sync::Arc;

use crate::importing::domain::raw_transaction::RawTransaction;
use crate::ledger::domain::repository::{AccountRepository, TransactionRepository};
use crate::shared::errors::ImportError;
use crate::shared::ids::{AccountID, Principal};
use crate::shared::period::Period;

/// Command to preview a batch of raw transactions before import.
///
/// Scans the target account for exact date+amount matches and flags them
/// as duplicates. The frontend uses this to let the user review before confirming.
pub struct PreviewCommand {
    pub principal: Principal,
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
pub struct PreviewHandler<A: AccountRepository, T: TransactionRepository> {
    account_repository: Arc<A>,
    transaction_repository: Arc<T>,
}

impl<A: AccountRepository, T: TransactionRepository> PreviewHandler<A, T> {
    pub fn new(account_repository: Arc<A>, transaction_repository: Arc<T>) -> Self {
        Self {
            account_repository,
            transaction_repository,
        }
    }

    /// Runs the preview: scans for duplicates and returns candidates with flags.
    ///
    /// # Errors
    /// Fails if the account does not belong to the authenticated user.
    pub async fn handle(&self, cmd: PreviewCommand) -> Result<Vec<PreviewCandidate>, ImportError> {
        let account = self
            .account_repository
            .find_by_id(cmd.account_id)
            .await?
            .ok_or_else(|| {
                ImportError::NotFound(format!("account not found: {}", cmd.account_id))
            })?;

        if account.owner_id != cmd.principal.user_id {
            return Err(ImportError::Forbidden(
                "not the owner of this account".into(),
            ));
        }

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
    use crate::ledger::domain::account::{Account, AccountType};
    use crate::shared::ids::{AccountID, Principal, UserID};
    use crate::shared::mock::{MockAccountRepository, MockTransactionRepository};
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
        let account_repo = Arc::new(MockAccountRepository::new());
        let tx_repo = Arc::new(MockTransactionRepository::new());
        let owner = UserID::new();
        let account_id = AccountID::new();

        let account = Account::new(
            account_id,
            owner,
            "Test".into(),
            AccountType::Checking,
            Currency::BRL,
            Money::from_cents(0, Currency::BRL),
        )
        .unwrap();
        account_repo.save(&account).await.unwrap();

        let handler = PreviewHandler::new(account_repo, tx_repo);
        let result = handler
            .handle(PreviewCommand {
                principal: Principal::new(owner),
                account_id,
                transactions: Vec::new(),
            })
            .await
            .unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_preview_no_duplicates() {
        let account_repo = Arc::new(MockAccountRepository::new());
        let tx_repo = Arc::new(MockTransactionRepository::new());
        let owner = UserID::new();
        let account_id = AccountID::new();

        let account = Account::new(
            account_id,
            owner,
            "Test".into(),
            AccountType::Checking,
            Currency::BRL,
            Money::from_cents(0, Currency::BRL),
        )
        .unwrap();
        account_repo.save(&account).await.unwrap();

        let handler = PreviewHandler::new(account_repo, tx_repo);
        let txs = vec![sample_raw("2026-03-15", 1000)];
        let result = handler
            .handle(PreviewCommand {
                principal: Principal::new(owner),
                account_id,
                transactions: txs,
            })
            .await
            .unwrap();
        assert_eq!(result.len(), 1);
        assert!(!result[0].is_duplicate);
    }

    #[tokio::test]
    async fn test_preview_wrong_owner_blocked() {
        let account_repo = Arc::new(MockAccountRepository::new());
        let tx_repo = Arc::new(MockTransactionRepository::new());
        let account_id = AccountID::new();

        let account = Account::new(
            account_id,
            UserID::new(),
            "Test".into(),
            AccountType::Checking,
            Currency::BRL,
            Money::from_cents(0, Currency::BRL),
        )
        .unwrap();
        account_repo.save(&account).await.unwrap();

        let handler = PreviewHandler::new(account_repo, tx_repo);
        let result = handler
            .handle(PreviewCommand {
                principal: Principal::new(UserID::new()),
                account_id,
                transactions: vec![sample_raw("2026-03-15", 1000)],
            })
            .await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ImportError::Forbidden(_)));
    }
}
