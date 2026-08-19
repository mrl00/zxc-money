use chrono::{DateTime, Datelike, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use crate::ledger::domain::transaction::TransactionType;
use crate::shared::errors::LedgerError;
use crate::shared::ids::{AccountID, CategoryID, RecurringTransactionID, TagID, UserID};
use crate::shared::money::Money;

/// Frequency at which a recurring transaction repeats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Frequency {
    /// Every day.
    Daily,
    /// Every week.
    Weekly,
    /// Every two weeks.
    Biweekly,
    /// Every month.
    Monthly,
    /// Every three months.
    Quarterly,
    /// Every year.
    Yearly,
}

impl Frequency {
    /// Returns the next date after `current` based on this frequency.
    pub fn next_date(&self, current: NaiveDate) -> NaiveDate {
        match self {
            Frequency::Daily => current + chrono::Duration::days(1),
            Frequency::Weekly => current + chrono::Duration::weeks(1),
            Frequency::Biweekly => current + chrono::Duration::weeks(2),
            Frequency::Monthly => {
                let mut year = current.year();
                let mut month = current.month() + 1;
                if month > 12 {
                    month = 1;
                    year += 1;
                }
                let last_day = days_in_month(year, month);
                let day = current.day().min(last_day);
                NaiveDate::from_ymd_opt(year, month, day).unwrap()
            }
            Frequency::Quarterly => {
                let mut year = current.year();
                let mut month = current.month() + 3;
                if month > 12 {
                    month -= 12;
                    year += 1;
                }
                let last_day = days_in_month(year, month);
                let day = current.day().min(last_day);
                NaiveDate::from_ymd_opt(year, month, day).unwrap()
            }
            Frequency::Yearly => {
                let last_day = days_in_month(current.year() + 1, current.month());
                let day = current.day().min(last_day);
                NaiveDate::from_ymd_opt(current.year() + 1, current.month(), day).unwrap()
            }
        }
    }
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// A recurring transaction that generates periodic
/// [`Transaction`](crate::ledger::domain::transaction::Transaction)s.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecurringTransaction {
    /// Unique identifier.
    pub id: RecurringTransactionID,
    /// Owner of this recurring transaction.
    pub owner_id: UserID,
    /// Account to debit/credit.
    pub account_id: AccountID,
    /// Type of transaction (income or expense).
    pub tx_type: TransactionType,
    /// Amount per occurrence.
    pub amount: Money,
    /// Description for generated transactions.
    pub description: String,
    /// Optional category for generated transactions.
    pub category_id: Option<CategoryID>,
    /// Tags for generated transactions.
    pub tags: Vec<TagID>,
    /// How often this transaction repeats.
    pub frequency: Frequency,
    /// Next date when a transaction should be generated.
    pub next_date: NaiveDate,
    /// Whether this recurring transaction is active.
    pub active: bool,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
}

impl RecurringTransaction {
    /// Creates a new active recurring transaction.
    ///
    /// # Errors
    /// Returns an error if `amount` is not positive, `description` is empty,
    /// the type is `Transfer`, or an income/expense has no category.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: RecurringTransactionID,
        owner_id: UserID,
        account_id: AccountID,
        tx_type: TransactionType,
        amount: Money,
        description: String,
        category_id: Option<CategoryID>,
        frequency: Frequency,
        next_date: NaiveDate,
    ) -> Result<Self, LedgerError> {
        if !amount.is_positive() {
            return Err(LedgerError::InvalidAmount(
                "recurring transaction amount must be positive".into(),
            ));
        }

        if description.is_empty() {
            return Err(LedgerError::InvariantViolation(
                "recurring transaction description must not be empty".into(),
            ));
        }

        match tx_type {
            TransactionType::Transfer => {
                return Err(LedgerError::InvariantViolation(
                    "recurring transaction cannot be a transfer".into(),
                ));
            }
            TransactionType::Income | TransactionType::Expense => {
                if category_id.is_none() {
                    return Err(LedgerError::InvariantViolation(
                        "income/expense recurring transaction must have category".into(),
                    ));
                }
            }
        }

        Ok(Self {
            id,
            owner_id,
            account_id,
            tx_type,
            amount,
            description,
            category_id,
            tags: Vec::new(),
            frequency,
            next_date,
            active: true,
            created_at: Utc::now(),
        })
    }

    /// Sets the tags on this recurring transaction.
    pub fn with_tags(mut self, tags: Vec<TagID>) -> Self {
        self.tags = tags;
        self
    }

    /// Pauses the recurring transaction so it will not be due until resumed.
    pub fn pause(&mut self) {
        self.active = false;
    }

    /// Resumes a paused recurring transaction.
    pub fn resume(&mut self) {
        self.active = true;
    }

    /// Permanently cancels this recurring transaction.
    pub fn cancel(&mut self) {
        self.active = false;
    }

    /// Advances `next_date` by one occurrence.
    pub fn advance(&mut self) {
        self.next_date = self.frequency.next_date(self.next_date);
    }

    /// Returns `true` if this transaction is active and its next date is on or before `today`.
    pub fn is_due(&self, today: NaiveDate) -> bool {
        self.active && self.next_date <= today
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::money::Currency;

    fn make_recurring(freq: Frequency, next_date: NaiveDate) -> RecurringTransaction {
        RecurringTransaction::new(
            RecurringTransactionID::new(),
            UserID::new(),
            AccountID::new(),
            TransactionType::Expense,
            Money::new(1000, Currency::BRL),
            "Netflix".into(),
            Some(CategoryID::new()),
            freq,
            next_date,
        )
        .unwrap()
    }

    #[test]
    fn test_frequency_next_date_monthly() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        let next = Frequency::Monthly.next_date(date);
        assert_eq!(next, NaiveDate::from_ymd_opt(2026, 2, 15).unwrap());
    }

    #[test]
    fn test_frequency_next_date_monthly_end_of_month() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 31).unwrap();
        let next = Frequency::Monthly.next_date(date);
        assert_eq!(next, NaiveDate::from_ymd_opt(2026, 2, 28).unwrap());
    }

    #[test]
    fn test_frequency_next_date_yearly() {
        let date = NaiveDate::from_ymd_opt(2026, 3, 10).unwrap();
        let next = Frequency::Yearly.next_date(date);
        assert_eq!(next, NaiveDate::from_ymd_opt(2027, 3, 10).unwrap());
    }

    #[test]
    fn test_frequency_next_date_quarterly() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        let next = Frequency::Quarterly.next_date(date);
        assert_eq!(next, NaiveDate::from_ymd_opt(2026, 4, 15).unwrap());
    }

    #[test]
    fn test_frequency_next_date_biweekly() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let next = Frequency::Biweekly.next_date(date);
        assert_eq!(next, NaiveDate::from_ymd_opt(2026, 1, 15).unwrap());
    }

    #[test]
    fn test_frequency_next_date_daily() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let next = Frequency::Daily.next_date(date);
        assert_eq!(next, NaiveDate::from_ymd_opt(2026, 1, 2).unwrap());
    }

    #[test]
    fn test_frequency_next_date_weekly() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let next = Frequency::Weekly.next_date(date);
        assert_eq!(next, NaiveDate::from_ymd_opt(2026, 1, 8).unwrap());
    }

    #[test]
    fn test_pause_resume() {
        let mut r = make_recurring(
            Frequency::Monthly,
            NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
        );
        assert!(r.active);
        r.pause();
        assert!(!r.active);
        r.resume();
        assert!(r.active);
    }

    #[test]
    fn test_cancel() {
        let mut r = make_recurring(
            Frequency::Monthly,
            NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
        );
        r.cancel();
        assert!(!r.active);
    }

    #[test]
    fn test_advance() {
        let mut r = make_recurring(
            Frequency::Monthly,
            NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
        );
        r.advance();
        assert_eq!(r.next_date, NaiveDate::from_ymd_opt(2026, 2, 15).unwrap());
    }

    #[test]
    fn test_is_due() {
        let r = make_recurring(
            Frequency::Monthly,
            NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
        );
        assert!(r.is_due(NaiveDate::from_ymd_opt(2026, 1, 15).unwrap()));
        assert!(r.is_due(NaiveDate::from_ymd_opt(2026, 1, 20).unwrap()));
        assert!(!r.is_due(NaiveDate::from_ymd_opt(2026, 1, 14).unwrap()));
    }

    #[test]
    fn test_is_due_paused() {
        let mut r = make_recurring(
            Frequency::Monthly,
            NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
        );
        r.pause();
        assert!(!r.is_due(NaiveDate::from_ymd_opt(2026, 1, 15).unwrap()));
    }

    #[test]
    fn test_negative_amount_rejected() {
        let result = RecurringTransaction::new(
            RecurringTransactionID::new(),
            UserID::new(),
            AccountID::new(),
            TransactionType::Expense,
            Money::new(-100, Currency::BRL),
            "Test".into(),
            Some(CategoryID::new()),
            Frequency::Monthly,
            NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_description_rejected() {
        let result = RecurringTransaction::new(
            RecurringTransactionID::new(),
            UserID::new(),
            AccountID::new(),
            TransactionType::Expense,
            Money::new(100, Currency::BRL),
            "".into(),
            Some(CategoryID::new()),
            Frequency::Monthly,
            NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_transfer_rejected() {
        let result = RecurringTransaction::new(
            RecurringTransactionID::new(),
            UserID::new(),
            AccountID::new(),
            TransactionType::Transfer,
            Money::new(100, Currency::BRL),
            "Transfer".into(),
            None,
            Frequency::Monthly,
            NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_income_requires_category() {
        let result = RecurringTransaction::new(
            RecurringTransactionID::new(),
            UserID::new(),
            AccountID::new(),
            TransactionType::Income,
            Money::new(100, Currency::BRL),
            "Salary".into(),
            None,
            Frequency::Monthly,
            NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        );
        assert!(result.is_err());
    }
}
