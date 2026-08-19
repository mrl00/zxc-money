use std::sync::Arc;

use crate::ledger::domain::repository::TransactionRepository;
use crate::reporting::application::export_data::{ExportDataHandler, ExportDataQuery};
use crate::reporting::application::export_transactions::ExportTransactionsHandler;
use crate::reporting::application::get_cash_flow::{GetCashFlowHandler, GetCashFlowQuery};
use crate::reporting::application::get_category_report::{
    GetCategoryReportHandler, GetCategoryReportQuery,
};
use crate::reporting::application::get_month_comparison::{
    GetMonthComparisonHandler, GetMonthComparisonQuery, MonthSummary,
};
use crate::reporting::application::get_monthly_summary::{
    GetMonthlySummaryHandler, GetMonthlySummaryQuery, MonthlySummary,
};
use crate::reporting::application::get_net_worth::{GetNetWorthHandler, GetNetWorthQuery};
use crate::reporting::application::get_net_worth_breakdown::{
    GetNetWorthBreakdownHandler, GetNetWorthBreakdownQuery, NetWorthBreakdown,
};
use crate::reporting::application::get_net_worth_history::{
    GetNetWorthHistoryHandler, GetNetWorthHistoryQuery,
};
use crate::reporting::application::get_top_expenses::{
    GetTopExpensesHandler, GetTopExpensesQuery, TopExpenseEntry,
};
use crate::reporting::application::get_year_comparison::{
    GetYearComparisonHandler, GetYearComparisonQuery, YearSummary,
};
use crate::reporting::projections::account_balance::{
    CashFlowEntry, CategoryReport, NetWorthSnapshot,
};
use crate::reporting::projections::cash_flow::CashFlowStore;
use crate::reporting::projections::net_worth::NetWorthStore;

/// Facade aggregating all reporting query use cases for front-end consumption.
pub struct ReportingFacade {
    get_monthly_summary: GetMonthlySummaryHandler,
    get_category_report: GetCategoryReportHandler,
    get_net_worth: GetNetWorthHandler,
    get_cash_flow: GetCashFlowHandler,
    get_top_expenses: GetTopExpensesHandler,
    get_net_worth_history: GetNetWorthHistoryHandler,
    get_net_worth_breakdown: GetNetWorthBreakdownHandler,
    get_month_comparison: GetMonthComparisonHandler,
    get_year_comparison: GetYearComparisonHandler,
    export_data: ExportDataHandler,
}

impl ReportingFacade {
    /// Creates a new [`ReportingFacade`] with the given stores.
    pub fn new(
        cash_flow_store: Arc<CashFlowStore>,
        net_worth_store: Arc<NetWorthStore>,
        events: Arc<Vec<Box<dyn crate::shared::events::DomainEvent + Send + Sync>>>,
    ) -> Self {
        Self {
            get_monthly_summary: GetMonthlySummaryHandler::new(cash_flow_store.clone()),
            get_category_report: GetCategoryReportHandler::new(events.clone()),
            get_net_worth: GetNetWorthHandler::new(net_worth_store.clone()),
            get_cash_flow: GetCashFlowHandler::new(cash_flow_store.clone()),
            get_top_expenses: GetTopExpensesHandler::new(events),
            get_net_worth_history: GetNetWorthHistoryHandler::new(net_worth_store.clone()),
            get_net_worth_breakdown: GetNetWorthBreakdownHandler::new(net_worth_store.clone()),
            get_month_comparison: GetMonthComparisonHandler::new(cash_flow_store.clone()),
            get_year_comparison: GetYearComparisonHandler::new(cash_flow_store.clone()),
            export_data: ExportDataHandler::new(net_worth_store, cash_flow_store),
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

    /// Gets net worth snapshots over a date range.
    pub fn get_net_worth_history(&self, query: GetNetWorthHistoryQuery) -> Vec<NetWorthSnapshot> {
        self.get_net_worth_history.handle(query)
    }

    /// Gets a breakdown of net worth by account and investment.
    pub fn get_net_worth_breakdown(&self, _query: GetNetWorthBreakdownQuery) -> NetWorthBreakdown {
        self.get_net_worth_breakdown.handle(_query)
    }

    /// Compares two months side by side.
    pub fn get_month_comparison(
        &self,
        query: GetMonthComparisonQuery,
    ) -> (MonthSummary, MonthSummary) {
        self.get_month_comparison.handle(query)
    }

    /// Compares multiple years.
    pub fn get_year_comparison(&self, query: GetYearComparisonQuery) -> Vec<YearSummary> {
        self.get_year_comparison.handle(query)
    }

    /// Exports all reporting data as JSON.
    pub fn export_data(&self, query: ExportDataQuery) -> Result<String, serde_json::Error> {
        self.export_data.handle(query)
    }

    /// Creates a CSV export handler for transactions.
    pub fn export_transactions<T: TransactionRepository>(
        &self,
        transaction_repository: Arc<T>,
    ) -> ExportTransactionsHandler<T> {
        ExportTransactionsHandler::new(transaction_repository)
    }
}
