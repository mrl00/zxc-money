//! Credit card domain and application layer.
//!
//! This module implements the credit card bounded context, containing
//! domain models for [`domain::card::CreditCard`], [`domain::invoice::Invoice`],
//! [`domain::purchase::Purchase`], and the application commands that orchestrate them.

pub mod application;
pub mod domain;
