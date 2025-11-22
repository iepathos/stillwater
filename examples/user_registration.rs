//! User Registration Example
//!
//! End-to-end example demonstrating how Validation and Effect work together.
//! Shows a realistic user registration flow:
//! 1. Validate input data (pure, error accumulation)
//! 2. Check uniqueness (effect with database)
//! 3. Hash password (effect)
//! 4. Save user (effect with database)
//! 5. Send email (effect with email service)

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use stillwater::{ContextError, Effect, EffectContext, Validation};

// ==================== Domain Types ====================

#[derive(Debug, Clone)]
struct User {
    id: u64,
    username: String,
    email: String,
    password_hash: String,
}

#[derive(Debug, Clone)]
struct RegistrationInput {
    username: String,
    email: String,
    password: String,
    confirm_password: String,
}

// ==================== Pure Validation ====================

/// Validate username format
fn validate_username(username: &str) -> Validation<(), Vec<String>> {
    let mut errors = Vec::new();

    if username.is_empty() {
        errors.push("Username is required".to_string());
    } else if username.len() < 3 || username.len() > 20 {
        errors.push("Username must be between 3 and 20 characters".to_string());
    } else if !username
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        errors.push(
            "Username can only contain letters, numbers, underscores, and hyphens".to_string(),
        );
    }

    if errors.is_empty() {
        Validation::success(())
    } else {
        Validation::failure(errors)
    }
}

/// Validate email format
fn validate_email(email: &str) -> Validation<(), Vec<String>> {
    let mut errors = Vec::new();

    if email.is_empty() {
        errors.push("Email is required".to_string());
    } else if !email.contains('@') {
        errors.push("Email must contain @".to_string());
    } else if !email.contains('.') {
        errors.push("Email must contain a domain".to_string());
    } else if email.len() > 254 {
        errors.push("Email is too long".to_string());
    }

    if errors.is_empty() {
        Validation::success(())
    } else {
        Validation::failure(errors)
    }
}

/// Validate password strength
fn validate_password(password: &str) -> Validation<(), Vec<String>> {
    let mut errors = Vec::new();

    if password.len() < 8 {
        errors.push("Password must be at least 8 characters".to_string());
    }
    if !password.chars().any(|c| c.is_uppercase()) {
        errors.push("Password must contain an uppercase letter".to_string());
    }
    if !password.chars().any(|c| c.is_lowercase()) {
        errors.push("Password must contain a lowercase letter".to_string());
    }
    if !password.chars().any(|c| c.is_numeric()) {
        errors.push("Password must contain a number".to_string());
    }

    if errors.is_empty() {
        Validation::success(())
    } else {
        Validation::failure(errors)
    }
}

/// Validate passwords match
fn validate_passwords_match(password: &str, confirm: &str) -> Validation<(), Vec<String>> {
    if password == confirm {
        Validation::success(())
    } else {
        Validation::failure(vec!["Passwords do not match".to_string()])
    }
}

/// Validate all input fields (pure validation with error accumulation)
fn validate_registration_input(input: &RegistrationInput) -> Validation<(), Vec<String>> {
    let v1 = validate_username(&input.username);
    let v2 = validate_email(&input.email);
    let v3 = validate_password(&input.password);
    let v4 = validate_passwords_match(&input.password, &input.confirm_password);
    Validation::<((), (), (), ()), Vec<String>>::all((v1, v2, v3, v4)).map(|_| ())
}

// ==================== Services (Environment) ====================

/// Mock database service
struct Database {
    users: Arc<Mutex<HashMap<u64, User>>>,
    next_id: Arc<Mutex<u64>>,
}

impl Database {
    fn new() -> Self {
        Self {
            users: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(1)),
        }
    }

    fn username_exists(&self, username: &str) -> bool {
        self.users
            .lock()
            .unwrap()
            .values()
            .any(|u| u.username == username)
    }

    fn email_exists(&self, email: &str) -> bool {
        self.users
            .lock()
            .unwrap()
            .values()
            .any(|u| u.email == email)
    }

    fn save_user(&self, user: User) {
        self.users.lock().unwrap().insert(user.id, user);
    }

    fn get_next_id(&self) -> u64 {
        let mut id = self.next_id.lock().unwrap();
        let current = *id;
        *id += 1;
        current
    }

    fn count(&self) -> usize {
        self.users.lock().unwrap().len()
    }
}

/// Mock password hasher
struct PasswordHasher;

impl PasswordHasher {
    fn hash(&self, password: &str) -> String {
        // In real code, use bcrypt or argon2
        format!("hashed_{}", password)
    }
}

/// Mock email service
struct EmailService {
    sent_emails: Arc<Mutex<Vec<String>>>,
}

impl EmailService {
    fn new() -> Self {
        Self {
            sent_emails: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn send_welcome_email(&self, email: &str) {
        let message = format!("Welcome email sent to {}", email);
        self.sent_emails.lock().unwrap().push(message.clone());
        println!("  [EMAIL] {}", message);
    }

    fn sent_count(&self) -> usize {
        self.sent_emails.lock().unwrap().len()
    }
}

/// Application environment
struct Env {
    db: Database,
    hasher: PasswordHasher,
    email: EmailService,
}

impl Env {
    fn new() -> Self {
        Self {
            db: Database::new(),
            hasher: PasswordHasher,
            email: EmailService::new(),
        }
    }
}

impl AsRef<Database> for Env {
    fn as_ref(&self) -> &Database {
        &self.db
    }
}

impl AsRef<PasswordHasher> for Env {
    fn as_ref(&self) -> &PasswordHasher {
        &self.hasher
    }
}

impl AsRef<EmailService> for Env {
    fn as_ref(&self) -> &EmailService {
        &self.email
    }
}

// ==================== Effects ====================

/// Check if username is already taken (effectful - requires database)
fn check_username_available(username: String) -> Effect<(), ContextError<String>, Env> {
    let username_for_context = username.clone();
    Effect::from_fn(move |env: &Env| {
        if env.db.username_exists(&username) {
            Err(format!("Username '{}' is already taken", username))
        } else {
            Ok(())
        }
    })
    .context(format!(
        "checking if username '{}' is available",
        username_for_context
    ))
}

/// Check if email is already registered (effectful - requires database)
fn check_email_available(email: String) -> Effect<(), ContextError<String>, Env> {
    let email_for_context = email.clone();
    Effect::from_fn(move |env: &Env| {
        if env.db.email_exists(&email) {
            Err(format!("Email '{}' is already registered", email))
        } else {
            Ok(())
        }
    })
    .context(format!(
        "checking if email '{}' is available",
        email_for_context
    ))
}

/// Hash password (effectful - uses hasher service)
fn hash_password(password: String) -> Effect<String, ContextError<String>, Env> {
    Effect::from_fn(move |env: &Env| Ok::<_, String>(env.hasher.hash(&password)))
        .context("hashing password".to_string())
}

/// Save user to database
fn save_user(user: User) -> Effect<User, ContextError<String>, Env> {
    let username_for_context = user.username.clone();
    Effect::from_fn(move |env: &Env| {
        env.db.save_user(user.clone());
        Ok(user.clone())
    })
    .context(format!("saving user '{}'", username_for_context))
}

/// Send welcome email
fn send_welcome_email(email: String) -> Effect<(), ContextError<String>, Env> {
    let email_for_context = email.clone();
    Effect::from_fn(move |env: &Env| {
        env.email.send_welcome_email(&email);
        Ok(())
    })
    .context(format!("sending welcome email to '{}'", email_for_context))
}

// ==================== Registration Workflow ====================

/// Complete registration workflow combining validation and effects
fn register_user(input: RegistrationInput) -> Effect<User, ContextError<String>, Env> {
    // Step 1: Pure validation (convert Validation to Effect)
    Effect::from_validation(validate_registration_input(&input).map_err(|errors| errors.join("; ")))
        .context("validating registration input".to_string())
        // Step 2: Check username availability
        .and_then(move |_| {
            let input_clone = input.clone();
            check_username_available(input.username.clone()).map(move |_| input_clone)
        })
        // Step 3: Check email availability
        .and_then(move |input| {
            let input_clone = input.clone();
            check_email_available(input.email.clone()).map(move |_| input_clone)
        })
        // Step 4: Hash password
        .and_then(move |input| {
            let input_clone = input.clone();
            hash_password(input.password.clone()).map(move |hash| (input_clone, hash))
        })
        // Step 5: Create user object
        .and_then(|(input, password_hash)| {
            Effect::from_fn(move |env: &Env| {
                let user = User {
                    id: env.db.get_next_id(),
                    username: input.username.clone(),
                    email: input.email.clone(),
                    password_hash,
                };
                Ok((user, input.email.clone()))
            })
        })
        // Step 6: Save user
        .and_then(|(user, email)| save_user(user.clone()).map(move |user| (user, email)))
        // Step 7: Send welcome email
        .and_then(|(user, email)| send_welcome_email(email).map(move |_| user.clone()))
}

// ==================== Examples ====================

async fn example_successful_registration() {
    println!("\n=== Example 1: Successful Registration ===");

    let env = Env::new();

    let input = RegistrationInput {
        username: "alice_smith".to_string(),
        email: "alice@example.com".to_string(),
        password: "SecurePass123".to_string(),
        confirm_password: "SecurePass123".to_string(),
    };

    match register_user(input).run(&env).await {
        Ok(user) => {
            println!("✓ User registered successfully!");
            println!("  ID: {}", user.id);
            println!("  Username: {}", user.username);
            println!("  Email: {}", user.email);
            println!(
                "  Password hashed: {}",
                user.password_hash.starts_with("hashed_")
            );
            println!("  Total users: {}", env.db.count());
            println!("  Emails sent: {}", env.email.sent_count());
        }
        Err(e) => println!("✗ Registration failed:\n{}", e),
    }
}

async fn example_validation_errors() {
    println!("\n=== Example 2: Validation Errors ===");

    let env = Env::new();

    let input = RegistrationInput {
        username: "ab".to_string(),                // Too short
        email: "invalid-email".to_string(),        // No @ or .
        password: "weak".to_string(),              // Missing uppercase, number
        confirm_password: "different".to_string(), // Doesn't match
    };

    match register_user(input).run(&env).await {
        Ok(user) => println!("✓ User registered: {}", user.username),
        Err(e) => {
            println!("✗ Registration failed (validation errors):");
            println!("{}", e);
        }
    }
}

async fn example_duplicate_username() {
    println!("\n=== Example 3: Duplicate Username ===");

    let env = Env::new();

    // Register first user
    let input1 = RegistrationInput {
        username: "bob".to_string(),
        email: "bob1@example.com".to_string(),
        password: "Password123".to_string(),
        confirm_password: "Password123".to_string(),
    };

    register_user(input1).run(&env).await.ok();
    println!("  First user registered");

    // Try to register with same username
    let input2 = RegistrationInput {
        username: "bob".to_string(), // Duplicate!
        email: "bob2@example.com".to_string(),
        password: "Password123".to_string(),
        confirm_password: "Password123".to_string(),
    };

    match register_user(input2).run(&env).await {
        Ok(_) => println!("✓ User registered"),
        Err(e) => {
            println!("✗ Registration failed:");
            println!("{}", e);
        }
    }
}

async fn example_duplicate_email() {
    println!("\n=== Example 4: Duplicate Email ===");

    let env = Env::new();

    // Register first user
    let input1 = RegistrationInput {
        username: "charlie".to_string(),
        email: "charlie@example.com".to_string(),
        password: "Password123".to_string(),
        confirm_password: "Password123".to_string(),
    };

    register_user(input1).run(&env).await.ok();
    println!("  First user registered");

    // Try to register with same email
    let input2 = RegistrationInput {
        username: "charlie2".to_string(),
        email: "charlie@example.com".to_string(), // Duplicate!
        password: "Password123".to_string(),
        confirm_password: "Password123".to_string(),
    };

    match register_user(input2).run(&env).await {
        Ok(_) => println!("✓ User registered"),
        Err(e) => {
            println!("✗ Registration failed:");
            println!("{}", e);
        }
    }
}

async fn example_multiple_registrations() {
    println!("\n=== Example 5: Multiple Successful Registrations ===");

    let env = Env::new();

    let users = vec![
        ("alice", "alice@example.com"),
        ("bob", "bob@example.com"),
        ("charlie", "charlie@example.com"),
    ];

    for (username, email) in users {
        let input = RegistrationInput {
            username: username.to_string(),
            email: email.to_string(),
            password: "SecurePass123".to_string(),
            confirm_password: "SecurePass123".to_string(),
        };

        match register_user(input).run(&env).await {
            Ok(user) => println!("  ✓ Registered: {} ({})", user.username, user.email),
            Err(e) => println!("  ✗ Failed: {}", e),
        }
    }

    println!("\nTotal users: {}", env.db.count());
    println!("Total emails sent: {}", env.email.sent_count());
}

// ==================== Main ====================

#[tokio::main]
async fn main() {
    println!("User Registration Examples");
    println!("==========================");
    println!();
    println!("This example demonstrates how Validation and Effect work together:");
    println!("1. Pure validation with error accumulation (Validation)");
    println!("2. Effectful operations with context (Effect)");
    println!("3. Composing a complete registration workflow");

    example_successful_registration().await;
    example_validation_errors().await;
    example_duplicate_username().await;
    example_duplicate_email().await;
    example_multiple_registrations().await;

    println!("\n=== All examples completed successfully! ===");
}
