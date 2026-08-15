use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("moeda incompatível: esperado {expected}, recebido {received}")]
    CurrencyMismatch { expected: String, received: String },

    #[error("conta não encontrada: {0}")]
    AccountNotFound(String),

    #[error("transação não encontrada: {0}")]
    TransactionNotFound(String),

    #[error("categoria não encontrada: {0}")]
    CategoryNotFound(String),

    #[error("tag não encontrada: {0}")]
    TagNotFound(String),

    #[error("budget não encontrado: {0}")]
    BudgetNotFound(String),

    #[error("goal não encontrado: {0}")]
    GoalNotFound(String),

    #[error("cartão de crédito não encontrado: {0}")]
    CreditCardNotFound(String),

    #[error("fatura não encontrada: {0}")]
    InvoiceNotFound(String),

    #[error("bill não encontrado: {0}")]
    BillNotFound(String),

    #[error("investimento não encontrado: {0}")]
    InvestmentNotFound(String),

    #[error("saldo insuficiente: disponível {available}, solicitado {requested}")]
    InsufficientFunds {
        available: String,
        requested: String,
    },

    #[error("valor inválido: {0}")]
    InvalidAmount(String),

    #[error("invariante violado: {0}")]
    InvariantViolation(String),

    #[error("duplicate entry: {0}")]
    AlreadyExists(String),
}
