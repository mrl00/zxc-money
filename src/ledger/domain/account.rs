use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::shared::ids::AccountID;
use crate::shared::money::Money;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountType {
    Checking,
    Savings,
    Wallet,
    Investment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: AccountID,
    pub name: String,
    pub account_type: AccountType,
    pub currency: crate::shared::money::Currency,
    pub opening_balance: Money,
    pub created_at: DateTime<Utc>,
}

impl Account {
    pub fn new(
        id: AccountID,
        name: String,
        account_type: AccountType,
        currency: crate::shared::money::Currency,
        opening_balance: Money,
    ) -> Self {
        Self {
            id,
            name,
            account_type,
            currency,
            opening_balance,
            created_at: Utc::now(),
        }
    }

    pub fn rename(&mut self, new_name: String) {
        self.name = new_name;
    }
}
