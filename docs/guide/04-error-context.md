# Error Context: Never Lose the Trail

## The Problem

Standard errors lose context as they bubble up:

```rust
async fn load_user_profile(id: u64) -> Result<Profile, Error> {
    let user = database.fetch_user(id).await?;
    // Error: "Connection refused"
    // Lost: Why were we connecting? What were we trying to do?
}
```

When an error occurs deep in your code, you lose valuable context:
- What operation was being attempted?
- What was the call path?
- Which resource failed?

## The Solution: ContextError

`ContextError` wraps errors and accumulates context as they propagate:

```rust
use stillwater::ContextError;

let err = ContextError::new("connection refused")
    .context("fetching user from database")
    .context("loading user profile")
    .context("rendering dashboard");

println!("{}", err);
// Output:
// Error: connection refused
//   -> fetching user from database
//   -> loading user profile
//   -> rendering dashboard
```

Now you know exactly what failed and why!

## Core API

### Creating Context Errors

```rust
use stillwater::ContextError;

// Wrap an error
let err = ContextError::new("file not found");

// Add context
let err = err.context("reading config file");
```

### Adding Context Layers

```rust
use stillwater::ContextError;

let err = ContextError::new("parse error")
    .context("reading config.toml")
    .context("initializing application")
    .context("startup sequence");

// Context accumulates in order (innermost to outermost)
assert_eq!(err.context_trail(), &[
    "reading config.toml",
    "initializing application",
    "startup sequence"
]);
```

### Accessing the Error

```rust
use stillwater::ContextError;

let err = ContextError::new("base error")
    .context("operation failed");

// Get reference to inner error
assert_eq!(err.inner(), &"base error");

// Consume and get inner error
let inner = err.into_inner();
assert_eq!(inner, "base error");

// Get context trail
let trail = err.context_trail();
assert_eq!(trail, &["operation failed"]);
```

## Using with Effect

Context errors integrate seamlessly with Effect:

```rust
use stillwater::{Effect, ContextError};

fn load_user(id: u64) -> Effect<User, ContextError<DbError>, AppEnv> {
    IO::read(|env: &AppEnv| env.db.fetch_user(id))
        .map_err(|e| ContextError::new(e).context("fetching user from database"))
        .and_then(|user| {
            load_profile(&user)
                .map_err(|e| e.context("loading user profile"))
        })
        .map_err(|e| e.context("rendering dashboard"))
}
```

Even better, Effect provides a `context` method:

```rust
use stillwater::Effect;

fn load_user(id: u64) -> Effect<User, String, AppEnv> {
    IO::read(|env: &AppEnv| env.db.fetch_user(id))
        .context("fetching user from database")
        .and_then(|user| {
            load_profile(&user)
                .context("loading user profile")
        })
        .context("rendering dashboard")
}
```

The Effect's `context` method automatically wraps errors in ContextError!

## Real-World Example

```rust
use stillwater::{Effect, IO, ContextError};

struct AppEnv {
    db: Database,
    cache: Cache,
}

#[derive(Debug)]
enum AppError {
    DatabaseError(String),
    CacheError(String),
    NotFound,
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            AppError::CacheError(msg) => write!(f, "Cache error: {}", msg),
            AppError::NotFound => write!(f, "Resource not found"),
        }
    }
}

impl std::error::Error for AppError {}

fn get_user_dashboard(
    user_id: u64
) -> Effect<Dashboard, ContextError<AppError>, AppEnv> {
    // Try cache first
    IO::read(|env: &AppEnv| {
        env.cache.get_user(user_id)
            .map_err(|e| AppError::CacheError(e.to_string()))
    })
    .map_err(|e| ContextError::new(e).context("checking user cache"))
    .and_then(|cached| {
        match cached {
            Some(user) => Effect::pure(user),
            None => {
                // Cache miss - fetch from database
                IO::read(|env: &AppEnv| {
                    env.db.fetch_user(user_id)
                        .map_err(|e| AppError::DatabaseError(e.to_string()))
                })
                .map_err(|e| ContextError::new(e).context("fetching user from database"))
            }
        }
    })
    .and_then(|user| {
        // Load user's projects
        IO::read(|env: &AppEnv| {
            env.db.fetch_user_projects(user.id)
                .map_err(|e| AppError::DatabaseError(e.to_string()))
        })
        .map_err(|e| ContextError::new(e).context("loading user projects"))
        .map(|projects| (user, projects))
    })
    .and_then(|(user, projects)| {
        // Load user's notifications
        IO::read(|env: &AppEnv| {
            env.db.fetch_notifications(user.id)
                .map_err(|e| AppError::DatabaseError(e.to_string()))
        })
        .map_err(|e| ContextError::new(e).context("loading notifications"))
        .map(|notifications| Dashboard { user, projects, notifications })
    })
    .map_err(|e| e.context("building user dashboard"))
}

// Usage
match get_user_dashboard(123).run(&env).await {
    Ok(dashboard) => println!("Dashboard: {:?}", dashboard),
    Err(err) => {
        eprintln!("{}", err);
        // Output might be:
        // Error: Database error: connection timeout
        //   -> fetching user from database
        //   -> building user dashboard
    }
}
```

## Best Practices

### Add Context at Boundaries

Don't add context to every function. Add it at major operation boundaries:

```rust
// ❌ Too much context
fn validate_email(email: &str) -> Result<Email, ContextError<Error>> {
    check_format(email)
        .context("checking email format")?;  // Too granular
    check_domain(email)
        .context("checking email domain")?;   // Too granular
    Ok(Email(email))
}

// ✓ Context at boundaries
fn register_user(input: UserInput) -> Effect<User, ContextError<Error>, Env> {
    validate_user(input)
        .context("validating user input")  // Good
        .and_then(|valid| {
            save_to_database(valid)
                .context("saving user to database")  // Good
        })
}
```

### Be Specific but Concise

```rust
// ❌ Too vague
.context("error occurred")

// ❌ Too verbose
.context("An error occurred while attempting to read the user configuration file from disk")

// ✓ Just right
.context("reading user config file")
```

### Include Relevant Identifiers

```rust
// ✓ Include user ID for debugging
.context(format!("loading profile for user {}", user_id))

// ✓ Include file path
.context(format!("reading config from {}", path.display()))
```

## When to Use ContextError

**Use ContextError when**:
- Debugging production issues
- Errors cross multiple layers
- You need to understand call paths
- Building user-facing error messages

**Don't use when**:
- Performance is critical (hot loops)
- Error types already have good messages
- Single-layer operations

## Display Format

ContextError formats nicely for logging:

```rust
use stillwater::ContextError;

let err = ContextError::new("connection timeout")
    .context("querying database")
    .context("loading user profile")
    .context("rendering dashboard");

println!("{}", err);
// Output:
// Error: connection timeout
//   -> querying database
//   -> loading user profile
//   -> rendering dashboard
```

The indentation makes the error trail clear and readable.

## Testing

Test error context accumulation:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_error_context() {
        let env = MockEnv::with_error();

        let result = load_user_dashboard(123).run(&env).await;

        match result {
            Err(err) => {
                // Check inner error
                assert_eq!(
                    err.inner().to_string(),
                    "Database error: connection failed"
                );

                // Check context trail
                let trail = err.context_trail();
                assert!(trail.contains(&"fetching user from database".to_string()));
                assert!(trail.contains(&"building user dashboard".to_string()));
            }
            Ok(_) => panic!("Expected error"),
        }
    }
}
```

## Integration with Other Error Crates

ContextError implements `std::error::Error`, so it works with libraries like anyhow:

```rust
use anyhow::Result;
use stillwater::ContextError;

fn example() -> Result<()> {
    let ctx_err = ContextError::new("base error")
        .context("operation failed");

    Err(ctx_err)?  // Converts to anyhow::Error
}
```

## Performance Considerations

ContextError has minimal overhead:
- Small allocation for context Vec
- String allocations for messages
- No runtime cost if not used

The benefits for debugging usually outweigh the costs.

## Summary

- **ContextError** accumulates context as errors propagate
- **Context trail** shows the call path
- **Effect.context()** makes adding context ergonomic
- **Add context at boundaries**, not everywhere
- **Display format** is clean and readable

## Next Steps

- Learn about the [IO Module](05-io-module.md)
- See [error_context example](../../examples/error_context.rs)
- Read about [Helper Combinators](06-helper-combinators.md)
