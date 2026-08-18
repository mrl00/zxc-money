//! Type-safe UUID wrappers for all aggregate identifiers.
//!
//! Each ID type is a newtype around `Uuid`, providing compile-time
//! distinction between different identifier domains. All IDs implement
//! `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Serialize`,
//! `Deserialize`, `Display`, and `Default`.
//!
//! # Example
//!
//! ```ignore
//! use zxc_money::shared::ids::AccountID;
//!
//! let id = AccountID::new();          // random UUID v4
//! let id2 = AccountID::from_uuid(uuid); // wrap existing UUID
//! println!("{id}");                   // displays as UUID string
//! ```

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Generates a type-safe ID newtype wrapper around `Uuid`.
///
/// Each invocation creates a new struct with `new()`, `from_uuid()`,
/// `as_uuid()`, `Default`, and `Display` implementations.
macro_rules! define_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub struct $name(Uuid);

        impl $name {
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }

            pub fn from_uuid(uuid: Uuid) -> Self {
                Self(uuid)
            }

            pub fn as_uuid(&self) -> &Uuid {
                &self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

// Identifier for `Account` aggregates.
define_id!(AccountID);
// Identifier for `Transaction` aggregates.
define_id!(TransactionID);
// Identifier for `Category` entities.
define_id!(CategoryID);
// Identifier for `Tag` entities.
define_id!(TagID);
// Identifier for `Budget` aggregates.
define_id!(BudgetID);
// Identifier for `FinancialGoal` aggregates.
define_id!(GoalID);
// Identifier for `CreditCard` aggregates.
define_id!(CreditCardID);
// Identifier for `Invoice` aggregates.
define_id!(InvoiceID);
// Identifier for `Purchase` entities.
define_id!(PurchaseID);
// Identifier for `Bill` aggregates.
define_id!(BillID);
// Identifier for `Asset` entities.
define_id!(AssetID);
// Identifier for `Portfolio` aggregates.
define_id!(PortfolioID);
// Identifier for the application user. Used as `owner_id` on all sensitive aggregates.
define_id!(UserID);
// Identifier for `RecurringTransaction` aggregates.
define_id!(RecurringTransactionID);
// Groups installment purchases of the same transaction across multiple invoices.
define_id!(InstallmentGroupID);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_creation() {
        let id = AccountID::new();
        assert!(!id.as_uuid().is_nil());
    }

    #[test]
    fn test_id_display() {
        let id = TransactionID::new();
        let s = format!("{id}");
        assert_eq!(s.len(), 36); // UUID format
    }

    #[test]
    fn test_id_equality() {
        let uuid = Uuid::new_v4();
        let a = AccountID::from_uuid(uuid);
        let b = AccountID::from_uuid(uuid);
        assert_eq!(a, b);
    }

    #[test]
    fn test_id_inequality() {
        let a = AccountID::new();
        let b = AccountID::new();
        assert_ne!(a, b);
    }

    #[test]
    fn test_serde_roundtrip() {
        let id = CategoryID::new();
        let json = serde_json::to_string(&id).unwrap();
        let deserialized: CategoryID = serde_json::from_str(&json).unwrap();
        assert_eq!(id, deserialized);
    }
}
