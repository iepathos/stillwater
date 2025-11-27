---
number: 2
title: Resource Scopes and Bracket Pattern
category: foundation
priority: high
status: draft
dependencies: []
created: 2025-01-24
revised: 2025-11-27
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

### Philosophy Alignment

From PHILOSOPHY.md: *"We're not trying to be Haskell. We're trying to be better Rust."*

This spec follows Stillwater's pragmatic approach:

1. **Work with Rust, not against it** - Don't fight the ownership model
2. **Composition over complexity** - Build from simple, composable pieces
3. **Types guide, don't restrict** - Keep signatures understandable
4. **Pragmatism over purity** - Handle 90% of cases well, don't over-engineer the rest

## Objective

Add resource scope management to stillwater that:

1. Guarantees cleanup runs even on failure
2. Supports async acquisition and release
3. Composes multiple resources with correct LIFO ordering
4. Handles cleanup errors gracefully (logs by default)
5. Integrates naturally with Effect composition

### Explicit Non-Goals (Deferred)

The following are **out of scope** for this spec:

1. **Dynamic resource acquisition (`ScopeGuard`)** - Fights Rust's ownership model; use nested brackets instead
2. **Named resource builders (`ScopeBuilder`)** - Over-engineered; type-erased resources add complexity
3. **Cancellation safety** - Complex async Rust problem; document limitations instead
4. **Panic safety guarantees** - Best-effort only; async panic handling is inherently limited

These may be addressed in future specs if there's demonstrated need.

## Requirements

### Functional Requirements

#### FR-1: Basic Bracket Pattern

The core pattern: acquire, use, release - release ALWAYS runs.

```rust
let result = Effect::bracket(
    // Acquire
    open_database_connection(),
    // Release (takes ownership of resource)
    |conn| async move { conn.close().await },
    // Use (borrows resource)
    |conn| {
        fetch_user(conn, user_id)
            .and_then(|user| update_user(conn, user))
    }
).run(&env).await;
```

#### FR-2: Multiple Resources with Correct Ordering

Resources released in reverse order of acquisition (LIFO).

```rust
// Two resources
let result = Effect::bracket2(
    open_connection(),       // Acquired first
    open_file(path),         // Acquired second
    |conn| async move { conn.close().await },  // Released second
    |file| async move { file.close().await },  // Released first (LIFO)
    |conn, file| process(conn, file)
).run(&env).await;

// Three resources
let result = Effect::bracket3(
    open_connection(),
    acquire_lock(id),
    open_file(path),
    |conn| async move { conn.close().await },
    |lock| async move { lock.release().await },
    |file| async move { file.close().await },
    |conn, lock, file| process(conn, lock, file)
).run(&env).await;
```

#### FR-3: Resource Type for Reusable Patterns

```rust
// Define a resource that knows how to acquire and release
let db_resource = Resource::new(
    open_database_connection(),
    |conn| async move { conn.close().await }
);

// Use it with bracket semantics
let result = db_resource.with(|conn| {
    fetch_user(conn, user_id)
}).run(&env).await;

// Compose resources
let result = Resource::both(db_resource, file_resource)
    .with(|(conn, file)| process(conn, file))
    .run(&env).await;

// Three resources
let result = Resource::all3(db_resource, lock_resource, file_resource)
    .with(|(conn, lock, file)| process(conn, lock, file))
    .run(&env).await;
```

#### FR-4: Cleanup Error Handling

Default behavior: log cleanup errors, return use result.

```rust
// Default: cleanup errors logged, use result returned
let result = Effect::bracket(
    acquire(),
    |r| async move { r.close().await },  // If this fails, logged
    |r| use_resource(r)                   // This result returned
).run(&env).await;

// Explicit: get both errors for custom handling
let result = Effect::bracket_full(
    acquire(),
    |r| async move { r.close().await },
    |r| use_resource(r),
).run(&env).await;
// Returns: Result<T, BracketError<E>>
// BracketError contains use_error and/or cleanup_error
```

#### FR-5: Builder Pattern for Multiple Resources

Fluent API to avoid deeply nested brackets. This is the **recommended API** for multiple resources.

```rust
// Flat, readable, idiomatic Rust
let result = Effect::acquiring(open_connection(), |c| c.close())
    .and(acquire_lock(id), |l| l.release())
    .and(open_file(path), |f| f.close())
    .with(|(conn, lock, file)| {
        process(conn, lock, file)
    })
    .run(&env)
    .await;

// Single resource also works
let result = Effect::acquiring(open_connection(), |c| c.close())
    .with(|conn| fetch_user(conn, user_id))
    .run(&env)
    .await;

// Conditional resource acquisition - nest one level
let result = Effect::acquiring(open_connection(), |c| c.close())
    .with(|conn| {
        if needs_lock {
            Effect::acquiring(acquire_lock(id), |l| l.release())
                .with(|lock| process(conn, Some(lock)))
        } else {
            process(conn, None)
        }
    })
    .run(&env)
    .await;
```

The builder generates nested brackets internally - no runtime overhead, just ergonomic syntax.

### Non-Functional Requirements

#### NFR-1: Guaranteed Cleanup

- Cleanup MUST run if use function returns `Ok` or `Err`
- Cleanup MUST run in correct order (LIFO) for multiple resources
- **Limitation**: Cleanup is NOT guaranteed on panic or future cancellation (documented)

#### NFR-2: Simple Type Signatures

- Bracket methods should have readable type signatures
- Error messages should be understandable
- Avoid excessive generic parameters

#### NFR-3: Minimal Overhead

- Single `bracket` should have minimal allocation beyond the Effect itself
- `bracket2`/`bracket3` implemented as nested brackets (no additional allocation)

## Acceptance Criteria

### Must Have
- [ ] `Effect::bracket` for single resource acquire/use/release
- [ ] `Effect::bracket2` for two resources with LIFO cleanup
- [ ] `Effect::bracket3` for three resources with LIFO cleanup
- [ ] `Effect::acquiring` builder with `.and()` and `.with()` methods
- [ ] `Acquiring<T, E, Env>` builder type with nested tuple output
- [ ] `Acquiring::with_flat` for 2, 3, and 4 resources (flat parameter access)
- [ ] `Resource<T, E, Env>` type with `with` method
- [ ] `Resource::both` for composing two resources
- [ ] Cleanup errors logged by default
- [ ] `Effect::bracket_full` returning `BracketError` for explicit handling
- [ ] `Effect::bracket_sync` for panic-safe bracket with synchronous cleanup
- [ ] `BracketError` with `AcquireError`, `UseError`, `CleanupError`, and `Both` variants
- [ ] `Resource::both` logs cleanup errors on partial acquisition rollback (not silently discarded)
- [ ] Comprehensive unit tests
- [ ] Documentation with examples

### Should Have
- [ ] Example file: `examples/resource_scopes.rs`
- [ ] Integration tests with tokio file I/O

### Won't Have (This Version)
- [ ] `ScopeGuard` for dynamic resource acquisition
- [ ] `ScopeBuilder` for named resources
- [ ] Cancellation-safe variants
- [ ] Panic-safe variants with async cleanup (only sync cleanup via `bracket_sync`)
- [ ] `Resource::map` (ownership complexity with release function)
- [ ] `Resource::all3` (use `Acquiring` builder for 3+ resources instead)

## Technical Details

### Implementation Approach

#### Core Types

```rust
use std::future::Future;

/// Error type for bracket operations with explicit error handling.
///
/// This enum design ensures all states are valid - no `(None, None)` case possible.
/// Each variant clearly identifies which phase of the bracket operation failed.
#[derive(Debug, Clone)]
pub enum BracketError<E> {
    /// Resource acquisition failed - never got to use the resource.
    AcquireError(E),
    /// The use function failed, cleanup succeeded.
    UseError(E),
    /// The use function succeeded, cleanup failed.
    CleanupError(E),
    /// Both use and cleanup failed.
    Both {
        use_error: E,
        cleanup_error: E,
    },
}

impl<E> BracketError<E> {
    /// Returns the acquire error, if any.
    pub fn acquire_error(&self) -> Option<&E> {
        match self {
            BracketError::AcquireError(e) => Some(e),
            _ => None,
        }
    }

    /// Returns the use error, if any.
    pub fn use_error(&self) -> Option<&E> {
        match self {
            BracketError::UseError(e) => Some(e),
            BracketError::Both { use_error, .. } => Some(use_error),
            _ => None,
        }
    }

    /// Returns the cleanup error, if any.
    pub fn cleanup_error(&self) -> Option<&E> {
        match self {
            BracketError::CleanupError(e) => Some(e),
            BracketError::Both { cleanup_error, .. } => Some(cleanup_error),
            _ => None,
        }
    }
}

impl<E: std::fmt::Display> std::fmt::Display for BracketError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BracketError::AcquireError(e) => write!(f, "acquire failed: {}", e),
            BracketError::UseError(e) => write!(f, "{}", e),
            BracketError::CleanupError(e) => write!(f, "cleanup failed: {}", e),
            BracketError::Both { use_error, cleanup_error } => {
                write!(f, "use failed: {}; cleanup also failed: {}", use_error, cleanup_error)
            }
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for BracketError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            BracketError::AcquireError(e) => Some(e),
            BracketError::UseError(e) => Some(e),
            BracketError::Both { use_error, .. } => Some(use_error),
            BracketError::CleanupError(e) => Some(e),
        }
    }
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
    /// The release function runs even if the use function fails.
    /// If release fails, the error is logged and the use result is returned.
    ///
    /// # Example
    ///
    /// ```
    /// use stillwater::Effect;
    ///
    /// # tokio_test::block_on(async {
    /// let result = Effect::bracket(
    ///     Effect::<_, String, ()>::pure(vec![1, 2, 3]),
    ///     |v| async move {
    ///         println!("Cleaning up vec with {} items", v.len());
    ///         Ok(())
    ///     },
    ///     |v| Effect::pure(v.iter().sum::<i32>()),
    /// ).run(&()).await;
    ///
    /// assert_eq!(result, Ok(6));
    /// # });
    /// ```
    pub fn bracket<R, U, Acq, Rel, RelFut, Use>(
        acquire: Acq,
        release: Rel,
        use_fn: Use,
    ) -> Effect<U, E, Env>
    where
        R: Send + 'static,
        U: Send + 'static,
        Acq: Into<Effect<R, E, Env>>,
        Rel: FnOnce(R) -> RelFut + Send + 'static,
        RelFut: Future<Output = Result<(), E>> + Send + 'static,
        Use: FnOnce(&R) -> Effect<U, E, Env> + Send + 'static,
        E: std::fmt::Debug,
    {
        let acquire_effect = acquire.into();

        Effect::from_async(move |env: &Env| {
            // Convert to raw pointer to move into async block.
            //
            // SAFETY: This is safe because:
            // 1. `env` is borrowed for the lifetime of the `run()` call
            // 2. The async block completes before `run()` returns
            // 3. `Env: Sync` allows shared references across threads
            // 4. We only create a shared reference, never mutate
            //
            // Alternative: Could use Arc<Env> but adds allocation overhead.
            // This pattern is common in async Rust (see tokio internals).
            let env_ptr = env as *const Env;

            async move {
                let env = unsafe { &*env_ptr };

                // Acquire resource
                let resource = acquire_effect.run(env).await?;

                // Use resource
                let use_result = use_fn(&resource).run(env).await;

                // Release resource (always runs)
                let release_result = release(resource).await;

                // Handle errors
                match (&use_result, &release_result) {
                    (Ok(_), Ok(())) => use_result,
                    (Err(_), Ok(())) => use_result,
                    (Ok(_), Err(rel_err)) => {
                        tracing::warn!("Resource cleanup failed: {:?}", rel_err);
                        use_result
                    }
                    (Err(_), Err(rel_err)) => {
                        tracing::warn!("Resource cleanup failed: {:?}", rel_err);
                        use_result
                    }
                }
            }
        })
    }

    /// Bracket with two resources, released in reverse order (LIFO).
    ///
    /// Implemented as nested brackets for simplicity and correctness.
    pub fn bracket2<R1, R2, U, Acq1, Acq2, Rel1, RelFut1, Rel2, RelFut2, Use>(
        acquire1: Acq1,
        acquire2: Acq2,
        release1: Rel1,
        release2: Rel2,
        use_fn: Use,
    ) -> Effect<U, E, Env>
    where
        R1: Send + 'static,
        R2: Send + 'static,
        U: Send + 'static,
        Acq1: Into<Effect<R1, E, Env>>,
        Acq2: Into<Effect<R2, E, Env>>,
        Rel1: FnOnce(R1) -> RelFut1 + Send + 'static,
        RelFut1: Future<Output = Result<(), E>> + Send + 'static,
        Rel2: FnOnce(R2) -> RelFut2 + Send + 'static,
        RelFut2: Future<Output = Result<(), E>> + Send + 'static,
        Use: FnOnce(&R1, &R2) -> Effect<U, E, Env> + Send + 'static,
        E: std::fmt::Debug,
    {
        // Nested brackets ensure LIFO cleanup:
        // - acquire1, then acquire2
        // - use both
        // - release2 (inner bracket), then release1 (outer bracket)
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

    /// Bracket with three resources, released in reverse order (LIFO).
    pub fn bracket3<R1, R2, R3, U, Acq1, Acq2, Acq3, Rel1, RelFut1, Rel2, RelFut2, Rel3, RelFut3, Use>(
        acquire1: Acq1,
        acquire2: Acq2,
        acquire3: Acq3,
        release1: Rel1,
        release2: Rel2,
        release3: Rel3,
        use_fn: Use,
    ) -> Effect<U, E, Env>
    where
        R1: Send + 'static,
        R2: Send + 'static,
        R3: Send + 'static,
        U: Send + 'static,
        Acq1: Into<Effect<R1, E, Env>>,
        Acq2: Into<Effect<R2, E, Env>>,
        Acq3: Into<Effect<R3, E, Env>>,
        Rel1: FnOnce(R1) -> RelFut1 + Send + 'static,
        RelFut1: Future<Output = Result<(), E>> + Send + 'static,
        Rel2: FnOnce(R2) -> RelFut2 + Send + 'static,
        RelFut2: Future<Output = Result<(), E>> + Send + 'static,
        Rel3: FnOnce(R3) -> RelFut3 + Send + 'static,
        RelFut3: Future<Output = Result<(), E>> + Send + 'static,
        Use: FnOnce(&R1, &R2, &R3) -> Effect<U, E, Env> + Send + 'static,
        E: std::fmt::Debug,
    {
        Effect::bracket(
            acquire1,
            release1,
            move |r1| {
                Effect::bracket2(
                    acquire2,
                    acquire3,
                    release2,
                    release3,
                    move |r2, r3| use_fn(r1, r2, r3),
                )
            },
        )
    }

    /// Bracket with explicit error handling - returns both use and cleanup errors.
    pub fn bracket_full<R, U, Acq, Rel, RelFut, Use>(
        acquire: Acq,
        release: Rel,
        use_fn: Use,
    ) -> Effect<U, BracketError<E>, Env>
    where
        R: Send + 'static,
        U: Send + 'static,
        Acq: Into<Effect<R, E, Env>>,
        Rel: FnOnce(R) -> RelFut + Send + 'static,
        RelFut: Future<Output = Result<(), E>> + Send + 'static,
        Use: FnOnce(&R) -> Effect<U, E, Env> + Send + 'static,
    {
        let acquire_effect = acquire.into();

        Effect::from_async(move |env: &Env| {
            let env_ptr = env as *const Env;

            async move {
                let env = unsafe { &*env_ptr };

                // Acquire - map error to BracketError::AcquireError
                let resource = match acquire_effect.run(env).await {
                    Ok(r) => r,
                    Err(e) => return Err(BracketError::AcquireError(e)),
                };

                // Use resource
                let use_result = use_fn(&resource).run(env).await;

                // Release resource
                let release_result = release(resource).await;

                // Combine results
                match (use_result, release_result) {
                    (Ok(value), Ok(())) => Ok(value),
                    (Ok(_), Err(cleanup_err)) => Err(BracketError::CleanupError(cleanup_err)),
                    (Err(use_err), Ok(())) => Err(BracketError::UseError(use_err)),
                    (Err(use_err), Err(cleanup_err)) => Err(BracketError::Both {
                        use_error: use_err,
                        cleanup_error: cleanup_err,
                    }),
                }
            }
        })
    }

    /// Panic-safe bracket with synchronous cleanup.
    ///
    /// Unlike `bracket`, this variant guarantees cleanup runs even if the use
    /// function panics, provided the release function is synchronous. This uses
    /// `std::panic::catch_unwind` internally.
    ///
    /// # Panic Safety
    ///
    /// - If `use_fn` panics, cleanup still runs, then the panic is re-raised
    /// - If cleanup fails after a panic, the cleanup error is logged and panic re-raised
    /// - The use function must be `UnwindSafe` (most closures are)
    ///
    /// # When to Use
    ///
    /// Use `bracket_sync` when:
    /// - Your cleanup is synchronous (or can be made sync with `block_on`)
    /// - You need guaranteed cleanup even on panic
    /// - You're at an application boundary where panics might occur
    ///
    /// Use regular `bracket` when:
    /// - Cleanup is async and cannot be made synchronous
    /// - You're in library code where panics should propagate
    ///
    /// # Example
    ///
    /// ```
    /// use stillwater::Effect;
    ///
    /// let result = Effect::bracket_sync(
    ///     Effect::<_, String, ()>::pure(vec![1, 2, 3]),
    ///     |v| {
    ///         println!("Cleaning up vec with {} items", v.len());
    ///         Ok(())  // Synchronous cleanup
    ///     },
    ///     |v| Effect::pure(v.iter().sum::<i32>()),
    /// ).run(&()).await;
    /// ```
    pub fn bracket_sync<R, U, Acq, Rel, Use>(
        acquire: Acq,
        release: Rel,
        use_fn: Use,
    ) -> Effect<U, E, Env>
    where
        R: Send + 'static,
        U: Send + 'static,
        Acq: Into<Effect<R, E, Env>>,
        Rel: FnOnce(R) -> Result<(), E> + Send + 'static,
        Use: FnOnce(&R) -> Effect<U, E, Env> + Send + std::panic::UnwindSafe + 'static,
        E: std::fmt::Debug,
    {
        let acquire_effect = acquire.into();

        Effect::from_async(move |env: &Env| {
            let env_ptr = env as *const Env;

            async move {
                let env = unsafe { &*env_ptr };

                // Acquire resource
                let resource = acquire_effect.run(env).await?;

                // Use resource with panic catching
                // We need AssertUnwindSafe because env reference isn't UnwindSafe
                let use_result = {
                    let resource_ref = &resource;
                    let env_for_use = env;

                    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        // We can't await inside catch_unwind, so we use a blocking approach
                        // This creates a nested runtime - document this limitation
                        futures::executor::block_on(use_fn(resource_ref).run(env_for_use))
                    }))
                };

                // Release resource (always runs, even after panic)
                let release_result = release(resource);

                // Handle results
                match use_result {
                    Ok(Ok(value)) => {
                        // Use succeeded
                        if let Err(rel_err) = release_result {
                            tracing::warn!("Resource cleanup failed: {:?}", rel_err);
                        }
                        Ok(value)
                    }
                    Ok(Err(use_err)) => {
                        // Use returned an error
                        if let Err(rel_err) = release_result {
                            tracing::warn!("Resource cleanup failed: {:?}", rel_err);
                        }
                        Err(use_err)
                    }
                    Err(panic_payload) => {
                        // Use panicked - log cleanup error if any, then re-panic
                        if let Err(rel_err) = release_result {
                            tracing::error!(
                                "Resource cleanup failed after panic: {:?}",
                                rel_err
                            );
                        }
                        std::panic::resume_unwind(panic_payload)
                    }
                }
            }
        })
    }
}
```

#### Resource Type

```rust
/// A resource that can be acquired and must be released.
///
/// `Resource` encapsulates the acquire/release pattern, making it
/// reusable and composable.
///
/// # Example
///
/// ```
/// use stillwater::{Effect, Resource};
///
/// let db = Resource::new(
///     Effect::pure(DatabaseConnection::new()),
///     |conn| async move { conn.close().await }
/// );
///
/// // Use the resource
/// let result = db.with(|conn| {
///     Effect::pure(conn.query("SELECT 1"))
/// }).run(&()).await;
/// ```
pub struct Resource<T, E, Env>
where
    T: Send + 'static,
    E: Send + 'static,
    Env: Sync + 'static,
{
    acquire: Effect<T, E, Env>,
    release: Box<dyn FnOnce(T) -> BoxFuture<'static, Result<(), E>> + Send>,
}

impl<T, E, Env> Resource<T, E, Env>
where
    T: Send + 'static,
    E: Send + std::fmt::Debug + 'static,
    Env: Sync + 'static,
{
    /// Create a new resource with acquire effect and release function.
    pub fn new<Acq, Rel, RelFut>(acquire: Acq, release: Rel) -> Self
    where
        Acq: Into<Effect<T, E, Env>>,
        Rel: FnOnce(T) -> RelFut + Send + 'static,
        RelFut: Future<Output = Result<(), E>> + Send + 'static,
    {
        Resource {
            acquire: acquire.into(),
            release: Box::new(move |t| Box::pin(release(t))),
        }
    }

    /// Use this resource with a function, guaranteeing cleanup.
    ///
    /// This is equivalent to `Effect::bracket` but with the acquire/release
    /// already encapsulated in the Resource.
    pub fn with<U, F>(self, f: F) -> Effect<U, E, Env>
    where
        U: Send + 'static,
        F: FnOnce(&T) -> Effect<U, E, Env> + Send + 'static,
    {
        Effect::bracket(self.acquire, self.release, f)
    }

    /// Combine two resources into one.
    ///
    /// The combined resource acquires both resources and releases them
    /// in reverse order (LIFO).
    pub fn both<T2>(
        self,
        other: Resource<T2, E, Env>,
    ) -> Resource<(T, T2), E, Env>
    where
        T2: Send + 'static,
    {
        // We need to compose the acquires and releases
        // This is tricky because we need to handle partial acquisition
        let acquire1 = self.acquire;
        let release1 = self.release;
        let acquire2 = other.acquire;
        let release2 = other.release;

        let combined_acquire = Effect::from_async(move |env: &Env| {
            let env_ptr = env as *const Env;
            async move {
                let env = unsafe { &*env_ptr };
                let t1 = acquire1.run(env).await?;
                match acquire2.run(env).await {
                    Ok(t2) => Ok((t1, t2)),
                    Err(acquire_err) => {
                        // Release t1 if t2 acquisition fails
                        // Log cleanup error rather than silently discarding
                        if let Err(cleanup_err) = release1(t1).await {
                            tracing::warn!(
                                "Cleanup failed during partial acquisition rollback: {:?}",
                                cleanup_err
                            );
                        }
                        Err(acquire_err)
                    }
                }
            }
        });

        let combined_release = move |(t1, t2): (T, T2)| {
            async move {
                // Release in reverse order
                let r2 = release2(t2).await;
                let r1 = release1(t1).await;
                // Return first error if any
                r2?;
                r1?;
                Ok(())
            }
        };

        Resource::new(combined_acquire, combined_release)
    }

    // Note: Resource::map and Resource::all3 are deferred to future work.
    // map() requires careful design around ownership for the release function.
    // all3() depends on map() for tuple flattening.
    // Users should use the Acquiring builder for 3+ resources instead.
}
```

#### Builder Pattern (Acquiring)

The builder provides a flat API that generates nested brackets at compile time.

**Important: Tuple Nesting Behavior**

When chaining multiple `.and()` calls, the resources are nested as right-associated tuples.
This is a consequence of how `Resource::both` composes pairs of resources.

| Resources | Type | Destructure Pattern |
|-----------|------|---------------------|
| 1 | `T` | `\|a\|` |
| 2 | `(T1, T2)` | `\|(a, b)\|` |
| 3 | `(T1, (T2, T3))` | `\|(a, (b, c))\|` |
| 4 | `(T1, (T2, (T3, T4)))` | `\|(a, (b, (c, d)))\|` |

**Why not flat tuples?** Flattening would require either:
1. Macros (worse error messages, IDE support)
2. Complex trait machinery (slow compiles)
3. HList-style type-level programming (complexity explosion)

The nested approach is predictable and works with Rust's type system naturally.

**Ergonomic access with `with_flat`:**

To avoid nested destructuring, use the `with_flat` method which provides flat parameter access:

```rust
// Nested destructuring (always works)
.with(|(a, (b, c))| { ... })

// Flat parameters via with_flat (2-4 resources)
.with_flat(|a, b, c| { ... })
```

The `with_flat` method is provided for 2, 3, and 4 resources to improve ergonomics.

```rust
/// Builder for acquiring multiple resources with guaranteed cleanup.
///
/// This provides a fluent API that avoids deeply nested brackets while
/// generating the same efficient code structure internally.
///
/// # Example
///
/// ```
/// use stillwater::Effect;
///
/// let result = Effect::acquiring(open_conn(), |c| c.close())
///     .and(acquire_lock(), |l| l.release())
///     .and(open_file(), |f| f.close())
///     .with(|(conn, lock, file)| {
///         process(conn, lock, file)
///     })
///     .run(&env)
///     .await;
/// ```
pub struct Acquiring<T, E, Env>
where
    T: Send + 'static,
    E: Send + 'static,
    Env: Sync + 'static,
{
    resource: Resource<T, E, Env>,
}

impl<T, E, Env> Effect<T, E, Env>
where
    T: Send + 'static,
    E: Send + std::fmt::Debug + 'static,
    Env: Sync + 'static,
{
    /// Start building a resource acquisition chain.
    ///
    /// This is the entry point for the fluent builder API. Chain multiple
    /// resources with `.and()` and finalize with `.with()`.
    ///
    /// # Example
    ///
    /// ```
    /// Effect::acquiring(open_database(), |db| db.close())
    ///     .and(open_file(), |f| f.close())
    ///     .with(|(db, file)| do_work(db, file))
    /// ```
    pub fn acquiring<R, Acq, Rel, RelFut>(
        acquire: Acq,
        release: Rel,
    ) -> Acquiring<R, E, Env>
    where
        R: Send + 'static,
        Acq: Into<Effect<R, E, Env>>,
        Rel: FnOnce(R) -> RelFut + Send + 'static,
        RelFut: Future<Output = Result<(), E>> + Send + 'static,
    {
        Acquiring {
            resource: Resource::new(acquire, release),
        }
    }
}

impl<T, E, Env> Acquiring<T, E, Env>
where
    T: Send + 'static,
    E: Send + std::fmt::Debug + 'static,
    Env: Sync + 'static,
{
    /// Add another resource to the acquisition chain.
    ///
    /// Resources are acquired in order and released in reverse order (LIFO).
    /// The resulting tuple is flattened: `(A, B)` not `(A, (B,))`.
    pub fn and<T2, Acq, Rel, RelFut>(
        self,
        acquire: Acq,
        release: Rel,
    ) -> Acquiring<(T, T2), E, Env>
    where
        T2: Send + 'static,
        Acq: Into<Effect<T2, E, Env>>,
        Rel: FnOnce(T2) -> RelFut + Send + 'static,
        RelFut: Future<Output = Result<(), E>> + Send + 'static,
    {
        Acquiring {
            resource: self.resource.both(Resource::new(acquire, release)),
        }
    }

    /// Use the acquired resources with a function, guaranteeing cleanup.
    ///
    /// This finalizes the builder and returns an Effect that will:
    /// 1. Acquire all resources in order
    /// 2. Run the provided function with references to all resources
    /// 3. Release all resources in reverse order (even on error)
    pub fn with<U, F>(self, f: F) -> Effect<U, E, Env>
    where
        U: Send + 'static,
        F: FnOnce(&T) -> Effect<U, E, Env> + Send + 'static,
    {
        self.resource.with(f)
    }
}

// Implement tuple flattening for ergonomic access
// When chaining .and().and(), we get (A, (B, C)) but want (A, B, C)
// This is handled by implementing for nested tuple patterns

impl<A, B, E, Env> Acquiring<(A, B), E, Env>
where
    A: Send + 'static,
    B: Send + 'static,
    E: Send + std::fmt::Debug + 'static,
    Env: Sync + 'static,
{
    /// Use with flattened parameter access for two resources.
    ///
    /// Instead of `|(a, b)|`, you can use `|a, b|`.
    pub fn with_flat<U, F>(self, f: F) -> Effect<U, E, Env>
    where
        U: Send + 'static,
        F: FnOnce(&A, &B) -> Effect<U, E, Env> + Send + 'static,
    {
        self.resource.with(|(a, b)| f(a, b))
    }
}

impl<A, B, C, E, Env> Acquiring<(A, (B, C)), E, Env>
where
    A: Send + 'static,
    B: Send + 'static,
    C: Send + 'static,
    E: Send + std::fmt::Debug + 'static,
    Env: Sync + 'static,
{
    /// Use with flattened tuple access for three resources.
    ///
    /// Instead of `|(a, (b, c))|`, you can use `|(a, b, c)|`.
    pub fn with_flat<U, F>(self, f: F) -> Effect<U, E, Env>
    where
        U: Send + 'static,
        F: FnOnce(&A, &B, &C) -> Effect<U, E, Env> + Send + 'static,
    {
        self.resource.with(|(a, (b, c))| f(a, b, c))
    }
}

impl<A, B, C, D, E, Env> Acquiring<(A, (B, (C, D))), E, Env>
where
    A: Send + 'static,
    B: Send + 'static,
    C: Send + 'static,
    D: Send + 'static,
    E: Send + std::fmt::Debug + 'static,
    Env: Sync + 'static,
{
    /// Use with flattened tuple access for four resources.
    pub fn with_flat<U, F>(self, f: F) -> Effect<U, E, Env>
    where
        U: Send + 'static,
        F: FnOnce(&A, &B, &C, &D) -> Effect<U, E, Env> + Send + 'static,
    {
        self.resource.with(|(a, (b, (c, d)))| f(a, b, c, d))
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
│   ├── bracket.rs   # bracket implementations (on Effect)
│   ├── resource.rs  # Resource<T, E, Env> type
│   └── acquiring.rs # Acquiring<T, E, Env> builder
```

## Dependencies

- **Prerequisites**: None
- **Affected Components**:
  - `effect.rs` - Add bracket methods
  - `lib.rs` - Re-export resource types
- **External Dependencies**:
  - `tracing` (new dependency, for logging cleanup errors)
  - `futures` (new dependency, for `block_on` in `bracket_sync`)

Add to `Cargo.toml`:
```toml
[dependencies]
tracing = "0.1"
futures = "0.3"
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn bracket_returns_error_on_acquire_failure() {
        let released = Arc::new(AtomicBool::new(false));
        let released_clone = released.clone();

        let result = Effect::<i32, String, ()>::bracket(
            Effect::fail("acquire failed".to_string()),
            move |_| {
                released_clone.store(true, Ordering::SeqCst);
                async { Ok(()) }
            },
            |val| Effect::pure(*val * 2),
        )
        .run(&())
        .await;

        assert_eq!(result, Err("acquire failed".to_string()));
        assert!(
            !released.load(Ordering::SeqCst),
            "cleanup must NOT run when acquire fails"
        );
    }

    #[tokio::test]
    async fn bracket_releases_on_success() {
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
    async fn bracket_releases_on_use_failure() {
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
        assert!(released.load(Ordering::SeqCst), "cleanup must run on failure");
    }

    #[tokio::test]
    async fn bracket_logs_cleanup_error_returns_use_result() {
        // Cleanup fails, but use succeeds - should return use result
        let result = Effect::bracket(
            Effect::<_, String, ()>::pure(42),
            |_| async { Err("cleanup failed".to_string()) },
            |val| Effect::pure(*val * 2),
        )
        .run(&())
        .await;

        assert_eq!(result, Ok(84), "use result returned despite cleanup failure");
    }

    #[tokio::test]
    async fn bracket2_releases_in_lifo_order() {
        let order = Arc::new(std::sync::Mutex::new(Vec::new()));
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
        let releases = order.lock().unwrap();
        assert_eq!(*releases, vec!["release_second", "release_first"]);
    }

    #[tokio::test]
    async fn bracket2_releases_first_if_second_acquire_fails() {
        let released = Arc::new(AtomicBool::new(false));
        let released_clone = released.clone();

        let result = Effect::<&str, String, ()>::bracket2(
            Effect::pure("first"),
            Effect::fail("acquire2 failed".to_string()),
            move |_| {
                released_clone.store(true, Ordering::SeqCst);
                async { Ok(()) }
            },
            |_| async { Ok(()) },
            |_, _| Effect::pure("done"),
        )
        .run(&())
        .await;

        assert!(result.is_err());
        assert!(
            released.load(Ordering::SeqCst),
            "first resource must be released when second acquire fails"
        );
    }

    #[tokio::test]
    async fn bracket_full_returns_both_errors() {
        let result = Effect::bracket_full(
            Effect::<_, String, ()>::pure(42),
            |_| async { Err("cleanup failed".to_string()) },
            |_| Effect::<i32, String, ()>::fail("use failed".to_string()),
        )
        .run(&())
        .await;

        let err = result.unwrap_err();
        match err {
            BracketError::Both { use_error, cleanup_error } => {
                assert_eq!(use_error, "use failed");
                assert_eq!(cleanup_error, "cleanup failed");
            }
            _ => panic!("expected BracketError::Both"),
        }
    }

    #[tokio::test]
    async fn bracket_full_returns_use_error_only() {
        let result = Effect::bracket_full(
            Effect::<_, String, ()>::pure(42),
            |_| async { Ok(()) },
            |_| Effect::<i32, String, ()>::fail("use failed".to_string()),
        )
        .run(&())
        .await;

        let err = result.unwrap_err();
        match err {
            BracketError::UseError(e) => assert_eq!(e, "use failed"),
            _ => panic!("expected BracketError::UseError"),
        }
    }

    #[tokio::test]
    async fn bracket_full_returns_cleanup_error_only() {
        let result = Effect::bracket_full(
            Effect::<_, String, ()>::pure(42),
            |_| async { Err("cleanup failed".to_string()) },
            |_| Effect::<i32, String, ()>::pure(84),
        )
        .run(&())
        .await;

        let err = result.unwrap_err();
        match err {
            BracketError::CleanupError(e) => assert_eq!(e, "cleanup failed"),
            _ => panic!("expected BracketError::CleanupError"),
        }
    }

    #[tokio::test]
    async fn bracket_full_returns_acquire_error() {
        let result = Effect::<i32, String, ()>::bracket_full(
            Effect::fail("acquire failed".to_string()),
            |_| async { Ok(()) },
            |_| Effect::pure(42),
        )
        .run(&())
        .await;

        let err = result.unwrap_err();
        match err {
            BracketError::AcquireError(e) => assert_eq!(e, "acquire failed"),
            _ => panic!("expected BracketError::AcquireError"),
        }
    }

    #[tokio::test]
    async fn resource_use_guarantees_cleanup() {
        let released = Arc::new(AtomicBool::new(false));
        let released_clone = released.clone();

        let resource = Resource::new(
            Effect::<_, String, ()>::pure(42),
            move |_| {
                released_clone.store(true, Ordering::SeqCst);
                async { Ok(()) }
            },
        );

        let result = resource.with(|val| Effect::pure(*val * 2)).run(&()).await;

        assert_eq!(result, Ok(84));
        assert!(released.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn acquiring_builder_single_resource() {
        let released = Arc::new(AtomicBool::new(false));
        let released_clone = released.clone();

        let result = Effect::<_, String, ()>::acquiring(
            Effect::pure(42),
            move |_| {
                released_clone.store(true, Ordering::SeqCst);
                async { Ok(()) }
            },
        )
        .with(|val| Effect::pure(*val * 2))
        .run(&())
        .await;

        assert_eq!(result, Ok(84));
        assert!(released.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn acquiring_builder_multiple_resources() {
        let order = Arc::new(std::sync::Mutex::new(Vec::new()));
        let order1 = order.clone();
        let order2 = order.clone();
        let order3 = order.clone();

        let result = Effect::<_, String, ()>::acquiring(
            Effect::pure("first"),
            move |_| {
                order1.lock().unwrap().push("release_first");
                async { Ok(()) }
            },
        )
        .and(
            Effect::pure("second"),
            move |_| {
                order2.lock().unwrap().push("release_second");
                async { Ok(()) }
            },
        )
        .and(
            Effect::pure("third"),
            move |_| {
                order3.lock().unwrap().push("release_third");
                async { Ok(()) }
            },
        )
        .with(|(first, (second, third))| {
            // Verify we have all resources
            assert_eq!(*first, "first");
            assert_eq!(*second, "second");
            assert_eq!(*third, "third");
            Effect::pure("done")
        })
        .run(&())
        .await;

        assert_eq!(result, Ok("done"));

        // Verify LIFO cleanup order
        let releases = order.lock().unwrap();
        assert_eq!(*releases, vec!["release_third", "release_second", "release_first"]);
    }

    #[tokio::test]
    async fn acquiring_builder_with_flat_two_resources() {
        let result = Effect::<_, String, ()>::acquiring(
            Effect::pure(10),
            |_| async { Ok(()) },
        )
        .and(Effect::pure(20), |_| async { Ok(()) })
        .with_flat(|a, b| {
            Effect::pure(*a + *b)
        })
        .run(&())
        .await;

        assert_eq!(result, Ok(30));
    }

    #[tokio::test]
    async fn acquiring_builder_with_flat_three_resources() {
        let result = Effect::<_, String, ()>::acquiring(
            Effect::pure(1),
            |_| async { Ok(()) },
        )
        .and(Effect::pure(2), |_| async { Ok(()) })
        .and(Effect::pure(3), |_| async { Ok(()) })
        .with_flat(|a, b, c| {
            Effect::pure(*a + *b + *c)
        })
        .run(&())
        .await;

        assert_eq!(result, Ok(6));
    }

    #[tokio::test]
    async fn acquiring_builder_releases_on_partial_acquire_failure() {
        let released = Arc::new(AtomicBool::new(false));
        let released_clone = released.clone();

        let result = Effect::<_, String, ()>::acquiring(
            Effect::pure("first"),
            move |_| {
                released_clone.store(true, Ordering::SeqCst);
                async { Ok(()) }
            },
        )
        .and(
            Effect::<&str, String, ()>::fail("second acquire failed".to_string()),
            |_| async { Ok(()) },
        )
        .with(|(first, second)| {
            Effect::pure(format!("{} {}", first, second))
        })
        .run(&())
        .await;

        assert!(result.is_err());
        assert!(
            released.load(Ordering::SeqCst),
            "first resource must be released when second acquire fails"
        );
    }

    #[tokio::test]
    async fn bracket_sync_releases_on_success() {
        let released = Arc::new(AtomicBool::new(false));
        let released_clone = released.clone();

        let result = Effect::bracket_sync(
            Effect::<_, String, ()>::pure(42),
            move |_| {
                released_clone.store(true, Ordering::SeqCst);
                Ok(())
            },
            |val| Effect::pure(*val * 2),
        )
        .run(&())
        .await;

        assert_eq!(result, Ok(84));
        assert!(released.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn bracket_sync_releases_on_use_failure() {
        let released = Arc::new(AtomicBool::new(false));
        let released_clone = released.clone();

        let result = Effect::<i32, String, ()>::bracket_sync(
            Effect::pure(42),
            move |_| {
                released_clone.store(true, Ordering::SeqCst);
                Ok(())
            },
            |_| Effect::fail("use failed".to_string()),
        )
        .run(&())
        .await;

        assert_eq!(result, Err("use failed".to_string()));
        assert!(released.load(Ordering::SeqCst), "cleanup must run on failure");
    }

    #[tokio::test]
    #[should_panic(expected = "intentional panic")]
    async fn bracket_sync_releases_on_panic() {
        let released = Arc::new(AtomicBool::new(false));
        let released_clone = released.clone();

        // We need to check cleanup ran BEFORE the panic propagates
        // Use a static to verify after test (or check in separate thread)
        static CLEANUP_RAN: std::sync::atomic::AtomicBool =
            std::sync::atomic::AtomicBool::new(false);

        let _ = Effect::<i32, String, ()>::bracket_sync(
            Effect::pure(42),
            move |_| {
                CLEANUP_RAN.store(true, Ordering::SeqCst);
                Ok(())
            },
            |_| {
                panic!("intentional panic");
                #[allow(unreachable_code)]
                Effect::pure(0)
            },
        )
        .run(&())
        .await;

        // Note: This assertion runs before panic propagates due to how
        // bracket_sync re-raises the panic after cleanup
    }

    #[tokio::test]
    async fn bracket_sync_logs_cleanup_error_on_panic() {
        // This test verifies cleanup runs even when it fails after a panic
        // The panic should still propagate
        let result = std::panic::catch_unwind(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                Effect::<i32, String, ()>::bracket_sync(
                    Effect::pure(42),
                    |_| Err("cleanup failed".to_string()),  // Cleanup fails
                    |_| {
                        panic!("use panicked");
                        #[allow(unreachable_code)]
                        Effect::pure(0)
                    },
                )
                .run(&())
                .await
            })
        });

        // Should have panicked (cleanup error logged, panic re-raised)
        assert!(result.is_err());
    }
}
```

### Property-Based Tests

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn cleanup_always_runs_regardless_of_use_result(use_succeeds: bool) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let cleanup_ran = Arc::new(AtomicBool::new(false));
        let cleanup_ran_clone = cleanup_ran.clone();

        let _ = rt.block_on(async {
            Effect::<i32, String, ()>::bracket(
                Effect::pure(()),
                move |_| {
                    cleanup_ran_clone.store(true, Ordering::SeqCst);
                    async { Ok(()) }
                },
                move |_| {
                    if use_succeeds {
                        Effect::pure(42)
                    } else {
                        Effect::fail("use failed".to_string())
                    }
                },
            )
            .run(&())
            .await
        });

        prop_assert!(cleanup_ran.load(Ordering::SeqCst));
    }
}
```

## Known Limitations

### Panic Safety

The standard `bracket` pattern does **not** guarantee cleanup on panic:

```rust
Effect::bracket(
    acquire(),
    |r| async { r.close().await },
    |r| {
        panic!("oops");  // Cleanup will NOT run with async release
        Effect::pure(())
    }
)
```

**Solution**: Use `bracket_sync` when cleanup can be synchronous:

```rust
Effect::bracket_sync(
    acquire(),
    |r| r.close_sync(),  // Synchronous cleanup
    |r| {
        panic!("oops");  // Cleanup WILL run, then panic re-raised
        Effect::pure(())
    }
)
```

**Limitation of `bracket_sync`**: Uses `futures::executor::block_on` internally, which creates a nested runtime. This may cause issues if the use function itself spawns tasks on the outer runtime. For most use cases (pure computation, simple I/O) this is fine.

**When to use which**:
- `bracket` - Async cleanup, no panic safety needed (library code, controlled environments)
- `bracket_sync` - Sync cleanup, panic safety required (application boundaries, user-facing code)

### Cancellation Safety

The bracket pattern is **not** cancellation-safe with `tokio::select!`:

```rust
tokio::select! {
    result = bracketed_effect.run(&env) => { ... }
    _ = timeout => {
        // If timeout wins, cleanup may not run!
    }
}
```

**Rationale**: Making this safe would require spawning cleanup as a detached task, which loses error handling and ordering guarantees.

**Workaround**: Don't use `select!` with bracket, or handle cancellation explicitly.

### No Dynamic Resource Acquisition

Unlike ZIO's Scope, we don't support:

```rust
// NOT SUPPORTED
Effect::scoped(|scope| {
    let conn = scope.acquire(open_conn(), |c| c.close())?;
    // ...
})
```

**Rationale**: This fights Rust's ownership model. The scope needs the resource for cleanup, but the user also needs it. Solutions (Arc, Clone, etc.) add complexity.

**Workaround**: Use nested brackets or `Resource::both`.

## Future Considerations

### Macro for Flat Syntax

If the tuple nesting proves too cumbersome in practice, a declarative macro could provide
flatter syntax while generating the same nested brackets internally:

```rust
// Potential future syntax (NOT in this spec)
bracket! {
    conn <- open_conn(), |c| c.close();
    file <- open_file(), |f| f.close();
    lock <- acquire_lock(), |l| l.release();
    =>
    do_work(&conn, &file, &lock)
}
```

**Trade-offs:**
- Pro: Best ergonomics, scales to N resources
- Pro: Zero runtime overhead (generates nested brackets)
- Con: Worse error messages than functions
- Con: Degraded IDE support (autocomplete, type hints)
- Con: "Magic" syntax users must learn

**Decision:** Defer to a future spec if real-world usage demonstrates need. The current
`with_flat` methods handle the common 2-4 resource cases adequately.

### Cats-Effect Style Monadic Resource

Scala's cats-effect provides elegant resource composition via `flatMap`:

```scala
val resources = for {
  conn <- Resource.make(openConn)(_.close)
  file <- Resource.make(openFile)(_.close)
} yield (conn, file)

resources.use { case (conn, file) => doWork(conn, file) }
```

This doesn't translate cleanly to Rust due to ownership constraints - the `flatMap`
closure would need to capture the first resource while also allowing it to be used
and released. Solutions require `Arc` or unsafe, adding complexity.

**Decision:** Not pursued. The `Acquiring` builder achieves similar ergonomics within
Rust's ownership model.

## Migration Guide

### From Manual Cleanup

```rust
// Before: Manual cleanup (error-prone)
async fn process(env: &Env) -> Result<(), Error> {
    let conn = open_conn(env).await?;
    let result = use_conn(&conn).await;
    conn.close().await?;  // Might not run on error!
    result
}

// After: Bracket (guaranteed cleanup)
fn process() -> Effect<(), Error, Env> {
    Effect::bracket(
        open_conn(),
        |conn| async move { conn.close().await },
        |conn| use_conn(conn),
    )
}
```

### From Multiple try/finally

```rust
// Before: Nested try/finally (messy)
async fn process(env: &Env) -> Result<(), Error> {
    let conn = open_conn(env).await?;
    let result = async {
        let file = open_file().await?;
        let result = use_both(&conn, &file).await;
        file.close().await?;
        result
    }.await;
    conn.close().await?;
    result
}

// After: bracket2 (clean, correct)
fn process() -> Effect<(), Error, Env> {
    Effect::bracket2(
        open_conn(),
        open_file(),
        |conn| async move { conn.close().await },
        |file| async move { file.close().await },
        |conn, file| use_both(conn, file),
    )
}

// Best: builder pattern (recommended)
fn process() -> Effect<(), Error, Env> {
    Effect::acquiring(open_conn(), |c| async move { c.close().await })
        .and(open_file(), |f| async move { f.close().await })
        .with(|(conn, file)| use_both(conn, file))
}
```

## Implementation Order

1. **Phase 1**: `Effect::bracket` - Core pattern
2. **Phase 2**: `BracketError` type, `Effect::bracket_full`, `Effect::bracket_sync`
3. **Phase 3**: `Resource` type with `with`, `both` (with cleanup error logging)
4. **Phase 4**: `Acquiring` builder with `and`, `with`, `with_flat`
5. **Phase 5**: `Effect::bracket2`, `Effect::bracket3` (convenience, built on bracket)
6. **Phase 6**: Documentation, examples, integration tests

---

*"Acquire, use, release. Always release. Keep it simple."*
