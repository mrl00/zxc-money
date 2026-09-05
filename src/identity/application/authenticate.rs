use crate::identity::domain::repository::{IdentityError, UserRepository};
use crate::identity::domain::user::PasswordHasher;
use crate::shared::ids::Principal;
use std::sync::Arc;

/// Command to authenticate a user.
pub struct AuthenticateUserCommand {
    pub email: String,
    pub password: String,
}

/// Handler for user authentication. Returns a [`Principal`] on success.
pub struct AuthenticateUserHandler<R: UserRepository, H: PasswordHasher> {
    user_repository: Arc<R>,
    password_hasher: Arc<H>,
}

impl<R: UserRepository, H: PasswordHasher> AuthenticateUserHandler<R, H> {
    pub fn new(user_repository: Arc<R>, password_hasher: Arc<H>) -> Self {
        Self {
            user_repository,
            password_hasher,
        }
    }

    pub async fn handle(&self, cmd: AuthenticateUserCommand) -> Result<Principal, IdentityError> {
        let user = self
            .user_repository
            .find_by_email(&cmd.email)
            .await?
            .ok_or(IdentityError::InvalidCredentials)?;

        let valid = self
            .password_hasher
            .verify(&cmd.password, &user.password_hash)
            .map_err(|e| IdentityError::InvalidInput(e.to_string()))?;

        if !valid {
            return Err(IdentityError::InvalidCredentials);
        }

        Ok(Principal::new(user.id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::application::create_user::{CreateUserCommand, CreateUserHandler};
    use crate::identity::domain::user::Argon2PasswordHasher;
    use crate::provider::id::MockIdGenerator;
    use crate::shared::mock::MockUserRepository;
    use uuid::Uuid;

    async fn setup_user() -> (Arc<MockUserRepository>, Arc<Argon2PasswordHasher>) {
        let user_repo = Arc::new(MockUserRepository::new());
        let hasher = Arc::new(Argon2PasswordHasher);
        let id_gen = Arc::new(MockIdGenerator::new(Uuid::nil()));
        let create_handler = CreateUserHandler::new(user_repo.clone(), hasher.clone(), id_gen);

        create_handler
            .handle(CreateUserCommand {
                email: "alice@example.com".into(),
                password: "secret123".into(),
                name: "Alice".into(),
            })
            .await
            .unwrap();

        (user_repo, hasher)
    }

    #[tokio::test]
    async fn test_authenticate_success() {
        let (user_repo, hasher) = setup_user().await;
        let handler = AuthenticateUserHandler::new(user_repo, hasher);

        let principal = handler
            .handle(AuthenticateUserCommand {
                email: "alice@example.com".into(),
                password: "secret123".into(),
            })
            .await
            .unwrap();

        // The user was created with MockIdGenerator(Uuid::nil())
        assert_eq!(*principal.user_id.as_uuid(), Uuid::nil());
    }

    #[tokio::test]
    async fn test_authenticate_wrong_password() {
        let (user_repo, hasher) = setup_user().await;
        let handler = AuthenticateUserHandler::new(user_repo, hasher);

        let result = handler
            .handle(AuthenticateUserCommand {
                email: "alice@example.com".into(),
                password: "wrong_password".into(),
            })
            .await;

        assert!(matches!(result, Err(IdentityError::InvalidCredentials)));
    }

    #[tokio::test]
    async fn test_authenticate_unknown_email() {
        let (user_repo, hasher) = setup_user().await;
        let handler = AuthenticateUserHandler::new(user_repo, hasher);

        let result = handler
            .handle(AuthenticateUserCommand {
                email: "bob@example.com".into(),
                password: "secret123".into(),
            })
            .await;

        assert!(matches!(result, Err(IdentityError::InvalidCredentials)));
    }
}
