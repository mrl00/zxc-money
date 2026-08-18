//! Error types for each bounded context.
//!
//! Every module defines its own error enum using `thiserror`. Handlers return
//! `Result<T, ModuleError>`, enabling exhaustive `match` on the frontend side.
//!
//! All error enums include `Repository(RepositoryError)` and `Publish(PublishError)`
//! variants with `#[from]` for automatic conversion from infrastructure errors.

use thiserror::Error;

/// Errors from the event publishing infrastructure.
#[derive(Debug, Error)]
pub enum PublishError {
    /// The event dispatcher failed to deliver an event.
    #[error("event publish failed: {0}")]
    DispatchFailed(String),
}

/// Errors from repository (persistence) operations.
#[derive(Debug, Error)]
pub enum RepositoryError {
    /// The requested entity was not found.
    #[error("not found: {0}")]
    NotFound(String),

    /// An entity with the same identity already exists.
    #[error("already exists: {0}")]
    AlreadyExists(String),

    /// The entity is in an invalid state for the requested operation.
    #[error("invalid state: {0}")]
    InvalidState(String),

    /// A storage-level error occurred (e.g. connection failure).
    #[error("storage error: {0}")]
    Storage(String),
}

/// Errors from the Ledger bounded context (accounts, transactions, transfers).
#[derive(Debug, Error)]
pub enum LedgerError {
    /// The specified account does not exist.
    #[error("account not found: {0}")]
    AccountNotFound(String),

    /// The specified transaction does not exist.
    #[error("transaction not found: {0}")]
    TransactionNotFound(String),

    /// The transaction currency does not match the account currency.
    #[error("currency mismatch: expected {expected}, got {received}")]
    CurrencyMismatch { expected: String, received: String },

    /// The account has insufficient funds for this operation.
    #[error("insufficient funds: available {available}, requested {requested}")]
    InsufficientFunds {
        available: String,
        requested: String,
    },

    /// The monetary amount is invalid (e.g. negative, zero, or overflow).
    #[error("invalid amount: {0}")]
    InvalidAmount(String),

    /// A domain invariant was violated.
    #[error("invariant violated: {0}")]
    InvariantViolation(String),

    /// The specified category does not exist.
    #[error("category not found: {0}")]
    CategoryNotFound(String),

    /// The specified tag does not exist.
    #[error("tag not found: {0}")]
    TagNotFound(String),

    /// The caller is not authorized to perform this operation.
    #[error("forbidden: {0}")]
    Forbidden(String),

    /// The specified recurring transaction does not exist.
    #[error("recurring transaction not found: {0}")]
    RecurringTransactionNotFound(String),

    /// An error from the persistence layer.
    #[error("repository error: {0}")]
    Repository(#[from] RepositoryError),

    /// An error from the event publishing layer.
    #[error("publish error: {0}")]
    Publish(#[from] PublishError),
}

/// Errors from the Budgeting bounded context (budgets, financial goals).
#[derive(Debug, Error)]
pub enum BudgetingError {
    /// The specified budget does not exist.
    #[error("budget not found: {0}")]
    BudgetNotFound(String),

    /// The specified financial goal does not exist.
    #[error("goal not found: {0}")]
    GoalNotFound(String),

    /// A domain invariant was violated.
    #[error("invariant violated: {0}")]
    InvariantViolation(String),

    /// Currency mismatch between related values.
    #[error("currency mismatch: expected {expected}, got {received}")]
    CurrencyMismatch { expected: String, received: String },

    /// The monetary amount is invalid.
    #[error("invalid amount: {0}")]
    InvalidAmount(String),

    /// An error from the persistence layer.
    #[error("repository error: {0}")]
    Repository(#[from] RepositoryError),

    /// An error from the event publishing layer.
    #[error("publish error: {0}")]
    Publish(#[from] PublishError),

    /// An error propagated from the Ledger context.
    #[error("ledger error: {0}")]
    Ledger(#[from] LedgerError),
}

/// Errors from the CreditCard bounded context (cards, invoices, purchases).
#[derive(Debug, Error)]
pub enum CreditCardError {
    /// The specified credit card does not exist.
    #[error("credit card not found: {0}")]
    CreditCardNotFound(String),

    /// The specified invoice does not exist.
    #[error("invoice not found: {0}")]
    InvoiceNotFound(String),

    /// The operation requires an open invoice, but the current one is not open.
    #[error("invoice is not open")]
    InvoiceNotOpen,

    /// A domain invariant was violated.
    #[error("invariant violated: {0}")]
    InvariantViolation(String),

    /// An error from the persistence layer.
    #[error("repository error: {0}")]
    Repository(#[from] RepositoryError),

    /// An error from the event publishing layer.
    #[error("publish error: {0}")]
    Publish(#[from] PublishError),
}

/// Errors from the BillsReminder bounded context.
#[derive(Debug, Error)]
pub enum BillsError {
    /// The specified bill does not exist.
    #[error("bill not found: {0}")]
    BillNotFound(String),

    /// A domain invariant was violated.
    #[error("invariant violated: {0}")]
    InvariantViolation(String),

    /// An error from the persistence layer.
    #[error("repository error: {0}")]
    Repository(#[from] RepositoryError),

    /// An error from the event publishing layer.
    #[error("publish error: {0}")]
    Publish(#[from] PublishError),
}

/// Errors from the Investment bounded context (portfolios, assets, positions).
#[derive(Debug, Error)]
pub enum InvestmentError {
    /// The specified portfolio does not exist.
    #[error("portfolio not found: {0}")]
    PortfolioNotFound(String),

    /// The specified asset does not exist.
    #[error("asset not found: {0}")]
    AssetNotFound(String),

    /// Not enough quantity to complete the sell operation.
    #[error("insufficient quantity: available {available}, requested {requested}")]
    InsufficientQuantity {
        available: String,
        requested: String,
    },

    /// A domain invariant was violated.
    #[error("invariant violated: {0}")]
    InvariantViolation(String),

    /// An error from the persistence layer.
    #[error("repository error: {0}")]
    Repository(#[from] RepositoryError),

    /// An error from the event publishing layer.
    #[error("publish error: {0}")]
    Publish(#[from] PublishError),
}
