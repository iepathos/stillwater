//! Canonical pure-core, imperative-shell example.
//!
//! The shell loads facts, the pure core decides whether registration is valid,
//! and the shell interprets the resulting plan. Saving and sending the welcome
//! email are deliberately sequential: a save failure prevents the email. An
//! email failure does not roll back a completed save; workflows that require
//! compensation should use a saga or another explicit transaction protocol.
//! Loaded facts are snapshots: the repository must enforce uniqueness atomically
//! when committing a plan. A conflict is distinct from an infrastructure failure.
//! The in-memory hasher is a test double, not a password-storage implementation.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use stillwater::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq)]
struct RegistrationInput {
    username: String,
    email: String,
    password: String,
    confirm_password: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct RegistrationFacts {
    username_taken: bool,
    email_taken: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RegistrationPlan {
    username: String,
    email: String,
    password: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct User {
    id: u64,
    username: String,
    email: String,
    password_hash: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum RegistrationError {
    UsernameRequired,
    UsernameLength,
    UsernameCharacters,
    EmailRequired,
    EmailFormat,
    PasswordLength,
    PasswordUppercase,
    PasswordLowercase,
    PasswordNumber,
    PasswordMismatch,
    UsernameTaken,
    EmailTaken,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum InfrastructureError {
    LoadFacts,
    HashPassword,
    SaveUser,
    SendEmail,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum AppError {
    Rejected(NonEmptyVec<RegistrationError>),
    Conflict(RegistrationError),
    Infrastructure(InfrastructureError),
}

#[derive(Debug)]
enum SaveError {
    Conflict(RegistrationError),
    Infrastructure(InfrastructureError),
}

impl From<SaveError> for AppError {
    fn from(error: SaveError) -> Self {
        match error {
            SaveError::Conflict(error) => Self::Conflict(error),
            SaveError::Infrastructure(error) => Self::Infrastructure(error),
        }
    }
}

fn validation_from_errors(
    errors: Vec<RegistrationError>,
) -> Validation<(), NonEmptyVec<RegistrationError>> {
    match NonEmptyVec::from_vec(errors) {
        Some(errors) => Validation::failure(errors),
        None => Validation::success(()),
    }
}

fn validate_username(username: &str) -> Validation<(), NonEmptyVec<RegistrationError>> {
    let mut errors = Vec::new();
    if username.is_empty() {
        errors.push(RegistrationError::UsernameRequired);
    }
    if !(3..=20).contains(&username.len()) {
        errors.push(RegistrationError::UsernameLength);
    }
    if !username
        .chars()
        .all(|character| character.is_alphanumeric() || matches!(character, '_' | '-'))
    {
        errors.push(RegistrationError::UsernameCharacters);
    }
    validation_from_errors(errors)
}

fn validate_email(email: &str) -> Validation<(), NonEmptyVec<RegistrationError>> {
    let mut errors = Vec::new();
    if email.is_empty() {
        errors.push(RegistrationError::EmailRequired);
    }
    if !email.contains('@') || !email.contains('.') || email.len() > 254 {
        errors.push(RegistrationError::EmailFormat);
    }
    validation_from_errors(errors)
}

fn validate_password(password: &str) -> Validation<(), NonEmptyVec<RegistrationError>> {
    let mut errors = Vec::new();
    if password.len() < 8 {
        errors.push(RegistrationError::PasswordLength);
    }
    if !password.chars().any(char::is_uppercase) {
        errors.push(RegistrationError::PasswordUppercase);
    }
    if !password.chars().any(char::is_lowercase) {
        errors.push(RegistrationError::PasswordLowercase);
    }
    if !password.chars().any(char::is_numeric) {
        errors.push(RegistrationError::PasswordNumber);
    }
    validation_from_errors(errors)
}

fn validate_password_match(
    password: &str,
    confirmation: &str,
) -> Validation<(), NonEmptyVec<RegistrationError>> {
    if password == confirmation {
        Validation::success(())
    } else {
        Validation::fail(RegistrationError::PasswordMismatch)
    }
}

fn require_available(
    available: bool,
    error: RegistrationError,
) -> Validation<(), NonEmptyVec<RegistrationError>> {
    if available {
        Validation::success(())
    } else {
        Validation::fail(error)
    }
}

/// Pure core: all decisions depend only on input data and loaded facts.
fn decide_registration(
    input: RegistrationInput,
    facts: RegistrationFacts,
) -> Validation<RegistrationPlan, NonEmptyVec<RegistrationError>> {
    Validation::<(), NonEmptyVec<RegistrationError>>::all((
        validate_username(&input.username),
        validate_email(&input.email),
        validate_password(&input.password),
        validate_password_match(&input.password, &input.confirm_password),
        require_available(!facts.username_taken, RegistrationError::UsernameTaken),
        require_available(!facts.email_taken, RegistrationError::EmailTaken),
    ))
    .map(|_| RegistrationPlan {
        username: input.username,
        email: input.email,
        password: input.password,
    })
}

trait UserRepository: Send + Sync {
    fn registration_facts<'a>(
        &'a self,
        username: &'a str,
        email: &'a str,
    ) -> BoxFuture<'a, Result<RegistrationFacts, InfrastructureError>>;

    // A real database adapter must use unique constraints, not another preflight query.
    fn save<'a>(&'a self, user: User) -> BoxFuture<'a, Result<User, SaveError>>;
}

trait PasswordHasher: Send + Sync {
    fn hash(&self, password: &str) -> Result<String, InfrastructureError>;
}

trait EmailSender: Send + Sync {
    fn send_welcome<'a>(&'a self, user: &'a User)
        -> BoxFuture<'a, Result<(), InfrastructureError>>;
}

#[derive(Clone)]
struct AppEnv {
    users: Arc<dyn UserRepository>,
    passwords: Arc<dyn PasswordHasher>,
    email: Arc<dyn EmailSender>,
}

fn load_registration_facts(
    input: RegistrationInput,
) -> impl Effect<Output = (RegistrationInput, RegistrationFacts), Error = AppError, Env = AppEnv> {
    from_async_ref(move |env: &AppEnv| {
        Box::pin(async move {
            env.users
                .registration_facts(&input.username, &input.email)
                .await
                .map(|facts| (input, facts))
                .map_err(AppError::Infrastructure)
        })
    })
}

fn interpret_registration(
    plan: RegistrationPlan,
) -> impl Effect<Output = User, Error = AppError, Env = AppEnv> {
    let RegistrationPlan {
        username,
        email,
        password,
    } = plan;

    from_fn(move |env: &AppEnv| {
        env.passwords
            .hash(&password)
            .map(|password_hash| User {
                id: 0,
                username,
                email,
                password_hash,
            })
            .map_err(AppError::Infrastructure)
    })
    .and_then(|user| {
        from_async_ref(move |env: &AppEnv| {
            Box::pin(async move { env.users.save(user).await.map_err(AppError::from) })
        })
    })
    .and_then(|user| {
        from_async_ref(move |env: &AppEnv| {
            Box::pin(async move {
                env.email
                    .send_welcome(&user)
                    .await
                    .map_err(AppError::Infrastructure)?;
                Ok(user)
            })
        })
    })
}

/// Imperative shell: load, decide, then interpret in an explicit order.
fn register_user(
    input: RegistrationInput,
) -> impl Effect<Output = User, Error = AppError, Env = AppEnv> {
    load_registration_facts(input)
        .and_then(|(input, facts)| {
            from_validation(decide_registration(input, facts).map_err(AppError::Rejected))
        })
        .and_then(interpret_registration)
}

#[derive(Default)]
struct InMemoryServices {
    users: Mutex<HashMap<u64, User>>,
    events: Mutex<Vec<&'static str>>,
    next_id: Mutex<u64>,
    load_error: Mutex<bool>,
    hash_error: Mutex<bool>,
    save_error: Mutex<bool>,
    email_error: Mutex<bool>,
}

impl InMemoryServices {
    fn app_env(self: &Arc<Self>) -> AppEnv {
        AppEnv {
            users: self.clone(),
            passwords: self.clone(),
            email: self.clone(),
        }
    }

    fn record(&self, event: &'static str) {
        self.events.lock().unwrap().push(event);
    }

    fn events(&self) -> Vec<&'static str> {
        self.events.lock().unwrap().clone()
    }
}

impl UserRepository for InMemoryServices {
    fn registration_facts<'a>(
        &'a self,
        username: &'a str,
        email: &'a str,
    ) -> BoxFuture<'a, Result<RegistrationFacts, InfrastructureError>> {
        Box::pin(async move {
            self.record("load_facts");
            if *self.load_error.lock().unwrap() {
                return Err(InfrastructureError::LoadFacts);
            }
            let users = self.users.lock().unwrap();
            Ok(RegistrationFacts {
                username_taken: users.values().any(|user| user.username == username),
                email_taken: users.values().any(|user| user.email == email),
            })
        })
    }

    fn save<'a>(&'a self, mut user: User) -> BoxFuture<'a, Result<User, SaveError>> {
        Box::pin(async move {
            self.record("save");
            if *self.save_error.lock().unwrap() {
                return Err(SaveError::Infrastructure(InfrastructureError::SaveUser));
            }
            let mut next_id = self.next_id.lock().unwrap();
            let mut users = self.users.lock().unwrap();
            if users.values().any(|saved| saved.username == user.username) {
                return Err(SaveError::Conflict(RegistrationError::UsernameTaken));
            }
            if users.values().any(|saved| saved.email == user.email) {
                return Err(SaveError::Conflict(RegistrationError::EmailTaken));
            }
            *next_id += 1;
            user.id = *next_id;
            users.insert(user.id, user.clone());
            Ok(user)
        })
    }
}

impl PasswordHasher for InMemoryServices {
    fn hash(&self, password: &str) -> Result<String, InfrastructureError> {
        self.record("hash");
        if *self.hash_error.lock().unwrap() {
            Err(InfrastructureError::HashPassword)
        } else {
            Ok(format!("hashed:{password}"))
        }
    }
}

impl EmailSender for InMemoryServices {
    fn send_welcome<'a>(
        &'a self,
        _user: &'a User,
    ) -> BoxFuture<'a, Result<(), InfrastructureError>> {
        Box::pin(async move {
            self.record("email");
            if *self.email_error.lock().unwrap() {
                Err(InfrastructureError::SendEmail)
            } else {
                Ok(())
            }
        })
    }
}

fn valid_input() -> RegistrationInput {
    RegistrationInput {
        username: "stillwater_user".to_string(),
        email: "user@example.com".to_string(),
        password: "CalmWater7".to_string(),
        confirm_password: "CalmWater7".to_string(),
    }
}

#[tokio::main]
async fn main() {
    let services = Arc::new(InMemoryServices::default());
    let result = register_user(valid_input()).run(&services.app_env()).await;
    println!("registration: {result:?}");
    println!("shell order: {:?}", services.events());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pure_decision_accepts_valid_input() {
        let result = decide_registration(valid_input(), RegistrationFacts::default());
        assert!(result.is_success());
    }

    #[test]
    fn pure_decision_accumulates_input_and_duplicate_errors() {
        let result = decide_registration(
            RegistrationInput {
                username: "!".to_string(),
                email: "bad".to_string(),
                password: "short".to_string(),
                confirm_password: "different".to_string(),
            },
            RegistrationFacts {
                username_taken: true,
                email_taken: true,
            },
        );

        let Validation::Failure(errors) = result else {
            panic!("invalid registration should be rejected");
        };
        let errors = errors.into_vec();
        assert!(errors.contains(&RegistrationError::UsernameLength));
        assert!(errors.contains(&RegistrationError::UsernameCharacters));
        assert!(errors.contains(&RegistrationError::EmailFormat));
        assert!(errors.contains(&RegistrationError::PasswordLength));
        assert!(errors.contains(&RegistrationError::PasswordUppercase));
        assert!(errors.contains(&RegistrationError::PasswordNumber));
        assert!(errors.contains(&RegistrationError::PasswordMismatch));
        assert!(errors.contains(&RegistrationError::UsernameTaken));
        assert!(errors.contains(&RegistrationError::EmailTaken));
    }

    #[tokio::test]
    async fn interpreter_saves_before_sending_email() {
        let services = Arc::new(InMemoryServices::default());
        let result = register_user(valid_input()).run(&services.app_env()).await;

        assert!(result.is_ok());
        assert_eq!(
            services.events(),
            vec!["load_facts", "hash", "save", "email"]
        );
    }

    #[tokio::test]
    async fn save_failure_prevents_email() {
        let services = Arc::new(InMemoryServices::default());
        *services.save_error.lock().unwrap() = true;

        let result = register_user(valid_input()).run(&services.app_env()).await;

        assert_eq!(
            result,
            Err(AppError::Infrastructure(InfrastructureError::SaveUser))
        );
        assert_eq!(services.events(), vec!["load_facts", "hash", "save"]);
    }

    #[tokio::test]
    async fn end_to_end_success_persists_the_user() {
        let services = Arc::new(InMemoryServices::default());
        let user = register_user(valid_input())
            .run(&services.app_env())
            .await
            .unwrap();

        assert!(user.id > 0);
        assert_eq!(user.password_hash, "hashed:CalmWater7");
        assert_eq!(services.users.lock().unwrap().get(&user.id), Some(&user));
    }

    #[tokio::test]
    async fn end_to_end_rejection_does_not_start_interpretation() {
        let services = Arc::new(InMemoryServices::default());
        let mut input = valid_input();
        input.confirm_password = "not-the-password".to_string();

        let result = register_user(input).run(&services.app_env()).await;

        assert!(matches!(result, Err(AppError::Rejected(_))));
        assert_eq!(services.events(), vec!["load_facts"]);
    }

    #[tokio::test]
    async fn end_to_end_infrastructure_error_is_preserved() {
        let services = Arc::new(InMemoryServices::default());
        *services.load_error.lock().unwrap() = true;

        let result = register_user(valid_input()).run(&services.app_env()).await;

        assert_eq!(
            result,
            Err(AppError::Infrastructure(InfrastructureError::LoadFacts))
        );
        assert_eq!(services.events(), vec!["load_facts"]);
    }

    #[tokio::test]
    async fn hash_failure_prevents_save_and_email() {
        let services = Arc::new(InMemoryServices::default());
        *services.hash_error.lock().unwrap() = true;

        let result = register_user(valid_input()).run(&services.app_env()).await;

        assert_eq!(
            result,
            Err(AppError::Infrastructure(InfrastructureError::HashPassword))
        );
        assert_eq!(services.events(), vec!["load_facts", "hash"]);
        assert!(services.users.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn email_failure_preserves_the_committed_user() {
        let services = Arc::new(InMemoryServices::default());
        *services.email_error.lock().unwrap() = true;

        let result = register_user(valid_input()).run(&services.app_env()).await;

        assert_eq!(
            result,
            Err(AppError::Infrastructure(InfrastructureError::SendEmail))
        );
        assert_eq!(
            services.events(),
            vec!["load_facts", "hash", "save", "email"]
        );
        let users = services.users.lock().unwrap();
        assert_eq!(users.len(), 1);
        assert_eq!(
            users.values().next().unwrap().username,
            valid_input().username
        );
    }

    #[tokio::test]
    async fn commit_rejects_stale_username_and_email_availability() {
        for conflict in [
            RegistrationError::UsernameTaken,
            RegistrationError::EmailTaken,
        ] {
            let services = Arc::new(InMemoryServices::default());
            let env = services.app_env();
            let mut second_input = valid_input();
            if conflict == RegistrationError::UsernameTaken {
                second_input.email = "another@example.com".into();
            } else {
                second_input.username = "another_user".into();
            }
            // Both decisions observe availability before either plan commits.
            let (input, facts) = load_registration_facts(valid_input())
                .run(&env)
                .await
                .unwrap();
            let first = decide_registration(input, facts).into_result().unwrap();
            let (input, facts) = load_registration_facts(second_input)
                .run(&env)
                .await
                .unwrap();
            let second = decide_registration(input, facts).into_result().unwrap();

            let first_user = interpret_registration(first).run(&env).await.unwrap();
            let result = interpret_registration(second).run(&env).await;

            assert_eq!(result, Err(AppError::Conflict(conflict)));
            let users = services.users.lock().unwrap();
            assert_eq!(users.len(), 1);
            assert_eq!(users.get(&first_user.id), Some(&first_user));
            assert_eq!(
                services
                    .events()
                    .iter()
                    .filter(|event| **event == "email")
                    .count(),
                1
            );
        }
    }
}
