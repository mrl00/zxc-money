//! Domain layer for the bills module.
//!
//! Contains the core business types: [`Bill`](bill::Bill), domain events, and the [`BillRepository`](repository::BillRepository) trait.

pub mod bill;
pub mod events;
pub mod repository;
