use thiserror::Error;

#[derive(Debug, Error)]
pub enum PublishError {
    #[error("event publish failed: {0}")]
    DispatchFailed(String),
}

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("already exists: {0}")]
    AlreadyExists(String),

    #[error("invalid state: {0}")]
    InvalidState(String),

    #[error("storage error: {0}")]
    Storage(String),
}

#[derive(Debug, Error)]
pub enum LedgerError {
    #[error("account not found: {0}")]
    AccountNotFound(String),

    #[error("transaction not found: {0}")]
    TransactionNotFound(String),

    #[error("currency mismatch: expected {expected}, got {received}")]
    CurrencyMismatch { expected: String, received: String },

    #[error("insufficient funds: available {available}, requested {requested}")]
    InsufficientFunds {
        available: String,
        requested: String,
    },

    #[error("invalid amount: {0}")]
    InvalidAmount(String),

    #[error("invariant violated: {0}")]
    InvariantViolation(String),

    #[error("category not found: {0}")]
    CategoryNotFound(String),

    #[error("tag not found: {0}")]
    TagNotFound(String),

    #[error("repository error: {0}")]
    Repository(#[from] RepositoryError),

    #[error("publish error: {0}")]
    Publish(#[from] PublishError),
}

#[derive(Debug, Error)]
pub enum BudgetingError {
    #[error("budget not found: {0}")]
    BudgetNotFound(String),

    #[error("goal not found: {0}")]
    GoalNotFound(String),

    #[error("invariant violated: {0}")]
    InvariantViolation(String),

    #[error("currency mismatch: expected {expected}, got {received}")]
    CurrencyMismatch { expected: String, received: String },

    #[error("invalid amount: {0}")]
    InvalidAmount(String),

    #[error("repository error: {0}")]
    Repository(#[from] RepositoryError),

    #[error("publish error: {0}")]
    Publish(#[from] PublishError),

    #[error("ledger error: {0}")]
    Ledger(#[from] LedgerError),
}

#[derive(Debug, Error)]
pub enum CreditCardError {
    #[error("credit card not found: {0}")]
    CreditCardNotFound(String),

    #[error("invoice not found: {0}")]
    InvoiceNotFound(String),

    #[error("invoice is not open")]
    InvoiceNotOpen,

    #[error("invariant violated: {0}")]
    InvariantViolation(String),

    #[error("repository error: {0}")]
    Repository(#[from] RepositoryError),

    #[error("publish error: {0}")]
    Publish(#[from] PublishError),
}

#[derive(Debug, Error)]
pub enum BillsError {
    #[error("bill not found: {0}")]
    BillNotFound(String),

    #[error("invariant violated: {0}")]
    InvariantViolation(String),

    #[error("repository error: {0}")]
    Repository(#[from] RepositoryError),

    #[error("publish error: {0}")]
    Publish(#[from] PublishError),
}

#[derive(Debug, Error)]
pub enum InvestmentError {
    #[error("portfolio not found: {0}")]
    PortfolioNotFound(String),

    #[error("asset not found: {0}")]
    AssetNotFound(String),

    #[error("insufficient quantity: available {available}, requested {requested}")]
    InsufficientQuantity {
        available: String,
        requested: String,
    },

    #[error("invariant violated: {0}")]
    InvariantViolation(String),

    #[error("repository error: {0}")]
    Repository(#[from] RepositoryError),

    #[error("publish error: {0}")]
    Publish(#[from] PublishError),
}
