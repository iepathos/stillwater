use stillwater::Effect;

#[derive(Clone, Debug, PartialEq)]
struct User {
    id: u64,
    email: String,
    age: u8,
    name: String,
}

#[derive(Debug, PartialEq)]
#[allow(dead_code)]
enum AppError {
    AgeTooYoung,
    EmailExists,
    DbError,
    LoggingError,
}

struct Database {
    users: Vec<User>,
}

impl Database {
    fn email_exists(&self, email: &str) -> bool {
        self.users.iter().any(|u| u.email == email)
    }

    #[allow(dead_code)]
    fn save(&mut self, user: &User) -> Result<(), String> {
        self.users.push(user.clone());
        Ok(())
    }
}

struct Logger {
    logs: Vec<String>,
}

impl Logger {
    #[allow(dead_code)]
    fn info(&mut self, msg: String) {
        self.logs.push(format!("INFO: {}", msg));
    }

    #[allow(dead_code)]
    fn warn(&mut self, msg: String) {
        self.logs.push(format!("WARN: {}", msg));
    }
}

struct EmailService {
    #[allow(dead_code)]
    sent_emails: Vec<String>,
}

impl EmailService {
    #[allow(dead_code)]
    fn send_welcome(&mut self, email: &str) -> Result<(), String> {
        self.sent_emails.push(email.to_string());
        Ok(())
    }
}

struct Env {
    db: Database,
    logger: Logger,
    #[allow(dead_code)]
    email_service: EmailService,
}

#[tokio::test]
async fn test_user_registration_workflow() {
    let user = User {
        id: 1,
        email: "test@example.com".to_string(),
        age: 25,
        name: "Test User".to_string(),
    };

    let env = Env {
        db: Database { users: vec![] },
        logger: Logger { logs: vec![] },
        email_service: EmailService {
            sent_emails: vec![],
        },
    };

    let effect = Effect::pure(user.clone())
        // Validate age using check()
        .check(|u| u.age >= 18, || AppError::AgeTooYoung)
        // Log the validation using tap()
        .tap(|u| {
            let user_id = u.id;
            Effect::from_fn(move |env: &Env| {
                let _ = env
                    .logger
                    .logs
                    .contains(&format!("INFO: Validated user: {}", user_id));
                Ok::<_, AppError>(())
            })
        })
        // Check if email exists using and_then_ref()
        .and_then_ref(|u| {
            let email = u.email.clone();
            Effect::from_fn(move |env: &Env| {
                if env.db.email_exists(&email) {
                    Err(AppError::EmailExists)
                } else {
                    Ok(())
                }
            })
        })
        // Combine with generated user ID using with()
        .with(|u| Effect::pure(format!("user-{}", u.id)))
        .map(|(user, user_id)| {
            let mut updated = user.clone();
            updated.id = user_id.len() as u64;
            updated
        });

    let result = effect.run(&env).await;
    assert!(result.is_ok());
    let registered_user = result.unwrap();
    assert_eq!(registered_user.email, "test@example.com");
}

#[tokio::test]
async fn test_user_registration_age_validation_fails() {
    let user = User {
        id: 1,
        email: "young@example.com".to_string(),
        age: 15,
        name: "Young User".to_string(),
    };

    let env = Env {
        db: Database { users: vec![] },
        logger: Logger { logs: vec![] },
        email_service: EmailService {
            sent_emails: vec![],
        },
    };

    let effect = Effect::pure(user)
        .check(|u| u.age >= 18, || AppError::AgeTooYoung)
        .and_then_ref(|u| {
            let email = u.email.clone();
            Effect::from_fn(move |env: &Env| {
                if env.db.email_exists(&email) {
                    Err(AppError::EmailExists)
                } else {
                    Ok(())
                }
            })
        });

    let result = effect.run(&env).await;
    assert_eq!(result, Err(AppError::AgeTooYoung));
}

#[tokio::test]
async fn test_user_registration_email_exists() {
    let existing_user = User {
        id: 1,
        email: "existing@example.com".to_string(),
        age: 25,
        name: "Existing User".to_string(),
    };

    let new_user = User {
        id: 2,
        email: "existing@example.com".to_string(),
        age: 30,
        name: "New User".to_string(),
    };

    let env = Env {
        db: Database {
            users: vec![existing_user],
        },
        logger: Logger { logs: vec![] },
        email_service: EmailService {
            sent_emails: vec![],
        },
    };

    let effect = Effect::pure(new_user)
        .check(|u| u.age >= 18, || AppError::AgeTooYoung)
        .and_then_ref(|u| {
            let email = u.email.clone();
            Effect::from_fn(move |env: &Env| {
                if env.db.email_exists(&email) {
                    Err(AppError::EmailExists)
                } else {
                    Ok(())
                }
            })
        });

    let result = effect.run(&env).await;
    assert_eq!(result, Err(AppError::EmailExists));
}

#[tokio::test]
async fn test_composition_with_multiple_helpers() {
    let user = User {
        id: 1,
        email: "test@example.com".to_string(),
        age: 25,
        name: "Test User".to_string(),
    };

    let env = Env {
        db: Database { users: vec![] },
        logger: Logger { logs: vec![] },
        email_service: EmailService {
            sent_emails: vec![],
        },
    };

    let effect = Effect::pure(user)
        // Validation
        .check(|u| u.age >= 18, || AppError::AgeTooYoung)
        .check(|u| !u.email.is_empty(), || AppError::DbError)
        // Side effect for logging
        .tap(|_u| Effect::pure(()))
        // Combine with additional data
        .with(|u| Effect::pure(format!("Welcome, {}!", u.name)))
        // Transform the tuple
        .map(|(user, welcome_msg)| (user, welcome_msg, 42))
        // Use and_then_ref to preserve the value while doing side effects
        .and_then_ref(|(user, _msg, _count)| {
            let user_id = user.id;
            Effect::from_fn(move |_env: &Env| Ok::<_, AppError>(user_id))
        });

    let result = effect.run(&env).await;
    assert!(result.is_ok());
}

#[derive(Debug, PartialEq)]
#[allow(dead_code)]
enum ValidationError {
    InvalidEmail,
    InvalidAge,
}

#[derive(Debug, PartialEq)]
#[allow(dead_code)]
enum DbError {
    ConnectionFailed,
    QueryFailed,
}

#[derive(Debug, PartialEq)]
enum ServiceError {
    Validation(ValidationError),
    Database(DbError),
}

impl From<ValidationError> for ServiceError {
    fn from(e: ValidationError) -> Self {
        ServiceError::Validation(e)
    }
}

impl From<DbError> for ServiceError {
    fn from(e: DbError) -> Self {
        ServiceError::Database(e)
    }
}

#[tokio::test]
async fn test_and_then_auto_with_multiple_error_types() {
    let effect = Effect::<_, ServiceError, ()>::pure(42)
        .and_then_auto(|_| Effect::<i32, ValidationError, ()>::pure(100))
        .and_then_auto(|_| Effect::<i32, DbError, ()>::pure(200))
        .and_then_auto(|_| Effect::<i32, ServiceError, ()>::pure(300));

    let result = effect.run(&()).await;
    assert_eq!(result, Ok(300));
}

#[tokio::test]
async fn test_and_then_auto_error_conversion() {
    let effect = Effect::<_, ServiceError, ()>::pure(42)
        .and_then_auto(|_| Effect::<i32, ValidationError, ()>::fail(ValidationError::InvalidEmail));

    let result = effect.run(&()).await;
    assert_eq!(
        result,
        Err(ServiceError::Validation(ValidationError::InvalidEmail))
    );
}

#[tokio::test]
async fn test_complex_workflow_with_all_combinators() {
    let user = User {
        id: 1,
        email: "complex@example.com".to_string(),
        age: 30,
        name: "Complex User".to_string(),
    };

    let env = Env {
        db: Database { users: vec![] },
        logger: Logger { logs: vec![] },
        email_service: EmailService {
            sent_emails: vec![],
        },
    };

    let effect = Effect::pure(user)
        // Multiple validations with check()
        .check(|u| u.age >= 18, || AppError::AgeTooYoung)
        .check(|u| u.age <= 120, || AppError::DbError)
        // Side effect with tap()
        .tap(|_u| Effect::pure(()))
        // Reference-based side effect with and_then_ref()
        .and_then_ref(|u| {
            let email = u.email.clone();
            Effect::from_fn(move |env: &Env| {
                if env.db.email_exists(&email) {
                    Err(AppError::EmailExists)
                } else {
                    Ok(())
                }
            })
        })
        // Combine with additional data using with()
        .with(|u| Effect::pure(format!("user-{}", u.id)))
        // Transform the result
        .map(|(user, user_key)| (user, user_key, true))
        // More reference-based operations
        .and_then_ref(|(user, _key, _active)| {
            let user_id = user.id;
            Effect::pure(user_id)
        })
        // Final transformation
        .map(|(_user, key, active)| format!("{}-{}", key, active));

    let result = effect.run(&env).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "user-1-true");
}
