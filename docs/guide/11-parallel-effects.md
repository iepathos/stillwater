# Parallel Effect Execution

## Overview

Stillwater's `Effect` type supports parallel execution of independent effects, enabling concurrent operations while preserving error handling and environment access patterns. This guide demonstrates the four parallel execution methods and when to use each.

## Why Parallel Effects?

Many real-world operations involve independent tasks that can run concurrently:

- Fetching multiple records from a database
- Validating multiple fields independently
- Making multiple API calls simultaneously
- Loading configuration from multiple sources
- Processing batches of items

Sequential execution:
```rust
// Slow - waits for each to complete before starting the next
let user1 = fetch_user(1).run(&env).await?;
let user2 = fetch_user(2).run(&env).await?;
let user3 = fetch_user(3).run(&env).await?;
```

Parallel execution:
```rust
// Fast - runs all three concurrently
let users = Effect::par_all([
    fetch_user(1),
    fetch_user(2),
    fetch_user(3),
]).run(&env).await?;
```

## The Four Parallel Methods

### 1. `par_all()` - Collect All Results

Runs multiple effects in parallel and collects all results. If any effects fail, **all errors are accumulated**.

**Use when:** You need all results and want to see all errors if multiple operations fail.

**Type signature:**
```rust
pub fn par_all<I>(effects: I) -> Effect<Vec<T>, Vec<E>, Env>
where
    I: IntoIterator<Item = Effect<T, E, Env>> + Send + 'static,
    I::IntoIter: Send,
```

**Example: Fetch Multiple Users**
```rust
use stillwater::Effect;

async fn fetch_user(id: i32) -> Effect<User, DbError, AppEnv> {
    Effect::from_async_fn(move |env| async move {
        env.db.find_user(id).await
    })
}

async fn load_team(ids: Vec<i32>) -> Result<Vec<User>, Vec<DbError>> {
    let env = AppEnv::new();

    let effects = ids.into_iter().map(|id| fetch_user(id));

    Effect::par_all(effects).run(&env).await
}

// Usage
# tokio_test::block_on(async {
match load_team(vec![1, 2, 3]).await {
    Ok(users) => println!("Loaded {} users", users.len()),
    Err(errors) => {
        // See ALL failures, not just the first
        for error in errors {
            eprintln!("Failed: {}", error);
        }
    }
}
# });
```

**Error behavior:**
```rust
# tokio_test::block_on(async {
let effects = vec![
    Effect::<i32, String, ()>::pure(1),
    Effect::fail("error1".to_string()),
    Effect::fail("error2".to_string()),
];

let result = Effect::par_all(effects).run_standalone().await;

// Result is Err(vec!["error1", "error2"])
// All errors are collected, not just the first
# });
```

### 2. `par_try_all()` - Fail Fast

Runs effects in parallel but **stops immediately when the first error occurs**.

**Use when:** You need all results but one failure invalidates the entire operation.

**Type signature:**
```rust
pub fn par_try_all<I>(effects: I) -> Effect<Vec<T>, E, Env>
where
    I: IntoIterator<Item = Effect<T, E, Env>> + Send + 'static,
    I::IntoIter: Send,
```

**Example: Health Check**
```rust
use stillwater::Effect;

async fn check_database() -> Effect<Status, Error, AppEnv> {
    Effect::from_async_fn(|env| async move {
        env.db.ping().await.map(|_| Status::Ok)
    })
}

async fn check_cache() -> Effect<Status, Error, AppEnv> {
    Effect::from_async_fn(|env| async move {
        env.cache.ping().await.map(|_| Status::Ok)
    })
}

async fn check_queue() -> Effect<Status, Error, AppEnv> {
    Effect::from_async_fn(|env| async move {
        env.queue.ping().await.map(|_| Status::Ok)
    })
}

async fn health_check() -> Effect<Vec<Status>, Error, AppEnv> {
    // If ANY service is down, fail immediately
    Effect::par_try_all([
        check_database(),
        check_cache(),
        check_queue(),
    ])
}

# tokio_test::block_on(async {
let env = AppEnv::new();

match health_check().run(&env).await {
    Ok(statuses) => println!("All services healthy"),
    Err(error) => {
        // Got the FIRST error, didn't wait for others
        eprintln!("Health check failed: {}", error);
    }
}
# });
```

**Comparison with par_all:**
```rust
# tokio_test::block_on(async {
let effects = vec![
    Effect::<i32, String, ()>::pure(1),
    Effect::fail("error1".to_string()),
    Effect::fail("error2".to_string()),
];

// par_try_all returns Err("error1")
let result = Effect::par_try_all(effects.clone()).run_standalone().await;
assert!(result.is_err());

// par_all returns Err(vec!["error1", "error2"])
let result = Effect::par_all(effects).run_standalone().await;
assert_eq!(result.unwrap_err().len(), 2);
# });
```

### 3. `race()` - First to Complete

Runs multiple effects in parallel and returns **the first successful result**. If all fail, collects all errors.

**Use when:** You have multiple equivalent sources and want the fastest response.

**Type signature:**
```rust
pub fn race<I>(effects: I) -> Effect<T, Vec<E>, Env>
where
    I: IntoIterator<Item = Effect<T, E, Env>> + Send + 'static,
    I::IntoIter: Send,
```

**Example: Fallback Data Sources**
```rust
use stillwater::Effect;

async fn fetch_from_cache(key: String) -> Effect<Data, Error, AppEnv> {
    Effect::from_async_fn(move |env| async move {
        env.cache.get(&key).await
    })
}

async fn fetch_from_primary(key: String) -> Effect<Data, Error, AppEnv> {
    Effect::from_async_fn(move |env| async move {
        env.primary_db.fetch(&key).await
    })
}

async fn fetch_from_backup(key: String) -> Effect<Data, Error, AppEnv> {
    Effect::from_async_fn(move |env| async move {
        env.backup_db.fetch(&key).await
    })
}

async fn fetch_data(key: String) -> Effect<Data, Vec<Error>, AppEnv> {
    // Try all sources, use whichever responds first
    Effect::race([
        fetch_from_cache(key.clone()),
        fetch_from_primary(key.clone()),
        fetch_from_backup(key),
    ])
}

# tokio_test::block_on(async {
let env = AppEnv::new();

match fetch_data("user:123".into()).run(&env).await {
    Ok(data) => {
        // Got data from the fastest source
        println!("Retrieved data: {:?}", data);
    }
    Err(errors) => {
        // ALL sources failed
        eprintln!("All sources failed:");
        for error in errors {
            eprintln!("  - {}", error);
        }
    }
}
# });
```

**Example: Timeout Pattern**
```rust
use stillwater::Effect;
use std::time::Duration;

async fn with_timeout<T, E, Env>(
    effect: Effect<T, E, Env>,
    timeout: Duration,
) -> Effect<T, Vec<E>, Env>
where
    T: Send + 'static,
    E: Send + From<String> + 'static,
    Env: Send + Sync + 'static,
{
    let timeout_effect = Effect::from_async_fn(move |_env| async move {
        tokio::time::sleep(timeout).await;
        Err(E::from("Operation timed out".to_string()))
    });

    Effect::race([effect, timeout_effect])
}

// Usage
# tokio_test::block_on(async {
let slow_operation = Effect::<i32, String, ()>::from_async_fn(|_| async {
    tokio::time::sleep(Duration::from_secs(10)).await;
    Ok(42)
});

let result = with_timeout(slow_operation, Duration::from_secs(1))
    .run_standalone()
    .await;

assert!(result.is_err()); // Times out after 1 second
# });
```

### 4. `par_all_limit()` - Bounded Concurrency

Runs effects in parallel but limits the number running concurrently.

**Use when:** You need to control resource usage (rate limiting, connection pools, etc.)

**Type signature:**
```rust
pub fn par_all_limit<I>(effects: I, limit: usize) -> Effect<Vec<T>, Vec<E>, Env>
where
    I: IntoIterator<Item = Effect<T, E, Env>> + Send + 'static,
    I::IntoIter: Send,
```

**Example: Batch Processing with Rate Limit**
```rust
use stillwater::Effect;

async fn process_item(item: Item) -> Effect<Result, Error, AppEnv> {
    Effect::from_async_fn(move |env| async move {
        env.api.process(item).await
    })
}

async fn process_batch(items: Vec<Item>) -> Effect<Vec<Result>, Vec<Error>, AppEnv> {
    let effects = items.into_iter().map(|item| process_item(item));

    // Process up to 5 items concurrently
    // If you have 100 items, only 5 run at once
    Effect::par_all_limit(effects, 5)
}

# tokio_test::block_on(async {
let env = AppEnv::new();
let items: Vec<Item> = vec![/* ... */];

match process_batch(items).run(&env).await {
    Ok(results) => {
        println!("Processed {} items", results.len());
    }
    Err(errors) => {
        eprintln!("Errors processing batch: {:?}", errors);
    }
}
# });
```

**Example: Database Connection Pool**
```rust
use stillwater::Effect;

async fn query_user(id: i32) -> Effect<User, DbError, AppEnv> {
    Effect::from_async_fn(move |env| async move {
        // Uses connection from pool
        env.db.query_user(id).await
    })
}

async fn load_users(ids: Vec<i32>) -> Effect<Vec<User>, Vec<DbError>, AppEnv> {
    let effects = ids.into_iter().map(|id| query_user(id));

    // Don't exhaust connection pool
    // If pool has 10 connections, use at most 8 for queries
    Effect::par_all_limit(effects, 8)
}
```

## Environment Access in Parallel Effects

All parallel effects share the same environment, which must be `Sync` (safe to share across threads).

```rust
use stillwater::Effect;

#[derive(Clone)]
struct AppEnv {
    config: Config,
    db: Database,
}

// Environment must be Sync for parallel execution
impl Sync for AppEnv {}

async fn parallel_with_env() -> Effect<Vec<i32>, String, AppEnv> {
    Effect::par_all([
        // All three effects access the same environment
        Effect::asks(|env: &AppEnv| env.config.timeout),
        Effect::asks(|env: &AppEnv| env.config.max_retries),
        Effect::asks(|env: &AppEnv| env.config.batch_size),
    ])
}
```

## Composing Parallel Effects

Parallel effects compose naturally with other Effect combinators:

```rust
use stillwater::Effect;

async fn load_and_process(ids: Vec<i32>) -> Effect<Vec<ProcessedData>, Error, AppEnv> {
    // Load in parallel
    let load_effects = ids.into_iter().map(|id| fetch_user(id));

    Effect::par_all(load_effects)
        // Then process each result
        .map(|users| {
            users.into_iter()
                .map(|user| process_user(user))
                .collect()
        })
        // Then save all in parallel
        .and_then(|processed| {
            let save_effects = processed.into_iter().map(|data| save_data(data));
            Effect::par_all(save_effects)
        })
}
```

## Practical Patterns

### Pattern 1: Parallel Validation

Validate multiple fields independently:

```rust
use stillwater::{Effect, Validation};

async fn validate_signup(data: SignupData) -> Effect<ValidUser, Vec<String>, AppEnv> {
    Effect::par_all([
        validate_email(data.email),
        validate_username(data.username),
        validate_password(data.password),
    ])
    .and_then(|results| {
        // All validations succeeded
        Effect::pure(ValidUser::new(results))
    })
}
```

### Pattern 2: Scatter-Gather

Query multiple services and combine results:

```rust
async fn dashboard_data(user_id: i32) -> Effect<Dashboard, Error, AppEnv> {
    Effect::par_all([
        fetch_user_profile(user_id),
        fetch_recent_activity(user_id),
        fetch_recommendations(user_id),
    ])
    .map(|results| Dashboard {
        profile: results[0],
        activity: results[1],
        recommendations: results[2],
    })
}
```

### Pattern 3: Defensive Timeout

Wrap risky operations with timeouts:

```rust
async fn with_timeout<T>(
    effect: Effect<T, Error, AppEnv>,
    timeout_ms: u64,
) -> Effect<T, Vec<Error>, AppEnv> {
    let timeout = Effect::from_async_fn(|_| async move {
        tokio::time::sleep(Duration::from_millis(timeout_ms)).await;
        Err(Error::Timeout)
    });

    Effect::race([effect, timeout])
}
```

### Pattern 4: Work Queue Processing

Process work items with concurrency control:

```rust
async fn process_queue(
    queue: Vec<WorkItem>,
    max_concurrent: usize,
) -> Effect<Vec<Result>, Vec<Error>, AppEnv> {
    let effects = queue.into_iter().map(|item| process_work_item(item));

    Effect::par_all_limit(effects, max_concurrent)
}
```

## Performance Considerations

### Choose the Right Method

- **par_all**: When you need all errors for debugging/reporting
- **par_try_all**: When one failure means the entire operation should stop
- **race**: When you have redundant data sources or want fastest response
- **par_all_limit**: When you need to protect resources (connections, memory, rate limits)

### Actual Concurrency

All parallel methods use actual async concurrency:

```rust
# tokio_test::block_on(async {
use std::time::Instant;

let start = Instant::now();

let effects = vec![
    Effect::<(), String, ()>::from_async_fn(|_| async {
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(())
    }),
    Effect::from_async_fn(|_| async {
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(())
    }),
    Effect::from_async_fn(|_| async {
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(())
    }),
];

Effect::par_all(effects).run_standalone().await.unwrap();

let elapsed = start.elapsed();

// Should take ~100ms, not ~300ms
assert!(elapsed < Duration::from_millis(150));
# });
```

### Memory Usage

- `par_all` and `par_try_all` spawn all tasks immediately
- `par_all_limit` only keeps `limit` tasks in flight at once
- For large batches with limited resources, use `par_all_limit`

## Error Handling Best Practices

### Accumulate Errors for Reporting

```rust
async fn import_records(records: Vec<Record>) -> ImportResult {
    let effects = records.iter().map(|r| validate_and_save(r));

    match Effect::par_all(effects).run(&env).await {
        Ok(results) => {
            ImportResult::success(results.len())
        }
        Err(errors) => {
            // Show user ALL validation errors, not just first
            ImportResult::failure(errors)
        }
    }
}
```

### Fail Fast for Critical Operations

```rust
async fn critical_setup() -> Effect<(), Error, AppEnv> {
    // If any step fails, abort immediately
    Effect::par_try_all([
        initialize_database(),
        connect_to_cache(),
        load_critical_config(),
    ])
    .map(|_| ())
}
```

### Graceful Degradation with Race

```rust
async fn fetch_with_fallback(key: String) -> Effect<Data, Error, AppEnv> {
    match Effect::race([
        fetch_from_cache(key.clone()),
        fetch_from_primary(key.clone()),
    ]).run(&env).await {
        Ok(data) => Ok(data),
        Err(_) => {
            // Both failed, try backup
            fetch_from_backup(key).run(&env).await
        }
    }
}
```

## Testing Parallel Effects

Parallel effects work with test environments just like sequential effects:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parallel_fetch() {
        let test_env = TestEnv {
            db: MockDatabase::new(),
        };

        let result = Effect::par_all([
            fetch_user(1),
            fetch_user(2),
            fetch_user(3),
        ]).run(&test_env).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 3);
    }
}
```

## Common Pitfalls

### Don't Use for Dependent Operations

```rust
// WRONG: second operation depends on first
Effect::par_all([
    create_user(data),
    send_welcome_email(user_id),  // Needs user_id from create_user!
])

// RIGHT: use sequential composition
create_user(data)
    .and_then(|user| send_welcome_email(user.id))
```

### Remember Sync Requirement

```rust
// WRONG: Environment not Sync
struct AppEnv {
    db: Rc<Database>,  // Rc is not Sync!
}

// RIGHT: Use Arc for shared ownership
struct AppEnv {
    db: Arc<Database>,  // Arc is Sync
}
```

## Summary

Stillwater provides four parallel execution methods:

1. **par_all** - Run all, collect all results and all errors
2. **par_try_all** - Run all, fail fast on first error
3. **race** - Return first success, collect all errors if all fail
4. **par_all_limit** - Run with concurrency limit

All methods:
- Provide true async concurrency
- Share environment safely across tasks
- Compose naturally with other Effect combinators
- Work seamlessly with the test environment pattern

Choose based on your error handling needs and resource constraints.
