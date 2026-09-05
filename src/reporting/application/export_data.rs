use std::sync::Arc;

use crate::reporting::projections::cash_flow::CashFlowStore;
use crate::reporting::projections::net_worth::NetWorthStore;
use crate::shared::ids::Principal;
use crate::shared::period::Period;

/// Query to export all reporting data as JSON.
pub struct ExportDataQuery {
    pub principal: Principal,
    pub period: Period,
}

/// Serialized reporting data for JSON export.
#[derive(serde::Serialize)]
pub struct ReportingExport {
    pub net_worth: NetWorthExport,
    pub cash_flow: Vec<CashFlowExport>,
}

#[derive(serde::Serialize)]
pub struct NetWorthExport {
    pub date: String,
    pub total_assets: rust_decimal::Decimal,
    pub total_liabilities: rust_decimal::Decimal,
    pub net_worth: rust_decimal::Decimal,
}

#[derive(serde::Serialize)]
pub struct CashFlowExport {
    pub date: String,
    pub income: rust_decimal::Decimal,
    pub expense: rust_decimal::Decimal,
    pub net: rust_decimal::Decimal,
}

/// Handles [`ExportDataQuery`] by serializing reporting projections to JSON.
pub struct ExportDataHandler {
    net_worth_store: Arc<NetWorthStore>,
    cash_flow_store: Arc<CashFlowStore>,
}

impl ExportDataHandler {
    /// Creates a new handler with the given stores.
    pub fn new(net_worth_store: Arc<NetWorthStore>, cash_flow_store: Arc<CashFlowStore>) -> Self {
        Self {
            net_worth_store,
            cash_flow_store,
        }
    }

    /// Exports reporting data as a JSON string.
    pub fn handle(&self, query: ExportDataQuery) -> Result<String, serde_json::Error> {
        let snapshot = self.net_worth_store.snapshot(query.period.end());

        let net_worth = NetWorthExport {
            date: snapshot.date.to_string(),
            total_assets: snapshot.total_assets.amount(),
            total_liabilities: snapshot.total_liabilities.amount(),
            net_worth: snapshot.net_worth.amount(),
        };

        let entries = self.cash_flow_store.get_period(query.period);
        let cash_flow: Vec<CashFlowExport> = entries
            .iter()
            .map(|e| CashFlowExport {
                date: e.date.to_string(),
                income: e.income.amount(),
                expense: e.expense.amount(),
                net: e.net.amount(),
            })
            .collect();

        let export = ReportingExport {
            net_worth,
            cash_flow,
        };

        serde_json::to_string_pretty(&export)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::domain::events::AccountOpened;
    use crate::ledger::domain::events::TransactionRecorded;
    use crate::ledger::domain::transaction::TransactionType;
    use crate::shared::ids::{AccountID, CategoryID, Principal, TransactionID, UserID};
    use crate::shared::money::{Currency, Money};

    #[test]
    fn test_export_json() {
        let nw_store = Arc::new(NetWorthStore::new());
        let cf_store = Arc::new(CashFlowStore::new());
        let handler = ExportDataHandler::new(nw_store.clone(), cf_store.clone());

        let account_id = AccountID::new();

        nw_store.handle_event(&AccountOpened {
            account_id,
            owner_id: UserID::new(),
            name: "Checking".into(),
            currency: Currency::BRL,
            opening_balance: Money::from_cents(1000000, Currency::BRL),
            timestamp: chrono::Utc::now(),
        });

        cf_store.handle_event(
            &TransactionRecorded {
                transaction_id: TransactionID::new(),
                account_id,
                owner_id: UserID::new(),
                tx_type: TransactionType::Income,
                amount: Money::from_cents(500000, Currency::BRL),
                category_id: Some(CategoryID::new()),
                description: "Salary".into(),
                date: chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
                timestamp: chrono::Utc::now(),
            },
            &std::sync::Mutex::new(std::collections::HashMap::new()),
        );

        let period = Period::new(
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
        );

        let json = handler
            .handle(ExportDataQuery {
                principal: Principal::new(UserID::new()),
                period,
            })
            .unwrap();
        assert!(json.contains("10000"));
        assert!(json.contains("5000"));
    }

    #[test]
    fn test_export_empty() {
        let nw_store = Arc::new(NetWorthStore::new());
        let cf_store = Arc::new(CashFlowStore::new());
        let handler = ExportDataHandler::new(nw_store, cf_store);

        let period = Period::new(
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
        );

        let json = handler
            .handle(ExportDataQuery {
                principal: Principal::new(UserID::new()),
                period,
            })
            .unwrap();
        assert!(json.contains("net_worth"));
        assert!(json.contains("cash_flow"));
    }
}
