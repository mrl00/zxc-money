//! Domain layer for the investment module.
//!
//! Contains [`Asset`](asset::Asset), [`Position`](position::Position), [`Portfolio`](portfolio::Portfolio),
//! domain events, and repository traits.

pub mod asset;
pub mod events;
pub mod portfolio;
pub mod position;
pub mod repository;
