use std::collections::HashMap;
use std::sync::Arc;

use crate::ledger::domain::events::TransactionRecorded;
use crate::ledger::domain::transaction::TransactionType;
use crate::reporting::projections::account_balance::CategoryReport;
use crate::shared::events::DomainEvent;
use crate::shared::ids::CategoryID;
use crate::shared::money::Money;
use crate::shared::period::Period;

/// Query to get spending breakdown by category within a date range.
pub struct GetCategoryReportQuery {
    pub from: chrono::NaiveDate,
    pub to: chrono::NaiveDate,
}

/// Handles [`GetCategoryReportQuery`] by aggregating expense transactions by category.
pub struct GetCategoryReportHandler {
    events: Arc<Vec<Box<dyn DomainEvent + Send + Sync>>>,
}

impl GetCategoryReportHandler {
    /// Creates a new handler with a set of pre-collected events.
    ///
    /// In a real system this would query a transaction repository.
    /// For now, we accept events to aggregate from.
    pub fn new(events: Arc<Vec<Box<dyn DomainEvent + Send + Sync>>>) -> Self {
        Self { events }
    }

    /// Executes the category report query.
    pub fn handle(&self, query: GetCategoryReportQuery) -> Vec<CategoryReport> {
        let period = Period::new(query.from, query.to);
        let mut category_totals: HashMap<CategoryID, (Money, usize)> = HashMap::new();

        for event in self.events.iter() {
            if let Some(e) = event.as_any().downcast_ref::<TransactionRecorded>() {
                if e.tx_type != TransactionType::Expense {
                    continue;
                }
                if !period.contains(e.date) {
                    continue;
                }
                if let Some(category_id) = e.category_id {
                    let entry = category_totals
                        .entry(category_id)
                        .or_insert_with(|| (Money::zero(crate::shared::money::Currency::BRL), 0));
                    entry.0 = (entry.0 + e.amount).unwrap();
                    entry.1 += 1;
                }
            }
        }

        let mut reports: Vec<CategoryReport> = category_totals
            .into_iter()
            .map(|(category_id, (total, count))| CategoryReport {
                category_id,
                category_name: String::new(), // Name resolution left to frontend
                total,
                transaction_count: count,
            })
            .collect();

        reports.sort_by_key(|b| std::cmp::Reverse(b.total.amount()));
        reports
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::ids::{AccountID, TransactionID};
    use crate::shared::money::Currency;

    fn expense_event(
        cat: CategoryID,
        amount: i64,
        date: chrono::NaiveDate,
    ) -> Box<TransactionRecorded> {
        Box::new(TransactionRecorded {
            transaction_id: TransactionID::new(),
            account_id: AccountID::new(),
            tx_type: TransactionType::Expense,
            amount: Money::from_cents(amount, Currency::BRL),
            category_id: Some(cat),
            description: "Test".into(),
            date,
            timestamp: chrono::Utc::now(),
        })
    }

    #[test]
    fn test_category_report_groups_by_category() {
        let cat1 = CategoryID::new();
        let cat2 = CategoryID::new();

        let events: Vec<Box<dyn DomainEvent + Send + Sync>> = vec![
            expense_event(
                cat1,
                10000,
                chrono::NaiveDate::from_ymd_opt(2026, 1, 5).unwrap(),
            ),
            expense_event(
                cat1,
                20000,
                chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            ),
            expense_event(
                cat2,
                50_00,
                chrono::NaiveDate::from_ymd_opt(2026, 1, 20).unwrap(),
            ),
        ];

        let handler = GetCategoryReportHandler::new(Arc::new(events));
        let reports = handler.handle(GetCategoryReportQuery {
            from: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            to: chrono::NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
        });

        assert_eq!(reports.len(), 2);
        assert_eq!(
            reports[0].total.amount(),
            rust_decimal::Decimal::from(30000) / rust_decimal::Decimal::from(100)
        ); // cat1 first (highest)
        assert_eq!(reports[0].transaction_count, 2);
        assert_eq!(
            reports[1].total.amount(),
            rust_decimal::Decimal::from(5000) / rust_decimal::Decimal::from(100)
        ); // cat2 second
    }

    #[test]
    fn test_category_report_filters_by_period() {
        let cat = CategoryID::new();

        let events: Vec<Box<dyn DomainEvent + Send + Sync>> = vec![
            expense_event(
                cat,
                10000,
                chrono::NaiveDate::from_ymd_opt(2026, 1, 5).unwrap(),
            ),
            expense_event(
                cat,
                20000,
                chrono::NaiveDate::from_ymd_opt(2026, 2, 5).unwrap(),
            ),
        ];

        let handler = GetCategoryReportHandler::new(Arc::new(events));
        let reports = handler.handle(GetCategoryReportQuery {
            from: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            to: chrono::NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
        });

        assert_eq!(reports.len(), 1);
        assert_eq!(
            reports[0].total.amount(),
            rust_decimal::Decimal::from(10000) / rust_decimal::Decimal::from(100)
        );
    }

    #[test]
    fn test_category_report_ignores_income() {
        let cat = CategoryID::new();

        let events: Vec<Box<dyn DomainEvent + Send + Sync>> = vec![Box::new(TransactionRecorded {
            transaction_id: TransactionID::new(),
            account_id: AccountID::new(),
            tx_type: TransactionType::Income,
            amount: Money::from_cents(500000, Currency::BRL),
            category_id: Some(cat),
            description: "Salary".into(),
            date: chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            timestamp: chrono::Utc::now(),
        })];

        let handler = GetCategoryReportHandler::new(Arc::new(events));
        let reports = handler.handle(GetCategoryReportQuery {
            from: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            to: chrono::NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
        });

        assert!(reports.is_empty());
    }
}
