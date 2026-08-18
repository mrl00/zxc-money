//! Provider ports for infrastructure concerns.
//!
//! Contains traits that abstract away infrastructure details (ID generation,
//! datetime). The core defines these ports; frontends provide concrete implementations.

pub mod datetime;
pub mod id;

pub use datetime::{DateTimeProvider, SystemDateTime};
pub use id::{IdGenerator, UuidGenerator};
