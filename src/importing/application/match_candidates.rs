use std::sync::Arc;

use chrono::Days;

use crate::importing::domain::raw_transaction::RawTransaction;
use crate::ledger::domain::repository::{AccountRepository, TransactionRepository};
use crate::shared::errors::ImportError;
use crate::shared::ids::{AccountID, Principal, TransactionID};
use crate::shared::money::Money;
use crate::shared::period::Period;

/// Command to find potential matches between raw transactions and existing ones.
///
/// Uses date-window + amount-tolerance matching. The frontend presents these
/// candidates so the user can decide which to skip or merge.
pub struct MatchCandidatesCommand {
    pub principal: Principal,
    /// Target account to match against.
    pub account_id: AccountID,
    /// Parsed raw transactions to find matches for.
    pub transactions: Vec<RawTransaction>,
    /// Number of days before/after the raw date to search for matches.
    pub date_tolerance_days: u32,
    /// Whether to consider amount equality as exact or allow a tolerance.
    /// When `true`, amounts must match exactly. When `false`, amounts within
    /// ±1% of each other are considered matches.
    pub exact_amount: bool,
}

/// A raw transaction paired with a potential match from the ledger.
#[derive(Debug, Clone)]
pub struct MatchCandidate {
    /// The raw transaction being imported.
    pub raw: RawTransaction,
    /// The matched existing transaction, if any.
    pub existing: Option<ExistingMatch>,
}

/// Summary of an existing transaction that matched a raw transaction.
#[derive(Debug, Clone)]
pub struct ExistingMatch {
    /// The existing transaction's ID.
    pub transaction_id: TransactionID,
    /// The existing transaction's date.
    pub date: chrono::NaiveDate,
    /// The existing transaction's amount.
    pub amount: Money,
    /// The existing transaction's description.
    pub description: String,
}

/// Handler that finds candidate matches between raw and existing transactions.
///
/// For each raw transaction, queries the target account within a date window
/// and returns any matching existing transactions.
pub struct MatchCandidatesHandler<A: AccountRepository, T: TransactionRepository> {
    account_repository: Arc<A>,
    transaction_repository: Arc<T>,
}

impl<A: AccountRepository, T: TransactionRepository> MatchCandidatesHandler<A, T> {
    pub fn new(account_repository: Arc<A>, transaction_repository: Arc<T>) -> Self {
        Self {
            account_repository,
            transaction_repository,
        }
    }

    /// Finds candidate matches for each raw transaction.
    ///
    /// # Errors
    /// Fails if the account does not belong to the authenticated user.
    pub async fn handle(
        &self,
        cmd: MatchCandidatesCommand,
    ) -> Result<Vec<MatchCandidate>, ImportError> {
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

        let tolerance = Days::new(cmd.date_tolerance_days.into());
        let search_start = min_date - tolerance;
        let search_end = max_date + tolerance;
        let period = Period::new(search_start, search_end);

        let existing = self
            .transaction_repository
            .find_by_account(cmd.account_id, period)
            .await?;

        let candidates = cmd
            .transactions
            .into_iter()
            .map(|raw| {
                let matched = existing.iter().find(|t| {
                    let date_ok = t.date == raw.date;
                    let amount_ok = if cmd.exact_amount {
                        t.amount == raw.amount
                    } else {
                        amounts_close(t.amount, raw.amount)
                    };
                    date_ok && amount_ok
                });

                let existing_match = matched.map(|t| ExistingMatch {
                    transaction_id: t.id,
                    date: t.date,
                    amount: t.amount,
                    description: t.description.clone(),
                });

                MatchCandidate {
                    raw,
                    existing: existing_match,
                }
            })
            .collect();

        Ok(candidates)
    }
}

/// Checks if two amounts are within ±1% of each other.
fn amounts_close(a: Money, b: Money) -> bool {
    if a == b {
        return true;
    }
    if a.is_zero() || b.is_zero() {
        return false;
    }
    let diff = (a - b).unwrap().abs();
    let a_abs = a.abs();
    let b_abs = b.abs();
    let base = if a_abs > b_abs { a_abs } else { b_abs };
    // diff / base < 0.01 → diff * 100 < base
    diff.amount() * rust_decimal::Decimal::from(100) < base.amount()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::money::Currency;

    use crate::ledger::domain::account::{Account, AccountType};
    use crate::shared::ids::{Principal, UserID};
    use crate::shared::mock::{MockAccountRepository, MockTransactionRepository};

    #[test]
    fn test_amounts_close_exact() {
        let a = Money::from_cents(1000, Currency::BRL);
        let b = Money::from_cents(1000, Currency::BRL);
        assert!(amounts_close(a, b));
    }

    #[test]
    fn test_amounts_close_within_tolerance() {
        let a = Money::from_cents(1000, Currency::BRL);
        let b = Money::from_cents(1005, Currency::BRL);
        assert!(amounts_close(a, b));
    }

    #[test]
    fn test_amounts_not_close() {
        let a = Money::from_cents(1000, Currency::BRL);
        let b = Money::from_cents(1100, Currency::BRL);
        assert!(!amounts_close(a, b));
    }

    #[tokio::test]
    async fn test_match_candidates_empty() {
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

        let handler = MatchCandidatesHandler::new(account_repo, tx_repo);
        let result = handler
            .handle(MatchCandidatesCommand {
                principal: Principal::new(owner),
                account_id,
                transactions: Vec::new(),
                date_tolerance_days: 3,
                exact_amount: true,
            })
            .await
            .unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_match_candidates_wrong_owner_blocked() {
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

        let handler = MatchCandidatesHandler::new(account_repo, tx_repo);
        let result = handler
            .handle(MatchCandidatesCommand {
                principal: Principal::new(UserID::new()),
                account_id,
                transactions: Vec::new(),
                date_tolerance_days: 3,
                exact_amount: true,
            })
            .await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ImportError::Forbidden(_)));
    }
}
