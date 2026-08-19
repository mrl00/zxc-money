use async_trait::async_trait;

use crate::importing::domain::raw_transaction::RawTransaction;
use crate::shared::errors::ImportError;

/// Configuration for mapping CSV/OFX columns to [`RawTransaction`] fields.
///
/// Each field specifies the 0-based column index in the source file.
/// The `date_format` is used to parse the date string (chrono format).
#[derive(Debug, Clone)]
pub struct ColumnMapping {
    /// Column index for the transaction date.
    pub date: usize,
    /// Column index for the amount (supports negative for expenses).
    pub amount: usize,
    /// Column index for the description/memo.
    pub description: usize,
    /// chrono format string for parsing the date (e.g. `"%Y-%m-%d"`, `"%d/%m/%Y"`).
    pub date_format: String,
}

/// Port trait for parsing financial statement files into [`RawTransaction`] records.
///
/// Frontends implement this trait for each supported format (CSV, OFX, QIF, etc.).
/// The core defines the contract; adapters live outside.
///
/// # Example
///
/// ```ignore
/// struct CsvStatementParser;
///
/// #[async_trait]
/// impl StatementParser for CsvStatementParser {
///     async fn parse(&self, input: &str, mapping: &ColumnMapping) -> Result<Vec<RawTransaction>, ImportError> {
///         // Parse CSV using `mapping` to map columns
///     }
/// }
/// ```
#[async_trait]
pub trait StatementParser: Send + Sync {
    /// Parses raw input text into structured [`RawTransaction`] records.
    ///
    /// # Arguments
    /// * `input` — The raw file content as a string.
    /// * `mapping` — Column index configuration for the input format.
    ///
    /// # Errors
    /// Returns [`ImportError::ParseError`] if the input cannot be parsed,
    /// or [`ImportError::InvalidRawTransaction`] if individual rows are malformed.
    async fn parse(
        &self,
        input: &str,
        mapping: &ColumnMapping,
    ) -> Result<Vec<RawTransaction>, ImportError>;
}
