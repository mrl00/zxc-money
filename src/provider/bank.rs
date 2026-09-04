use async_trait::async_trait;
use thiserror::Error;

use crate::importing::domain::raw_transaction::RawTransaction;
use crate::shared::period::Period;

/// Errors from the bank integration infrastructure.
#[derive(Debug, Error)]
pub enum BankError {
    /// The bank API returned an error.
    #[error("bank API error: {0}")]
    ApiError(String),

    /// The connection to the bank failed.
    #[error("connection failed: {0}")]
    ConnectionFailed(String),

    /// The account was not found at the bank.
    #[error("account not found: {0}")]
    AccountNotFound(String),

    /// The bank returned invalid data.
    #[error("invalid data: {0}")]
    InvalidData(String),
}

/// Port for fetching transactions from a bank via Open Finance.
///
/// Frontends must implement this trait to connect to real bank APIs.
/// This is distinct from [`StatementParser`](crate::provider::parser::StatementParser),
/// which parses manual CSV/text imports. `BankProvider` is for automated
/// API-based imports through Open Finance or similar protocols.
///
/// # Example
///
/// A frontend might implement this with an Open Finance client:
/// ```ignore
/// struct PluggyClient { api_key: String }
///
/// #[async_trait]
/// impl BankProvider for PluggyClient {
///     async fn fetch_transactions(
///         &self,
///         account_id: &str,
///         range: &Period,
///     ) -> Result<Vec<RawTransaction>, BankError> {
///         // call Pluggy API
///         Ok(vec![])
///     }
/// }
/// ```
#[async_trait]
pub trait BankProvider: Send + Sync {
    /// Fetches transactions for the given account within the specified date range.
    async fn fetch_transactions(
        &self,
        account_id: &str,
        range: &Period,
    ) -> Result<Vec<RawTransaction>, BankError>;
}

/// Mock bank provider for testing.
///
/// Returns a predefined list of transactions regardless of account/range.
pub struct MockBankProvider {
    transactions: Vec<RawTransaction>,
}

impl MockBankProvider {
    /// Creates a mock that returns the given transactions.
    pub fn new(transactions: Vec<RawTransaction>) -> Self {
        Self { transactions }
    }

    /// Creates an empty mock that returns no transactions.
    pub fn empty() -> Self {
        Self {
            transactions: Vec::new(),
        }
    }
}

#[async_trait]
impl BankProvider for MockBankProvider {
    async fn fetch_transactions(
        &self,
        _account_id: &str,
        _range: &Period,
    ) -> Result<Vec<RawTransaction>, BankError> {
        Ok(self.transactions.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::money::{Currency, Money};

    fn sample_raw() -> RawTransaction {
        RawTransaction {
            date: chrono::NaiveDate::from_ymd_opt(2026, 3, 15).unwrap(),
            amount: Money::new(-2500, Currency::BRL),
            description: "Supermarket".into(),
            raw_line: "15/03/2026,-25.00,Supermarket".into(),
        }
    }

    #[tokio::test]
    async fn test_mock_bank_returns_transactions() {
        let mock = MockBankProvider::new(vec![sample_raw()]);
        let range = Period::new(
            chrono::NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2026, 3, 31).unwrap(),
        );

        let txns = mock.fetch_transactions("acc-1", &range).await.unwrap();
        assert_eq!(txns.len(), 1);
        assert_eq!(txns[0].description, "Supermarket");
    }

    #[tokio::test]
    async fn test_mock_bank_empty() {
        let mock = MockBankProvider::empty();
        let range = Period::new(
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
        );

        let txns = mock.fetch_transactions("acc-1", &range).await.unwrap();
        assert!(txns.is_empty());
    }
}
