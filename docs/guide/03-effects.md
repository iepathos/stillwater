# Effect Composition: Pure Core, Imperative Shell

## The Philosophy

Effect helps you structure applications with:
- **Pure core**: Business logic with no side effects (easy to test)
- **Imperative shell**: I/O operations at the boundaries (controlled)

This separation makes code more testable, maintainable, and composable.

## Zero-Cost by Default

Stillwater's Effect system follows the `futures` crate pattern: **zero-cost by default, explicit boxing when needed**.

```rust
use stillwater::prelude::*;

// Zero heap allocations - compiler can inline everything
let effect = pure::<_, String, ()>(42)
    .map(|x| x + 1)           // Returns Map<Pure<...>, ...>
    .and_then(|x| pure(x * 2)) // Returns AndThen<Map<...>, ...>
    .map(|x| x.to_string());   // Returns Map<AndThen<...>, ...>

// Type: Map<AndThen<Map<Pure<i32, String, ()>, ...>, ...>, ...>
// NO heap allocation!
```

Each combinator returns a concrete type. The compiler knows the exact type at compile time and can fully optimize the effect chain.

## The Problem

How do you test this code?

```rust
async fn create_user(email: String, age: u8) -> Result<User, Error> {
    // Validation mixed with I/O
    if !email.contains('@') {
        return Err(Error::InvalidEmail);
    }

    // Database call (requires real/mock DB)
    let existing = database.find_by_email(&email).await?;
    if existing.is_some() {
        return Err(Error::EmailExists);
    }

    // More I/O
    let user = User { email, age };
    database.save(&user).await?;
    Ok(user)
}
```

Problems:
- Can't test without database
- Business logic mixed with I/O
- Hard to reason about what's pure vs effectful

## The Solution: Effect

Effect separates pure logic from I/O:

```rust
use stillwater::prelude::*;

#[derive(Clone)]
struct AppEnv {
    db: Database,
}

fn create_user(email: String, age: u8) -> impl Effect<Output = User, Error = AppError, Env = AppEnv> {
    // Pure validation first
    from_validation(validate_user(&email, age).map_err(AppError::Validation))
        // Then I/O
        .and_then(move |_| {
            from_fn(move |env: &AppEnv| env.db.find_by_email(&email))
        })
        // Pure logic
        .and_then(move |existing| {
            if existing.is_some() {
                fail(AppError::EmailExists)
            } else {
                pure(User { email, age })
            }
        })
        // More I/O
        .and_then(|user| {
            from_fn(move |env: &AppEnv| env.db.save(&user))
                .map(move |_| user)
        })
}

// Run at application boundary
let env = AppEnv { db };
let user = create_user(email, age).run(&env).await?;
```

Benefits:
- Pure functions need no mocks
- I/O is explicit via `from_fn`, `from_async`
- Easy to test with mock environments
- Zero heap allocations in the effect chain

## Core API

### Creating Effects

```rust
use stillwater::prelude::*;

// Pure value (no I/O)
let effect = pure::<_, String, ()>(42);

// Failed effect
let effect = fail::<i32, _, ()>("error".to_string());

// From Result
let effect = from_result::<_, String, ()>(Ok(42));

// From Validation
let validation = Validation::success(42);
let effect = from_validation(validation);

// From sync function
let effect = from_fn(|env: &Env| {
    Ok::<_, String>(env.config.value)
});

// From async function
let effect = from_async(|env: &Env| async {
    env.db.fetch_user(123).await
});

// From Option
let effect = from_option::<_, _, ()>(Some(42), || "value missing");
```

### Transforming Effects

```rust
use stillwater::prelude::*;

// Map success value
let effect = pure::<_, String, ()>(21).map(|x| x * 2);
let result = effect.run(&()).await; // Ok(42)

// Map error value
let effect = fail::<i32, _, ()>("oops").map_err(|e| format!("Error: {}", e));

// Chain dependent effects
let effect = pure::<_, String, ()>(5)
    .and_then(|x| pure(x * 2))
    .and_then(|x| pure(x + 10));
let result = effect.run(&()).await; // Ok(20)
```

### Running Effects

```rust
use stillwater::prelude::*;

// With environment
let env = AppEnv { /* ... */ };
let result = effect.run(&env).await;

// With unit environment (when Env = ())
use stillwater::RunStandalone;
let result = effect.run_standalone().await;
```

## When to Use `.boxed()`

Boxing is needed in exactly three situations:

### 1. Storing in Collections

```rust
use stillwater::prelude::*;

// Different effect types can't be stored in the same Vec
// Boxing gives them a uniform type
let effects: Vec<BoxedEffect<i32, String, ()>> = vec![
    pure(1).boxed(),
    pure(2).map(|x| x * 2).boxed(),
    pure(3).and_then(|x| pure(x * 3)).boxed(),
];

// Process them
for effect in effects {
    let result = effect.run(&()).await?;
    println!("Result: {}", result);
}
```

### 2. Recursive Effects

```rust
use stillwater::prelude::*;

// Recursive function needs concrete return type
fn countdown(n: i32) -> BoxedEffect<i32, String, ()> {
    if n <= 0 {
        pure(0).boxed()
    } else {
        pure(n)
            .and_then(move |x| countdown(x - 1).map(move |sum| x + sum))
            .boxed()
    }
}

let sum = countdown(5).run(&()).await?; // 15
```

### 3. Match Arms with Different Effect Types

```rust
use stillwater::prelude::*;

enum DataSource {
    Cache,
    Database,
    Remote,
}

fn fetch_data(source: DataSource) -> BoxedEffect<String, String, ()> {
    match source {
        DataSource::Cache => {
            // Just pure value
            pure("cached data".to_string()).boxed()
        }
        DataSource::Database => {
            // Effect with map
            pure("db")
                .map(|s| format!("{} data", s))
                .boxed()
        }
        DataSource::Remote => {
            // Effect with and_then
            pure("remote")
                .and_then(|s| pure(format!("{} data", s)))
                .boxed()
        }
    }
}
```

## Reader Pattern

The Reader pattern provides functional dependency injection. Stillwater includes three helpers:

### `ask()` - Access the Environment

Returns the entire environment:

```rust
use stillwater::prelude::*;

#[derive(Clone)]
struct Config {
    api_key: String,
    timeout: u64,
}

// Get the whole environment
let effect = ask::<String, Config>();

let config = Config {
    api_key: "secret".into(),
    timeout: 30,
};

let result = effect.run(&config).await.unwrap();
assert_eq!(result.api_key, "secret");
```

### `asks()` - Query Environment

Extract a specific value:

```rust
use stillwater::prelude::*;

#[derive(Clone)]
struct AppEnv {
    database: String,
    cache: String,
}

// Query just the database field
let effect = asks(|env: &AppEnv| env.database.clone());

let env = AppEnv {
    database: "postgres".into(),
    cache: "redis".into(),
};

let result = effect.run(&env).await.unwrap();
assert_eq!(result, "postgres");
```

### `local()` - Modify Environment

Run an effect with a temporarily modified environment:

```rust
use stillwater::prelude::*;

#[derive(Clone)]
struct Config {
    debug: bool,
    timeout: u64,
}

fn fetch_data() -> impl Effect<Output = String, Error = String, Env = Config> {
    asks(|cfg: &Config| format!("fetched with timeout {}", cfg.timeout))
}

let config = Config {
    debug: false,
    timeout: 30,
};

// Run with modified timeout
let effect = local(
    |cfg: &Config| Config { timeout: 60, ..*cfg },
    fetch_data()
);

let result = effect.run(&config).await.unwrap();
assert_eq!(result, "fetched with timeout 60");
// Original config still has timeout=30
```

## Parallel Effects

### Heterogeneous Parallel (Zero-Cost)

For 2-4 effects of different types, use `par2`, `par3`, `par4`:

```rust
use stillwater::prelude::*;

let (num, text) = par2(
    pure::<_, String, ()>(42),
    pure::<_, String, ()>("hello".to_string()),
).run(&()).await?;
```

### Homogeneous Parallel (Requires Boxing)

For collections of effects, use `par_all`, `race`, `par_all_limit`:

```rust
use stillwater::prelude::*;

// par_all - run all, collect all results
let effects: Vec<BoxedEffect<i32, String, ()>> = vec![
    pure(1).boxed(),
    pure(2).boxed(),
    pure(3).boxed(),
];
let results = par_all(effects).run(&()).await?; // [1, 2, 3]

// race - return first success
let effects: Vec<BoxedEffect<String, String, ()>> = vec![
    fail("first failed".to_string()).boxed(),
    pure("second succeeded".to_string()).boxed(),
];
let result = race(effects).run(&()).await?; // "second succeeded"

// par_all_limit - run with concurrency limit
let effects: Vec<BoxedEffect<i32, String, ()>> = /* many effects */;
let results = par_all_limit(effects, 10).run(&()).await?; // max 10 concurrent
```

## Real-World Example: User Registration

```rust
use stillwater::prelude::*;

// Environment with dependencies
#[derive(Clone)]
struct AppEnv {
    db: Database,
    email_service: EmailService,
}

// Error type
#[derive(Debug)]
enum AppError {
    ValidationError(Vec<String>),
    EmailExists,
    DatabaseError(String),
    EmailError(String),
}

// Pure validation (no I/O, easy to test)
fn validate_user(email: &str, age: u8) -> Validation<(), Vec<String>> {
    Validation::all((
        validate_email(email),
        validate_age(age),
    ))
    .map(|_| ())
}

// Effect composition (I/O at boundaries)
fn register_user(
    email: String,
    age: u8,
) -> impl Effect<Output = User, Error = AppError, Env = AppEnv> {
    // 1. Validate input (pure)
    from_validation(
        validate_user(&email, age)
            .map_err(AppError::ValidationError)
    )
    // 2. Check if email exists (I/O)
    .and_then(move |_| {
        from_fn(move |env: &AppEnv| {
            env.db.find_by_email(&email)
                .map_err(|e| AppError::DatabaseError(e.to_string()))
        })
    })
    // 3. Check uniqueness (pure logic)
    .and_then(move |existing| {
        if existing.is_some() {
            fail(AppError::EmailExists)
        } else {
            pure(())
        }
    })
    // 4. Create user (pure)
    .map(move |_| User { email: email.clone(), age })
    // 5. Save to database (I/O)
    .and_then(|user| {
        from_fn(move |env: &AppEnv| {
            env.db.save_user(&user)
                .map_err(|e| AppError::DatabaseError(e.to_string()))
        })
        .map(move |_| user)
    })
    // 6. Send welcome email (I/O)
    .and_then(|user| {
        from_fn(move |env: &AppEnv| {
            env.email_service.send_welcome(&user.email)
                .map_err(|e| AppError::EmailError(e.to_string()))
        })
        .map(move |_| user)
    })
}

// Usage at application boundary
#[tokio::main]
async fn main() -> Result<(), AppError> {
    let env = AppEnv {
        db: Database::connect("postgres://...").await?,
        email_service: EmailService::new(),
    };

    let user = register_user(
        "user@example.com".to_string(),
        25
    ).run(&env).await?;

    println!("Registered: {:?}", user);
    Ok(())
}
```

## Testing Effects

The key benefit: pure functions need no mocks!

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Test pure validation (no mocks needed!)
    #[test]
    fn test_validate_user() {
        let result = validate_user("user@example.com", 25);
        assert!(result.is_success());

        let result = validate_user("invalid", 15);
        assert!(result.is_failure());
    }

    // Test effectful code with mock environment
    #[derive(Clone)]
    struct MockEnv {
        users: Vec<User>,
    }

    impl MockEnv {
        fn find_by_email(&self, email: &str) -> Result<Option<User>, String> {
            Ok(self.users.iter().find(|u| u.email == email).cloned())
        }
    }

    #[tokio::test]
    async fn test_effect_with_mock_env() {
        let env = MockEnv { users: vec![] };

        let effect = from_fn(|env: &MockEnv| env.find_by_email("test@example.com"))
            .and_then(|existing| {
                if existing.is_some() {
                    fail("Email exists")
                } else {
                    pure(User {
                        email: "test@example.com".to_string(),
                        age: 25,
                    })
                }
            });

        let result = effect.run(&env).await;
        assert!(result.is_ok());
    }
}
```

## Performance Considerations

The Effect trait is zero-cost by default:
- No heap allocations for effect chains
- Compiler can fully inline combinators
- Same performance as hand-written async code

Boxing happens only when you call `.boxed()`:
- Collections of effects
- Recursive effects
- Match arms with different types

For I/O-bound work (API calls, database queries), boxing overhead is negligible compared to actual work.

## Common Patterns

### Pattern 1: Validate Then Execute

```rust
from_validation(validate_input(input))
    .and_then(|valid| execute_with_db(valid))
```

### Pattern 2: Read, Decide, Write

```rust
from_fn(|env: &Env| env.db.fetch(id))
    .and_then(|data| {
        let result = pure_business_logic(data);
        from_fn(move |env: &Env| env.db.save(result))
    })
```

### Pattern 3: Error Context

```rust
create_user(email, age)
    .context("Creating user account")
    .and_then(|user| {
        send_welcome_email(&user)
            .context("Sending welcome email")
    })
```

### Pattern 4: Conditional Effect

```rust
fn conditional_fetch(use_cache: bool) -> BoxedEffect<String, String, AppEnv> {
    if use_cache {
        from_fn(|env: &AppEnv| Ok(env.cache.get("data"))).boxed()
    } else {
        from_async(|env: &AppEnv| async { env.db.fetch().await }).boxed()
    }
}
```

## When to Use Effect

**Use Effect when**:
- Separating I/O from business logic
- Testing effectful code
- Composing async operations
- Dependency injection needed

**Use plain async fn when**:
- Simple CRUD operations
- No complex composition
- Testing not critical
- Maximum simplicity needed

## Summary

- **Effect trait**: Zero-cost effect composition following `futures` pattern
- **Pure core**: Business logic is easy to test (no mocks)
- **Imperative shell**: I/O at boundaries via `from_fn`, `from_async`
- **Environment**: Provides dependency injection
- **Boxing**: Use `.boxed()` only when type erasure is needed
- **Composition**: Via `map`, `and_then`, `or_else`, etc.
- **Reader pattern**: `ask()`, `asks()`, `local()` for environment access

## Next Steps

- Learn about [Error Context](04-error-context.md)
- Explore the [Reader Pattern](09-reader-pattern.md) in depth
- See the [Migration Guide](../MIGRATION.md) if upgrading from 0.10.x
- Check out [testing_patterns example](../../examples/testing_patterns.rs)
- Read the [API docs](https://docs.rs/stillwater)
