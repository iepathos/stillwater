---
number: 005
title: IO Module with Read/Write Helpers
category: foundation
priority: high
status: draft
dependencies: [003]
created: 2025-11-21
---

# Specification 005: IO Module with Read/Write Helpers

**Category**: foundation
**Priority**: high
**Status**: draft
**Dependencies**: Spec 003 (Effect Type)

## Context

Creating Effect instances for I/O operations requires boilerplate: wrapping functions, handling environment extraction, etc. The IO module provides convenient helpers that make common I/O patterns ergonomic.

Based on our design analysis, we chose `read` and `write` over `query` and `execute` because:
- More universal (works for files, DB, network, cache, logger)
- Matches Rust stdlib conventions (std::fs::read, std::fs::write)
- Crystal clear intent (read = get data, write = mutate/send data)

The IO module handles environment extraction automatically using trait bounds, making it easy to access specific services from a composite environment.

## Objective

Implement an `IO` module that provides convenient helpers (`read`, `write`, `read_async`, `write_async`) for creating Effects from I/O operations, with automatic environment extraction.

## Requirements

### Functional Requirements

- Provide `IO::read()` for read-only operations (immutable borrow)
- Provide `IO::write()` for mutating operations (mutable borrow)
- Provide `IO::read_async()` for async read operations
- Provide `IO::write_async()` for async write operations
- Support automatic environment extraction via type inference
- Work with any type that implements AsRef/AsMut for the service
- Clear error messages when environment doesn't have required service
- Integrate seamlessly with Effect composition

### Non-Functional Requirements

- Type inference works for common cases
- Zero-cost (inlines to direct function calls)
- Clear documentation
- Works with custom environment types

## Acceptance Criteria

- [ ] IO struct defined in `src/io.rs`
- [ ] `IO::read()` takes closure, returns Effect
- [ ] `IO::write()` takes closure, returns Effect
- [ ] `IO::read_async()` takes async closure, returns Effect
- [ ] `IO::write_async()` takes async closure, returns Effect
- [ ] Type inference determines service type from closure parameter
- [ ] Works with AsRef/AsMut pattern for environment extraction
- [ ] Comprehensive tests (>95% coverage)
- [ ] Documentation with examples showing different services
- [ ] Examples in examples/ directory

## Technical Details

### Implementation Approach

```rust
use std::convert::Infallible;

/// Helper for creating I/O effects
pub struct IO;

impl IO {
    /// Create an effect from a read-only operation
    ///
    /// The closure receives an immutable reference to the service
    /// extracted from the environment.
    pub fn read<T, R, F, Env>(f: F) -> Effect<R, Infallible, Env>
    where
        F: FnOnce(&T) -> R + Send + 'static,
        R: Send + 'static,
        T: Send + Sync + 'static,
        Env: AsRef<T> + Send + Sync + 'static,
    {
        Effect::from_fn(move |env: &Env| Ok(f(env.as_ref())))
    }

    /// Create an effect from a mutating operation
    ///
    /// Note: Since Effect runs with &Env, true mutation requires
    /// interior mutability (RefCell, Mutex, etc.) in practice.
    /// This is a design choice to explore.
    pub fn write<T, R, F, Env>(f: F) -> Effect<R, Infallible, Env>
    where
        F: FnOnce(&T) -> R + Send + 'static,
        R: Send + 'static,
        T: Send + Sync + 'static,
        Env: AsRef<T> + Send + Sync + 'static,
    {
        Effect::from_fn(move |env: &Env| Ok(f(env.as_ref())))
    }

    /// Create an effect from an async read-only operation
    pub fn read_async<T, R, F, Fut, Env>(f: F) -> Effect<R, Infallible, Env>
    where
        F: FnOnce(&T) -> Fut + Send + 'static,
        Fut: Future<Output = R> + Send + 'static,
        R: Send + 'static,
        T: Send + Sync + 'static,
        Env: AsRef<T> + Send + Sync + 'static,
    {
        Effect::from_async(move |env: &Env| async move {
            let service = env.as_ref();
            Ok(f(service).await)
        })
    }

    /// Create an effect from an async mutating operation
    pub fn write_async<T, R, F, Fut, Env>(f: F) -> Effect<R, Infallible, Env>
    where
        F: FnOnce(&T) -> Fut + Send + 'static,
        Fut: Future<Output = R> + Send + 'static,
        R: Send + 'static,
        T: Send + Sync + 'static,
        Env: AsRef<T> + Send + Sync + 'static,
    {
        Effect::from_async(move |env: &Env| async move {
            let service = env.as_ref();
            Ok(f(service).await)
        })
    }
}
```

### Architecture Changes

- New module: `src/io.rs`
- Export from `src/lib.rs`
- Re-export in `prelude`

### Data Structures

```rust
pub struct IO;  // Zero-sized type, just a namespace
```

### APIs and Interfaces

See Implementation Approach above.

### Environment Pattern

Users implement AsRef for their environment:

```rust
struct AppEnv {
    db: Database,
    cache: Cache,
    logger: Logger,
}

impl AsRef<Database> for AppEnv {
    fn as_ref(&self) -> &Database {
        &self.db
    }
}

impl AsRef<Cache> for AppEnv {
    fn as_ref(&self) -> &Cache {
        &self.cache
    }
}

// Now IO helpers work automatically:
let effect = IO::read(|db: &Database| db.find_user(id));
```

## Dependencies

- **Prerequisites**: Spec 003 (Effect type)
- **Affected Components**: None (new module)
- **External Dependencies**: None

## Testing Strategy

### Unit Tests

```rust
#[tokio::test]
async fn test_io_read() {
    struct Database {
        users: Vec<User>,
    }

    impl Database {
        fn find_user(&self, id: u64) -> Option<User> {
            self.users.iter().find(|u| u.id == id).cloned()
        }
    }

    struct Env {
        db: Database,
    }

    impl AsRef<Database> for Env {
        fn as_ref(&self) -> &Database {
            &self.db
        }
    }

    let env = Env {
        db: Database {
            users: vec![User { id: 1, name: "Alice" }],
        },
    };

    let effect = IO::read(|db: &Database| db.find_user(1));
    let result = effect.run(&env).await;

    assert_eq!(result, Ok(Some(User { id: 1, name: "Alice" })));
}

#[tokio::test]
async fn test_io_read_async() {
    struct Database;

    impl Database {
        async fn query(&self, sql: &str) -> String {
            format!("Result of: {}", sql)
        }
    }

    struct Env {
        db: Database,
    }

    impl AsRef<Database> for Env {
        fn as_ref(&self) -> &Database {
            &self.db
        }
    }

    let env = Env { db: Database };

    let effect = IO::read_async(|db: &Database| async move {
        db.query("SELECT * FROM users").await
    });

    let result = effect.run(&env).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_multiple_services() {
    struct Database;
    struct Cache;
    struct Logger;

    struct Env {
        db: Database,
        cache: Cache,
        logger: Logger,
    }

    impl AsRef<Database> for Env {
        fn as_ref(&self) -> &Database { &self.db }
    }
    impl AsRef<Cache> for Env {
        fn as_ref(&self) -> &Cache { &self.cache }
    }
    impl AsRef<Logger> for Env {
        fn as_ref(&self) -> &Logger { &self.logger }
    }

    let env = Env {
        db: Database,
        cache: Cache,
        logger: Logger,
    };

    // Type inference figures out which service to use
    let db_effect = IO::read(|db: &Database| "db data");
    let cache_effect = IO::read(|cache: &Cache| "cache data");

    assert_eq!(db_effect.run(&env).await, Ok("db data"));
    assert_eq!(cache_effect.run(&env).await, Ok("cache data"));
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_real_world_composition() {
    // Simulate real database and cache
    struct Database {
        data: std::collections::HashMap<u64, String>,
    }

    impl Database {
        async fn find(&self, id: u64) -> Option<String> {
            self.data.get(&id).cloned()
        }
    }

    struct Cache {
        data: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<u64, String>>>,
    }

    impl Cache {
        fn get(&self, id: u64) -> Option<String> {
            self.data.lock().unwrap().get(&id).cloned()
        }

        fn set(&self, id: u64, value: String) {
            self.data.lock().unwrap().insert(id, value);
        }
    }

    struct Env {
        db: Database,
        cache: Cache,
    }

    impl AsRef<Database> for Env {
        fn as_ref(&self) -> &Database { &self.db }
    }

    impl AsRef<Cache> for Env {
        fn as_ref(&self) -> &Cache { &self.cache }
    }

    // Check cache, fallback to DB
    async fn get_user(id: u64) -> Effect<Option<String>, Infallible, Env> {
        IO::read(move |cache: &Cache| cache.get(id))
            .and_then(|cached| {
                if cached.is_some() {
                    Effect::pure(cached)
                } else {
                    IO::read_async(move |db: &Database| async move {
                        db.find(id).await
                    })
                    .and_then(move |value| {
                        if let Some(ref v) = value {
                            IO::read(move |cache: &Cache| {
                                cache.set(id, v.clone());
                            })
                            .map(|_| value)
                        } else {
                            Effect::pure(value)
                        }
                    })
                }
            })
    }

    let mut db_data = std::collections::HashMap::new();
    db_data.insert(1, "Alice".to_string());

    let env = Env {
        db: Database { data: db_data },
        cache: Cache {
            data: std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
        },
    };

    let result = get_user(1).run(&env).await;
    assert_eq!(result, Ok(Some("Alice".to_string())));

    // Should be cached now
    let cached = env.cache.get(1);
    assert_eq!(cached, Some("Alice".to_string()));
}
```

## Documentation Requirements

### Code Documentation

- Comprehensive rustdoc for IO helpers
- Examples for each method
- Explain environment extraction pattern
- Show AsRef implementation examples

### User Documentation

- Add "IO Module" section to README
- Create section in docs/guide/03-effects.md
- Show environment setup patterns
- Explain read vs write distinction

### Architecture Updates

- Document IO module in DESIGN.md
- Explain AsRef pattern choice

## Implementation Notes

### Mutation Challenge

With `Effect::run(&Env)`, we can't take `&mut Env`. Options:

1. **Interior Mutability** (Recommended for MVP):
   ```rust
   struct Env {
       db: Arc<Mutex<Database>>,  // Thread-safe mutation
   }
   ```

2. **Separate Read/Write Environments**:
   - Defer to future improvement

3. **Read-Only for MVP**:
   - Accept that true mutation needs interior mutability

Decision: Document interior mutability pattern for MVP.

### Type Inference

Works great:
```rust
IO::read(|db: &Database| db.query())  // Infers Env: AsRef<Database>
```

Sometimes needs help:
```rust
IO::read::<Database, _, _, AppEnv>(|db| db.query())  // Explicit
```

### Zero-Cost

All helpers inline to direct function calls. No runtime overhead.

## Migration and Compatibility

No migration needed - this is a new feature.

## Open Questions

1. Should we provide helpers for common patterns like "try cache, fallback to DB"?
   - Decision: No, users can compose from IO::read primitives

2. Should write really take &mut or accept interior mutability?
   - Decision: Document interior mutability pattern, revisit if painful

3. Should we provide IO::from_env() for direct environment access?
   ```rust
   IO::from_env(|env: &AppEnv| env.config.database_url.clone())
   ```
   - Decision: Not needed, IO::read works fine

4. Should we support fallible I/O operations directly?
   ```rust
   IO::try_read(|db: &Database| db.find(id))  // Returns Effect<T, E, Env>
   ```
   - Decision: Add in helper combinators spec (Spec 006)
