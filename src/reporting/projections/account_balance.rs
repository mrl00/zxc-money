use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::shared::ids::AccountID;
use crate::shared::money::Money;

/// Real-time balance projection for a single account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountBalanceProjection {
    pub account_id: AccountID,
    pub balance: Money,
    pub reconciled_balance: Money,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl AccountBalanceProjection {
    /// Creates a new projection with the given balance and current timestamp.
    pub fn new(account_id: AccountID, balance: Money) -> Self {
        Self {
            account_id,
            balance,
            reconciled_balance: balance,
            last_updated: chrono::Utc::now(),
        }
    }
}

/// Point-in-time snapshot of total assets, liabilities, and net worth.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetWorthSnapshot {
    pub date: NaiveDate,
    pub total_assets: Money,
    pub total_liabilities: Money,
    pub net_worth: Money,
}

/// Spending progress for a budget within a period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetProgress {
    pub category_id: crate::shared::ids::CategoryID,
    pub planned: Money,
    pub spent: Money,
    pub remaining: Money,
    pub pct_used: f64,
    pub is_over: bool,
}

/// Income, expense, and net cash flow for a single day.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashFlowEntry {
    pub date: NaiveDate,
    pub income: Money,
    pub expense: Money,
    pub net: Money,
}

/// Aggregated spending report for a single category.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryReport {
    pub category_id: crate::shared::ids::CategoryID,
    pub category_name: String,
    pub total: Money,
    pub transaction_count: usize,
}
