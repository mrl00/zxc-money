use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::shared::ids::AccountID;
use crate::shared::money::Money;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountBalanceProjection {
    pub account_id: AccountID,
    pub balance: Money,
    pub reconciled_balance: Money,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl AccountBalanceProjection {
    pub fn new(account_id: AccountID, balance: Money) -> Self {
        Self {
            account_id,
            balance,
            reconciled_balance: balance,
            last_updated: chrono::Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetWorthSnapshot {
    pub date: NaiveDate,
    pub total_assets: Money,
    pub total_liabilities: Money,
    pub net_worth: Money,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetProgress {
    pub category_id: crate::shared::ids::CategoryID,
    pub planned: Money,
    pub spent: Money,
    pub remaining: Money,
    pub pct_used: f64,
    pub is_over: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashFlowEntry {
    pub date: NaiveDate,
    pub income: Money,
    pub expense: Money,
    pub net: Money,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryReport {
    pub category_id: crate::shared::ids::CategoryID,
    pub category_name: String,
    pub total: Money,
    pub transaction_count: usize,
}
