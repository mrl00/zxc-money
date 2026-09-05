//! Domain layer for the identity bounded context.
//!
//! Contains the [`user::User`] aggregate, [`user::PasswordHasher`] port,
//! and [`repository::UserRepository`] trait.

pub mod repository;
pub mod user;
