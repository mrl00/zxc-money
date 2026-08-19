//! Domain layer for the investment module.
//!
//! Contains aggregates ([`Asset`](asset::Asset),
//! [`Portfolio`](portfolio::Portfolio)), the [`Position`](position::Position)
//! entity, domain events, and the [`QuoteProvider`](quote::QuoteProvider) port.

pub mod asset;
pub mod events;
pub mod portfolio;
pub mod position;
pub mod quote;
pub mod repository;
