//! User registration example - testing Effect composition ergonomics
//!
//! This tests the "pure core, imperative shell" pattern.
//! Pure functions are separated from I/O effects.

use std::fmt;
use stillwater::{Effect, Validation, IO};

// ============================================================================
// Domain Types
// ============================================================================

#[derive(Debug, Clone)]
struct UserId(u64);

#[derive(Debug, Clone)]
struct Email(String);

#[derive(Debug, Clone)]
struct PasswordHash(String);

#[derive(Debug, Clone)]
struct User {
    id: UserId,
    email: Email,
    password_hash: PasswordHash,
}

#[derive(Clone)]
struct NewUser {
    email: Email,
    password: String,
}

// ============================================================================
// Application Environment (what our effects need)
// ============================================================================

struct AppEnv {
    db: Database,
    email_service: EmailService,
}

// Mock implementations for testing
#[derive(Clone)]
struct Database {
    users: Vec<User>,
}

impl Database {
    fn find_by_email(&self, email: &Email) -> Result<Option<User>, DbError> {
        Ok(self.users.iter().find(|u| u.email.0 == email.0).cloned())
    }

    fn insert_user(&mut self, user: User) -> Result<UserId, DbError> {
        self.users.push(user.clone());
        Ok(user.id)
    }
}

#[derive(Clone)]
struct EmailService;

impl EmailService {
    fn send_welcome(&self, email: &Email) -> Result<(), EmailError> {
        println!("  [Email] Sending welcome email to {}", email.0);
        Ok(())
    }
}

// ============================================================================
// Errors
// ============================================================================

#[derive(Debug)]
enum AppError {
    EmailAlreadyExists(Email),
    Validation(Vec<ValidationError>),
    Database(DbError),
    Email(EmailError),
}

#[derive(Debug, Clone)]
enum ValidationError {
    InvalidEmail(String),
    PasswordTooWeak,
}

#[derive(Debug, Clone)]
struct DbError(String);

#[derive(Debug, Clone)]
struct EmailError(String);

impl From<Vec<ValidationError>> for AppError {
    fn from(errors: Vec<ValidationError>) -> Self {
        AppError::Validation(errors)
    }
}

impl From<DbError> for AppError {
    fn from(err: DbError) -> Self {
        AppError::Database(err)
    }
}

impl From<EmailError> for AppError {
    fn from(err: EmailError) -> Self {
        AppError::Email(err)
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AppError::EmailAlreadyExists(email) => {
                write!(f, "Email {} already registered", email.0)
            }
            AppError::Validation(errors) => {
                write!(f, "Validation failed: {} errors", errors.len())
            }
            AppError::Database(e) => write!(f, "Database error: {:?}", e),
            AppError::Email(e) => write!(f, "Email error: {:?}", e),
        }
    }
}

// ============================================================================
// Pure Functions (the "still" core)
// ============================================================================

fn validate_email(email: &str) -> Validation<Email, Vec<ValidationError>> {
    if email.contains('@') && email.contains('.') {
        Validation::success(Email(email.to_string()))
    } else {
        Validation::failure(vec![ValidationError::InvalidEmail(email.to_string())])
    }
}

fn validate_password(pwd: &str) -> Validation<String, Vec<ValidationError>> {
    if pwd.len() >= 8 {
        Validation::success(pwd.to_string())
    } else {
        Validation::failure(vec![ValidationError::PasswordTooWeak])
    }
}

fn validate_new_user(email: &str, password: &str) -> Validation<NewUser, Vec<ValidationError>> {
    Validation::all((validate_email(email), validate_password(password)))
        .map(|(email, password)| NewUser { email, password })
}

fn hash_password(password: &str) -> PasswordHash {
    // In reality, use bcrypt or argon2
    PasswordHash(format!("hashed_{}", password))
}

fn create_user_from_new(new_user: NewUser, id: UserId) -> User {
    User {
        id,
        email: new_user.email,
        password_hash: hash_password(&new_user.password),
    }
}

fn next_user_id() -> UserId {
    // In reality, DB would generate this
    UserId(42)
}

// ============================================================================
// Effects (the "water" shell)
// ============================================================================

// Question: How does this composition feel?
// Is the flow clear?

fn register_user(email: String, password: String) -> Effect<User, AppError, AppEnv> {
    // Step 1: Validate input (pure, can fail with multiple errors)
    Effect::from_validation(validate_new_user(&email, &password))
        // Step 2: Check if email exists (I/O)
        .and_then(|new_user| {
            IO::query(|db: &Database| db.find_by_email(&new_user.email))
                .map_err(AppError::from)
                .and_then(move |existing| {
                    if existing.is_some() {
                        Effect::fail(AppError::EmailAlreadyExists(new_user.email.clone()))
                    } else {
                        Effect::pure(new_user)
                    }
                })
        })
        // Step 3: Create user entity (pure)
        .map(|new_user| {
            let id = next_user_id();
            create_user_from_new(new_user, id)
        })
        // Step 4: Save to database (I/O)
        .and_then(|user| {
            let user_clone = user.clone();
            IO::execute(move |db: &mut Database| db.insert_user(user))
                .map_err(AppError::from)
                .map(move |_| user_clone)
        })
        // Step 5: Send welcome email (I/O, non-critical)
        // Question: How should we handle non-critical failures?
        .and_then(|user| {
            let user_clone = user.clone();
            IO::execute(move |email_service: &EmailService| email_service.send_welcome(&user.email))
                .map_err(AppError::from)
                // Option A: Ignore email errors
                .or_else(|_err| {
                    println!("  [Warning] Failed to send welcome email, continuing...");
                    Effect::pure(())
                })
                .map(move |_| user_clone)
        })
}

// Alternative: Should we have a helper for non-critical effects?
fn register_user_alt(email: String, password: String) -> Effect<User, AppError, AppEnv> {
    Effect::from_validation(validate_new_user(&email, &password))
        .and_then(|new_user| check_email_not_exists(new_user))
        .map(|new_user| create_user_from_new(new_user, next_user_id()))
        .and_then(save_user)
        .and_then(send_welcome_email_optional) // Hypothetical helper
}

// Helper effects - does this feel cleaner?
fn check_email_not_exists(new_user: NewUser) -> Effect<NewUser, AppError, AppEnv> {
    IO::query(|db: &Database| db.find_by_email(&new_user.email))
        .map_err(AppError::from)
        .and_then(move |existing| {
            if existing.is_some() {
                Effect::fail(AppError::EmailAlreadyExists(new_user.email.clone()))
            } else {
                Effect::pure(new_user)
            }
        })
}

fn save_user(user: User) -> Effect<User, AppError, AppEnv> {
    let user_clone = user.clone();
    IO::execute(move |db: &mut Database| db.insert_user(user))
        .map_err(AppError::from)
        .map(move |_| user_clone)
}

fn send_welcome_email_optional(user: User) -> Effect<User, AppError, AppEnv> {
    let user_clone = user.clone();
    IO::execute(move |email_service: &EmailService| email_service.send_welcome(&user.email))
        .map_err(AppError::from)
        .or_else(|_err| {
            println!("  [Warning] Failed to send welcome email");
            Effect::pure(())
        })
        .map(move |_| user_clone)
}

// ============================================================================
// Usage
// ============================================================================

fn main() {
    println!("=== User Registration Effect Composition Test ===\n");

    let env = AppEnv {
        db: Database { users: vec![] },
        email_service: EmailService,
    };

    // Test 1: Successful registration
    println!("Test 1: Valid registration");
    let effect = register_user(
        "newuser@example.com".to_string(),
        "securepass123".to_string(),
    );

    match effect.run(&env) {
        Ok(user) => {
            println!("✓ User registered successfully!");
            println!("  ID: {:?}", user.id);
            println!("  Email: {:?}", user.email);
        }
        Err(err) => {
            println!("✗ Registration failed: {}", err);
        }
    }

    println!("\n---\n");

    // Test 2: Validation errors
    println!("Test 2: Invalid input");
    let effect = register_user("not-an-email".to_string(), "weak".to_string());

    match effect.run(&env) {
        Ok(_) => println!("✓ Unexpected success"),
        Err(err) => {
            println!("✗ Registration failed: {}", err);
            if let AppError::Validation(errors) = err {
                for e in errors {
                    println!("  - {:?}", e);
                }
            }
        }
    }

    println!("\n---\n");

    // Test 3: Email already exists
    println!("Test 3: Duplicate email");

    // First registration
    let effect1 = register_user(
        "duplicate@example.com".to_string(),
        "password123".to_string(),
    );

    let mut env_with_user = env.clone();
    let _ = effect1.run(&env_with_user);

    // Try to register again with same email
    let effect2 = register_user(
        "duplicate@example.com".to_string(),
        "different-password".to_string(),
    );

    match effect2.run(&env_with_user) {
        Ok(_) => println!("✓ Unexpected success"),
        Err(err) => {
            println!("✗ Registration blocked: {}", err);
        }
    }

    // Ergonomics questions to evaluate:
    // 1. Is the Effect composition readable?
    // 2. Do the I/O helpers (IO::query, IO::execute) feel natural?
    // 3. Is error conversion (map_err) too verbose?
    // 4. Should we have better helpers for "optional" effects?
    // 5. Is the separation of pure/effectful clear?
    // 6. Would context() help here? e.g. .context("Checking email exists")
}

/* Expected output:

=== User Registration Effect Composition Test ===

Test 1: Valid registration
  [Email] Sending welcome email to newuser@example.com
✓ User registered successfully!
  ID: UserId(42)
  Email: Email("newuser@example.com")

---

Test 2: Invalid input
✗ Registration failed: Validation failed: 2 errors
  - InvalidEmail("not-an-email")
  - PasswordTooWeak

---

Test 3: Duplicate email
  [Email] Sending welcome email to duplicate@example.com
✗ Registration blocked: Email duplicate@example.com already registered

*/
