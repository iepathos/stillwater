---
number: 009
title: Compiled Runnable Examples
category: documentation
priority: high
status: draft
dependencies: [001, 002, 003, 004, 005, 006, 008]
created: 2025-11-21
---

# Specification 009: Compiled Runnable Examples

**Category**: documentation
**Priority**: high
**Status**: draft
**Dependencies**: Specs 001-006 (all core features), Spec 008 (project structure)

## Context

We created fictional example code early in the design process to test ergonomics:
- form_validation.rs
- user_registration.rs
- error_context.rs
- data_pipeline.rs
- testing_patterns.rs

These examples helped validate the API design, but they don't actually compile because the library doesn't exist yet. This spec converts them into real, runnable examples that:
1. Compile successfully
2. Demonstrate real-world usage
3. Serve as copy-paste starting points for users
4. Test the ergonomics of the implemented APIs
5. Run in CI to ensure they stay working

## Objective

Convert all fictional examples into fully functional, well-documented, runnable code that compiles and executes successfully, demonstrating the library's capabilities.

## Requirements

### Functional Requirements

- Convert form_validation.rs to compilable code
- Convert user_registration.rs to compilable code
- Convert error_context.rs to compilable code
- Convert data_pipeline.rs to compilable code
- Convert testing_patterns.rs to compilable code
- All examples must compile with `cargo build --examples`
- All examples must run successfully with `cargo run --example <name>`
- Examples produce visible output to demonstrate behavior
- Code follows Rust best practices
- Examples are self-contained (no external dependencies beyond stillwater)

### Non-Functional Requirements

- Clear, readable code (not overly clever)
- Comprehensive comments explaining patterns
- Output shows both success and failure cases
- Examples are realistic (not toy problems)
- Fast execution (<1 second per example)
- Works on stable Rust (no nightly required)

## Acceptance Criteria

- [ ] form_validation.rs compiles and runs
- [ ] user_registration.rs compiles and runs
- [ ] error_context.rs compiles and runs
- [ ] data_pipeline.rs compiles and runs
- [ ] testing_patterns.rs compiles and runs
- [ ] All examples produce meaningful output
- [ ] Examples documented in README.md
- [ ] CI runs all examples successfully
- [ ] Examples follow consistent style
- [ ] Each example has header comment explaining purpose

## Technical Details

### Implementation Approach

Each example follows this structure:

```rust
//! Example: [Name]
//!
//! Demonstrates: [What patterns/features]
//!
//! Run with: cargo run --example [name]

use stillwater::prelude::*;

// Domain types and pure functions
// ...

// Example scenarios
fn main() {
    println!("Example: [Name]\n");

    // Scenario 1: Success case
    println!("=== Success Case ===");
    // ...

    // Scenario 2: Failure case
    println!("\n=== Failure Case ===");
    // ...

    // Scenario 3: Edge case
    println!("\n=== Edge Case ===");
    // ...
}
```

### Example 1: Form Validation

**File**: `examples/form_validation.rs`

```rust
//! Example: Form Validation with Error Accumulation
//!
//! Demonstrates:
//! - Validation type with error accumulation
//! - Validation::all() with tuples
//! - Custom error types
//! - Collecting all validation errors at once
//!
//! Run with: cargo run --example form_validation

use stillwater::prelude::*;

#[derive(Debug, Clone, PartialEq)]
struct User {
    email: String,
    password: String,
    age: u8,
}

#[derive(Debug, Clone, PartialEq)]
enum ValidationError {
    InvalidEmail(String),
    PasswordTooShort { min_length: usize, actual: usize },
    AgeTooYoung { min_age: u8, actual: u8 },
}

// Implement Semigroup for Vec<ValidationError> (automatic via Vec<T>)
// This allows error accumulation

fn validate_email(email: &str) -> Validation<String, Vec<ValidationError>> {
    if email.contains('@') && email.contains('.') {
        Validation::success(email.to_string())
    } else {
        Validation::failure(vec![ValidationError::InvalidEmail(email.to_string())])
    }
}

fn validate_password(password: &str) -> Validation<String, Vec<ValidationError>> {
    const MIN_LENGTH: usize = 8;
    if password.len() >= MIN_LENGTH {
        Validation::success(password.to_string())
    } else {
        Validation::failure(vec![ValidationError::PasswordTooShort {
            min_length: MIN_LENGTH,
            actual: password.len(),
        }])
    }
}

fn validate_age(age: u8) -> Validation<u8, Vec<ValidationError>> {
    const MIN_AGE: u8 = 18;
    if age >= MIN_AGE {
        Validation::success(age)
    } else {
        Validation::failure(vec![ValidationError::AgeTooYoung {
            min_age: MIN_AGE,
            actual: age,
        }])
    }
}

fn validate_registration(
    email: &str,
    password: &str,
    age: u8,
) -> Validation<User, Vec<ValidationError>> {
    // Validation::all runs ALL validations and accumulates ALL errors
    Validation::all((
        validate_email(email),
        validate_password(password),
        validate_age(age),
    ))
    .map(|(email, password, age)| User {
        email,
        password,
        age,
    })
}

fn main() {
    println!("Example: Form Validation with Error Accumulation\n");

    // Scenario 1: All fields valid
    println!("=== Valid Registration ===");
    let result = validate_registration("alice@example.com", "secure123", 25);
    match result {
        Validation::Success(user) => println!("✓ Success: {:#?}", user),
        Validation::Failure(errors) => println!("✗ Errors: {:#?}", errors),
    }

    // Scenario 2: All fields invalid (shows error accumulation!)
    println!("\n=== Invalid Registration (All Fields) ===");
    let result = validate_registration("invalid-email", "short", 15);
    match result {
        Validation::Success(user) => println!("✓ Success: {:#?}", user),
        Validation::Failure(errors) => {
            println!("✗ Accumulated {} errors:", errors.len());
            for (i, error) in errors.iter().enumerate() {
                println!("  {}. {:?}", i + 1, error);
            }
        }
    }

    // Scenario 3: Some fields invalid
    println!("\n=== Partially Invalid Registration ===");
    let result = validate_registration("bob@example.com", "tiny", 30);
    match result {
        Validation::Success(user) => println!("✓ Success: {:#?}", user),
        Validation::Failure(errors) => {
            println!("✗ Accumulated {} error(s):", errors.len());
            for (i, error) in errors.iter().enumerate() {
                println!("  {}. {:?}", i + 1, error);
            }
        }
    }

    println!("\n✨ Key Takeaway: All validation errors are collected and reported at once!");
    println!("   This provides better UX than failing on the first error.");
}
```

### Example 2: User Registration with Effects

**File**: `examples/user_registration.rs`

```rust
//! Example: User Registration with Effect Composition
//!
//! Demonstrates:
//! - Effect type for I/O separation
//! - IO::read and IO::write helpers
//! - Pure business logic with effectful I/O
//! - Environment pattern for dependency injection
//! - Testability through mock environments
//!
//! Run with: cargo run --example user_registration

use stillwater::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// ========== Domain Types (Pure) ==========

#[derive(Debug, Clone, PartialEq)]
struct User {
    id: u64,
    email: String,
    password_hash: String,
}

#[derive(Debug, Clone, PartialEq)]
enum AppError {
    EmailExists(String),
    DatabaseError(String),
    EmailServiceError(String),
}

// ========== Services (Effectful) ==========

#[derive(Clone)]
struct Database {
    users: Arc<Mutex<HashMap<String, User>>>,
    next_id: Arc<Mutex<u64>>,
}

impl Database {
    fn new() -> Self {
        Database {
            users: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(1)),
        }
    }

    fn email_exists(&self, email: &str) -> bool {
        self.users.lock().unwrap().contains_key(email)
    }

    fn save(&self, user: User) -> Result<(), String> {
        self.users
            .lock()
            .unwrap()
            .insert(user.email.clone(), user);
        Ok(())
    }

    fn next_id(&self) -> u64 {
        let mut id = self.next_id.lock().unwrap();
        let current = *id;
        *id += 1;
        current
    }
}

#[derive(Clone)]
struct EmailService;

impl EmailService {
    fn send_welcome(&self, email: &str) -> Result<(), String> {
        println!("  📧 Sending welcome email to: {}", email);
        Ok(())
    }
}

#[derive(Clone)]
struct Logger;

impl Logger {
    fn info(&self, msg: String) {
        println!("  ℹ️  {}", msg);
    }

    fn warn(&self, msg: String) {
        println!("  ⚠️  {}", msg);
    }
}

// ========== Environment Setup ==========

#[derive(Clone)]
struct AppEnv {
    db: Database,
    email_service: EmailService,
    logger: Logger,
}

impl AppEnv {
    fn new() -> Self {
        AppEnv {
            db: Database::new(),
            email_service: EmailService,
            logger: Logger,
        }
    }
}

// Implement AsRef for each service type
impl AsRef<Database> for AppEnv {
    fn as_ref(&self) -> &Database {
        &self.db
    }
}

impl AsRef<EmailService> for AppEnv {
    fn as_ref(&self) -> &EmailService {
        &self.email_service
    }
}

impl AsRef<Logger> for AppEnv {
    fn as_ref(&self) -> &Logger {
        &self.logger
    }
}

// ========== Pure Business Logic ==========

fn hash_password(password: &str) -> String {
    // In real code, use bcrypt or argon2
    format!("hashed_{}", password)
}

fn create_user(id: u64, email: String, password: &str) -> User {
    User {
        id,
        email,
        password_hash: hash_password(password),
    }
}

// ========== Effectful Operations ==========

fn register_user(email: String, password: String) -> Effect<User, AppError, AppEnv> {
    // Check if email exists
    IO::read(move |db: &Database| {
        if db.email_exists(&email) {
            Err(AppError::EmailExists(email.clone()))
        } else {
            Ok(email.clone())
        }
    })
    .and_then(move |email| {
        // Generate ID (pure function would be better, but demonstrating effects)
        IO::read(move |db: &Database| {
            let id = db.next_id();
            let user = create_user(id, email.clone(), &password);
            (user, email)
        })
        .and_then(|(user, email)| {
            // Save to database
            let user_clone = user.clone();
            IO::read(move |db: &Database| {
                db.save(user_clone.clone())
                    .map_err(AppError::DatabaseError)?;
                Ok((user_clone, email))
            })
            .and_then(|(user, email)| {
                // Send welcome email (non-critical, don't fail if it errors)
                let user_clone = user.clone();
                IO::read(move |email_svc: &EmailService| email_svc.send_welcome(&email))
                    .or_else(move |err| {
                        // Log warning but don't fail registration
                        IO::read(move |logger: &Logger| {
                            logger.warn(format!("Failed to send email: {}", err));
                        })
                        .map(|_| ())
                    })
                    .map(move |_| user_clone)
            })
        })
    })
}

// ========== Main ==========

#[tokio::main]
async fn main() {
    println!("Example: User Registration with Effect Composition\n");

    let env = AppEnv::new();

    // Scenario 1: Successful registration
    println!("=== Registering alice@example.com ===");
    match register_user("alice@example.com".to_string(), "password123".to_string())
        .run(&env)
        .await
    {
        Ok(user) => println!("✓ Registered: {:#?}", user),
        Err(err) => println!("✗ Error: {:?}", err),
    }

    // Scenario 2: Duplicate email
    println!("\n=== Registering alice@example.com again ===");
    match register_user("alice@example.com".to_string(), "password456".to_string())
        .run(&env)
        .await
    {
        Ok(user) => println!("✓ Registered: {:#?}", user),
        Err(err) => println!("✗ Error: {:?}", err),
    }

    // Scenario 3: Different user
    println!("\n=== Registering bob@example.com ===");
    match register_user("bob@example.com".to_string(), "secure789".to_string())
        .run(&env)
        .await
    {
        Ok(user) => println!("✓ Registered: {:#?}", user),
        Err(err) => println!("✗ Error: {:?}", err),
    }

    println!("\n✨ Key Takeaway: Effects separate I/O from business logic.");
    println!("   Pure functions (hash_password, create_user) need no mocks to test!");
}
```

### Example 3: Error Context

**File**: `examples/error_context.rs`

```rust
//! Example: Error Context Trails
//!
//! Demonstrates:
//! - ContextError for error trails
//! - .context() method for adding breadcrumbs
//! - Preserving "why" and "where" information
//! - Better debugging through error context
//!
//! Run with: cargo run --example error_context

use stillwater::prelude::*;
use std::fs;

#[derive(Debug, Clone, PartialEq)]
enum ConfigError {
    FileNotFound(String),
    ParseError(String),
    ValidationError(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ConfigError::FileNotFound(path) => write!(f, "File not found: {}", path),
            ConfigError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            ConfigError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl std::error::Error for ConfigError {}

// Simulated file reading (would use std::fs in real code)
fn read_file(path: &str) -> Effect<String, ConfigError, ()> {
    if path.contains("nonexistent") {
        Effect::fail(ConfigError::FileNotFound(path.to_string()))
    } else {
        Effect::pure("database_url=postgres://localhost\nport=5432".to_string())
    }
}

fn parse_config(content: String) -> Effect<Config, ConfigError, ()> {
    // Simulated parsing (would use serde or toml crate in real code)
    if content.is_empty() {
        Effect::fail(ConfigError::ParseError("Empty content".to_string()))
    } else {
        Effect::pure(Config {
            database_url: "postgres://localhost".to_string(),
            port: 5432,
        })
    }
}

fn validate_config(config: Config) -> Effect<Config, ConfigError, ()> {
    if config.port < 1024 {
        Effect::fail(ConfigError::ValidationError(
            "Port must be >= 1024".to_string(),
        ))
    } else {
        Effect::pure(config)
    }
}

#[derive(Debug)]
struct Config {
    database_url: String,
    port: u16,
}

fn load_config(path: &str) -> Effect<Config, ContextError<ConfigError>, ()> {
    read_file(path)
        .context(format!("Reading config file: {}", path))
        .and_then(|content| {
            parse_config(content).context("Parsing configuration")
        })
        .and_then(|config| {
            validate_config(config).context("Validating configuration")
        })
        .context("Loading application configuration")
}

#[tokio::main]
async fn main() {
    println!("Example: Error Context Trails\n");

    // Scenario 1: Success
    println!("=== Loading valid config ===");
    match load_config("config.toml").run(&()).await {
        Ok(config) => println!("✓ Config loaded: {:#?}", config),
        Err(err) => println!("✗ Error:\n{}", err),
    }

    // Scenario 2: File not found (shows full context trail)
    println!("\n=== Loading nonexistent config ===");
    match load_config("nonexistent.toml").run(&()).await {
        Ok(config) => println!("✓ Config loaded: {:#?}", config),
        Err(err) => {
            println!("✗ Error with context trail:\n{}", err);
            println!("\nContext trail:");
            for (i, ctx) in err.context_trail().iter().enumerate() {
                println!("  {}. {}", i + 1, ctx);
            }
        }
    }

    println!("\n✨ Key Takeaway: Context errors preserve the path errors take through code.");
    println!("   This makes debugging much easier than plain error messages!");
}
```

### Example 4: Data Pipeline

**File**: `examples/data_pipeline.rs`

```rust
//! Example: Data Pipeline (ETL)
//!
//! Demonstrates:
//! - Combining Validation + Effect
//! - Real-world data processing pipeline
//! - Pure transformation functions
//! - Strict vs permissive validation strategies
//!
//! Run with: cargo run --example data_pipeline

use stillwater::prelude::*;

// ========== Domain Types ==========

#[derive(Debug, Clone)]
struct RawRecord {
    id: String,
    email: String,
    age: String,
    score: String,
}

#[derive(Debug, Clone, PartialEq)]
struct ValidRecord {
    id: u64,
    email: String,
    age: u8,
    score: f64,
}

#[derive(Debug, Clone)]
struct EnrichedRecord {
    id: u64,
    email: String,
    age: u8,
    score: f64,
    category: Category,
}

#[derive(Debug, Clone, PartialEq)]
enum Category {
    Young,
    Adult,
    Senior,
}

#[derive(Debug, Clone, PartialEq)]
enum ParseError {
    InvalidId(String),
    InvalidEmail(String),
    InvalidAge(String),
    InvalidScore(String),
}

// ========== Pure Validation Functions ==========

fn validate_id(raw: &str) -> Validation<u64, Vec<ParseError>> {
    raw.parse::<u64>()
        .map(Validation::success)
        .unwrap_or_else(|_| {
            Validation::failure(vec![ParseError::InvalidId(raw.to_string())])
        })
}

fn validate_email(raw: &str) -> Validation<String, Vec<ParseError>> {
    if raw.contains('@') {
        Validation::success(raw.to_string())
    } else {
        Validation::failure(vec![ParseError::InvalidEmail(raw.to_string())])
    }
}

fn validate_age(raw: &str) -> Validation<u8, Vec<ParseError>> {
    raw.parse::<u8>()
        .map(Validation::success)
        .unwrap_or_else(|_| Validation::failure(vec![ParseError::InvalidAge(raw.to_string())]))
}

fn validate_score(raw: &str) -> Validation<f64, Vec<ParseError>> {
    raw.parse::<f64>()
        .map(Validation::success)
        .unwrap_or_else(|_| {
            Validation::failure(vec![ParseError::InvalidScore(raw.to_string())])
        })
}

fn validate_record(raw: RawRecord) -> Validation<ValidRecord, Vec<ParseError>> {
    Validation::all((
        validate_id(&raw.id),
        validate_email(&raw.email),
        validate_age(&raw.age),
        validate_score(&raw.score),
    ))
    .map(|(id, email, age, score)| ValidRecord {
        id,
        email,
        age,
        score,
    })
}

// ========== Pure Enrichment Functions ==========

fn categorize_by_age(age: u8) -> Category {
    match age {
        0..=17 => Category::Young,
        18..=64 => Category::Adult,
        _ => Category::Senior,
    }
}

fn enrich_record(record: ValidRecord) -> EnrichedRecord {
    EnrichedRecord {
        category: categorize_by_age(record.age),
        id: record.id,
        email: record.email,
        age: record.age,
        score: record.score,
    }
}

// ========== Pipeline Strategies ==========

fn strict_pipeline(records: Vec<RawRecord>) -> Validation<Vec<EnrichedRecord>, Vec<ParseError>> {
    // Validate all, fail if ANY record is invalid
    Validation::all_vec(records.into_iter().map(validate_record).collect()).map(|valid_records| {
        valid_records.into_iter().map(enrich_record).collect()
    })
}

fn permissive_pipeline(records: Vec<RawRecord>) -> (Vec<EnrichedRecord>, Vec<Vec<ParseError>>) {
    // Filter out invalid records, process valid ones
    let mut valid = Vec::new();
    let mut errors = Vec::new();

    for record in records {
        match validate_record(record) {
            Validation::Success(valid_record) => valid.push(enrich_record(valid_record)),
            Validation::Failure(errs) => errors.push(errs),
        }
    }

    (valid, errors)
}

// ========== Main ==========

fn main() {
    println!("Example: Data Pipeline (ETL)\n");

    let records = vec![
        RawRecord {
            id: "1".to_string(),
            email: "alice@example.com".to_string(),
            age: "25".to_string(),
            score: "95.5".to_string(),
        },
        RawRecord {
            id: "2".to_string(),
            email: "bob@example.com".to_string(),
            age: "30".to_string(),
            score: "87.3".to_string(),
        },
        RawRecord {
            id: "bad".to_string(), // Invalid ID
            email: "invalid-email".to_string(), // Invalid email
            age: "25".to_string(),
            score: "92.1".to_string(),
        },
        RawRecord {
            id: "4".to_string(),
            email: "carol@example.com".to_string(),
            age: "70".to_string(),
            score: "88.8".to_string(),
        },
    ];

    // Strategy 1: Strict (fail if any record is invalid)
    println!("=== Strict Pipeline ===");
    match strict_pipeline(records.clone()) {
        Validation::Success(enriched) => {
            println!("✓ Processed {} records:", enriched.len());
            for record in enriched {
                println!("  {:?}", record);
            }
        }
        Validation::Failure(errors) => {
            println!("✗ Pipeline failed with errors:");
            for err in errors {
                println!("  {:?}", err);
            }
        }
    }

    // Strategy 2: Permissive (process valid, report invalid)
    println!("\n=== Permissive Pipeline ===");
    let (valid, errors) = permissive_pipeline(records);
    println!("✓ Processed {} valid records:", valid.len());
    for record in valid {
        println!("  {:?}", record);
    }
    if !errors.is_empty() {
        println!("\n⚠️  {} invalid records:", errors.len());
        for (i, err) in errors.iter().enumerate() {
            println!("  Record {}: {:?}", i + 1, err);
        }
    }

    println!("\n✨ Key Takeaway: Choose validation strategy based on business requirements.");
    println!("   Strict = all-or-nothing, Permissive = process what you can.");
}
```

### Example 5: Testing Patterns

**File**: `examples/testing_patterns.rs`

```rust
//! Example: Testing Patterns
//!
//! Demonstrates:
//! - Pure functions need zero mocks
//! - Effect testing with simple mock environments
//! - Testability benefits of separation
//!
//! Run with: cargo run --example testing_patterns

use stillwater::prelude::*;

// ========== Pure Business Logic (Zero Mocks Needed!) ==========

#[derive(Debug, Clone, PartialEq)]
struct Product {
    price: f64,
    quantity: u32,
}

#[derive(Debug, Clone, PartialEq)]
struct Discount {
    percentage: f64,
}

// Pure function - easy to test!
fn calculate_discount(product: &Product, discount: &Discount) -> f64 {
    product.price * product.quantity as f64 * (discount.percentage / 100.0)
}

// Pure function - easy to test!
fn apply_discount(product: Product, discount: Discount) -> Product {
    let discount_amount = calculate_discount(&product, &discount);
    Product {
        price: product.price - (discount_amount / product.quantity as f64),
        ..product
    }
}

#[cfg(test)]
mod pure_tests {
    use super::*;

    #[test]
    fn test_calculate_discount() {
        let product = Product {
            price: 100.0,
            quantity: 2,
        };
        let discount = Discount { percentage: 10.0 };

        // No mocks, no setup, just test the logic!
        assert_eq!(calculate_discount(&product, &discount), 20.0);
    }

    #[test]
    fn test_apply_discount() {
        let product = Product {
            price: 100.0,
            quantity: 2,
        };
        let discount = Discount { percentage: 10.0 };

        let result = apply_discount(product, discount);

        // No mocks needed!
        assert_eq!(result.price, 90.0);
    }
}

// ========== Effectful Operations (Simple Mock Env) ==========

#[derive(Clone)]
struct PricingService {
    base_prices: std::collections::HashMap<String, f64>,
}

impl PricingService {
    fn get_price(&self, product_id: &str) -> Option<f64> {
        self.base_prices.get(product_id).copied()
    }
}

#[derive(Clone)]
struct Env {
    pricing: PricingService,
}

impl AsRef<PricingService> for Env {
    fn as_ref(&self) -> &PricingService {
        &self.pricing
    }
}

fn fetch_price(product_id: String) -> Effect<f64, String, Env> {
    IO::read(move |pricing: &PricingService| {
        pricing
            .get_price(&product_id)
            .ok_or_else(|| format!("Product not found: {}", product_id))
    })
}

#[cfg(test)]
mod effect_tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_price_found() {
        // Simple mock environment - just a HashMap!
        let mut prices = std::collections::HashMap::new();
        prices.insert("WIDGET".to_string(), 99.99);

        let env = Env {
            pricing: PricingService { base_prices: prices },
        };

        let result = fetch_price("WIDGET".to_string()).run(&env).await;

        assert_eq!(result, Ok(99.99));
    }

    #[tokio::test]
    async fn test_fetch_price_not_found() {
        let env = Env {
            pricing: PricingService {
                base_prices: std::collections::HashMap::new(),
            },
        };

        let result = fetch_price("MISSING".to_string()).run(&env).await;

        assert!(result.is_err());
    }
}

// ========== Main Demo ==========

fn main() {
    println!("Example: Testing Patterns\n");

    println!("=== Pure Function Tests ===");
    println!("Run with: cargo test --example testing_patterns");
    println!("\nPure functions like calculate_discount() need ZERO mocks!");
    println!("Just call the function and assert the result.");

    println!("\n=== Effect Tests ===");
    println!("Effects need simple mock environments.");
    println!("Environment is just data - easy to construct for tests!");

    let product = Product {
        price: 100.0,
        quantity: 2,
    };
    let discount = Discount { percentage: 15.0 };

    println!("\n=== Example Calculation ===");
    println!("Product: {:?}", product);
    println!("Discount: {:?}", discount);
    println!(
        "Discount amount: ${:.2}",
        calculate_discount(&product, &discount)
    );
    println!("After discount: {:?}", apply_discount(product, discount));

    println!("\n✨ Key Takeaway: Separation makes testing trivial!");
    println!("   Pure functions = zero mocks");
    println!("   Effects = simple data mocks");
}
```

### Architecture Changes

- All examples converted from fictional to real code
- Examples integrated into CI pipeline
- README updated with example links

## Dependencies

- **Prerequisites**: Specs 001-006 (implementation), Spec 008 (project structure)
- **Affected Components**: examples/ directory, CI pipeline
- **External Dependencies**: tokio (for async examples)

## Testing Strategy

### Compilation Tests

```bash
# All examples must compile
cargo build --examples --all-features

# No warnings allowed
cargo build --examples --all-features 2>&1 | grep warning && exit 1
```

### Execution Tests

```bash
# All examples must run successfully
for example in form_validation user_registration error_context data_pipeline testing_patterns; do
  cargo run --example $example || exit 1
done
```

### CI Integration

Add to `.github/workflows/ci.yml`:
```yaml
- name: Build examples
  run: cargo build --examples --all-features

- name: Run examples
  run: |
    cargo run --example form_validation
    cargo run --example user_registration
    cargo run --example error_context
    cargo run --example data_pipeline
    cargo run --example testing_patterns
```

## Documentation Requirements

### Code Documentation

- Each example has header comment explaining:
  - What it demonstrates
  - How to run it
  - Key takeaways
- Inline comments explain non-obvious patterns
- Output shows both success and failure paths

### User Documentation

Update README.md:
```markdown
## Examples

Run any example with `cargo run --example <name>`:

- **form_validation**: Error accumulation with Validation
- **user_registration**: Effect composition and I/O separation
- **error_context**: Error trails for better debugging
- **data_pipeline**: Real-world ETL pipeline
- **testing_patterns**: Pure functions vs effectful code testing

See [examples/](examples/) directory for full code.
```

### Architecture Updates

- Document examples in DESIGN.md
- Link to examples from relevant specs

## Implementation Notes

### Output Format

All examples follow consistent output format:
```
Example: <Name>

=== Scenario Name ===
[Output here]

✨ Key Takeaway: [Main lesson]
```

### Self-Contained

Examples don't depend on each other or external files. All data is inline.

### Realistic but Simple

Examples are realistic enough to demonstrate real patterns, but simple enough to understand quickly.

### Comments vs Documentation

- Header comments: Explain what and why
- Inline comments: Explain how (only when non-obvious)
- Let code be self-documenting where possible

## Migration and Compatibility

No migration - these are new examples based on earlier fictional versions.

## Open Questions

1. Should we add more examples?
   - Decision: These 5 cover core use cases, add more post-MVP based on user requests

2. Should examples use macros for less boilerplate?
   - Decision: No, show explicit patterns so users understand what's happening

3. Should we add a "kitchen sink" example?
   - Decision: No, focused examples are better than one giant example

4. Should examples be integration tests too?
   - Decision: No, keep examples and tests separate for clarity
