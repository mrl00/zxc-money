//! Domain layer for the budgeting module.
//!
//! Contains [`Budget`](budget::Budget), [`FinancialGoal`](goal::FinancialGoal), domain events, and repository traits.

pub mod budget;
pub mod events;
pub mod goal;
pub mod repository;
