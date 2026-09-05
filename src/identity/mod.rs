//! Identity bounded context — user management and authentication.
//!
//! This module provides the capability for multi-user authentication.
//! In single-user scenarios (e.g. local TUI), the front may bypass
//! this module entirely and inject a hardcoded [`Principal`].
//!
//! [`Principal`]: crate::shared::ids::Principal

pub mod application;
pub mod domain;
