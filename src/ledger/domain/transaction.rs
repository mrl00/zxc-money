use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use crate::shared::ids::{AccountID, CategoryID, TransactionID};
use crate::shared::money::Money;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionType {
    Income,
    Expense,
    Transfer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: TransactionID,
    pub account_id: AccountID,
    pub tx_type: TransactionType,
    pub amount: Money,
    pub description: String,
    pub date: NaiveDate,
    pub category_id: Option<CategoryID>,
    pub counterpart_account_id: Option<AccountID>,
    pub reconciled: bool,
    pub created_at: DateTime<Utc>,
}
