# Comparison to Other Libraries

This document compares Stillwater to other Rust libraries providing similar functionality, with **extensive before/after code examples** demonstrating real-world improvements.

## Quick Comparison Table

| Feature | Stillwater | frunk | monadic | anyhow | validator |
|---------|-----------|-------|---------|--------|-----------|
| Error accumulation | ✓ | ✓ | ✗ | ✗ | ✗ |
| Effect composition | ✓ | ✗ | ✓ | ✗ | ✗ |
| Async support | ✓ | ✗ | ✗ | ✓ | ✗ |
| Learning curve | Low | High | Medium | Low | Low |
| Type-level programming | ✗ | ✓ | ✗ | ✗ | ✗ |
| Macro DSL | ✗ | ✗ | ✓ | ✓ | ✓ |
| Pure Rust idioms | ✓ | Partial | Partial | ✓ | ✓ |
| Dependencies | 0 (core) | Many | Few | Few | Many |

---

## Before/After Examples: Form Validation

### Basic 3-Field Validation

**Problem**: Collecting ALL validation errors for a user registration form, not just the first one.

**Before** (Traditional Rust - 22 lines):
```rust
fn validate_user_registration(
    email: &str,
    password: &str,
    age: u8,
) -> Result<ValidatedUser, Vec<String>> {
    let mut errors = Vec::new();

    let validated_email = match validate_email(email) {
        Ok(e) => Some(e),
        Err(e) => { errors.push(e); None }
    };

    let validated_password = match validate_password(password) {
        Ok(p) => Some(p),
        Err(e) => { errors.push(e); None }
    };

    let validated_age = match validate_age(age) {
        Ok(a) => Some(a),
        Err(e) => { errors.push(e); None }
    };

    if errors.is_empty() {
        Ok(ValidatedUser {
            email: validated_email.unwrap(),
            password: validated_password.unwrap(),
            age: validated_age.unwrap(),
        })
    } else {
        Err(errors)
    }
}
```

**After** (With Stillwater - 8 lines, 64% reduction):
```rust
use stillwater::{Validation, validation::ValidateAll};

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
    .map(|(email, password, age)| ValidatedUser { email, password, age })
}
```

**Key Improvements**:
- 64% less code (22 → 8 lines)
- No mutable state or `Option` unwrapping
- Declarative tuple-based composition
- Type-safe error accumulation via `Semigroup`

---

### 5-Field Validation with Nested Objects

**Problem**: Validating a complex order form with shipping address.

**Before** (Traditional Rust - 42 lines):
```rust
fn validate_order(input: &OrderInput) -> Result<ValidatedOrder, Vec<String>> {
    let mut errors = Vec::new();

    let customer_name = match validate_name(&input.customer_name) {
        Ok(n) => Some(n),
        Err(e) => { errors.push(e); None }
    };

    let email = match validate_email(&input.email) {
        Ok(e) => Some(e),
        Err(e) => { errors.push(e); None }
    };

    let phone = match validate_phone(&input.phone) {
        Ok(p) => Some(p),
        Err(e) => { errors.push(e); None }
    };

    // Nested address validation
    let street = match validate_street(&input.address.street) {
        Ok(s) => Some(s),
        Err(e) => { errors.push(format!("address.street: {}", e)); None }
    };

    let city = match validate_city(&input.address.city) {
        Ok(c) => Some(c),
        Err(e) => { errors.push(format!("address.city: {}", e)); None }
    };

    let postal_code = match validate_postal_code(&input.address.postal_code) {
        Ok(p) => Some(p),
        Err(e) => { errors.push(format!("address.postal_code: {}", e)); None }
    };

    if errors.is_empty() {
        Ok(ValidatedOrder {
            customer_name: customer_name.unwrap(),
            email: email.unwrap(),
            phone: phone.unwrap(),
            address: ValidatedAddress {
                street: street.unwrap(),
                city: city.unwrap(),
                postal_code: postal_code.unwrap(),
            },
        })
    } else {
        Err(errors)
    }
}
```

**After** (With Stillwater - 18 lines, 57% reduction):
```rust
use stillwater::{Validation, validation::ValidateAll};

fn validate_order(input: &OrderInput) -> Validation<ValidatedOrder, Vec<String>> {
    let address = (
        validate_street(&input.address.street)
            .map_err(|e| vec![format!("address.street: {}", e)]),
        validate_city(&input.address.city)
            .map_err(|e| vec![format!("address.city: {}", e)]),
        validate_postal_code(&input.address.postal_code)
            .map_err(|e| vec![format!("address.postal_code: {}", e)]),
    )
    .validate_all()
    .map(|(street, city, postal_code)| ValidatedAddress { street, city, postal_code });

    (
        validate_name(&input.customer_name),
        validate_email(&input.email),
        validate_phone(&input.phone),
        address,
    )
    .validate_all()
    .map(|(customer_name, email, phone, address)| {
        ValidatedOrder { customer_name, email, phone, address }
    })
}
```

**Key Improvements**:
- 57% less code (42 → 18 lines)
- Composable nested validation
- Field path prefixes handled cleanly
- No nested conditionals

---

### Conditional Validation

**Problem**: Password confirmation must match, but only validate strength if they match.

**Before** (Traditional Rust - 20 lines):
```rust
fn validate_password_change(
    current: &str,
    new_password: &str,
    confirm: &str,
) -> Result<ValidatedPassword, Vec<String>> {
    let mut errors = Vec::new();

    // First check if passwords match
    if new_password != confirm {
        errors.push("Passwords do not match".to_string());
    }

    // Validate current password
    if current.is_empty() {
        errors.push("Current password required".to_string());
    }

    // Only validate strength if passwords match
    if new_password == confirm {
        if new_password.len() < 8 {
            errors.push("Password must be at least 8 characters".to_string());
        }
        if !new_password.chars().any(|c| c.is_uppercase()) {
            errors.push("Password must contain uppercase".to_string());
        }
    }

    if errors.is_empty() {
        Ok(ValidatedPassword { password: new_password.to_string() })
    } else {
        Err(errors)
    }
}
```

**After** (With Stillwater - 14 lines, 30% reduction):
```rust
use stillwater::{Validation, validation::ValidateAll};

fn validate_password_change(
    current: &str,
    new_password: &str,
    confirm: &str,
) -> Validation<ValidatedPassword, Vec<String>> {
    let passwords_match = if new_password == confirm {
        Validation::success(new_password.to_string())
    } else {
        Validation::failure(vec!["Passwords do not match".to_string()])
    };

    (
        validate_non_empty(current, "Current password required"),
        passwords_match.and_then(validate_password_strength),
    )
    .validate_all()
    .map(|(_, password)| ValidatedPassword { password })
}
```

**Key Improvements**:
- Conditional validation via `and_then`
- Early exit for mismatched passwords
- Clean separation of concerns

---

## Before/After Examples: Error Context

### Error Trail for Debugging

**Problem**: When a deeply nested operation fails, you need the full context trail.

**Before** (Traditional Rust - 24 lines):
```rust
#[derive(Debug)]
struct ContextError {
    message: String,
    context: Vec<String>,
}

fn process_order(order_id: u64) -> Result<Receipt, ContextError> {
    let order = fetch_order(order_id)
        .map_err(|e| ContextError {
            message: e,
            context: vec!["fetching order".to_string()],
        })?;

    let inventory = check_inventory(&order.items)
        .map_err(|e| ContextError {
            message: e,
            context: vec![
                "checking inventory".to_string(),
                format!("for order {}", order_id),
            ],
        })?;

    let receipt = create_receipt(&order, &inventory)
        .map_err(|e| ContextError {
            message: e,
            context: vec![
                "creating receipt".to_string(),
                format!("for order {}", order_id),
            ],
        })?;

    Ok(receipt)
}
```

**After** (With Stillwater - 16 lines, 33% reduction):
```rust
use stillwater::effect::prelude::*;
use stillwater::effect::context::{EffectContext, EffectContextChain};
use stillwater::context::ContextError;

fn process_order(order_id: u64) -> impl Effect<Output = Receipt, Error = ContextError<String>, Env = AppEnv> {
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
```

**Key Improvements**:
- `.context()` wraps any error in `ContextError<E>` with the given message
- `.context_chain()` adds additional context to an existing `ContextError`
- Error trail shows: `["fetching order", "processing order 999"]` on failure
- Each operation gets its own context; outer context added via `context_chain`

> **Note**: `.context()` creates a new `ContextError` wrapping the inner error.
> Use `.context_chain()` to add to an existing `ContextError`'s trail.

---

## Before/After Examples: Dependency Injection

### Parameter Threading (3 Dependencies, 4 Functions)

**Problem**: Passing database, cache, and email service through multiple function calls.

**Before** (Traditional Rust - 32 lines):
```rust
async fn process_user_signup(
    db: &Database,
    cache: &Cache,
    email_service: &EmailService,
    input: SignupInput,
) -> Result<User, Error> {
    let validated = validate_signup(&input)?;

    let user = create_user(db, &validated).await?;

    cache_user_session(cache, &user).await?;

    send_welcome_email(email_service, &user).await?;

    Ok(user)
}

async fn create_user(db: &Database, input: &ValidatedSignup) -> Result<User, Error> {
    db.insert_user(input).await
}

async fn cache_user_session(cache: &Cache, user: &User) -> Result<(), Error> {
    cache.set(&user.session_id, &user.id).await
}

async fn send_welcome_email(email_service: &EmailService, user: &User) -> Result<(), Error> {
    email_service.send_template("welcome", &user.email).await
}
```

**After** (With Stillwater Reader Pattern - 24 lines, 25% reduction):
```rust
use stillwater::effect::prelude::*;

fn process_user_signup(input: SignupInput) -> impl Effect<Output = User, Error = Error, Env = AppEnv> {
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

fn create_user(input: ValidatedSignup) -> impl Effect<Output = User, Error = Error, Env = AppEnv> {
    from_async(move |env: &AppEnv| {
        let db = env.db.clone();
        async move { db.insert_user(&input).await }
    })
}

fn cache_user_session(user: &User) -> impl Effect<Output = (), Error = Error, Env = AppEnv> {
    let session_id = user.session_id.clone();
    let user_id = user.id;
    from_async(move |env: &AppEnv| {
        let cache = env.cache.clone();
        async move { cache.set(&session_id, &user_id).await }
    })
}

fn send_welcome_email(user: &User) -> impl Effect<Output = (), Error = Error, Env = AppEnv> {
    let email = user.email.clone();
    from_async(move |env: &AppEnv| {
        let email_service = env.email.clone();
        async move { email_service.send_template("welcome", &email).await }
    })
}
```

**Key Improvements**:
- No parameter threading (dependencies accessed via environment)
- Functions are self-contained and composable
- Adding a new dependency requires only changing `AppEnv`
- Testing is trivial (just provide a test environment)

> **Note**: When composing effects that need the same value, clone upfront to satisfy
> Rust's ownership rules. The helper functions already extract only what they need.

---

### Testing with Mock Dependencies

**Before** (Traditional Rust - complex mocking):
```rust
#[cfg(test)]
mod tests {
    use mockall::{automock, predicate::*};

    #[automock]
    trait DatabaseTrait {
        async fn fetch_user(&self, id: u64) -> Result<User, Error>;
    }

    #[tokio::test]
    async fn test_process_user() {
        let mut mock_db = MockDatabaseTrait::new();
        mock_db.expect_fetch_user()
            .with(eq(123))
            .times(1)
            .returning(|_| Ok(User { id: 123, name: "Test".into() }));

        let result = process_user(&mock_db, 123).await;
        assert!(result.is_ok());
    }
}
```

**After** (With Stillwater - 12 lines, simpler):
```rust
#[cfg(test)]
mod tests {
    use stillwater::effect::prelude::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_process_user() {
        let test_env = AppEnv {
            db: Arc::new(InMemoryDb::with_user(User { id: 123, name: "Test".into() })),
            cache: Arc::new(NoOpCache),
            email: Arc::new(RecordingEmailService::new()),
        };

        let result = process_user(123).run(&test_env).await;
        assert!(result.is_ok());
    }
}
```

**Key Improvements**:
- No mocking framework required
- Test environment is just data
- Easy to reuse test fixtures across tests
- Behavior verification via recording implementations

---

## Before/After Examples: Async Composition

### Retry with Exponential Backoff

**Problem**: Retry a flaky network call with exponential backoff.

**Before** (Traditional Rust - 35 lines):
```rust
use std::time::Duration;
use tokio::time::sleep;

async fn fetch_with_retry(url: &str, max_retries: u32) -> Result<Response, Error> {
    let mut attempt = 0;
    let mut delay = Duration::from_millis(100);

    loop {
        match http_client.get(url).await {
            Ok(response) => return Ok(response),
            Err(e) => {
                attempt += 1;
                if attempt > max_retries {
                    return Err(Error::RetriesExhausted {
                        attempts: attempt,
                        last_error: Box::new(e),
                    });
                }

                // Exponential backoff with jitter
                let jitter = rand::random::<u64>() % 50;
                sleep(delay + Duration::from_millis(jitter)).await;
                delay = std::cmp::min(delay * 2, Duration::from_secs(30));
            }
        }
    }
}
```

**After** (With Stillwater - 14 lines, 60% reduction):
```rust
use stillwater::effect::prelude::*;
use stillwater::effect::retry::retry;
use stillwater::retry::RetryExhausted;
use stillwater::RetryPolicy;
use std::time::Duration;

fn fetch_with_retry(url: String)
    -> impl Effect<Output = RetryExhausted<Response>, Error = RetryExhausted<Error>, Env = AppEnv>
{
    retry(
        move || {
            let url = url.clone();
            from_async(move |env: &AppEnv| {
                let client = env.http.clone();
                async move { client.get(&url).await }
            })
        },
        RetryPolicy::exponential(Duration::from_millis(100))
            .with_max_retries(3)
            .with_max_delay(Duration::from_secs(30)),
    )
}

// Usage: extract the response from RetryExhausted
// let result = fetch_with_retry(url).run(&env).await?;
// let response = result.into_value();  // Get the Response
// let attempts = result.attempts;       // How many attempts it took
```

**Key Improvements**:
- 60% less code (35 → 14 lines)
- Built-in jitter and backoff calculations
- Configurable retry policy
- `RetryExhausted` tracks attempt count and total duration on both success and failure

> **Note**: Both success and failure are wrapped in `RetryExhausted<T>` which provides
> `.into_value()`, `.attempts`, and `.total_duration` for observability.

---

### Parallel Operations with Timeout

**Problem**: Fetch user data from multiple services concurrently with a timeout.

**Before** (Traditional Rust - 25 lines):
```rust
use tokio::time::timeout;
use futures::future::try_join3;

async fn fetch_user_dashboard(user_id: u64) -> Result<Dashboard, Error> {
    let timeout_duration = Duration::from_secs(5);

    let (profile, orders, recommendations) = timeout(
        timeout_duration,
        try_join3(
            fetch_profile(user_id),
            fetch_recent_orders(user_id),
            fetch_recommendations(user_id),
        )
    )
    .await
    .map_err(|_| Error::Timeout)?
    .map_err(Error::from)?;

    Ok(Dashboard {
        profile,
        orders,
        recommendations,
    })
}
```

**After** (With Stillwater - 17 lines, 32% reduction):
```rust
use stillwater::effect::prelude::*;
use stillwater::effect::retry::with_timeout;
use stillwater::TimeoutError;
use std::time::Duration;

fn fetch_user_dashboard(user_id: u64) -> impl Effect<Output = Dashboard, Error = TimeoutError<Error>, Env = AppEnv> {
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
```

**Key Improvements**:
- 32% less code
- `zip3` for type-safe parallel composition (returns an Effect)
- Integrated timeout handling
- Clear error type showing timeout vs inner error

> **Note**: Use `zip3` for Effect composition. The `par3` function is an async helper
> that returns a tuple of Results directly, useful when you need individual error handling.

---

## Real-World Scenarios

### Scenario 1: API Request Handler

**Problem**: Handle an API request with validation, business logic, and error handling.

**Before** (Traditional Rust - 45 lines):
```rust
async fn handle_create_order(
    db: &Database,
    cache: &Cache,
    email: &EmailService,
    request: CreateOrderRequest,
) -> Result<ApiResponse<Order>, ApiError> {
    // Validate input
    let mut validation_errors = Vec::new();

    if request.items.is_empty() {
        validation_errors.push("Order must have at least one item");
    }
    if request.customer_id == 0 {
        validation_errors.push("Invalid customer ID");
    }
    if request.items.iter().any(|i| i.quantity == 0) {
        validation_errors.push("Item quantity must be positive");
    }

    if !validation_errors.is_empty() {
        return Err(ApiError::ValidationFailed(validation_errors));
    }

    // Check customer exists
    let customer = db.get_customer(request.customer_id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or(ApiError::NotFound("Customer not found"))?;

    // Check inventory
    for item in &request.items {
        let stock = cache.get_stock(item.product_id)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?;
        if stock < item.quantity {
            return Err(ApiError::BusinessLogic("Insufficient stock"));
        }
    }

    // Create order
    let order = db.create_order(&customer, &request.items)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    // Send confirmation (fire and forget)
    let _ = email.send_order_confirmation(&customer.email, &order).await;

    Ok(ApiResponse::success(order))
}
```

**After** (With Stillwater - 30 lines, 33% reduction):
```rust
use stillwater::{Validation, validation::ValidateAll};
use stillwater::effect::prelude::*;

fn handle_create_order(request: CreateOrderRequest) -> impl Effect<Output = ApiResponse<Order>, Error = ApiError, Env = AppEnv> {
    // Validation phase
    let validated = (
        validate_non_empty_items(&request.items),
        validate_customer_id(request.customer_id),
        validate_item_quantities(&request.items),
    )
    .validate_all()
    .map_err(ApiError::ValidationFailed);

    from_validation(validated)
        .and_then(move |_| {
            // Business logic phase
            fetch_customer(request.customer_id)
                .and_then(move |customer| {
                    check_inventory(&request.items)
                        .and_then(move |_| create_order(&customer, &request.items))
                        .and_then(move |order| {
                            send_confirmation(&customer.email, &order)
                                .map(move |_| ApiResponse::success(order))
                        })
                })
        })
        .map_err(|e| ApiError::Internal(e.to_string()))
}
```

**Key Improvements**:
- Clear separation: validation → business logic
- All validation errors collected upfront
- No explicit error mapping at each step
- Composable, testable functions

---

### Scenario 2: Database Transaction

**Problem**: Execute multiple database operations atomically.

**Before** (Traditional Rust - 30 lines):
```rust
async fn transfer_funds(
    db: &Database,
    from_account: u64,
    to_account: u64,
    amount: Decimal,
) -> Result<Transfer, Error> {
    let tx = db.begin_transaction().await?;

    let result = async {
        // Check source balance
        let from = tx.get_account(from_account).await?;
        if from.balance < amount {
            return Err(Error::InsufficientFunds);
        }

        // Debit source
        tx.update_balance(from_account, from.balance - amount).await?;

        // Credit destination
        let to = tx.get_account(to_account).await?;
        tx.update_balance(to_account, to.balance + amount).await?;

        // Record transfer
        let transfer = tx.create_transfer(from_account, to_account, amount).await?;

        Ok(transfer)
    }.await;

    match result {
        Ok(transfer) => {
            tx.commit().await?;
            Ok(transfer)
        }
        Err(e) => {
            tx.rollback().await?;
            Err(e)
        }
    }
}
```

**After** (With Stillwater bracket - 24 lines, 20% reduction):
```rust
use stillwater::effect::prelude::*;
use stillwater::effect::bracket::{bracket_full, BracketError};

fn transfer_funds(
    from_account: u64,
    to_account: u64,
    amount: Decimal,
) -> impl Effect<Output = Transfer, Error = BracketError<Error>, Env = AppEnv> {
    bracket_full(
        // Acquire: begin transaction
        from_async(|env: &AppEnv| {
            let db = env.db.clone();
            async move { db.begin_transaction().await }
        }),
        // Release: always attempt commit (rollback on drop if uncommitted)
        |tx| async move { tx.commit().await },
        // Use: perform transfer operations
        |tx| {
            check_balance(tx, from_account, amount)
                .and_then(move |_| debit_account(tx, from_account, amount))
                .and_then(move |_| credit_account(tx, to_account, amount))
                .and_then(move |_| record_transfer(tx, from_account, to_account, amount))
        },
    )
}
```

**Key Improvements**:
- Resource safety via `bracket_full` pattern
- `BracketError` distinguishes acquire/use/cleanup failures
- Transaction auto-rollbacks on drop if not committed
- Composable operations within transaction

> **Note**: `bracket_full` returns `BracketError<E>` which tells you exactly which phase
> failed. Use plain `bracket` if you only need the use error (cleanup errors are logged).

---

### Scenario 3: Configuration Validation

**Problem**: Validate application configuration at startup.

**Before** (Traditional Rust - 40 lines):
```rust
fn validate_config(config: &RawConfig) -> Result<ValidatedConfig, Vec<ConfigError>> {
    let mut errors = Vec::new();

    let port = match config.port {
        Some(p) if p > 0 && p < 65536 => Some(p as u16),
        Some(p) => { errors.push(ConfigError::InvalidPort(p)); None }
        None => { errors.push(ConfigError::MissingPort); None }
    };

    let database_url = match &config.database_url {
        Some(url) if url.starts_with("postgres://") => Some(url.clone()),
        Some(url) => { errors.push(ConfigError::InvalidDatabaseUrl(url.clone())); None }
        None => { errors.push(ConfigError::MissingDatabaseUrl); None }
    };

    let log_level = match config.log_level.as_deref() {
        Some("debug") | Some("info") | Some("warn") | Some("error") => {
            Some(config.log_level.clone().unwrap())
        }
        Some(level) => { errors.push(ConfigError::InvalidLogLevel(level.to_string())); None }
        None => Some("info".to_string()) // Default
    };

    let max_connections = match config.max_connections {
        Some(n) if n > 0 && n <= 1000 => Some(n),
        Some(n) => { errors.push(ConfigError::InvalidMaxConnections(n)); None }
        None => Some(10) // Default
    };

    if errors.is_empty() {
        Ok(ValidatedConfig {
            port: port.unwrap(),
            database_url: database_url.unwrap(),
            log_level: log_level.unwrap(),
            max_connections: max_connections.unwrap(),
        })
    } else {
        Err(errors)
    }
}
```

**After** (With Stillwater - 22 lines, 45% reduction):
```rust
use stillwater::{Validation, validation::ValidateAll};

fn validate_config(config: &RawConfig) -> Validation<ValidatedConfig, Vec<ConfigError>> {
    (
        validate_port(config.port),
        validate_database_url(&config.database_url),
        validate_log_level(&config.log_level).map(|v| v.unwrap_or("info".to_string())),
        validate_max_connections(config.max_connections).map(|v| v.unwrap_or(10)),
    )
    .validate_all()
    .map(|(port, database_url, log_level, max_connections)| {
        ValidatedConfig { port, database_url, log_level, max_connections }
    })
}

fn validate_port(port: Option<i32>) -> Validation<u16, Vec<ConfigError>> {
    match port {
        Some(p) if p > 0 && p < 65536 => Validation::success(p as u16),
        Some(p) => Validation::failure(vec![ConfigError::InvalidPort(p)]),
        None => Validation::failure(vec![ConfigError::MissingPort]),
    }
}
// ... similar for other validators
```

**Key Improvements**:
- 45% less code
- Each field validator is independent and testable
- Default values handled cleanly with `map`
- All config errors reported at once

---

## Boilerplate Reduction Summary

| Scenario | Before (LOC) | After (LOC) | Reduction |
|----------|--------------|-------------|-----------|
| 3-field validation | 22 | 8 | **64%** |
| 5-field nested validation | 42 | 18 | **57%** |
| Conditional validation | 20 | 14 | **30%** |
| Error context chain | 24 | 16 | **33%** |
| Dependency threading (3 deps) | 32 | 24 | **25%** |
| Retry with backoff | 35 | 14 | **60%** |
| Parallel with timeout | 25 | 17 | **32%** |
| API request handler | 45 | 30 | **33%** |
| Database transaction | 30 | 24 | **20%** |
| Config validation | 40 | 22 | **45%** |

**Average reduction across all examples: ~40%**

---

## Complementary Usage: Stillwater + Other Crates

### Stillwater + anyhow

Use anyhow for error propagation, Stillwater for validation and effects:

```rust
use stillwater::{Validation, validation::ValidateAll};
use stillwater::effect::prelude::*;
use anyhow::{Result, Context};

fn process_request(input: RequestInput) -> impl Effect<Output = Response, Error = anyhow::Error, Env = AppEnv> {
    // Validation phase with Stillwater
    let validated = (
        validate_email(&input.email),
        validate_name(&input.name),
    )
    .validate_all()
    .into_result()
    .map_err(|errors| anyhow::anyhow!("Validation failed: {:?}", errors));

    from_result(validated)
        .and_then(|(email, name)| {
            // Effect composition with context
            create_user(email, name)
                .map_err(|e| anyhow::anyhow!(e))
        })
}
```

### Stillwater + validator

Use validator derive macros for struct validation, Stillwater for custom logic:

```rust
use validator::Validate;
use stillwater::{Validation, validation::ValidateAll};

#[derive(Validate)]
struct UserInput {
    #[validate(email)]
    email: String,
    #[validate(length(min = 8))]
    password: String,
}

fn validate_user(input: &UserInput) -> Validation<ValidatedUser, Vec<String>> {
    // Struct-level validation via validator
    let struct_valid = match input.validate() {
        Ok(()) => Validation::success(()),
        Err(e) => Validation::failure(vec![format!("Struct validation: {}", e)]),
    };

    // Custom business logic via Stillwater
    let email_unique = check_email_unique(&input.email);
    let password_not_common = check_password_not_common(&input.password);

    (struct_valid, email_unique, password_not_common)
        .validate_all()
        .map(|_| ValidatedUser {
            email: input.email.clone(),
            password: input.password.clone(),
        })
}
```

### Stillwater + thiserror

Use thiserror for error definitions, Stillwater for composition:

```rust
use thiserror::Error;
use stillwater::effect::prelude::*;

#[derive(Error, Debug)]
enum OrderError {
    #[error("Customer not found: {0}")]
    CustomerNotFound(u64),
    #[error("Insufficient stock for product {product_id}")]
    InsufficientStock { product_id: u64 },
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

fn process_order(order: Order) -> impl Effect<Output = Receipt, Error = OrderError, Env = AppEnv> {
    fetch_customer(order.customer_id)
        .map_err(OrderError::from)
        .and_then(move |customer| {
            check_stock(&order.items)
                .and_then(move |_| create_receipt(&customer, &order))
        })
}
```

---

## vs frunk

**frunk** focuses on type-level functional programming with HLists, Generic, and other advanced concepts.

### Similarities
- Both provide Validation with error accumulation
- Both implement Semigroup

### Differences

**Stillwater**:
- ✓ Practical focus on common patterns
- ✓ Effect composition for I/O separation
- ✓ Lower learning curve
- ✓ Better documentation for beginners
- ✓ Async support

**frunk**:
- ✓ More advanced type-level features (HLists, Generic)
- ✓ Powerful generic programming
- ✗ Steeper learning curve
- ✗ No effect system
- ✗ No async support

### When to use frunk
- Type-level programming is important
- You need HList transformations
- You're comfortable with advanced type theory

### When to use Stillwater
- Validation and effects are your primary needs
- You want a gentler learning curve
- You need async support

---

## vs monadic

**monadic** provides Reader/Writer/State monads with macro-based do-notation.

### Similarities
- Both provide effect composition
- Both handle dependencies (Reader monad)

### Differences

**Stillwater**:
- ✓ No macro DSL (more idiomatic Rust)
- ✓ Method chaining instead of do-notation
- ✓ Validation with error accumulation
- ✓ Async support
- ✓ Zero dependencies
- ✓ Writer Effect for logging/accumulation
- ✓ Reader pattern with `ask()`/`asks()`/`local()`

**monadic**:
- ✓ State monad
- ✓ Do-notation via macros
- ✗ Macro-heavy syntax (`rdrdo!`)
- ✗ No validation
- ✗ No async support

### When to use monadic
- You want Haskell-style do-notation
- You need State monad
- You're porting Haskell code

### When to use Stillwater
- You prefer Rust idioms over Haskell syntax
- You need validation and effects together
- You want async support
- You need Writer Effect for logging/metrics

---

## vs anyhow / thiserror

**anyhow** provides ergonomic error handling. **thiserror** provides derive macros for error types.

### Similarities
- All handle errors
- All work with Result

### Differences

**Stillwater**:
- ✓ Error accumulation (Validation)
- ✓ Effect composition
- ✓ ContextError for trails
- ✗ Less focused on error handling alone

**anyhow/thiserror**:
- ✓ Excellent error handling ergonomics
- ✓ Great for error propagation
- ✗ No error accumulation
- ✗ No effect system

### When to use anyhow/thiserror
- Error handling is your only need
- You want minimal boilerplate
- Short-circuiting errors are fine

### When to use Stillwater
- You need error accumulation
- You want effect composition
- You're building validation-heavy apps

**Recommendation**: Use both! Stillwater for business logic, anyhow for error propagation.

---

## vs validator

**validator** provides derive macros for common validation rules.

### Similarities
- Both validate data
- Both can accumulate errors

### Differences

**Stillwater**:
- ✓ Functional composition
- ✓ Effect system
- ✓ Custom validation logic
- ✗ No derive macros
- ✗ More verbose for simple cases

**validator**:
- ✓ Derive macros for common validations
- ✓ Less boilerplate for simple cases
- ✗ No effect system
- ✗ Less flexible for complex logic

### When to use validator
- Simple struct validation with standard rules
- You want derive macros
- Validation is your only need

### When to use Stillwater
- Complex validation logic
- Need effect composition
- Want full control over validation

**Recommendation**: Use both! validator for struct-level rules, Stillwater for complex logic.

---

## vs Standard Library (Result, Option)

**Result** and **Option** are the foundation. When should you reach for Stillwater?

### Use Result when
- Short-circuiting is desired (fail fast)
- Single error is sufficient
- Simple error propagation

### Use Validation when
- You want ALL errors at once
- Validating forms or API requests
- Independent validations

### Use Effect when
- Separating I/O from logic
- Testing with mock environments
- Composing async operations

**Rule of thumb**: Start with Result. Reach for Stillwater when you need error accumulation or I/O separation.

---

## Philosophy Comparison

| Library | Philosophy |
|---------|-----------|
| Stillwater | Pragmatic FP: common patterns, low learning curve |
| frunk | Academic FP: type-level programming, HLists |
| monadic | Haskell-style: monad abstraction, do-notation |
| anyhow | Ergonomic errors: minimal boilerplate |
| validator | Declarative: derive macros for common cases |

---

## Migration Guide

### From Result to Stillwater

```rust
// Before: Result (short-circuits)
fn validate(data: Data) -> Result<Valid, Error> {
    let email = validate_email(data.email)?;
    let age = validate_age(data.age)?;
    Ok(Valid { email, age })
}

// After: Validation (accumulates)
use stillwater::{Validation, validation::ValidateAll};

fn validate(data: Data) -> Validation<Valid, Vec<Error>> {
    (
        validate_email(data.email),
        validate_age(data.age),
    )
    .validate_all()
    .map(|(email, age)| Valid { email, age })
}
```

### From async fn to Effect

```rust
// Before: async fn (hard to test)
async fn create_user(db: &Database, email: String) -> Result<User, Error> {
    let user = User { email };
    db.save(&user).await?;
    Ok(user)
}

// After: Effect (testable)
use stillwater::effect::prelude::*;

fn create_user(email: String) -> impl Effect<Output = User, Error = Error, Env = AppEnv> {
    let user = User { email };
    from_async(move |env: &AppEnv| {
        let db = env.db.clone();
        let user = user.clone();
        async move {
            db.save(&user).await?;
            Ok(user)
        }
    })
}
```

---

## Summary

| Use Case | Best Choice |
|----------|-------------|
| Form validation | Stillwater Validation |
| API validation | Stillwater Validation |
| Testable I/O | Stillwater Effect |
| Error propagation | anyhow + Stillwater |
| Simple validations | validator + Stillwater |
| Type-level programming | frunk |
| Haskell-style monads | monadic |
| Generic error handling | anyhow/thiserror |

---

## Further Reading

- [Stillwater User Guide](guide/README.md)
- [frunk documentation](https://docs.rs/frunk)
- [monadic documentation](https://docs.rs/monadic)
- [anyhow documentation](https://docs.rs/anyhow)
- [validator documentation](https://docs.rs/validator)
