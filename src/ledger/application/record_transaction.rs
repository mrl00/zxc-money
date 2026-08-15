use crate::ledger::domain::transaction::TransactionType;
use crate::shared::errors::DomainError;
use crate::shared::ids::{AccountID, CategoryID};
use crate::shared::money::Money;
use chrono::NaiveDate;

pub struct RecordTransactionCommand {
    pub account_id: AccountID,
    pub tx_type: TransactionType,
    pub amount: Money,
    pub description: String,
    pub date: NaiveDate,
    pub category_id: Option<CategoryID>,
}

#[derive(Default)]
pub struct RecordTransactionHandler {
    // repository and event publisher injected via constructor
}

impl RecordTransactionHandler {
    pub fn new() -> Self {
        Self {}
    }

    pub fn validate(&self, cmd: &RecordTransactionCommand) -> Result<(), DomainError> {
        if !cmd.amount.is_positive() {
            return Err(DomainError::InvalidAmount("valor deve ser positivo".into()));
        }

        match cmd.tx_type {
            TransactionType::Transfer => {
                if cmd.category_id.is_some() {
                    return Err(DomainError::InvariantViolation(
                        "transferência não deve ter categoria".into(),
                    ));
                }
            }
            TransactionType::Income | TransactionType::Expense => {
                if cmd.category_id.is_none() {
                    return Err(DomainError::InvariantViolation(
                        "receita/despesa deve ter categoria".into(),
                    ));
                }
            }
        }

        Ok(())
    }
}
