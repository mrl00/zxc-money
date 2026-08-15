use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

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

define_id!(AccountID);
define_id!(TransactionID);
define_id!(CategoryID);
define_id!(TagID);
define_id!(BudgetID);
define_id!(GoalID);
define_id!(CreditCardID);
define_id!(InvoiceID);
define_id!(PurchaseID);
define_id!(BillID);
define_id!(AssetID);
define_id!(PortfolioID);

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
