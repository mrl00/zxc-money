use std::sync::Arc;

use crate::reporting::application::get_cash_flow::{GetCashFlowHandler, GetCashFlowQuery};
use crate::reporting::application::get_category_report::{
    GetCategoryReportHandler, GetCategoryReportQuery,
};
use crate::reporting::application::get_monthly_summary::{
    GetMonthlySummaryHandler, GetMonthlySummaryQuery, MonthlySummary,
};
use crate::reporting::application::get_net_worth::{GetNetWorthHandler, GetNetWorthQuery};
use crate::reporting::application::get_top_expenses::{
    GetTopExpensesHandler, GetTopExpensesQuery, TopExpenseEntry,
};
use crate::reporting::projections::account_balance::{
    CashFlowEntry, CategoryReport, NetWorthSnapshot,
};
use crate::reporting::projections::cash_flow::CashFlowStore;
use crate::reporting::projections::net_worth::NetWorthStore;

/// Facade aggregating all reporting query use cases for front-end consumption.
pub struct Facade {
    get_monthly_summary: GetMonthlySummaryHandler,
    get_category_report: GetCategoryReportHandler,
    get_net_worth: GetNetWorthHandler,
    get_cash_flow: GetCashFlowHandler,
    get_top_expenses: GetTopExpensesHandler,
}

impl Facade {
    /// Creates a new [`Facade`] with the given stores.
    pub fn new(
        cash_flow_store: Arc<CashFlowStore>,
        net_worth_store: Arc<NetWorthStore>,
        events: Arc<Vec<Box<dyn crate::shared::events::DomainEvent + Send + Sync>>>,
    ) -> Self {
        Self {
            get_monthly_summary: GetMonthlySummaryHandler::new(cash_flow_store.clone()),
            get_category_report: GetCategoryReportHandler::new(events.clone()),
            get_net_worth: GetNetWorthHandler::new(net_worth_store),
            get_cash_flow: GetCashFlowHandler::new(cash_flow_store),
            get_top_expenses: GetTopExpensesHandler::new(events),
        }
    }

    /// Gets a monthly summary of income, expenses, and balance.
    pub fn get_monthly_summary(&self, query: GetMonthlySummaryQuery) -> MonthlySummary {
        self.get_monthly_summary.handle(query)
    }

    /// Gets spending breakdown by category within a date range.
    pub fn get_category_report(&self, query: GetCategoryReportQuery) -> Vec<CategoryReport> {
        self.get_category_report.handle(query)
    }

    /// Gets a net worth snapshot.
    pub fn get_net_worth(&self, query: GetNetWorthQuery) -> NetWorthSnapshot {
        self.get_net_worth.handle(query)
    }

    /// Gets cash flow entries for the last N months.
    pub fn get_cash_flow(&self, query: GetCashFlowQuery) -> Vec<CashFlowEntry> {
        self.get_cash_flow.handle(query)
    }

    /// Gets the top N expenses within a date range.
    pub fn get_top_expenses(&self, query: GetTopExpensesQuery) -> Vec<TopExpenseEntry> {
        self.get_top_expenses.handle(query)
    }
}
