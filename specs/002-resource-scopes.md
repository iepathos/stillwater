---
number: 2
title: Resource Scopes and Bracket Pattern
category: foundation
priority: high
status: draft
dependencies: []
created: 2025-01-24
---

# Specification 002: Resource Scopes and Bracket Pattern

**Category**: foundation
**Priority**: high
**Status**: draft
**Dependencies**: None (builds on existing Effect type)

## Context

Resource management in async Rust is fundamentally broken. The `Drop` trait works for synchronous cleanup, but async operations cannot be performed in `Drop`. This leads to:

1. **Leaked resources**: Database connections, file handles, network sockets left open
2. **Duplicate cleanup code**: Every error path needs manual cleanup
3. **Silent failures**: Cleanup errors are often ignored or cause panics
4. **Ordering bugs**: Resources released in wrong order causing deadlocks

### The Problem in Practice

```rust
// Typical async code - riddled with cleanup issues
async fn process_order(order_id: OrderId) -> Result<(), Error> {
    let conn = db_pool.acquire().await?;
    let lock = redis.lock(order_id).await?;

    // If this fails, neither conn nor lock are released!
    let order = fetch_order(&conn, order_id).await?;

    // More operations that might fail...
    validate_order(&order)?;

    let file = File::create(format!("/tmp/{}.json", order_id)).await?;
    // If THIS fails, conn and lock leak, file handle leaks

    serde_json::to_writer(&file, &order)?;

    // Manual cleanup - easy to forget, easy to get wrong
    file.sync_all().await?;
    drop(file);
    lock.release().await?;  // What if this fails?
    conn.close().await?;    // What if THIS fails?

    Ok(())
}
```

### Prior Art

- **Haskell's `bracket`**: `bracket acquire release use` - guarantees release runs
- **ZIO's `Scope`**: Structured resource management with automatic cleanup
- **Python's `with`/`async with`**: Context managers for resource cleanup
- **C#'s `using`/`await using`**: Deterministic disposal
- **Rust's `Drop`**: Only works for sync cleanup
- **scopeguard**: Sync-only scope guards
- **async-scoped**: Exists but not Effect-integrated, limited API

### Why This Matters for Stillwater

Stillwater's philosophy is **"pure core, imperative shell"**. Resource acquisition and cleanup are inherently effectful operations that belong at the shell. By modeling resource scopes as Effects, we:

1. **Guarantee cleanup**: Even on panic or early return
2. **Compose resources**: Acquire multiple resources safely
3. **Order cleanup correctly**: LIFO (last-in-first-out) release
4. **Handle cleanup errors**: Don't silently swallow them
5. **Integrate with Effect chains**: Natural composition

## Objective

Add resource scope management to stillwater that:

1. Guarantees cleanup runs even on failure/panic
2. Supports async acquisition and release
3. Composes multiple resources with correct ordering
4. Handles cleanup errors gracefully
5. Integrates naturally with Effect composition
6. Provides both scoped (bracket) and manual (Scope) APIs

## Requirements

### Functional Requirements

#### FR-1: Basic Bracket Pattern

```rust
// Acquire, use, release - release ALWAYS runs
let result = Effect::bracket(
    // Acquire
    open_database_connection(),
    // Release (receives the acquired resource)
    |conn| conn.close(),
    // Use (receives the acquired resource)
    |conn| {
        fetch_user(&conn, user_id)
            .and_then(|user| update_user(&conn, user))
    }
).run(&env).await;
```

#### FR-2: Multiple Resources with Correct Ordering

```rust
// Resources released in reverse order of acquisition
let result = Effect::bracket2(
    open_connection(),       // Acquired first
    open_file(path),         // Acquired second
    |conn| conn.close(),     // Released second
    |file| file.close(),     // Released first (LIFO)
    |conn, file| {
        // Use both resources
        process(&conn, &file)
    }
).run(&env).await;

// Or with arbitrary number of resources
let result = Effect::scoped(|scope| {
    let conn = scope.acquire(open_connection(), |c| c.close())?;
    let lock = scope.acquire(acquire_lock(id), |l| l.release())?;
    let file = scope.acquire(open_file(path), |f| f.close())?;

    // Use resources...
    process(&conn, &lock, &file)

    // Cleanup runs in reverse: file, lock, conn
}).run(&env).await;
```

#### FR-3: Resource Type with RAII-like Semantics

```rust
// Define a resource type that knows how to clean itself up
let db_resource = Resource::new(
    open_database_connection(),  // acquire effect
    |conn| conn.close()          // release function
);

// Use it with bracket
let result = db_resource.use_with(|conn| {
    fetch_user(&conn, user_id)
}).run(&env).await;

// Or acquire multiple
let result = Resource::all((db_resource, file_resource, lock_resource))
    .use_with(|(conn, file, lock)| {
        process(&conn, &file, &lock)
    })
    .run(&env).await;
```

#### FR-4: Scope Builder for Complex Resource Management

```rust
let result = Scope::new()
    .acquire("database", open_connection(), |c| c.close())
    .acquire("cache", open_redis(), |r| r.disconnect())
    .acquire_when(need_lock, "lock", acquire_lock(), |l| l.release())
    .run(|resources| {
        let conn = resources.get::<Connection>("database")?;
        let cache = resources.get::<Redis>("cache")?;

        // Use resources...
        process(conn, cache)
    })
    .run(&env).await;
```

#### FR-5: Cleanup Error Handling

```rust
// Option 1: Cleanup errors are logged but don't override use error
let result = Effect::bracket(
    acquire(),
    |r| r.close(),  // If this fails, error is logged
    |r| use_resource(r)  // This error is returned
).run(&env).await;

// Option 2: Explicit cleanup error handling
let result = Effect::bracket_with_cleanup_error(
    acquire(),
    |r| r.close(),
    |r| use_resource(r),
    |use_result, cleanup_result| {
        // Decide how to combine errors
        match (use_result, cleanup_result) {
            (Ok(v), Ok(())) => Ok(v),
            (Err(e), _) => Err(e),  // Prefer use error
            (Ok(_), Err(e)) => Err(e.into()),  // Cleanup error
        }
    }
).run(&env).await;

// Option 3: Collect all errors
let result = Effect::bracket_collecting(
    acquire(),
    |r| r.close(),
    |r| use_resource(r)
).run(&env).await;
// Returns Result<T, BracketError<E>> where BracketError contains both
```

#### FR-6: Async Release Functions

```rust
// Release can be async
let result = Effect::bracket(
    open_connection(),
    |conn| async move {
        conn.flush().await?;
        conn.close().await
    },
    |conn| fetch_data(&conn)
).run(&env).await;
```

#### FR-7: Conditional Resource Acquisition

```rust
let result = Effect::scoped(|scope| {
    let conn = scope.acquire(open_connection(), |c| c.close())?;

    // Only acquire lock if needed
    let lock = if needs_lock {
        Some(scope.acquire(acquire_lock(), |l| l.release())?)
    } else {
        None
    };

    process(&conn, lock.as_ref())
}).run(&env).await;
```

#### FR-8: Nested Scopes

```rust
let result = Effect::scoped(|outer| {
    let conn = outer.acquire(open_connection(), |c| c.close())?;

    // Inner scope for temporary resources
    Effect::scoped(|inner| {
        let temp_file = inner.acquire(create_temp_file(), |f| f.delete())?;

        export_to_file(&conn, &temp_file)?;
        upload_file(&temp_file)

        // temp_file deleted here
    })?;

    // conn still available here
    finalize(&conn)

    // conn closed here
}).run(&env).await;
```

### Non-Functional Requirements

#### NFR-1: Guaranteed Cleanup

- Cleanup MUST run even if use function panics
- Cleanup MUST run even if use function returns early
- Cleanup MUST run in correct order (LIFO)

#### NFR-2: Minimal Overhead

- No allocation for simple bracket with single resource
- Scope should use small-vec optimization for typical 1-4 resources

#### NFR-3: Clear Error Messages

- Cleanup failures should include resource identifier
- Stack of cleanup errors when multiple cleanups fail

#### NFR-4: Cancellation Safety

- Resources must be cleaned up if the future is dropped/cancelled
- Use `tokio::select!` safely with scoped resources

## Acceptance Criteria

- [ ] `Effect::bracket` for single resource acquire/use/release
- [ ] `Effect::bracket2`, `Effect::bracket3` for multiple resources
- [ ] `Effect::scoped` for dynamic resource acquisition
- [ ] `Resource<T>` type representing an acquirable resource
- [ ] `Scope` builder for complex resource management
- [ ] Async release functions supported
- [ ] Cleanup errors handled (logged by default, configurable)
- [ ] LIFO cleanup ordering guaranteed
- [ ] Works with `tokio::select!` (cancellation safe)
- [ ] Cleanup runs on panic (using `std::panic::catch_unwind` or similar)
- [ ] Comprehensive unit tests
- [ ] Integration tests with real async resources
- [ ] Documentation with examples
- [ ] Example file: `examples/resource_scopes.rs`

## Technical Details

### Implementation Approach

#### Core Types

```rust
/// A resource that can be acquired and must be released.
///
/// `Resource<T, E, Env>` represents a pattern for safely managing
/// resources with guaranteed cleanup.
pub struct Resource<T, E, Env> {
    acquire: Effect<T, E, Env>,
    release: Box<dyn FnOnce(T) -> BoxFuture<'static, Result<(), E>> + Send>,
}

impl<T, E, Env> Resource<T, E, Env>
where
    T: Send + 'static,
    E: Send + 'static,
    Env: Sync + 'static,
{
    /// Create a new resource with acquire effect and release function.
    pub fn new<F, Fut>(acquire: Effect<T, E, Env>, release: F) -> Self
    where
        F: FnOnce(T) -> Fut + Send + 'static,
        Fut: Future<Output = Result<(), E>> + Send + 'static,
    {
        Resource {
            acquire,
            release: Box::new(move |t| Box::pin(release(t))),
        }
    }

    /// Use this resource with a function, guaranteeing cleanup.
    pub fn use_with<U, F>(self, f: F) -> Effect<U, E, Env>
    where
        F: FnOnce(&T) -> Effect<U, E, Env> + Send + 'static,
        U: Send + 'static,
    {
        Effect::bracket(self.acquire, self.release, f)
    }
}

/// A scope for managing multiple resources with guaranteed cleanup.
pub struct Scope<'env, E, Env> {
    resources: Vec<Box<dyn FnOnce() -> BoxFuture<'static, Result<(), E>> + Send>>,
    env: &'env Env,
}

impl<'env, E, Env> Scope<'env, E, Env>
where
    E: Send + 'static,
    Env: Sync + 'static,
{
    /// Acquire a resource within this scope.
    ///
    /// The resource will be released when the scope exits,
    /// in reverse order of acquisition.
    pub async fn acquire<T, F, Fut>(
        &mut self,
        acquire: Effect<T, E, Env>,
        release: F,
    ) -> Result<T, E>
    where
        T: Send + 'static,
        F: FnOnce(T) -> Fut + Send + 'static,
        Fut: Future<Output = Result<(), E>> + Send + 'static,
    {
        let value = acquire.run(self.env).await?;

        // Store cleanup function
        // Note: We need to clone/move the value for cleanup
        // This requires T: Clone or some form of shared ownership
        // See Implementation Notes for solutions

        Ok(value)
    }
}

/// Error type for bracket operations that includes cleanup errors.
#[derive(Debug)]
pub struct BracketError<E> {
    /// The error from the use function, if any.
    pub use_error: Option<E>,
    /// Errors from cleanup, in order they occurred.
    pub cleanup_errors: Vec<CleanupError<E>>,
}

#[derive(Debug)]
pub struct CleanupError<E> {
    /// Identifier for the resource that failed to clean up.
    pub resource_id: Option<String>,
    /// The actual error.
    pub error: E,
}
```

#### Bracket Implementation

```rust
impl<T, E, Env> Effect<T, E, Env>
where
    T: Send + 'static,
    E: Send + 'static,
    Env: Sync + 'static,
{
    /// Acquire a resource, use it, and guarantee cleanup.
    ///
    /// The release function is called even if the use function fails.
    /// If both use and release fail, the use error is returned and
    /// the release error is logged.
    ///
    /// # Examples
    ///
    /// ```
    /// use stillwater::Effect;
    ///
    /// async fn example() {
    ///     let result = Effect::bracket(
    ///         Effect::pure(Connection::new()),  // acquire
    ///         |conn| async move { conn.close().await },  // release
    ///         |conn| Effect::pure(conn.query("SELECT 1")),  // use
    ///     ).run(&()).await;
    /// }
    /// ```
    pub fn bracket<R, U, Rel, RelFut, Use>(
        acquire: Effect<R, E, Env>,
        release: Rel,
        use_fn: Use,
    ) -> Effect<U, E, Env>
    where
        R: Send + 'static,
        U: Send + 'static,
        Rel: FnOnce(R) -> RelFut + Send + 'static,
        RelFut: Future<Output = Result<(), E>> + Send + 'static,
        Use: FnOnce(&R) -> Effect<U, E, Env> + Send + 'static,
    {
        Effect::from_async(move |env| async move {
            // Acquire resource
            let resource = acquire.run(env).await?;

            // Use resource, catching any errors
            let use_result = use_fn(&resource).run(env).await;

            // Release resource (always runs)
            let release_result = release(resource).await;

            // Handle errors
            match (use_result, release_result) {
                (Ok(value), Ok(())) => Ok(value),
                (Err(e), Ok(())) => Err(e),
                (Ok(value), Err(rel_err)) => {
                    // Log release error, return success
                    tracing::error!("Resource cleanup failed: {:?}", rel_err);
                    Ok(value)
                }
                (Err(use_err), Err(rel_err)) => {
                    // Log release error, return use error
                    tracing::error!("Resource cleanup failed: {:?}", rel_err);
                    Err(use_err)
                }
            }
        })
    }

    /// Bracket with two resources, released in reverse order.
    pub fn bracket2<R1, R2, U, Rel1, RelFut1, Rel2, RelFut2, Use>(
        acquire1: Effect<R1, E, Env>,
        acquire2: Effect<R2, E, Env>,
        release1: Rel1,
        release2: Rel2,
        use_fn: Use,
    ) -> Effect<U, E, Env>
    where
        R1: Send + 'static,
        R2: Send + 'static,
        U: Send + 'static,
        Rel1: FnOnce(R1) -> RelFut1 + Send + 'static,
        RelFut1: Future<Output = Result<(), E>> + Send + 'static,
        Rel2: FnOnce(R2) -> RelFut2 + Send + 'static,
        RelFut2: Future<Output = Result<(), E>> + Send + 'static,
        Use: FnOnce(&R1, &R2) -> Effect<U, E, Env> + Send + 'static,
    {
        Effect::bracket(
            acquire1,
            release1,
            move |r1| {
                Effect::bracket(
                    acquire2,
                    release2,
                    move |r2| use_fn(r1, r2),
                )
            },
        )
    }

    /// Run a function with a managed scope for resource acquisition.
    ///
    /// Resources acquired within the scope are automatically released
    /// when the scope exits, in reverse order of acquisition.
    pub fn scoped<F, U>(f: F) -> Effect<U, E, Env>
    where
        F: FnOnce(&mut ScopeGuard<E, Env>) -> Effect<U, E, Env> + Send + 'static,
        U: Send + 'static,
    {
        Effect::from_async(move |env| async move {
            let mut scope = ScopeGuard::new(env);

            // Run the user function
            let result = f(&mut scope).run(env).await;

            // Cleanup all resources in reverse order
            let cleanup_errors = scope.cleanup().await;

            // Handle errors
            if !cleanup_errors.is_empty() {
                for err in &cleanup_errors {
                    tracing::error!("Resource cleanup failed: {:?}", err);
                }
            }

            result
        })
    }
}
```

### Architecture Changes

New module structure:

```
src/
├── lib.rs           # Re-export resource types
├── effect.rs        # Add bracket methods
├── resource/
│   ├── mod.rs       # Module root, re-exports
│   ├── bracket.rs   # bracket, bracket2, bracket3 implementations
│   ├── scope.rs     # Scope and ScopeGuard implementations
│   ├── resource.rs  # Resource<T, E, Env> type
│   └── error.rs     # BracketError, CleanupError
```

### Data Structures

```rust
/// A scope guard that tracks resources for cleanup.
pub struct ScopeGuard<'env, E, Env> {
    env: &'env Env,
    // Cleanup functions stored in acquisition order
    // Executed in reverse order
    cleanups: Vec<CleanupFn<E>>,
}

type CleanupFn<E> = Box<dyn FnOnce() -> BoxFuture<'static, Result<(), E>> + Send>;

/// Builder for complex resource scopes.
pub struct ScopeBuilder<E, Env> {
    resources: Vec<ResourceDef<E, Env>>,
}

struct ResourceDef<E, Env> {
    name: &'static str,
    acquire: Box<dyn FnOnce(&Env) -> BoxFuture<'_, Result<Box<dyn Any + Send>, E>> + Send>,
    release: Box<dyn FnOnce(Box<dyn Any + Send>) -> BoxFuture<'static, Result<(), E>> + Send>,
    condition: Option<Box<dyn Fn(&Env) -> bool + Send + Sync>>,
}
```

### APIs and Interfaces

#### Effect Extensions

```rust
impl<T, E, Env> Effect<T, E, Env> {
    // Single resource
    pub fn bracket<R, U, Rel, RelFut, Use>(
        acquire: Effect<R, E, Env>,
        release: Rel,
        use_fn: Use,
    ) -> Effect<U, E, Env>;

    // Two resources
    pub fn bracket2<R1, R2, U, ...>(...) -> Effect<U, E, Env>;

    // Three resources
    pub fn bracket3<R1, R2, R3, U, ...>(...) -> Effect<U, E, Env>;

    // Dynamic scope
    pub fn scoped<F, U>(f: F) -> Effect<U, E, Env>;

    // With explicit error handling
    pub fn bracket_with<R, U, Rel, RelFut, Use, ErrHandler>(
        acquire: Effect<R, E, Env>,
        release: Rel,
        use_fn: Use,
        error_handler: ErrHandler,
    ) -> Effect<U, BracketError<E>, Env>;
}
```

#### Resource Type

```rust
impl<T, E, Env> Resource<T, E, Env> {
    pub fn new<F, Fut>(acquire: Effect<T, E, Env>, release: F) -> Self;
    pub fn use_with<U, F>(self, f: F) -> Effect<U, E, Env>;
    pub fn map_acquire<U, F>(self, f: F) -> Resource<U, E, Env>;
    pub fn map_error<E2, F>(self, f: F) -> Resource<T, E2, Env>;
}

impl<E, Env> Resource<(), E, Env> {
    /// Combine multiple resources into one.
    pub fn all<R1, R2>(r1: Resource<R1, E, Env>, r2: Resource<R2, E, Env>)
        -> Resource<(R1, R2), E, Env>;
}
```

#### ScopeGuard

```rust
impl<'env, E, Env> ScopeGuard<'env, E, Env> {
    /// Acquire a resource, registering it for cleanup.
    pub async fn acquire<T, F, Fut>(
        &mut self,
        effect: Effect<T, E, Env>,
        release: F,
    ) -> Result<T, E>;

    /// Acquire with a name for error reporting.
    pub async fn acquire_named<T, F, Fut>(
        &mut self,
        name: &'static str,
        effect: Effect<T, E, Env>,
        release: F,
    ) -> Result<T, E>;
}
```

## Dependencies

- **Prerequisites**: None
- **Affected Components**:
  - `effect.rs` - Add bracket methods
  - `lib.rs` - Re-export resource types
- **External Dependencies**:
  - `tracing` (existing, for logging cleanup errors)
  - `tokio` (existing, for async)

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_bracket_releases_on_success() {
        let released = Arc::new(AtomicBool::new(false));
        let released_clone = released.clone();

        let result = Effect::bracket(
            Effect::<_, String, ()>::pure(42),
            move |_| {
                released_clone.store(true, Ordering::SeqCst);
                async { Ok(()) }
            },
            |val| Effect::pure(*val * 2),
        )
        .run(&())
        .await;

        assert_eq!(result, Ok(84));
        assert!(released.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_bracket_releases_on_failure() {
        let released = Arc::new(AtomicBool::new(false));
        let released_clone = released.clone();

        let result = Effect::<i32, String, ()>::bracket(
            Effect::pure(42),
            move |_| {
                released_clone.store(true, Ordering::SeqCst);
                async { Ok(()) }
            },
            |_| Effect::fail("use failed".to_string()),
        )
        .run(&())
        .await;

        assert_eq!(result, Err("use failed".to_string()));
        assert!(released.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_bracket2_releases_in_reverse_order() {
        let order = Arc::new(Mutex::new(Vec::new()));
        let order1 = order.clone();
        let order2 = order.clone();

        let result = Effect::bracket2(
            Effect::<_, String, ()>::pure("first"),
            Effect::pure("second"),
            move |_| {
                order1.lock().unwrap().push("release_first");
                async { Ok(()) }
            },
            move |_| {
                order2.lock().unwrap().push("release_second");
                async { Ok(()) }
            },
            |_, _| Effect::pure("done"),
        )
        .run(&())
        .await;

        assert_eq!(result, Ok("done"));
        assert_eq!(*order.lock().unwrap(), vec!["release_second", "release_first"]);
    }

    #[tokio::test]
    async fn test_scoped_cleanup_on_early_return() {
        let cleanup_count = Arc::new(AtomicU32::new(0));
        let count = cleanup_count.clone();

        let result = Effect::<_, String, ()>::scoped(|scope| {
            let count = count.clone();
            async move {
                scope.acquire(
                    Effect::pure(1),
                    move |_| {
                        count.fetch_add(1, Ordering::SeqCst);
                        async { Ok(()) }
                    },
                ).await?;

                // Early return
                return Err("early exit".to_string());

                #[allow(unreachable_code)]
                Ok(())
            }
        })
        .run(&())
        .await;

        assert!(result.is_err());
        assert_eq!(cleanup_count.load(Ordering::SeqCst), 1);
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_real_file_cleanup() {
    let temp_path = "/tmp/stillwater_test_file.txt";

    let result = Effect::bracket(
        Effect::from_async(|_: &()| async {
            tokio::fs::File::create(temp_path).await
                .map_err(|e| e.to_string())
        }),
        |file| async move {
            drop(file);
            tokio::fs::remove_file(temp_path).await
                .map_err(|e| e.to_string())
        },
        |file| Effect::from_async(move |_| async move {
            use tokio::io::AsyncWriteExt;
            let mut f = file;
            f.write_all(b"test data").await
                .map_err(|e| e.to_string())
        }),
    )
    .run(&())
    .await;

    assert!(result.is_ok());
    assert!(!std::path::Path::new(temp_path).exists());
}

#[tokio::test]
async fn test_database_connection_cleanup() {
    // Integration test with real database
}
```

### Property-Based Tests

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn cleanup_always_runs(
        use_succeeds: bool,
        cleanup_succeeds: bool
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let cleanup_ran = Arc::new(AtomicBool::new(false));
        let cleanup_ran_clone = cleanup_ran.clone();

        let result = rt.block_on(async {
            Effect::<_, String, ()>::bracket(
                Effect::pure(()),
                move |_| {
                    cleanup_ran_clone.store(true, Ordering::SeqCst);
                    async move {
                        if cleanup_succeeds { Ok(()) } else { Err("cleanup failed".to_string()) }
                    }
                },
                move |_| {
                    if use_succeeds {
                        Effect::pure(())
                    } else {
                        Effect::fail("use failed".to_string())
                    }
                },
            )
            .run(&())
            .await
        });

        // Cleanup should ALWAYS run
        prop_assert!(cleanup_ran.load(Ordering::SeqCst));
    }
}
```

## Documentation Requirements

### Code Documentation
- Comprehensive rustdoc for all public types
- Examples showing common patterns
- Safety documentation for panic handling

### User Documentation
- Add "Resource Management" chapter to user guide
- Document when to use bracket vs scoped
- Provide migration guide from manual cleanup

### Architecture Updates
- Document resource module in DESIGN.md
- Explain panic safety guarantees

## Implementation Notes

### Handling Resource Ownership in Scope

The `ScopeGuard::acquire` method has a challenge: we need to both return the resource to the user AND keep it for cleanup. Solutions:

**Option A: Clone requirement**
```rust
pub async fn acquire<T: Clone, ...>(...) -> Result<T, E>
```
Requires T: Clone, limiting usability.

**Option B: Shared ownership**
```rust
pub async fn acquire<T, ...>(...) -> Result<Arc<T>, E>
```
Returns Arc<T>, user works with shared reference.

**Option C: Split resource**
```rust
pub async fn acquire<T, ...>(...) -> Result<(T, CleanupHandle), E>
```
Returns resource and separate cleanup handle.

**Recommendation**: Option B for simplicity, with Option C available for advanced cases.

### Panic Safety

To ensure cleanup runs even on panic:

```rust
pub fn bracket<...>(...) -> Effect<U, E, Env> {
    Effect::from_async(move |env| async move {
        let resource = acquire.run(env).await?;

        // Use AssertUnwindSafe or catch_unwind
        let use_result = std::panic::AssertUnwindSafe(use_fn(&resource).run(env))
            .catch_unwind()
            .await;

        // Always release
        let release_result = release(resource).await;

        match use_result {
            Ok(Ok(value)) => { /* ... */ }
            Ok(Err(e)) => { /* ... */ }
            Err(panic) => {
                // Log release result, re-panic
                std::panic::resume_unwind(panic);
            }
        }
    })
}
```

### Cancellation Safety

When used with `tokio::select!`, the future might be dropped mid-execution. We need to ensure cleanup still runs:

```rust
// Use tokio's on_cancel or similar pattern
let guard = scopeguard::guard(resource, |r| {
    // Spawn cleanup task that will run even if this future is dropped
    tokio::spawn(async move { release(r).await });
});
```

This is complex and may require a dedicated cancellation-safe variant.

## Migration and Compatibility

### Breaking Changes
None - this is a purely additive feature.

### Feature Flags
```toml
[features]
default = ["resource"]
resource = []  # Resource scope functionality
```

### Deprecations
None.

## Future Considerations (Out of Scope)

These are explicitly NOT part of this specification:

1. **Resource Pools**: Managed pools of reusable resources (use deadpool/bb8)
2. **Distributed Resources**: Resources across multiple machines
3. **Resource Limits**: Limiting concurrent resource acquisition
4. **Resource Metrics**: Tracking resource usage statistics

---

*"Acquire, use, release. Always release. Even when everything goes wrong."*
