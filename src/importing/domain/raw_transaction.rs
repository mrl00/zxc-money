use chrono::NaiveDate;

use crate::shared::money::Money;

/// A single row parsed from a financial statement, before import into the ledger.
///
/// This is the output of [`StatementParser`](crate::provider::parser::StatementParser)
/// and the input to the import pipeline (preview → match → confirm).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawTransaction {
    /// Transaction date as parsed from the statement.
    pub date: NaiveDate,
    /// Monetary amount (positive = income, negative = expense).
    pub amount: Money,
    /// Description or memo from the statement.
    pub description: String,
    /// The original unparsed line, kept for audit/debug purposes.
    pub raw_line: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::money::Currency;

    fn sample_raw() -> RawTransaction {
        RawTransaction {
            date: NaiveDate::from_ymd_opt(2026, 3, 15).unwrap(),
            amount: Money::new(-2500, Currency::BRL),
            description: "Supermarket".into(),
            raw_line: "15/03/2026,-25.00,Supermarket".into(),
        }
    }

    #[test]
    fn test_raw_transaction_creation() {
        let raw = sample_raw();
        assert_eq!(raw.date, NaiveDate::from_ymd_opt(2026, 3, 15).unwrap());
        assert_eq!(raw.amount, Money::new(-2500, Currency::BRL));
        assert_eq!(raw.description, "Supermarket");
    }

    #[test]
    fn test_raw_transaction_clone() {
        let raw = sample_raw();
        let cloned = raw.clone();
        assert_eq!(raw, cloned);
    }

    #[test]
    fn test_raw_transaction_positive_amount() {
        let raw = RawTransaction {
            date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            amount: Money::new(50000, Currency::BRL),
            description: "Salary".into(),
            raw_line: "01/01/2026,500.00,Salary".into(),
        };
        assert!(raw.amount.is_positive());
    }
}
