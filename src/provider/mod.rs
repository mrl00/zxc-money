//! Provider ports for infrastructure concerns.
//!
//! Contains traits that abstract away infrastructure details (ID generation,
//! datetime, statement parsing, notifications, file storage, bank APIs).
//! The core defines these ports; frontends provide concrete implementations.

pub mod bank;
pub mod datetime;
pub mod id;
pub mod notification;
pub mod parser;
pub mod storage;

pub use bank::{BankError, BankProvider};
pub use datetime::{DateTimeProvider, SystemDateTime};
pub use id::{IdGenerator, UuidGenerator};
pub use notification::{NotificationError, NotificationProvider};
pub use parser::{ColumnMapping, StatementParser};
pub use storage::{FileStorage, StorageError};
