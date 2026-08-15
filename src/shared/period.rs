use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Period {
    start: NaiveDate,
    end: NaiveDate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct YearMonth {
    pub year: i32,
    pub month: u32,
}

impl Period {
    pub fn new(start: NaiveDate, end: NaiveDate) -> Self {
        assert!(start <= end, "start date must be <= end date");
        Self { start, end }
    }

    pub fn start(&self) -> NaiveDate {
        self.start
    }

    pub fn end(&self) -> NaiveDate {
        self.end
    }

    pub fn contains(&self, date: NaiveDate) -> bool {
        date >= self.start && date <= self.end
    }

    pub fn overlaps(&self, other: &Period) -> bool {
        self.start <= other.end && other.start <= self.end
    }

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
    pub fn new(year: i32, month: u32) -> Self {
        assert!((1..=12).contains(&month), "month must be 1-12");
        Self { year, month }
    }

    pub fn from_date(date: NaiveDate) -> Self {
        Self {
            year: date.year(),
            month: date.month(),
        }
    }

    pub fn first_day(&self) -> NaiveDate {
        NaiveDate::from_ymd_opt(self.year, self.month, 1).expect("invalid date")
    }

    pub fn last_day(&self) -> NaiveDate {
        let next_month = self.next();
        next_month.first_day().pred_opt().expect("invalid date")
    }

    pub fn period(&self) -> Period {
        Period::new(self.first_day(), self.last_day())
    }

    pub fn next(&self) -> YearMonth {
        if self.month == 12 {
            YearMonth::new(self.year + 1, 1)
        } else {
            YearMonth::new(self.year, self.month + 1)
        }
    }

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
