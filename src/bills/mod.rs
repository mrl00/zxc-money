//! Bill management module.
//!
//! Handles scheduling, payment tracking, and due-date alerts for recurring
//! and one-time bills. Integrates with the [`ledger`](crate::ledger) context
//! via events: when a bill is paid, an expense transaction is automatically
//! created.
//!
//! # Architecture
//!
//! - [`domain`] — `Bill` aggregate, events, repository trait
//! - [`application`] — command handlers (schedule, mark paid,
//!   cross-context Ledger integration) and query handlers (calendar view)
//! - [`projections`] — event-driven read models for calendar queries
//! - [`facade`] — unified entry point for front-end consumption

pub mod application;
pub mod domain;
pub mod facade;
pub mod projections;
