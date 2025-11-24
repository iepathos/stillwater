# Reader Pattern in Stillwater

## What is the Reader Pattern?

The Reader pattern is a functional programming technique for dependency injection. Instead of passing dependencies through every function parameter, you "ask" for them from an environment when needed.

In Stillwater, the Reader pattern is built into the `Effect` type through three key functions:
- `ask()` - Get the entire environment
- `asks(f)` - Extract a specific value from the environment
- `local(f, effect)` - Run an effect with a modified environment

## Why Use the Reader Pattern?

### Problem: Dependency Threading

Without Reader, you pass dependencies everywhere:

```rust
fn process_order(order: Order, db: &Database, cache: &Cache, logger: &Logger) -> Result<()> {
    validate_order(order, db)?;
    save_order(order, db, cache, logger)?;
    notify_customer(order, logger)?;
    Ok(())
}

fn validate_order(order: Order, db: &Database) -> Result<()> {
    // Only needs db, but caller must provide it
}

fn save_order(order: Order, db: &Database, cache: &Cache, logger: &Logger) -> Result<()> {
    // All three dependencies needed
}
```

This gets tedious and error-prone as the application grows.

### Solution: Reader Pattern

With Reader, dependencies live in an environment:

```rust
use stillwater::{Effect, IO};

struct AppEnv {
    db: Database,
    cache: Cache,
    logger: Logger,
}

fn process_order(order: Order) -> Effect<(), Error, AppEnv> {
    validate_order(order.clone())
        .and_then(|_| save_order(order.clone()))
        .and_then(|_| notify_customer(order))
}

fn validate_order(order: Order) -> Effect<(), Error, AppEnv> {
    // Ask for just the database when needed
    IO::read(|env: &AppEnv| env.db.validate(&order))
}

fn save_order(order: Order) -> Effect<(), Error, AppEnv> {
    // Functions get dependencies implicitly
    IO::read(|env: &AppEnv| env.db.save(&order))
        .and_then(|_| IO::write(|env: &mut AppEnv| {
            env.cache.invalidate(&order.id)
        }))
}
```

Benefits:
- No threading dependencies through parameters
- Easy to add new dependencies without changing function signatures
- Environment is explicit at type level (`AppEnv`)
- Testing is easier with mock environments

## Core Functions

### `ask()` - Access the Whole Environment

Use `ask()` when you need the entire environment:

```rust
use stillwater::Effect;

#[derive(Clone)]
struct Config {
    api_key: String,
    timeout: u64,
    debug: bool,
}

fn log_config() -> Effect<String, String, Config> {
    Effect::ask()
        .map(|cfg: Config| {
            format!(
                "Config: timeout={}, debug={}",
                cfg.timeout,
                cfg.debug
            )
        })
}

// Run it
# tokio_test::block_on(async {
let config = Config {
    api_key: "secret".into(),
    timeout: 30,
    debug: true,
};

let result = log_config().run(&config).await.unwrap();
assert_eq!(result, "Config: timeout=30, debug=true");
# });
```

### `asks()` - Query Specific Values

Use `asks(f)` when you only need part of the environment:

```rust
use stillwater::Effect;

struct AppEnv {
    database_url: String,
    cache_url: String,
    max_connections: u32,
}

fn get_db_url() -> Effect<String, String, AppEnv> {
    Effect::asks(|env: &AppEnv| env.database_url.clone())
}

fn get_max_connections() -> Effect<u32, String, AppEnv> {
    Effect::asks(|env: &AppEnv| env.max_connections)
}

// Compose them
fn connect() -> Effect<String, String, AppEnv> {
    Effect::asks(|env: &AppEnv| env.database_url.clone())
        .and_then(|url| {
            Effect::asks(|env: &AppEnv| env.max_connections)
                .map(move |max| {
                    format!("Connecting to {} with {} connections", url, max)
                })
        })
}

# tokio_test::block_on(async {
let env = AppEnv {
    database_url: "postgres://localhost".into(),
    cache_url: "redis://localhost".into(),
    max_connections: 10,
};

let result = connect().run(&env).await.unwrap();
assert_eq!(result, "Connecting to postgres://localhost with 10 connections");
# });
```

### `local()` - Temporary Environment Modifications

Use `local(f, effect)` to run an effect with a modified environment:

```rust
use stillwater::Effect;

#[derive(Clone)]
struct Config {
    timeout: u64,
    retries: u32,
}

fn fetch_data() -> Effect<String, String, Config> {
    Effect::asks(|cfg: &Config| {
        format!("Fetching with timeout={}, retries={}", cfg.timeout, cfg.retries)
    })
}

fn fetch_with_extended_timeout() -> Effect<String, String, Config> {
    // Temporarily increase timeout for this operation
    Effect::local(
        |cfg: &Config| Config {
            timeout: cfg.timeout * 2,
            retries: cfg.retries,
        },
        fetch_data()
    )
}

# tokio_test::block_on(async {
let config = Config {
    timeout: 30,
    retries: 3,
};

// Normal fetch
let result = fetch_data().run(&config).await.unwrap();
assert_eq!(result, "Fetching with timeout=30, retries=3");

// With extended timeout
let result = fetch_with_extended_timeout().run(&config).await.unwrap();
assert_eq!(result, "Fetching with timeout=60, retries=3");

// Original config unchanged
assert_eq!(config.timeout, 30);
# });
```

## Composition Patterns

### Pattern 1: Combining asks() with Business Logic

```rust
use stillwater::Effect;

struct PricingEnv {
    tax_rate: f64,
    discount_rate: f64,
}

fn calculate_price(base_price: f64) -> Effect<f64, String, PricingEnv> {
    Effect::asks(|env: &PricingEnv| env.tax_rate)
        .and_then(move |tax| {
            Effect::asks(|env: &PricingEnv| env.discount_rate)
                .map(move |discount| {
                    let discounted = base_price * (1.0 - discount);
                    discounted * (1.0 + tax)
                })
        })
}

# tokio_test::block_on(async {
let env = PricingEnv {
    tax_rate: 0.08,
    discount_rate: 0.10,
};

let final_price = calculate_price(100.0).run(&env).await.unwrap();
assert_eq!(final_price, 97.2); // (100 * 0.9) * 1.08
# });
```

### Pattern 2: Environment-Dependent Decisions

```rust
use stillwater::{Effect, IO};

struct AppEnv {
    debug_mode: bool,
    log_level: String,
}

fn log_message(msg: String) -> Effect<(), String, AppEnv> {
    Effect::asks(|env: &AppEnv| env.debug_mode)
        .and_then(move |debug| {
            if debug {
                Effect::from_fn(move |env: &AppEnv| {
                    println!("[{}] {}", env.log_level, msg);
                    Ok(())
                })
            } else {
                Effect::pure(())
            }
        })
}
```

### Pattern 3: Nested Environments with local()

```rust
use stillwater::Effect;

#[derive(Clone)]
struct ServerConfig {
    host: String,
    port: u16,
    timeout: u64,
}

fn make_request(path: &str) -> Effect<String, String, ServerConfig> {
    Effect::asks(move |cfg: &ServerConfig| {
        format!("GET {}:{}{} (timeout={})", cfg.host, cfg.port, path, cfg.timeout)
    })
}

fn make_critical_request(path: &str) -> Effect<String, String, ServerConfig> {
    // Critical requests get longer timeout
    let path = path.to_string();
    Effect::local(
        |cfg: &ServerConfig| ServerConfig {
            timeout: cfg.timeout * 3,
            ..cfg.clone()
        },
        Effect::asks(move |cfg: &ServerConfig| {
            format!("GET {}:{}{} (timeout={})", cfg.host, cfg.port, path, cfg.timeout)
        })
    )
}

# tokio_test::block_on(async {
let config = ServerConfig {
    host: "api.example.com".into(),
    port: 443,
    timeout: 10,
};

let normal = make_request("/users").run(&config).await.unwrap();
assert_eq!(normal, "GET api.example.com:443/users (timeout=10)");

let critical = make_critical_request("/payment").run(&config).await.unwrap();
assert_eq!(critical, "GET api.example.com:443/payment (timeout=30)");
# });
```

## Real-World Example: Multi-Tier Application

```rust
use stillwater::{Effect, IO};

// Environment with multiple dependencies
struct AppEnv {
    database: Database,
    cache: Cache,
    email: EmailService,
    config: AppConfig,
}

struct AppConfig {
    max_retries: u32,
    cache_ttl: u64,
}

#[derive(Clone)]
struct User {
    id: u64,
    email: String,
}

// Business logic uses Reader pattern
fn register_user(email: String) -> Effect<User, AppError, AppEnv> {
    // Validate email format (pure)
    validate_email(&email)?

    // Check if user exists (asks for database)
    check_user_exists(email.clone())
        .and_then(|exists| {
            if exists {
                Effect::fail(AppError::UserExists)
            } else {
                create_and_save_user(email)
            }
        })
}

fn check_user_exists(email: String) -> Effect<bool, AppError, AppEnv> {
    IO::read(move |env: &AppEnv| {
        env.database.find_by_email(&email)
            .map(|opt| opt.is_some())
            .map_err(AppError::DatabaseError)
    })
}

fn create_and_save_user(email: String) -> Effect<User, AppError, AppEnv> {
    let user = User {
        id: generate_id(),
        email: email.clone(),
    };

    // Save to database
    save_user_to_db(user.clone())
        // Update cache
        .and_then(|user| cache_user(user))
        // Send welcome email
        .and_then(|user| {
            send_welcome_email(user.clone())
                .map(|_| user)
        })
}

fn save_user_to_db(user: User) -> Effect<User, AppError, AppEnv> {
    IO::execute(move |env: &AppEnv| {
        env.database.insert_user(&user)
            .map(|_| user.clone())
            .map_err(AppError::DatabaseError)
    })
}

fn cache_user(user: User) -> Effect<User, AppError, AppEnv> {
    // Get cache TTL from config
    Effect::asks(|env: &AppEnv| env.config.cache_ttl)
        .and_then(move |ttl| {
            IO::execute(move |env: &AppEnv| {
                env.cache.set(&user.id, &user, ttl)
                    .map(|_| user.clone())
                    .map_err(AppError::CacheError)
            })
        })
}

fn send_welcome_email(user: User) -> Effect<(), AppError, AppEnv> {
    IO::execute(move |env: &AppEnv| {
        env.email.send(&user.email, "Welcome!", "Thanks for joining!")
            .map_err(AppError::EmailError)
    })
}

// At application boundary
# tokio_test::block_on(async {
let env = AppEnv {
    database: Database::connect("postgres://...").await?,
    cache: Cache::connect("redis://...").await?,
    email: EmailService::new("smtp://..."),
    config: AppConfig {
        max_retries: 3,
        cache_ttl: 3600,
    },
};

match register_user("user@example.com".into()).run(&env).await {
    Ok(user) => println!("User registered: {:?}", user),
    Err(e) => eprintln!("Registration failed: {:?}", e),
}
# });
```

## Testing with Reader Pattern

The Reader pattern makes testing easier with mock environments:

```rust
use stillwater::Effect;

struct AppEnv {
    database: Box<dyn UserRepository>,
}

trait UserRepository {
    fn find_by_email(&self, email: &str) -> Result<Option<User>, String>;
}

fn get_user_by_email(email: String) -> Effect<Option<User>, String, AppEnv> {
    IO::read(move |env: &AppEnv| {
        env.database.find_by_email(&email)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockDatabase {
        users: Vec<User>,
    }

    impl UserRepository for MockDatabase {
        fn find_by_email(&self, email: &str) -> Result<Option<User>, String> {
            Ok(self.users.iter()
                .find(|u| u.email == email)
                .cloned())
        }
    }

    #[tokio::test]
    async fn test_get_user() {
        let mock_db = MockDatabase {
            users: vec![
                User { id: 1, email: "test@example.com".into() },
            ],
        };

        let env = AppEnv {
            database: Box::new(mock_db),
        };

        let result = get_user_by_email("test@example.com".into())
            .run(&env)
            .await
            .unwrap();

        assert!(result.is_some());
        assert_eq!(result.unwrap().id, 1);
    }
}
```

## When to Use Each Function

### Use `ask()` when:
- You need the whole environment
- Passing the environment to another function
- Inspecting multiple environment fields at once

### Use `asks(f)` when:
- You only need one or two fields
- Extracting configuration values
- Computing derived values from environment

### Use `local(f, effect)` when:
- Temporarily overriding configuration
- Testing with modified settings
- Scoping changes to specific operations
- Implementing feature flags or A/B tests

## Best Practices

### 1. Keep Environments Small and Focused

```rust
// ❌ Bad: Kitchen sink environment
struct AppEnv {
    db: Database,
    cache: Cache,
    email: Email,
    sms: SMS,
    payment: Payment,
    analytics: Analytics,
    // ... 20 more dependencies
}

// ✓ Good: Focused environments
struct DataEnv {
    db: Database,
    cache: Cache,
}

struct NotificationEnv {
    email: Email,
    sms: SMS,
}

struct PaymentEnv {
    payment: Payment,
    analytics: Analytics,
}
```

### 2. Use asks() for Simple Queries

```rust
// ❌ Verbose
Effect::ask().map(|env: Config| env.timeout)

// ✓ Concise
Effect::asks(|env: &Config| env.timeout)
```

### 3. Compose with and_then for Dependent Operations

```rust
fn process() -> Effect<Result, Error, AppEnv> {
    Effect::asks(|env: &AppEnv| env.config.max_retries)
        .and_then(|retries| {
            Effect::asks(|env: &AppEnv| env.database.clone())
                .and_then(move |db| {
                    retry_with_db(db, retries)
                })
        })
}
```

### 4. Use local() Sparingly

Only use `local()` when you truly need to modify the environment temporarily. Most of the time, `asks()` is sufficient.

## Common Pitfalls

### Pitfall 1: Cloning Large Environments

```rust
// ❌ ask() clones the entire environment
let effect = Effect::<LargeEnv, _, LargeEnv>::ask();

// ✓ asks() only extracts what you need
let effect = Effect::asks(|env: &LargeEnv| env.small_field.clone());
```

### Pitfall 2: Nested local() Calls

```rust
// ❌ Hard to follow
Effect::local(
    |cfg| modify1(cfg),
    Effect::local(
        |cfg| modify2(cfg),
        Effect::local(
            |cfg| modify3(cfg),
            do_work()
        )
    )
)

// ✓ Compose modifications
Effect::local(
    |cfg| modify3(&modify2(&modify1(cfg))),
    do_work()
)
```

## Summary

The Reader pattern in Stillwater provides:
- Clean dependency injection without parameter threading
- Type-safe environment access
- Easy testing with mock environments
- Functional composition of environment-dependent operations

Key functions:
- `ask()` - Get the whole environment
- `asks(f)` - Query specific values
- `local(f, effect)` - Temporary modifications

Use the Reader pattern when you want clean, testable dependency injection in your Effect compositions.

## Next Steps

- Review the [Effects guide](03-effects.md) for more Effect patterns
- Check out the [IO Module](05-io-module.md) for I/O helpers
- See [testing_patterns example](../../examples/testing_patterns.rs)
- Read about [Error Context](04-error-context.md)
