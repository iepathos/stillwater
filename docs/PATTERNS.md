# Common Patterns and Recipes

This document collects common patterns and recipes for using Stillwater effectively.

## Validation Patterns

### Pattern 1: Independent Field Validation

When validating multiple independent fields:

```rust
use stillwater::Validation;

fn validate_user_registration(input: UserInput) -> Validation<User, Vec<Error>> {
    Validation::all((
        validate_email(&input.email),
        validate_password(&input.password),
        validate_age(input.age),
        validate_username(&input.username),
    ))
    .map(|(email, password, age, username)| {
        User { email, password, age, username }
    })
}
```

### Pattern 2: Dependent Validation

When one validation depends on another's result:

```rust
use stillwater::Validation;

fn validate_and_check_unique(email: &str) -> Validation<Email, Vec<Error>> {
    validate_email_format(email)
        .and_then(|email| check_email_not_taken(email))
}
```

### Pattern 3: Validating Collections

Validate all items in a collection:

```rust
use stillwater::Validation;

fn validate_all(items: Vec<Item>) -> Validation<Vec<ValidItem>, Vec<Error>> {
    let validations: Vec<_> = items
        .into_iter()
        .map(|item| validate_item(item))
        .collect();

    Validation::all_vec(validations)
}
```

### Pattern 4: Conditional Validation

Validate different fields based on conditions:

```rust
use stillwater::Validation;

fn validate_payment(method: PaymentMethod, data: PaymentData) -> Validation<Payment, Vec<Error>> {
    match method {
        PaymentMethod::CreditCard => {
            Validation::all((
                validate_card_number(&data.card_number),
                validate_cvv(&data.cvv),
                validate_expiry(&data.expiry),
            ))
            .map(|(card, cvv, expiry)| Payment::CreditCard { card, cvv, expiry })
        }
        PaymentMethod::BankTransfer => {
            Validation::all((
                validate_account_number(&data.account),
                validate_routing_number(&data.routing),
            ))
            .map(|(account, routing)| Payment::BankTransfer { account, routing })
        }
    }
}
```

## Effect Patterns

### Pattern 1: Read, Transform, Write

Classic pattern for processing data:

```rust
use stillwater::{Effect, IO};

fn process_user_data(id: u64) -> Effect<(), Error, Env> {
    IO::read(|env: &Env| env.db.fetch_user(id))
        .map(|user| transform_user_data(user))  // Pure transformation
        .and_then(|transformed| {
            IO::execute(|env: &Env| env.db.save_user(&transformed))
        })
}
```

### Pattern 2: Validate Then Execute

Validate input, then perform I/O if valid:

```rust
use stillwater::{Effect, Validation};

fn create_user(input: UserInput) -> Effect<User, Error, Env> {
    Effect::from_validation(validate_user(input))
        .and_then(|valid| {
            IO::execute(|env: &Env| env.db.insert_user(&valid))
        })
}
```

### Pattern 3: Try Cache, Fall Back to DB

Common caching pattern:

```rust
use stillwater::{Effect, IO};

fn get_user(id: u64) -> Effect<User, Error, Env> {
    IO::read(|env: &Env| env.cache.get_user(id))
        .and_then(|cached| {
            match cached {
                Some(user) => Effect::pure(user),
                None => {
                    IO::read(|env: &Env| env.db.fetch_user(id))
                        .and_then(|user| {
                            IO::write(|env: &Env| env.cache.set_user(id, user.clone()))
                                .map(|_| user)
                        })
                }
            }
        })
}
```

### Pattern 4: Sequential Operations with Context

Add context at each step:

```rust
use stillwater::Effect;

fn process_order(id: u64) -> Effect<Receipt, Error, Env> {
    fetch_order(id)
        .context("fetching order")
        .and_then(|order| {
            validate_order(&order)
                .context("validating order")
        })
        .and_then(|order| {
            charge_payment(&order)
                .context("processing payment")
        })
        .and_then(|charge| {
            generate_receipt(charge)
                .context("generating receipt")
        })
}
```

### Pattern 5: Parallel Operations (using tokio)

When effects are independent:

```rust
use stillwater::Effect;
use tokio;

async fn load_dashboard(user_id: u64, env: &Env) -> Result<Dashboard, Error> {
    let (user, projects, notifications) = tokio::try_join!(
        fetch_user(user_id).run(env),
        fetch_projects(user_id).run(env),
        fetch_notifications(user_id).run(env),
    )?;

    Ok(Dashboard { user, projects, notifications })
}
```

### Pattern 6: Combining Independent Effects with Zip

Use `zip` when you need both results from independent effects:

```rust
use stillwater::prelude::*;

// Basic zip: combine two independent effects into a tuple
fn load_user_with_settings(id: UserId) -> impl Effect<Output = (User, Settings), Error = AppError, Env = AppEnv> {
    fetch_user(id).zip(fetch_settings(id))
}

// zip_with: combine with a function directly (more efficient than zip + map)
fn calculate_total(order_id: OrderId) -> impl Effect<Output = Money, Error = AppError, Env = AppEnv> {
    fetch_price(order_id)
        .zip_with(fetch_quantity(order_id), |price, qty| price * qty)
}

// zip3 through zip8: flat tuple results for multiple effects
fn load_profile(id: UserId) -> impl Effect<Output = Profile, Error = AppError, Env = AppEnv> {
    zip3(
        fetch_user(id),
        fetch_settings(id),
        fetch_preferences(id),
    )
    .map(|(user, settings, prefs)| Profile { user, settings, prefs })
}

// Chained zips create nested tuples
fn chained_example() -> impl Effect<Output = i32, Error = String, Env = ()> {
    pure(1)
        .zip(pure(2))
        .zip(pure(3))
        .map(|((a, b), c)| a + b + c)  // Note the nested tuple
}
```

**Key points:**
- `zip` expresses independence - neither effect depends on the other's output
- Uses fail-fast semantics (first error wins), same as `and_then`
- For error accumulation with independent operations, use `Validation::all()` instead
- `zip3` through `zip8` return flat tuples for cleaner pattern matching

## Testing Patterns

### Pattern 1: Testing Pure Functions

Pure functions need no mocking:

```rust
#[test]
fn test_pure_validation() {
    let result = validate_email("user@example.com");
    assert!(result.is_success());

    let result = validate_email("invalid");
    assert!(result.is_failure());
}
```

### Pattern 2: Testing Effects with Mock Environment

```rust
struct MockEnv {
    users: HashMap<u64, User>,
}

impl MockEnv {
    fn fetch_user(&self, id: u64) -> Result<User, Error> {
        self.users.get(&id).cloned().ok_or(Error::NotFound)
    }
}

#[tokio::test]
async fn test_user_workflow() {
    let mut env = MockEnv {
        users: HashMap::new(),
    };
    env.users.insert(1, User { name: "Alice".into() });

    let effect = IO::read(|env: &MockEnv| env.fetch_user(1));
    let result = effect.run(&env).await;

    assert!(result.is_ok());
}
```

### Pattern 3: Testing Error Cases

```rust
#[tokio::test]
async fn test_user_not_found() {
    let env = MockEnv {
        users: HashMap::new(),  // Empty
    };

    let effect = IO::read(|env: &MockEnv| env.fetch_user(999));
    let result = effect.run(&env).await;

    assert_eq!(result, Err(Error::NotFound));
}
```

## Error Handling Patterns

### Pattern 1: Domain-Specific Errors

```rust
#[derive(Debug)]
enum UserError {
    NotFound(u64),
    InvalidEmail(String),
    PermissionDenied,
}

impl std::fmt::Display for UserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserError::NotFound(id) => write!(f, "User {} not found", id),
            UserError::InvalidEmail(email) => write!(f, "Invalid email: {}", email),
            UserError::PermissionDenied => write!(f, "Permission denied"),
        }
    }
}

impl std::error::Error for UserError {}
```

### Pattern 2: Error Conversion

```rust
fn fetch_user(id: u64) -> Effect<User, AppError, Env> {
    IO::read(|env: &Env| {
        env.db.fetch_user(id)
            .map_err(|e| AppError::Database(e.to_string()))
    })
}
```

### Pattern 3: Error Context Trails

```rust
fn complex_operation() -> Effect<Result, ContextError<Error>, Env> {
    step1()
        .context("performing step 1")
        .and_then(|r1| {
            step2(r1).context("performing step 2")
        })
        .and_then(|r2| {
            step3(r2).context("performing step 3")
        })
        .context("complex operation")
}
```

## Composition Patterns

### Pattern 1: Building Complex Validations

```rust
fn validate_address(addr: &Address) -> Validation<ValidAddress, Vec<Error>> {
    Validation::all((
        validate_street(&addr.street),
        validate_city(&addr.city),
        validate_zip(&addr.zip),
        validate_country(&addr.country),
    ))
    .map(|(street, city, zip, country)| {
        ValidAddress { street, city, zip, country }
    })
}

fn validate_contact(contact: &Contact) -> Validation<ValidContact, Vec<Error>> {
    Validation::all((
        validate_email(&contact.email),
        validate_phone(&contact.phone),
        validate_address(&contact.address),
    ))
    .map(|(email, phone, address)| {
        ValidContact { email, phone, address }
    })
}
```

### Pattern 2: Effect Pipelines

```rust
fn user_registration_pipeline(input: UserInput) -> Effect<User, Error, Env> {
    validate_input(input)
        .and_then(|valid| check_uniqueness(valid))
        .and_then(|valid| create_user(valid))
        .and_then(|user| send_welcome_email(user))
        .and_then(|user| create_default_settings(user))
        .context("user registration")
}
```

## Resource Management Patterns

The bracket pattern ensures resources are properly released even when errors occur.

### Pattern 1: Single Resource with Guaranteed Cleanup

```rust
use stillwater::effect::bracket::bracket;
use stillwater::prelude::*;

fn with_database_connection<T>(
    f: impl FnOnce(&Connection) -> impl Effect<Output = T, Error = AppError, Env = AppEnv>
) -> impl Effect<Output = T, Error = AppError, Env = AppEnv> {
    bracket(
        from_fn(|env: &AppEnv| env.pool.get_connection()),  // Acquire
        |conn| async move { conn.release().await },          // Release (always runs)
        f,                                                   // Use
    )
}

// Usage
let result = with_database_connection(|conn| {
    from_fn(move |_| conn.query("SELECT * FROM users"))
}).run(&env).await;
```

### Pattern 2: Multiple Resources with LIFO Cleanup

Resources are released in reverse order of acquisition (Last In, First Out):

```rust
use stillwater::effect::bracket::bracket2;

fn with_db_and_file(
    path: &Path,
) -> impl Effect<Output = Data, Error = AppError, Env = AppEnv> {
    bracket2(
        open_database(),                                    // Acquired first
        open_file(path),                                    // Acquired second
        |db| async move { db.close().await },               // Released second
        |file| async move { file.close().await },           // Released first (LIFO)
        |db, file| process_data(db, file),
    )
}
```

### Pattern 3: Fluent Builder for Multiple Resources

The `acquiring` builder provides ergonomic multi-resource management:

```rust
use stillwater::effect::bracket::acquiring;

fn complex_operation() -> impl Effect<Output = Result, Error = AppError, Env = AppEnv> {
    acquiring(
        open_connection(),
        |conn| async move { conn.close().await },
    )
    .and(acquire_lock(), |lock| async move { lock.release().await })
    .and(open_file(), |file| async move { file.close().await })
    .with_flat3(|conn, lock, file| {
        // All three resources available here
        // Cleanup happens in reverse order: file, lock, conn
        do_work(conn, lock, file)
    })
}
```

### Pattern 4: Explicit Error Handling with BracketError

When you need to distinguish between use errors and cleanup errors:

```rust
use stillwater::effect::bracket::{bracket_full, BracketError};

fn with_explicit_errors() -> impl Effect<Output = Data, Error = BracketError<AppError>, Env = AppEnv> {
    bracket_full(
        acquire_resource(),
        |r| async move { r.cleanup().await },
        |r| use_resource(r),
    )
}

// Handle all error cases explicitly
let result = with_explicit_errors().run(&env).await;
match result {
    Ok(data) => println!("Success: {:?}", data),
    Err(BracketError::AcquireError(e)) => {
        // Resource was never acquired, no cleanup needed
        log::error!("Failed to acquire: {:?}", e);
    }
    Err(BracketError::UseError(e)) => {
        // Use failed, but cleanup succeeded
        log::error!("Operation failed: {:?}", e);
    }
    Err(BracketError::CleanupError(e)) => {
        // Use succeeded, but cleanup failed - may need manual intervention
        log::warn!("Cleanup failed: {:?}", e);
    }
    Err(BracketError::Both { use_error, cleanup_error }) => {
        // Both failed - log both for debugging
        log::error!("Use failed: {:?}, cleanup also failed: {:?}", use_error, cleanup_error);
    }
}
```

### Pattern 5: Partial Acquisition Rollback

When acquiring multiple resources, earlier acquisitions are rolled back if later ones fail:

```rust
use stillwater::effect::bracket::acquiring;

// If file acquisition fails, connection is automatically released
let effect = acquiring(
    open_connection(),  // Succeeds
    |c| async move { c.close().await },
)
.and(
    open_file(path),    // Fails!
    |f| async move { f.close().await },
)
.with(|(conn, file)| use_both(conn, file));

// Connection is properly cleaned up even though file failed
let result = effect.run(&env).await;  // Returns file acquisition error
```

### Pattern 6: Connection Pool Pattern

Encapsulate resource management in reusable abstractions:

```rust
use stillwater::effect::bracket::Resource;

struct ConnectionPool {
    // ... pool internals
}

impl ConnectionPool {
    fn connection(&self) -> Resource<Connection, PoolError, AppEnv> {
        Resource::new(
            from_fn(|env: &AppEnv| env.pool.checkout()),
            |conn| async move { conn.checkin().await },
        )
    }
}

// Usage - cleanup is automatic
let pool = ConnectionPool::new();
let result = pool.connection()
    .with(|conn| from_fn(move |_| conn.query("SELECT 1")))
    .run(&env)
    .await;
```

## Performance Patterns

### Pattern 1: Avoid Excessive Boxing

```rust
// Instead of creating many small effects:
let effect = Effect::pure(1)
    .map(|x| x + 1)
    .map(|x| x * 2)
    .map(|x| x - 3);

// Combine transformations:
let effect = Effect::pure(1)
    .map(|x| (x + 1) * 2 - 3);
```

### Pattern 2: Batch Operations

```rust
// Instead of many individual queries:
for id in ids {
    fetch_user(id).run(&env).await?;
}

// Batch fetch:
let users = fetch_users_batch(ids).run(&env).await?;
```

## Summary

These patterns cover common use cases. Mix and match them based on your needs!

## Next Steps

- See [FAQ](FAQ.md) for common questions
- Read [Comparison](COMPARISON.md) vs other libraries
- Check [examples/](../examples/) for working code
