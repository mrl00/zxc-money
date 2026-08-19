//! Provider ports for infrastructure concerns.
//!
//! Contains traits that abstract away infrastructure details (ID generation,
//! datetime, statement parsing). The core defines these ports; frontends
//! provide concrete implementations.

pub mod datetime;
pub mod id;
pub mod parser;

pub use datetime::{DateTimeProvider, SystemDateTime};
pub use id::{IdGenerator, UuidGenerator};
pub use parser::{ColumnMapping, StatementParser};
