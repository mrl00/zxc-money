use crate::shared::ids::TransactionID;

pub struct ReconcileTransactionCommand {
    pub transaction_id: TransactionID,
    pub reconciled: bool,
}

#[derive(Default)]
pub struct ReconcileTransactionHandler {
    // repository injected via constructor
}

impl ReconcileTransactionHandler {
    pub fn new() -> Self {
        Self {}
    }
}
