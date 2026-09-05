use crate::shared::ids::UserID;
use chrono::{DateTime, Utc};

/// Authenticated user aggregate.
///
/// Stores credentials and profile information. The `password_hash`
/// field contains an Argon2id hash — never store plaintext passwords.
#[derive(Debug, Clone)]
pub struct User {
    pub id: UserID,
    pub email: String,
    pub password_hash: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

impl User {
    /// Creates a new user. The password must already be hashed by the caller.
    pub fn new(
        id: UserID,
        email: String,
        password_hash: String,
        name: String,
    ) -> Result<Self, super::repository::IdentityError> {
        if email.trim().is_empty() {
            return Err(super::repository::IdentityError::InvalidInput(
                "email must not be empty".into(),
            ));
        }
        if name.trim().is_empty() {
            return Err(super::repository::IdentityError::InvalidInput(
                "name must not be empty".into(),
            ));
        }
        Ok(Self {
            id,
            email,
            password_hash,
            name,
            created_at: Utc::now(),
        })
    }
}

/// Port for password hashing. Implementations live outside the domain
/// (or in the core for pure-computation adapters like Argon2).
pub trait PasswordHasher: Send + Sync {
    /// Hash a plaintext password.
    fn hash(&self, password: &str) -> Result<String, PasswordError>;
    /// Verify a plaintext password against a stored hash.
    fn verify(&self, password: &str, hash: &str) -> Result<bool, PasswordError>;
}

/// Errors from password hashing operations.
#[derive(Debug, thiserror::Error)]
pub enum PasswordError {
    #[error("hash failed: {0}")]
    HashFailed(String),
    #[error("invalid hash format: {0}")]
    InvalidFormat(String),
}

/// Argon2id password hasher (pure computation, no I/O).
pub struct Argon2PasswordHasher;

impl PasswordHasher for Argon2PasswordHasher {
    fn hash(&self, password: &str) -> Result<String, PasswordError> {
        use argon2::password_hash::SaltString;
        use argon2::{Argon2, PasswordHasher};
        use rand::rngs::OsRng;

        let salt = SaltString::generate(&mut OsRng);
        let hash = Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| PasswordError::HashFailed(e.to_string()))?;
        Ok(hash.to_string())
    }

    fn verify(&self, password: &str, hash: &str) -> Result<bool, PasswordError> {
        use argon2::password_hash::PasswordHash;
        use argon2::{Argon2, PasswordVerifier};

        let parsed =
            PasswordHash::new(hash).map_err(|e| PasswordError::InvalidFormat(e.to_string()))?;
        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_creation() {
        let id = UserID::new();
        let user = User::new(
            id,
            "alice@example.com".into(),
            "hash123".into(),
            "Alice".into(),
        )
        .unwrap();
        assert_eq!(user.id, id);
        assert_eq!(user.email, "alice@example.com");
        assert_eq!(user.name, "Alice");
        assert_eq!(user.password_hash, "hash123");
    }

    #[test]
    fn test_user_empty_email_rejected() {
        let result = User::new(UserID::new(), "".into(), "hash".into(), "Alice".into());
        assert!(result.is_err());
    }

    #[test]
    fn test_user_whitespace_email_rejected() {
        let result = User::new(UserID::new(), "   ".into(), "hash".into(), "Alice".into());
        assert!(result.is_err());
    }

    #[test]
    fn test_user_empty_name_rejected() {
        let result = User::new(
            UserID::new(),
            "alice@example.com".into(),
            "hash".into(),
            "".into(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_argon2_hash_and_verify() {
        let hasher = Argon2PasswordHasher;
        let hash = hasher.hash("my_secret_password").unwrap();
        assert!(hasher.verify("my_secret_password", &hash).unwrap());
        assert!(!hasher.verify("wrong_password", &hash).unwrap());
    }

    #[test]
    fn test_argon2_different_hashes() {
        let hasher = Argon2PasswordHasher;
        let h1 = hasher.hash("password").unwrap();
        let h2 = hasher.hash("password").unwrap();
        // Salt is random, so hashes should differ
        assert_ne!(h1, h2);
        // But both should verify
        assert!(hasher.verify("password", &h1).unwrap());
        assert!(hasher.verify("password", &h2).unwrap());
    }

    #[test]
    fn test_argon2_invalid_hash_format() {
        let hasher = Argon2PasswordHasher;
        let result = hasher.verify("password", "not_a_valid_hash");
        assert!(result.is_err());
    }
}
