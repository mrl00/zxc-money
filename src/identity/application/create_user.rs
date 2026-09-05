use crate::identity::domain::repository::{IdentityError, UserRepository};
use crate::identity::domain::user::{PasswordHasher, User};
use crate::provider::IdGenerator;
use crate::shared::ids::UserID;
use std::sync::Arc;

/// Command to create a new user account.
pub struct CreateUserCommand {
    pub email: String,
    pub password: String,
    pub name: String,
}

/// Handler for creating new user accounts.
pub struct CreateUserHandler<R: UserRepository, H: PasswordHasher, I: IdGenerator> {
    user_repository: Arc<R>,
    password_hasher: Arc<H>,
    id_generator: Arc<I>,
}

impl<R: UserRepository, H: PasswordHasher, I: IdGenerator> CreateUserHandler<R, H, I> {
    /// Creates a new handler with the given dependencies.
    pub fn new(user_repository: Arc<R>, password_hasher: Arc<H>, id_generator: Arc<I>) -> Self {
        Self {
            user_repository,
            password_hasher,
            id_generator,
        }
    }

    /// Executes the create-user command.
    ///
    /// # Errors
    /// - [`IdentityError::EmailAlreadyExists`] if the email is already registered.
    /// - [`IdentityError::InvalidInput`] if email or name are empty, or hashing fails.
    /// - [`IdentityError::Repository`] on persistence failures.
    pub async fn handle(&self, cmd: CreateUserCommand) -> Result<UserID, IdentityError> {
        // Check email uniqueness
        if self
            .user_repository
            .find_by_email(&cmd.email)
            .await?
            .is_some()
        {
            return Err(IdentityError::EmailAlreadyExists(cmd.email));
        }

        // Hash password
        let password_hash = self
            .password_hasher
            .hash(&cmd.password)
            .map_err(|e| IdentityError::InvalidInput(e.to_string()))?;

        // Create user
        let id = UserID::from_uuid(self.id_generator.new_id());
        let user = User::new(id, cmd.email, password_hash, cmd.name)?;

        // Persist
        self.user_repository.save(&user).await?;

        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::domain::user::Argon2PasswordHasher;
    use crate::provider::id::MockIdGenerator;
    use crate::shared::mock::MockUserRepository;
    use uuid::Uuid;

    fn setup() -> (
        Arc<MockUserRepository>,
        Arc<Argon2PasswordHasher>,
        Arc<MockIdGenerator>,
    ) {
        let user_repo = Arc::new(MockUserRepository::new());
        let hasher = Arc::new(Argon2PasswordHasher);
        let id_gen = Arc::new(MockIdGenerator::new(Uuid::nil()));
        (user_repo, hasher, id_gen)
    }

    #[tokio::test]
    async fn test_create_user_happy_path() {
        let (user_repo, hasher, id_gen) = setup();
        let handler = CreateUserHandler::new(user_repo.clone(), hasher, id_gen);

        let cmd = CreateUserCommand {
            email: "alice@example.com".into(),
            password: "secret123".into(),
            name: "Alice".into(),
        };

        let user_id = handler.handle(cmd).await.unwrap();
        let stored = user_repo.find_by_id(user_id).await.unwrap().unwrap();
        assert_eq!(stored.email, "alice@example.com");
        assert_eq!(stored.name, "Alice");
    }

    #[tokio::test]
    async fn test_create_user_duplicate_email() {
        let (user_repo, hasher, id_gen) = setup();
        let handler = CreateUserHandler::new(user_repo.clone(), hasher.clone(), id_gen.clone());

        let cmd = CreateUserCommand {
            email: "alice@example.com".into(),
            password: "secret123".into(),
            name: "Alice".into(),
        };
        handler.handle(cmd).await.unwrap();

        let cmd2 = CreateUserCommand {
            email: "alice@example.com".into(),
            password: "another_password".into(),
            name: "Alice2".into(),
        };
        let result = handler.handle(cmd2).await;
        assert!(matches!(result, Err(IdentityError::EmailAlreadyExists(_))));
    }

    #[tokio::test]
    async fn test_create_user_empty_email() {
        let (user_repo, hasher, id_gen) = setup();
        let handler = CreateUserHandler::new(user_repo, hasher, id_gen);

        let cmd = CreateUserCommand {
            email: "".into(),
            password: "secret123".into(),
            name: "Alice".into(),
        };
        let result = handler.handle(cmd).await;
        assert!(matches!(result, Err(IdentityError::InvalidInput(_))));
    }
}
