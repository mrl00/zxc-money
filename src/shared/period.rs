//! Date range and year-month value objects.
//!
//! [`Period`] represents an inclusive date range. [`YearMonth`] represents
//! a specific month and provides utilities for first/last day, navigation,
//! and conversion to `Period`.

use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};

/// An inclusive date range from `start` to `end`.
///
/// # Invariants
///
/// - `start <= end` (enforced by constructor assertion).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Period {
    start: NaiveDate,
    end: NaiveDate,
}

/// A specific year and month combination.
///
/// Implements `Ord` for chronological comparison (year first, then month).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct YearMonth {
    /// The year (e.g. `2026`).
    pub year: i32,
    /// The month (1–12).
    pub month: u32,
}

impl Period {
    /// Create a new period. Panics if `start > end`.
    pub fn new(start: NaiveDate, end: NaiveDate) -> Self {
        assert!(start <= end, "start date must be <= end date");
        Self { start, end }
    }

    /// Return the start date.
    pub fn start(&self) -> NaiveDate {
        self.start
    }

    /// Return the end date.
    pub fn end(&self) -> NaiveDate {
        self.end
    }

    /// Returns `true` if `date` falls within this period (inclusive).
    pub fn contains(&self, date: NaiveDate) -> bool {
        date >= self.start && date <= self.end
    }

    /// Returns `true` if this period overlaps with `other`.
    pub fn overlaps(&self, other: &Period) -> bool {
        self.start <= other.end && other.start <= self.end
    }

    /// Return all [`YearMonth`]s that this period spans.
    pub fn months(&self) -> Vec<YearMonth> {
        let mut result = Vec::new();
        let mut current = YearMonth::from_date(self.start);
        let end = YearMonth::from_date(self.end);

        while current <= end {
            result.push(current);
            current = current.next();
        }
        result
    }
}

impl YearMonth {
    /// Create a new `YearMonth`. Panics if `month` is not in 1–12.
    pub fn new(year: i32, month: u32) -> Self {
        assert!((1..=12).contains(&month), "month must be 1-12");
        Self { year, month }
    }

    /// Extract the year-month from a `NaiveDate`.
    pub fn from_date(date: NaiveDate) -> Self {
        Self {
            year: date.year(),
            month: date.month(),
        }
    }

    /// Return the first day of this month.
    pub fn first_day(&self) -> NaiveDate {
        NaiveDate::from_ymd_opt(self.year, self.month, 1).expect("invalid date")
    }

    /// Return the last day of this month.
    pub fn last_day(&self) -> NaiveDate {
        let next_month = self.next();
        next_month.first_day().pred_opt().expect("invalid date")
    }

    /// Convert to a [`Period`] spanning the full month.
    pub fn period(&self) -> Period {
        Period::new(self.first_day(), self.last_day())
    }

    /// Return the next month.
    pub fn next(&self) -> YearMonth {
        if self.month == 12 {
            YearMonth::new(self.year + 1, 1)
        } else {
            YearMonth::new(self.year, self.month + 1)
        }
    }

    /// Return the previous month.
    pub fn previous(&self) -> YearMonth {
        if self.month == 1 {
            YearMonth::new(self.year - 1, 12)
        } else {
            YearMonth::new(self.year, self.month - 1)
        }
    }
}

impl PartialOrd for YearMonth {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for YearMonth {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.year
            .cmp(&other.year)
            .then(self.month.cmp(&other.month))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_period_creation() {
        let start = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2026, 12, 31).unwrap();
        let period = Period::new(start, end);
        assert_eq!(period.start(), start);
        assert_eq!(period.end(), end);
    }

    #[test]
    fn test_period_contains() {
        let start = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2026, 3, 31).unwrap();
        let period = Period::new(start, end);

        let inside = NaiveDate::from_ymd_opt(2026, 2, 15).unwrap();
        let outside = NaiveDate::from_ymd_opt(2026, 4, 1).unwrap();

        assert!(period.contains(inside));
        assert!(!period.contains(outside));
    }

    #[test]
    fn test_period_overlaps() {
        let p1 = Period::new(
            NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2026, 3, 31).unwrap(),
        );
        let p2 = Period::new(
            NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
            NaiveDate::from_ymd_opt(2026, 4, 30).unwrap(),
        );
        let p3 = Period::new(
            NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(),
            NaiveDate::from_ymd_opt(2026, 6, 30).unwrap(),
        );

        assert!(p1.overlaps(&p2));
        assert!(!p1.overlaps(&p3));
    }

    #[test]
    fn test_year_month_next() {
        let ym = YearMonth::new(2026, 11);
        let next = ym.next();
        assert_eq!(next, YearMonth::new(2026, 12));

        let dec = YearMonth::new(2026, 12);
        let jan = dec.next();
        assert_eq!(jan, YearMonth::new(2027, 1));
    }

    #[test]
    fn test_year_month_first_last_day() {
        let ym = YearMonth::new(2026, 2);
        assert_eq!(ym.first_day(), NaiveDate::from_ymd_opt(2026, 2, 1).unwrap());
        assert_eq!(ym.last_day(), NaiveDate::from_ymd_opt(2026, 2, 28).unwrap());
    }

    #[test]
    fn test_year_month_ordering() {
        let a = YearMonth::new(2026, 1);
        let b = YearMonth::new(2026, 6);
        let c = YearMonth::new(2027, 1);
        assert!(a < b);
        assert!(b < c);
    }

    #[test]
    fn test_year_month_period() {
        let ym = YearMonth::new(2026, 3);
        let period = ym.period();
        assert_eq!(period.start(), NaiveDate::from_ymd_opt(2026, 3, 1).unwrap());
        assert_eq!(period.end(), NaiveDate::from_ymd_opt(2026, 3, 31).unwrap());
    }

    #[test]
    fn test_serde_roundtrip() {
        let ym = YearMonth::new(2026, 7);
        let json = serde_json::to_string(&ym).unwrap();
        let deserialized: YearMonth = serde_json::from_str(&json).unwrap();
        assert_eq!(ym, deserialized);
    }
}
