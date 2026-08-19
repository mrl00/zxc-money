//! Shared value objects, error types, event system, and utilities.
//!
//! This module contains the building blocks used across all bounded contexts:
//!
//! - [`ids`] — Type-safe UUID wrappers for all aggregate identifiers
//! - [`money`] — `Money` value object with currency-safe arithmetic
//! - [`period`] — `Period` and `YearMonth` date range types
//! - [`errors`] — Error enums per bounded context
//! - [`events`] — Domain event trait, publisher, and in-memory dispatcher
//! - [`repository`] — Generic repository and unit-of-work traits
//! - [`mock`] — In-memory mock repositories for testing

pub mod errors;
pub mod events;
pub mod ids;
pub mod mock;
pub mod money;
pub mod period;
pub mod repository;
