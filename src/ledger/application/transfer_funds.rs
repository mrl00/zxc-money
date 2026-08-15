use crate::shared::errors::DomainError;
use crate::shared::ids::AccountID;
use crate::shared::money::Money;

pub struct TransferFundsCommand {
    pub from_account_id: AccountID,
    pub to_account_id: AccountID,
    pub amount: Money,
    pub description: String,
}

#[derive(Default)]
pub struct TransferFundsHandler {
    // repository, event publisher injected via constructor
}

impl TransferFundsHandler {
    pub fn new() -> Self {
        Self {}
    }

    pub fn validate(&self, cmd: &TransferFundsCommand) -> Result<(), DomainError> {
        if cmd.from_account_id == cmd.to_account_id {
            return Err(DomainError::InvariantViolation(
                "conta de origem e destino devem ser diferentes".into(),
            ));
        }

        if !cmd.amount.is_positive() {
            return Err(DomainError::InvalidAmount("valor deve ser positivo".into()));
        }

        Ok(())
    }
}
