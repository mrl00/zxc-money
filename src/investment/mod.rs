//! Investment module.
//!
//! Manages asset tracking, portfolio positions, buy/sell operations,
//! and profitability calculations.
//!
//! # Architecture
//!
//! - [`domain`] — [`Asset`](domain::asset::Asset) catalog,
//!   [`Portfolio`](domain::portfolio::Portfolio) aggregate with
//!   [`Position`](domain::position::Position) entities, and the
//!   [`QuoteProvider`](domain::quote::QuoteProvider) port
//! - [`application`] — command handlers (register, buy, sell) and
//!   query handlers (profitability, portfolio summary)
//! - [`facade`] — unified entry point for front-end consumption

pub mod application;
pub mod domain;
pub mod facade;
