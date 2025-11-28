//! Comparison Documentation Examples
//!
//! This file verifies that the code examples in docs/COMPARISON.md compile correctly.
//! Each section corresponds to an example from the comparison document.
//!
//! Run with: cargo run --example comparison_doc --features async

use std::sync::Arc;
use std::time::Duration;

use stillwater::context::ContextError;
use stillwater::effect::bracket::{bracket_full, BracketError};
use stillwater::effect::context::{EffectContext, EffectContextChain};
use stillwater::effect::prelude::*;
use stillwater::effect::retry::{retry, with_timeout};
use stillwater::retry::RetryExhausted;
use stillwater::validation::ValidateAll;
use stillwater::{RetryPolicy, TimeoutError, Validation};

// ============================================================================
// Mock types for examples
// ============================================================================

#[derive(Debug, Clone)]
struct ValidatedUser {
    email: String,
    password: String,
    age: u8,
}

impl ValidatedUser {
    fn summary(&self) -> String {
        let password_strength = if self.password.len() >= 12 {
            "strong"
        } else {
            "ok"
        };
        format!(
            "{} (age {}, password: {})",
            self.email, self.age, password_strength
        )
    }
}

#[derive(Debug, Clone)]
struct User {
    id: u64,
    email: String,
    session_id: String,
}

impl User {
    fn greeting(&self) -> String {
        format!("Welcome user {} ({})", self.id, self.email)
    }
}

#[derive(Debug, Clone)]
struct Dashboard {
    profile: String,
    orders: Vec<String>,
    recommendations: Vec<String>,
}

impl Dashboard {
    fn summary(&self) -> String {
        format!(
            "{}: {} orders, {} recommendations",
            self.profile,
            self.orders.len(),
            self.recommendations.len()
        )
    }
}

#[derive(Debug, Clone)]
struct Transfer {
    id: u64,
}

impl Transfer {
    fn receipt(&self) -> String {
        format!("Transfer #{} completed", self.id)
    }
}

#[derive(Debug, Clone)]
struct AppEnv {
    db: Arc<MockDb>,
    cache: Arc<MockCache>,
    email: Arc<MockEmail>,
    http: Arc<MockHttp>,
}

#[derive(Debug, Clone)]
struct MockDb;
#[derive(Debug, Clone)]
struct MockCache;
#[derive(Debug, Clone)]
struct MockEmail;
#[derive(Debug, Clone)]
struct MockHttp;
#[derive(Debug, Clone)]
struct MockTransaction;

impl MockDb {
    async fn insert_user(&self, email: &str) -> Result<User, String> {
        Ok(User {
            id: 1,
            email: email.to_string(),
            session_id: format!("session-{}", email.len()),
        })
    }

    async fn begin_transaction(&self) -> Result<MockTransaction, String> {
        Ok(MockTransaction)
    }
}

impl MockCache {
    async fn set(&self, _key: &str, _value: &u64) -> Result<(), String> {
        Ok(())
    }
}

impl MockEmail {
    async fn send_template(&self, _template: &str, _email: &str) -> Result<(), String> {
        Ok(())
    }
}

impl MockHttp {
    async fn get(&self, _url: &str) -> Result<String, String> {
        Ok("response".to_string())
    }
}

impl MockTransaction {
    async fn commit(self) -> Result<(), String> {
        Ok(())
    }
}

/// Validated signup - password is validated but not stored
/// (in a real app, you'd hash it during validation)
#[derive(Debug, Clone)]
struct ValidatedSignup {
    email: String,
}

// ============================================================================
// Example 1: Basic 3-Field Validation (COMPARISON.md lines 62-78)
// ============================================================================

fn validate_email(email: &str) -> Validation<String, Vec<String>> {
    if email.contains('@') {
        Validation::success(email.to_string())
    } else {
        Validation::failure(vec!["Invalid email".to_string()])
    }
}

fn validate_password(password: &str) -> Validation<String, Vec<String>> {
    if password.len() >= 8 {
        Validation::success(password.to_string())
    } else {
        Validation::failure(vec!["Password too short".to_string()])
    }
}

fn validate_age(age: u8) -> Validation<u8, Vec<String>> {
    if age >= 18 {
        Validation::success(age)
    } else {
        Validation::failure(vec!["Must be 18 or older".to_string()])
    }
}

/// From COMPARISON.md: Basic 3-field validation with error accumulation
fn validate_user_registration(
    email: &str,
    password: &str,
    age: u8,
) -> Validation<ValidatedUser, Vec<String>> {
    (
        validate_email(email),
        validate_password(password),
        validate_age(age),
    )
        .validate_all()
        .map(|(email, password, age)| ValidatedUser {
            email,
            password,
            age,
        })
}

// ============================================================================
// Example 2: Dependency Injection (COMPARISON.md lines 370-408)
// ============================================================================

fn validate_signup(input: &SignupInput) -> Result<ValidatedSignup, String> {
    if !input.email.contains('@') {
        return Err("Invalid email".to_string());
    }
    if input.password.len() < 8 {
        return Err("Password too short".to_string());
    }
    // Password validated but not stored (would be hashed in real app)
    Ok(ValidatedSignup {
        email: input.email.clone(),
    })
}

struct SignupInput {
    email: String,
    password: String,
}

fn create_user(input: ValidatedSignup) -> impl Effect<Output = User, Error = String, Env = AppEnv> {
    let email = input.email;
    from_async(move |env: &AppEnv| {
        let db = env.db.clone();
        let email = email.clone();
        async move { db.insert_user(&email).await }
    })
}

fn cache_user_session(user: &User) -> impl Effect<Output = (), Error = String, Env = AppEnv> {
    let session_id = user.session_id.clone();
    let user_id = user.id;
    from_async(move |env: &AppEnv| {
        let cache = env.cache.clone();
        async move { cache.set(&session_id, &user_id).await }
    })
}

fn send_welcome_email(user: &User) -> impl Effect<Output = (), Error = String, Env = AppEnv> {
    let email = user.email.clone();
    from_async(move |env: &AppEnv| {
        let email_service = env.email.clone();
        async move { email_service.send_template("welcome", &email).await }
    })
}

/// From COMPARISON.md: Dependency injection via Reader pattern
fn process_user_signup(
    input: SignupInput,
) -> impl Effect<Output = User, Error = String, Env = AppEnv> {
    from_result(validate_signup(&input))
        .and_then(create_user)
        .and_then(|user| {
            let user_for_email = user.clone();
            let user_to_return = user.clone();
            cache_user_session(&user)
                .and_then(move |_| send_welcome_email(&user_for_email))
                .map(move |_| user_to_return)
        })
}

// ============================================================================
// Example 3: Error Context (COMPARISON.md lines 298-328)
// ============================================================================

#[derive(Debug, Clone)]
struct Order {
    id: u64,
    items: Vec<String>,
    total: f64,
}

#[derive(Debug, Clone)]
struct Receipt {
    order_id: u64,
    items_count: usize,
    total: f64,
    status: String,
}

impl Receipt {
    fn summary(&self) -> String {
        format!(
            "Receipt #{}: {} items, ${:.2} - {}",
            self.order_id, self.items_count, self.total, self.status
        )
    }
}

fn fetch_order_effect(order_id: u64) -> impl Effect<Output = Order, Error = String, Env = AppEnv> {
    // Simulate: order 999 doesn't exist
    from_result(if order_id == 999 {
        Err("order not found".to_string())
    } else {
        Ok(Order {
            id: order_id,
            items: vec!["Widget".to_string(), "Gadget".to_string()],
            total: 99.99,
        })
    })
}

fn check_inventory_effect(
    order: Order,
) -> impl Effect<Output = Order, Error = String, Env = AppEnv> {
    // Simulate: order 888 has items out of stock
    from_result(if order.id == 888 {
        Err("items out of stock".to_string())
    } else {
        Ok(order)
    })
}

fn create_receipt_effect(
    order: Order,
) -> impl Effect<Output = Receipt, Error = String, Env = AppEnv> {
    pure(Receipt {
        order_id: order.id,
        items_count: order.items.len(),
        total: order.total,
        status: "confirmed".to_string(),
    })
}

/// From COMPARISON.md: Error context chaining
fn process_order(
    order_id: u64,
) -> impl Effect<Output = Receipt, Error = ContextError<String>, Env = AppEnv> {
    fetch_order_effect(order_id)
        .context("fetching order")
        .context_chain(format!("processing order {}", order_id))
        .and_then(move |order| {
            check_inventory_effect(order)
                .context("checking inventory")
                .context_chain(format!("processing order {}", order_id))
                .and_then(move |order| {
                    create_receipt_effect(order)
                        .context("creating receipt")
                        .context_chain(format!("processing order {}", order_id))
                })
        })
}

// ============================================================================
// Example 4: Retry with Exponential Backoff (COMPARISON.md lines 510-547)
// ============================================================================

/// From COMPARISON.md: Retry with exponential backoff
fn fetch_with_retry(
    url: String,
) -> impl Effect<Output = RetryExhausted<String>, Error = RetryExhausted<String>, Env = AppEnv> {
    retry(
        move || {
            let url = url.clone();
            from_async(move |env: &AppEnv| {
                let client = env.http.clone();
                let url = url.clone();
                async move { client.get(&url).await }
            })
        },
        RetryPolicy::exponential(Duration::from_millis(100))
            .with_max_retries(3)
            .with_max_delay(Duration::from_secs(30)),
    )
}

// ============================================================================
// Example 5: Parallel with Timeout (COMPARISON.md lines 572-601)
// ============================================================================

fn fetch_profile(_user_id: u64) -> impl Effect<Output = String, Error = String, Env = AppEnv> {
    pure("profile".to_string())
}

fn fetch_recent_orders(
    _user_id: u64,
) -> impl Effect<Output = Vec<String>, Error = String, Env = AppEnv> {
    pure(vec!["order1".to_string()])
}

fn fetch_recommendations(
    _user_id: u64,
) -> impl Effect<Output = Vec<String>, Error = String, Env = AppEnv> {
    pure(vec!["rec1".to_string()])
}

/// From COMPARISON.md: Parallel operations with timeout using zip3
fn fetch_user_dashboard(
    user_id: u64,
) -> impl Effect<Output = Dashboard, Error = TimeoutError<String>, Env = AppEnv> {
    with_timeout(
        zip3(
            fetch_profile(user_id),
            fetch_recent_orders(user_id),
            fetch_recommendations(user_id),
        )
        .map(|(profile, orders, recommendations)| Dashboard {
            profile,
            orders,
            recommendations,
        }),
        Duration::from_secs(5),
    )
}

// ============================================================================
// Example 6: Database Transaction with bracket_full (COMPARISON.md lines 751-787)
// ============================================================================

/// From COMPARISON.md: Database transaction with bracket_full
///
/// Note: The actual use of bracket_full requires careful handling of the
/// resource lifetime. This simplified example shows the pattern.
fn transfer_funds(
    from_account: u64,
    to_account: u64,
    amount: u64,
) -> impl Effect<Output = Transfer, Error = BracketError<String>, Env = AppEnv> {
    bracket_full(
        // Acquire: begin transaction
        from_async(|env: &AppEnv| {
            let db = env.db.clone();
            async move { db.begin_transaction().await }
        }),
        // Release: always attempt commit
        |tx| async move { tx.commit().await },
        // Use: perform transfer operations (simplified - just returns a Transfer)
        move |_tx: &MockTransaction| {
            // In a real implementation, you'd call helper functions here
            // that capture the necessary data from tx
            let _ = (from_account, to_account, amount); // Use the values
            pure::<_, String, AppEnv>(Transfer { id: 1 })
        },
    )
}

// ============================================================================
// Main: Run all examples
// ============================================================================

#[tokio::main]
async fn main() {
    println!("=== COMPARISON.md Examples Verification ===\n");

    // Example 1: Validation
    println!("1. Basic 3-Field Validation");
    let result = validate_user_registration("test@example.com", "password123", 25);
    match result {
        Validation::Success(user) => println!("   Valid: {}", user.summary()),
        Validation::Failure(errors) => println!("   Errors: {:?}", errors),
    }

    let result = validate_user_registration("invalid", "short", 16);
    match result {
        Validation::Success(user) => println!("   Valid: {}", user.summary()),
        Validation::Failure(errors) => println!("   All errors collected: {:?}", errors),
    }

    // Setup environment for effect examples
    let env = AppEnv {
        db: Arc::new(MockDb),
        cache: Arc::new(MockCache),
        email: Arc::new(MockEmail),
        http: Arc::new(MockHttp),
    };

    // Example 2: Dependency Injection
    println!("\n2. Dependency Injection (Reader Pattern)");
    let signup = SignupInput {
        email: "new@example.com".to_string(),
        password: "secure123".to_string(),
    };
    let result = process_user_signup(signup).run(&env).await;
    match result {
        Ok(user) => println!("   {}", user.greeting()),
        Err(e) => println!("   Error: {}", e),
    }

    // Example 3: Error Context
    println!("\n3. Error Context Chaining");

    // Success case
    let result = process_order(123).run(&env).await;
    match result {
        Ok(receipt) => println!("   Success: {}", receipt.summary()),
        Err(e) => println!("   Error: {} (context: {:?})", e.inner(), e.context_trail()),
    }

    // Error case - demonstrates context trail
    let result = process_order(999).run(&env).await;
    match result {
        Ok(receipt) => println!("   Success: {}", receipt.summary()),
        Err(e) => println!(
            "   Error: \"{}\" with context: {:?}",
            e.inner(),
            e.context_trail()
        ),
    }

    // Example 4: Retry
    println!("\n4. Retry with Exponential Backoff");
    let result = fetch_with_retry("https://example.com".to_string())
        .run(&env)
        .await;
    match result {
        Ok(success) => {
            let attempts = success.attempts;
            let response = success.into_value();
            println!("   Success after {} attempts: {}", attempts, response);
        }
        Err(exhausted) => println!(
            "   Failed after {} attempts: {}",
            exhausted.attempts, exhausted.final_error
        ),
    }

    // Example 5: Parallel with Timeout
    println!("\n5. Parallel Operations with Timeout (zip3)");
    let result = fetch_user_dashboard(42).run(&env).await;
    match result {
        Ok(dashboard) => println!("   {}", dashboard.summary()),
        Err(TimeoutError::Timeout { duration }) => {
            println!("   Timed out after {:?}", duration)
        }
        Err(TimeoutError::Inner(e)) => println!("   Inner error: {:?}", e),
    }

    // Example 6: Database Transaction
    println!("\n6. Database Transaction (bracket_full)");
    let result = transfer_funds(1, 2, 100).run(&env).await;
    match result {
        Ok(transfer) => println!("   {}", transfer.receipt()),
        Err(BracketError::AcquireError(e)) => println!("   Acquire failed: {}", e),
        Err(BracketError::UseError(e)) => println!("   Use failed: {}", e),
        Err(BracketError::CleanupError(e)) => println!("   Cleanup failed: {}", e),
        Err(BracketError::Both {
            use_error,
            cleanup_error,
        }) => {
            println!(
                "   Both failed: use={}, cleanup={}",
                use_error, cleanup_error
            )
        }
    }

    println!("\n=== All examples compiled and ran successfully! ===");
}
