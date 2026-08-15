use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use crate::shared::ids::{AccountID, CategoryID, PurchaseID, TagID, TransactionID};
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
    pub tags: Vec<TagID>,
    pub counterpart_account_id: Option<AccountID>,
    pub source_purchase_id: Option<PurchaseID>,
    pub reconciled: bool,
    pub created_at: DateTime<Utc>,
}

impl Transaction {
    pub fn new(
        id: TransactionID,
        account_id: AccountID,
        tx_type: TransactionType,
        amount: Money,
        description: String,
        date: NaiveDate,
    ) -> Self {
        Self {
            id,
            account_id,
            tx_type,
            amount,
            description,
            date,
            category_id: None,
            tags: Vec::new(),
            counterpart_account_id: None,
            source_purchase_id: None,
            reconciled: false,
            created_at: Utc::now(),
        }
    }

    pub fn with_category(mut self, category_id: CategoryID) -> Self {
        self.category_id = Some(category_id);
        self
    }

    pub fn with_counterpart(mut self, counterpart_account_id: AccountID) -> Self {
        self.counterpart_account_id = Some(counterpart_account_id);
        self
    }

    pub fn with_tags(mut self, tags: Vec<TagID>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_source_purchase(mut self, purchase_id: PurchaseID) -> Self {
        self.source_purchase_id = Some(purchase_id);
        self
    }

    pub fn mark_reconciled(&mut self) {
        self.reconciled = true;
    }
}
