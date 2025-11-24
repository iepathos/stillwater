# Effect Composition: Pure Core, Imperative Shell

## The Philosophy

Effect helps you structure applications with:
- **Pure core**: Business logic with no side effects (easy to test)
- **Imperative shell**: I/O operations at the boundaries (controlled)

This separation makes code more testable, maintainable, and composable.

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
use stillwater::{Effect, IO};

struct AppEnv {
    db: Database,
}

fn create_user(email: String, age: u8) -> Effect<User, Error, AppEnv> {
    // Pure validation first
    Effect::from_validation(validate_user(&email, age))
        // Then I/O
        .and_then(|_| {
            IO::read(|env: &AppEnv| env.db.find_by_email(&email))
        })
        // Pure logic
        .and_then(|existing| {
            if existing.is_some() {
                Effect::fail(Error::EmailExists)
            } else {
                Effect::pure(User { email, age })
            }
        })
        // More I/O
        .and_then(|user| {
            IO::execute(|env: &AppEnv| env.db.save(&user))
                .map(|_| user)
        })
}

// Run at application boundary
let env = AppEnv { db };
let user = create_user(email, age).run(&env).await?;
```

Benefits:
- Pure functions need no mocks
- I/O is explicit via `IO::read/write/execute`
- Easy to test with mock environments

## Core API

### Creating Effects

```rust
use stillwater::Effect;

// Pure value (no I/O)
let effect = Effect::pure(42);

// Failed effect
let effect = Effect::fail("error");

// From Result
let effect = Effect::from_result(Ok(42));

// From Validation
let validation = Validation::success(42);
let effect = Effect::from_validation(validation);

// From sync function
let effect = Effect::from_fn(|env: &Env| {
    Ok::<_, Error>(env.config.value)
});

// From async function
let effect = Effect::from_async(|env: &Env| async {
    env.db.fetch_user(123).await
});
```

### Transforming Effects

```rust
use stillwater::Effect;

// Map success value
let effect = Effect::pure(21).map(|x| x * 2);
assert_eq!(effect.run(&()).await, Ok(42));

// Map error value
let effect = Effect::fail("oops").map_err(|e| format!("Error: {}", e));

// Chain dependent effects
let effect = Effect::pure(5)
    .and_then(|x| Effect::pure(x * 2))
    .and_then(|x| Effect::pure(x + 10));
assert_eq!(effect.run(&()).await, Ok(20));
```

### Running Effects

```rust
use stillwater::Effect;

let effect = Effect::pure(42);

// Run with environment
let env = AppEnv { /* ... */ };
let result = effect.run(&env).await;
```

## The IO Module

The `IO` module provides ergonomic helpers for common patterns:

```rust
use stillwater::IO;

// Read from environment (immutable)
let effect = IO::read(|env: &AppEnv| env.db.fetch_user(id));

// Write to environment (mutable)
let effect = IO::write(|env: &mut AppEnv| {
    env.cache.insert(key, value);
    Ok(())
});

// Execute async operation
let effect = IO::execute(|env: &AppEnv| {
    env.db.save_user(&user)
});
```

See [IO Module guide](05-io-module.md) for details.

## Real-World Example: User Registration

```rust
use stillwater::{Effect, IO, Validation};

// Environment with dependencies
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
) -> Effect<User, AppError, AppEnv> {
    // 1. Validate input (pure)
    Effect::from_validation(
        validate_user(&email, age)
            .map_err(AppError::ValidationError)
    )
    // 2. Check if email exists (I/O)
    .and_then(|_| {
        IO::read(|env: &AppEnv| {
            env.db.find_by_email(&email)
                .map_err(|e| AppError::DatabaseError(e.to_string()))
        })
    })
    // 3. Check uniqueness (pure logic)
    .and_then(|existing| {
        if existing.is_some() {
            Effect::fail(AppError::EmailExists)
        } else {
            Effect::pure(())
        }
    })
    // 4. Create user (pure)
    .map(|_| User { email: email.clone(), age })
    // 5. Save to database (I/O)
    .and_then(|user| {
        IO::execute(|env: &AppEnv| {
            env.db.save_user(&user)
                .map_err(|e| AppError::DatabaseError(e.to_string()))
        })
        .map(|_| user)
    })
    // 6. Send welcome email (I/O)
    .and_then(|user| {
        IO::execute(|env: &AppEnv| {
            env.email_service.send_welcome(&user.email)
                .map_err(|e| AppError::EmailError(e.to_string()))
        })
        .map(|_| user)
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
    struct MockEnv {
        users: Vec<User>,
    }

    impl MockEnv {
        fn find_by_email(&self, email: &str) -> Result<Option<User>, String> {
            Ok(self.users.iter().find(|u| u.email == email).cloned())
        }

        fn save_user(&mut self, user: &User) -> Result<(), String> {
            self.users.push(user.clone());
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_register_user_success() {
        let env = MockEnv { users: vec![] };

        // Create simplified effect for testing
        let effect = IO::read(|env: &MockEnv| env.find_by_email("test@example.com"))
            .and_then(|existing| {
                if existing.is_some() {
                    Effect::fail("Email exists")
                } else {
                    Effect::pure(User {
                        email: "test@example.com".to_string(),
                        age: 25,
                    })
                }
            });

        let result = effect.run(&env).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_register_user_email_exists() {
        let env = MockEnv {
            users: vec![User {
                email: "existing@example.com".to_string(),
                age: 30,
            }],
        };

        let effect = IO::read(|env: &MockEnv| env.find_by_email("existing@example.com"))
            .and_then(|existing| {
                if existing.is_some() {
                    Effect::fail("Email exists")
                } else {
                    Effect::pure(User {
                        email: "existing@example.com".to_string(),
                        age: 25,
                    })
                }
            });

        let result = effect.run(&env).await;
        assert!(result.is_err());
    }
}
```

## Patterns

### Pattern 1: Validate Then Execute

```rust
Effect::from_validation(validate_input(input))
    .and_then(|valid| execute_with_db(valid))
```

### Pattern 2: Read, Decide, Write

```rust
IO::read(|env: &Env| env.db.fetch(id))
    .and_then(|data| {
        let result = pure_business_logic(data);
        IO::execute(|env: &Env| env.db.save(result))
    })
```

### Pattern 3: Parallel Effects (future enhancement)

Currently, effects run sequentially. For parallel execution, use tokio directly:

```rust
let (result1, result2) = tokio::join!(
    effect1.run(&env),
    effect2.run(&env)
);
```

### Pattern 4: Error Context

```rust
create_user(email, age)
    .context("Creating user account")
    .and_then(|user| {
        send_welcome_email(&user)
            .context("Sending welcome email")
    })
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
- Performance is paramount

## Common Pitfalls

### Don't mix pure and effectful code

```rust
// ❌ Wrong: I/O mixed with logic
fn process(id: u64) -> Effect<Result, Error, Env> {
    IO::read(|env: &Env| {
        let data = env.db.fetch(id)?;
        let processed = expensive_calculation(data); // Pure logic in I/O!
        env.db.save(processed)?;
        Ok(processed)
    })
}

// ✓ Right: Separate pure and effectful
fn process(id: u64) -> Effect<Result, Error, Env> {
    IO::read(|env: &Env| env.db.fetch(id))
        .map(|data| expensive_calculation(data))  // Pure!
        .and_then(|processed| {
            IO::execute(|env: &Env| env.db.save(processed))
        })
}
```

### Remember to run() at the boundary

```rust
// ❌ Wrong: Effect is lazy, nothing happens
let effect = create_user(email, age);
// User not created yet!

// ✓ Right: Run at application boundary
let user = create_user(email, age).run(&env).await?;
```

## Performance Considerations

Effect has minimal overhead:
- One box allocation per effect creation
- Zero-cost for pure values
- Async overhead only when using async operations

For tight loops or hot paths, consider using plain Result/async fn.

## Reader Pattern

The Reader pattern provides a functional approach to dependency injection. Stillwater includes three helpers for working with environments:

### `ask()` - Access the Environment

Returns the entire environment as an Effect:

```rust
use stillwater::Effect;

struct Config {
    api_key: String,
    timeout: u64,
}

// Get the whole environment
let effect = Effect::<Config, String, Config>::ask();

let config = Config {
    api_key: "secret".into(),
    timeout: 30,
};

let result = effect.run(&config).await.unwrap();
assert_eq!(result.api_key, "secret");
```

### `asks()` - Query Environment

Extract a specific value from the environment:

```rust
use stillwater::Effect;

struct AppEnv {
    database: String,
    cache: String,
}

// Query just the database field
let effect = Effect::asks(|env: &AppEnv| env.database.clone());

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
use stillwater::Effect;

#[derive(Clone)]
struct Config {
    debug: bool,
    timeout: u64,
}

fn fetch_data() -> Effect<String, String, Config> {
    Effect::asks(|cfg: &Config| {
        format!("fetched with timeout {}", cfg.timeout)
    })
}

let config = Config {
    debug: false,
    timeout: 30,
};

// Run with modified timeout for this specific fetch
let effect = Effect::local(
    |cfg: &Config| Config { timeout: 60, ..*cfg },
    fetch_data()
);

let result = effect.run(&config).await.unwrap();
assert_eq!(result, "fetched with timeout 60");
// Original config still has timeout=30
```

### Composing Reader Patterns

Combine these helpers with other Effect methods:

```rust
use stillwater::{Effect, IO};

struct AppEnv {
    db: Database,
    max_retries: u32,
}

fn save_with_retries(data: Data) -> Effect<(), Error, AppEnv> {
    // Get max retries from environment
    Effect::asks(|env: &AppEnv| env.max_retries)
        .and_then(|retries| {
            // Use it in our logic
            retry_operation(data, retries)
        })
}

fn retry_operation(data: Data, max: u32) -> Effect<(), Error, AppEnv> {
    // Implementation...
    Effect::pure(())
}
```

The Reader pattern is particularly useful when:
- Multiple functions need the same configuration
- You want to avoid passing environment through every function call
- Testing requires different environment configurations
- Environment needs temporary modifications for specific operations

See the [Reader Pattern guide](09-reader-pattern.md) for comprehensive examples and patterns.

## Summary

- **Effect** separates pure logic from I/O
- **Pure core** is easy to test (no mocks)
- **Imperative shell** handles I/O at boundaries
- **Environment** provides dependency injection
- **Composition** via map, and_then, etc.
- **Reader pattern** helpers: ask(), asks(), local()

## Next Steps

- Learn about [Error Context](04-error-context.md)
- See the [IO Module](05-io-module.md) guide
- Explore the [Reader Pattern](09-reader-pattern.md) in depth
- Check out [testing_patterns example](../../examples/testing_patterns.rs)
- Read the [API docs](https://docs.rs/stillwater)
