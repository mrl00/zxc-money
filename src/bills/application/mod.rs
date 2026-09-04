//! Application layer for the bills module.
//!
//! Contains use-case handlers for scheduling and paying bills, querying
//! the calendar view, and cross-context integration with the Ledger.

pub mod bill_paid_handler;
pub mod get_bills_by_month;
pub mod get_daily_bill_totals;
pub mod get_upcoming_bills;
pub mod mark_bill_paid;
pub mod schedule_bill;
