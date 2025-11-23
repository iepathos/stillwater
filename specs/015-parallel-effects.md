---
number: 015
title: Parallel Effect Execution
category: parallel
priority: high
status: draft
dependencies: []
created: 2025-11-22
---

# Specification 015: Parallel Effect Execution

**Category**: parallel
**Priority**: high
**Status**: draft
**Dependencies**: None (extends existing Effect type)

## Context

Stillwater's `Effect` type currently executes sequentially via `and_then`. However, many real-world use cases involve independent effects that could run concurrently:

- Fetching multiple users from a database in parallel
- Validating multiple fields independently
- Making multiple API calls simultaneously
- Loading configuration from multiple sources

The DESIGN.md roadmap (Phase 3) explicitly mentions parallel effects, and the async-design.md document discusses this as a future enhancement.

Current limitation:
```rust
// Sequential - waits for each to complete
let user1 = fetch_user(1).run(&env)?;
let user2 = fetch_user(2).run(&env)?;
let user3 = fetch_user(3).run(&env)?;
```

Desired capability:
```rust
// Parallel - runs concurrently
let users = Effect::par_all([
    fetch_user(1),
    fetch_user(2),
    fetch_user(3),
]).run(&env)?;
```

## Objective

Add parallel execution capabilities to `Effect`, enabling concurrent execution of independent effects while preserving error handling and environment access patterns.

## Requirements

### Functional Requirements

- `Effect::par_all()` - run multiple effects in parallel, collect all results
- `Effect::par_all_any_error()` - short-circuit on first error
- `Effect::race()` - return first effect to complete successfully
- `Effect::par_try_all()` - parallel execution with fail-fast semantics
- Support both sync and async effects
- Preserve environment access for all parallel effects
- Maintain error context chains
- Type-safe parallel composition

### Non-Functional Requirements

- Actual concurrency (use async runtime efficiently)
- No significant overhead vs manual parallel execution
- Clear error messages for parallel failures
- Composable with existing Effect combinators
- Works with tokio, async-std, and other runtimes

## Acceptance Criteria

- [ ] `Effect::par_all()` implemented for collecting all results
- [ ] `Effect::race()` implemented for first success
- [ ] Async versions work with tokio/async-std
- [ ] Environment shared correctly across parallel tasks
- [ ] Error context preserved in parallel execution
- [ ] Tests verify actual parallelism
- [ ] Examples demonstrate use cases
- [ ] Documentation guide created
- [ ] Performance benchmarks show speedup
- [ ] All tests pass

## Technical Details

### Implementation Approach

#### Parallel All (Collect All Results)

```rust
impl<T, E, Env> Effect<T, E, Env>
where
    T: Send,
    E: Send,
    Env: Sync,
{
    /// Run multiple effects in parallel, collecting all results.
    ///
    /// All effects run concurrently. If any fail, all errors are collected.
    /// Success only if all effects succeed.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::Effect;
    ///
    /// async fn fetch_user(id: i32) -> Effect<User, Error, DbEnv> {
    ///     IO::query(move |db| db.find_user(id))
    /// }
    ///
    /// let users = Effect::par_all(vec![
    ///     fetch_user(1),
    ///     fetch_user(2),
    ///     fetch_user(3),
    /// ]).run_async(&env).await?;
    ///
    /// assert_eq!(users.len(), 3);
    /// ```
    pub fn par_all<I>(effects: I) -> Effect<Vec<T>, Vec<E>, Env>
    where
        I: IntoIterator<Item = Effect<T, E, Env>>,
    {
        Effect::from_async_fn(|env| async move {
            let futures: Vec<_> = effects
                .into_iter()
                .map(|effect| {
                    let env_ref = env;
                    async move { effect.run_async(env_ref).await }
                })
                .collect();

            let results = futures::future::join_all(futures).await;

            // Collect successes and failures
            let mut successes = Vec::new();
            let mut failures = Vec::new();

            for result in results {
                match result {
                    Ok(value) => successes.push(value),
                    Err(error) => failures.push(error),
                }
            }

            if failures.is_empty() {
                Ok(successes)
            } else {
                Err(failures)
            }
        })
    }

    /// Run effects in parallel, short-circuit on first error.
    ///
    /// More efficient than `par_all` if you don't need all errors.
    pub fn par_try_all<I>(effects: I) -> Effect<Vec<T>, E, Env>
    where
        I: IntoIterator<Item = Effect<T, E, Env>>,
    {
        Effect::from_async_fn(|env| async move {
            let futures: Vec<_> = effects
                .into_iter()
                .map(|effect| {
                    let env_ref = env;
                    async move { effect.run_async(env_ref).await }
                })
                .collect();

            futures::future::try_join_all(futures).await
        })
    }
}
```

#### Race (First Success)

```rust
impl<T, E, Env> Effect<T, E, Env>
where
    T: Send,
    E: Send,
    Env: Sync,
{
    /// Race multiple effects, returning the first to succeed.
    ///
    /// If all fail, returns all errors.
    ///
    /// # Example
    ///
    /// ```rust
    /// // Try multiple data sources, use whichever responds first
    /// let data = Effect::race([
    ///     fetch_from_cache(),
    ///     fetch_from_primary_db(),
    ///     fetch_from_backup_db(),
    /// ]).run_async(&env).await?;
    /// ```
    pub fn race<I>(effects: I) -> Effect<T, Vec<E>, Env>
    where
        I: IntoIterator<Item = Effect<T, E, Env>>,
        T: Clone,
    {
        Effect::from_async_fn(|env| async move {
            let futures: Vec<_> = effects
                .into_iter()
                .map(|effect| {
                    let env_ref = env;
                    async move { effect.run_async(env_ref).await }
                })
                .collect();

            // Use futures::future::select_ok or similar
            match futures::future::select_ok(futures).await {
                Ok((value, _remaining)) => Ok(value),
                Err(errors) => Err(errors),
            }
        })
    }
}
```

#### Parallel With Concurrency Limit

```rust
impl<T, E, Env> Effect<T, E, Env>
where
    T: Send,
    E: Send,
    Env: Sync,
{
    /// Run effects in parallel with a concurrency limit.
    ///
    /// Useful for rate limiting or resource constraints.
    ///
    /// # Example
    ///
    /// ```rust
    /// // Fetch 100 users, but only 10 concurrent requests
    /// let user_ids: Vec<_> = (1..=100).collect();
    /// let effects: Vec<_> = user_ids.iter()
    ///     .map(|&id| fetch_user(id))
    ///     .collect();
    ///
    /// let users = Effect::par_all_limit(effects, 10)
    ///     .run_async(&env).await?;
    /// ```
    pub fn par_all_limit<I>(
        effects: I,
        limit: usize,
    ) -> Effect<Vec<T>, Vec<E>, Env>
    where
        I: IntoIterator<Item = Effect<T, E, Env>>,
    {
        Effect::from_async_fn(move |env| async move {
            use futures::stream::{self, StreamExt};

            let results: Vec<_> = stream::iter(effects)
                .map(|effect| {
                    let env_ref = env;
                    async move { effect.run_async(env_ref).await }
                })
                .buffer_unordered(limit)
                .collect()
                .await;

            let mut successes = Vec::new();
            let mut failures = Vec::new();

            for result in results {
                match result {
                    Ok(value) => successes.push(value),
                    Err(error) => failures.push(error),
                }
            }

            if failures.is_empty() {
                Ok(successes)
            } else {
                Err(failures)
            }
        })
    }
}
```

### Architecture Changes

- Extend `src/effect.rs` with parallel methods
- Add async utilities module if needed
- Update prelude
- Requires `futures` crate dependency

### Integration Patterns

#### Pattern 1: Parallel Data Fetching

```rust
async fn load_dashboard_data() -> Effect<Dashboard, Error, AppEnv> {
    let (user, orders, notifications) = Effect::par_all((
        fetch_user(),
        fetch_recent_orders(),
        fetch_notifications(),
    ))
    .run_async(&env)
    .await?;

    Ok(Dashboard { user, orders, notifications })
}
```

#### Pattern 2: Fallback Strategy

```rust
// Try primary, fallback to secondary, fallback to cache
let data = Effect::race([
    fetch_from_primary().context("primary"),
    fetch_from_secondary().context("secondary"),
    fetch_from_cache().context("cache"),
])
.run_async(&env)
.await?;
```

#### Pattern 3: Batch Processing

```rust
let user_ids: Vec<i32> = (1..=1000).collect();
let effects: Vec<_> = user_ids.iter()
    .map(|&id| fetch_and_process_user(id))
    .collect();

// Process in batches of 50
let results = Effect::par_all_limit(effects, 50)
    .run_async(&env)
    .await?;
```

## Dependencies

- **Prerequisites**: None
- **Affected Components**:
  - `src/effect.rs` (add parallel methods)
  - Examples demonstrating parallelism
- **External Dependencies**:
  - `futures` crate (for join_all, select_ok, streams)
  - Runtime-agnostic (works with tokio, async-std)

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[tokio::test]
    async fn test_par_all() {
        let env = ();

        let effects = vec![
            Effect::pure(1),
            Effect::pure(2),
            Effect::pure(3),
        ];

        let result = Effect::par_all(effects).run_async(&env).await;
        assert_eq!(result.unwrap(), vec![1, 2, 3]);
    }

    #[tokio::test]
    async fn test_par_all_with_errors() {
        let env = ();

        let effects = vec![
            Effect::pure(1),
            Effect::fail("error1"),
            Effect::fail("error2"),
        ];

        let result = Effect::par_all(effects).run_async(&env).await;
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 2);
    }

    #[tokio::test]
    async fn test_race() {
        let env = ();

        let start = Instant::now();

        let effects = vec![
            Effect::from_async_fn(|_| async {
                tokio::time::sleep(Duration::from_millis(100)).await;
                Ok(1)
            }),
            Effect::from_async_fn(|_| async {
                tokio::time::sleep(Duration::from_millis(50)).await;
                Ok(2)
            }),
            Effect::from_async_fn(|_| async {
                tokio::time::sleep(Duration::from_millis(200)).await;
                Ok(3)
            }),
        ];

        let result = Effect::race(effects).run_async(&env).await;
        let elapsed = start.elapsed();

        assert_eq!(result.unwrap(), 2); // Fastest
        assert!(elapsed < Duration::from_millis(100)); // Didn't wait for all
    }

    #[tokio::test]
    async fn test_actual_parallelism() {
        let env = ();
        let start = Instant::now();

        // Three 100ms tasks
        let effects = vec![
            delay_effect(100, 1),
            delay_effect(100, 2),
            delay_effect(100, 3),
        ];

        Effect::par_all(effects).run_async(&env).await.unwrap();

        let elapsed = start.elapsed();

        // Should take ~100ms if parallel, ~300ms if sequential
        assert!(elapsed < Duration::from_millis(200));
    }
}

fn delay_effect<T: Send>(ms: u64, value: T) -> Effect<T, (), ()>
where
    T: 'static,
{
    Effect::from_async_fn(move |_| async move {
        tokio::time::sleep(Duration::from_millis(ms)).await;
        Ok(value)
    })
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_parallel_with_environment() {
    #[derive(Clone)]
    struct TestEnv {
        multiplier: i32,
    }

    fn compute(value: i32) -> Effect<i32, (), TestEnv> {
        Effect::asks(move |env: &TestEnv| value * env.multiplier)
    }

    let env = TestEnv { multiplier: 2 };

    let results = Effect::par_all(vec![
        compute(1),
        compute(2),
        compute(3),
    ])
    .run_async(&env)
    .await
    .unwrap();

    assert_eq!(results, vec![2, 4, 6]);
}
```

## Documentation Requirements

### Code Documentation

- Comprehensive rustdoc for each parallel method
- Examples showing performance benefits
- Explain concurrency vs parallelism
- Document when to use each method

### User Documentation

- New guide: `docs/guide/11-parallel-effects.md`
- Update DESIGN.md with parallel patterns
- Add performance comparison benchmarks
- FAQ: "When to use parallel vs sequential?"

## Implementation Notes

### Design Decisions

**Why require Send + Sync?**
- Necessary for actual concurrent execution
- Most real-world types satisfy this
- Compiler enforces safety

**Why return Vec<E> for errors instead of single E?**
- Parallel execution means multiple errors possible
- Similar to Validation accumulation
- User can choose first error if desired

**Why support concurrency limits?**
- Prevent resource exhaustion
- Rate limiting for APIs
- Database connection pool limits

### Gotchas

- Environment must be `Sync` (thread-safe sharing)
- Results may not be in original order
- Error order is non-deterministic

### Best Practices

```rust
// Good: Independent effects that can run in parallel
Effect::par_all([fetch_user(), fetch_orders(), fetch_profile()])

// Bad: Dependent effects (use and_then instead)
Effect::par_all([fetch_user(), process_user(), save_user()])
// ❌ process/save depend on fetch!

// Good: Use limit for external resources
Effect::par_all_limit(api_calls, 10) // Don't overwhelm API

// Good: Race for redundancy/fallback
Effect::race([primary_source(), backup_source()])
```

## Migration and Compatibility

### Breaking Changes

None - pure additions.

### Compatibility

- Fully backward compatible
- Opt-in parallelism
- Works with existing Effect code

## Success Metrics

- Actual concurrency verified by timing tests
- Speedup proportional to number of parallel tasks
- No race conditions or data races
- Positive user feedback

## Future Enhancements

- `par_traverse()` - parallel map over collections
- Integration with rayon for CPU-bound parallel work
- Cancellation support
- Retry with jitter for failed parallel tasks
- Progress tracking for long-running parallel tasks
