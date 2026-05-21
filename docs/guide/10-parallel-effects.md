# Parallel Effect Execution

## Overview

Stillwater provides free functions for running independent effects concurrently while preserving the same environment and error model used by sequential effects.

There are two families of parallel helpers:

- Fixed-arity helpers: `par2`, `par3`, and `par4` for heterogeneous effects without boxing.
- Collection helpers: `par_all`, `par_try_all`, `race`, and `par_all_limit` for homogeneous `Vec<BoxedEffect<...>>` batches.

This guide focuses on when to use each helper and how to structure real application code around them.

## Why Parallel Effects?

Many real-world operations are independent and can run concurrently:

- Fetching multiple records from a database
- Making several API calls at once
- Loading independent configuration sources
- Validating independent inputs
- Processing batches with a concurrency limit

Sequential execution waits for each operation before starting the next:

```rust
let user = fetch_user(id).run(&env).await?;
let settings = fetch_settings(id).run(&env).await?;
let preferences = fetch_preferences(id).run(&env).await?;
```

Parallel execution starts independent work together:

```rust
use stillwater::effect::prelude::*;

let (user, settings, preferences) = par3(
    fetch_user(id),
    fetch_settings(id),
    fetch_preferences(id),
    &env,
).await;

let profile = UserProfile {
    user: user?,
    settings: settings?,
    preferences: preferences?,
};
```

## Choosing A Helper

| Need | Helper | Shape |
|------|--------|-------|
| 2-4 effects with different output types | `par2`, `par3`, `par4` | Returns a tuple of `Result`s |
| A batch where all errors should be reported | `par_all` | `Result<Vec<T>, Vec<E>>` |
| A batch where one error is enough | `par_try_all` | `Result<Vec<T>, E>` |
| The first completed result should decide | `race` | `Result<T, E>` |
| A large batch needs bounded concurrency | `par_all_limit` | `Result<Vec<T>, Vec<E>>` |

Collection helpers require boxed effects because a `Vec` needs one concrete item type:

```rust
use stillwater::effect::prelude::*;

let effects: Vec<BoxedEffect<User, DbError, AppEnv>> = user_ids
    .into_iter()
    .map(|id| fetch_user(id).boxed())
    .collect();

let users = par_all(effects, &env).await?;
```

## Heterogeneous Parallel Effects

Use `par2`, `par3`, or `par4` when effects have different output types, or when you want to avoid boxing.

```rust
use stillwater::effect::prelude::*;

let (price, inventory, shipping) = par3(
    fetch_price(item_id),
    fetch_inventory(item_id),
    fetch_shipping_options(item_id),
    &env,
).await;

let quote = Quote {
    price: price?,
    inventory: inventory?,
    shipping: shipping?,
};
```

These helpers return a tuple of results instead of short-circuiting. That makes each outcome explicit:

```rust
let (database, cache) = par2(check_database(), check_cache(), &env).await;

match (database, cache) {
    (Ok(db), Ok(cache)) => Health::healthy(db, cache),
    (db_result, cache_result) => Health::degraded(db_result.err(), cache_result.err()),
}
```

This is useful for diagnostics and health checks where you want to inspect every independent subsystem.

## `par_all` - Collect All Results Or All Errors

Use `par_all` when every operation should run to completion and callers benefit from a complete error report.

```rust
use stillwater::effect::prelude::*;

async fn validate_import(records: Vec<Record>, env: &AppEnv) -> Result<Vec<ValidRecord>, Vec<ValidationError>> {
    let effects: Vec<BoxedEffect<ValidRecord, ValidationError, AppEnv>> = records
        .into_iter()
        .map(|record| validate_record(record).boxed())
        .collect();

    par_all(effects, env).await
}
```

If any effect fails, `par_all` returns all failures:

```rust
let effects: Vec<BoxedEffect<i32, String, ()>> = vec![
    pure(1).boxed(),
    fail("bad input".to_string()).boxed(),
    fail("missing field".to_string()).boxed(),
];

let result = par_all(effects, &()).await;
assert_eq!(
    result,
    Err(vec!["bad input".to_string(), "missing field".to_string()])
);
```

This is the right choice for form validation, import validation, batch reporting, and admin tools where users need a full list of failures.

## `par_try_all` - Return A Single Error

Use `par_try_all` when one error is enough for the caller.

```rust
use stillwater::effect::prelude::*;

async fn load_required_services(env: &AppEnv) -> Result<Vec<ServiceStatus>, ServiceError> {
    let checks: Vec<BoxedEffect<ServiceStatus, ServiceError, AppEnv>> = vec![
        check_database().boxed(),
        check_cache().boxed(),
        check_queue().boxed(),
    ];

    par_try_all(checks, env).await
}
```

`par_try_all` awaits the batch and then collects with normal `Result` semantics, returning the first error in result order. It is not a cancellation primitive.

```rust
let effects: Vec<BoxedEffect<i32, String, ()>> = vec![
    pure(1).boxed(),
    fail("first error".to_string()).boxed(),
    fail("second error".to_string()).boxed(),
];

let result = par_try_all(effects, &()).await;
assert_eq!(result, Err("first error".to_string()));
```

Use `par_all` when you need every error. Use `par_try_all` when the caller only needs to know that the batch failed.

## `race` - First Completed Result

Use `race` when the first completed result should decide the outcome.

```rust
use stillwater::effect::prelude::*;

async fn fetch_from_fastest_replica(key: String, env: &AppEnv) -> Result<Data, FetchError> {
    let effects: Vec<BoxedEffect<Data, FetchError, AppEnv>> = vec![
        fetch_from_replica_a(key.clone()).boxed(),
        fetch_from_replica_b(key.clone()).boxed(),
        fetch_from_replica_c(key).boxed(),
    ];

    race(effects, env).await
}
```

`race` returns the first completed result, whether success or error. It does not wait to find the first success.

```rust
let effects: Vec<BoxedEffect<i32, String, ()>> = vec![
    fail("fast failure".to_string()).boxed(),
    pure(42).boxed(),
];

let result = race(effects, &()).await;
assert_eq!(result, Err("fast failure".to_string()));
```

This behavior is useful for deadline effects, fastest-result wins workflows, or cases where a fast failure should abort the attempt. For fallback semantics where failures should be ignored until every source fails, compose effects with `or_else`, `fallback_to`, or explicit retry/fallback logic instead of `race`.

## `par_all_limit` - Bounded Concurrency

Use `par_all_limit` for large batches or limited resources such as connection pools, file descriptors, or API rate limits.

```rust
use stillwater::effect::prelude::*;

async fn process_queue(
    queue: Vec<WorkItem>,
    max_concurrent: usize,
    env: &AppEnv,
) -> Result<Vec<ProcessedItem>, Vec<ProcessingError>> {
    let effects: Vec<BoxedEffect<ProcessedItem, ProcessingError, AppEnv>> = queue
        .into_iter()
        .map(|item| process_item(item).boxed())
        .collect();

    par_all_limit(effects, max_concurrent, env).await
}
```

The function still runs every effect and collects all errors, but it only keeps `limit` futures in flight at once.

```rust
let effects: Vec<BoxedEffect<i32, String, ()>> = (1..=10)
    .map(|n| pure(n).boxed())
    .collect();

let result = par_all_limit(effects, 3, &()).await;
assert_eq!(result.as_ref().map(|values| values.len()), Ok(10));
```

## Environment Access

Parallel helpers receive a shared `&Env`. Boxed collection helpers require `Env: Clone + Send + Sync + 'static`, so application environments usually store services in cheap-to-clone handles:

```rust
use std::sync::Arc;

#[derive(Clone)]
struct AppEnv {
    config: Arc<Config>,
    db: Arc<DatabasePool>,
    http: Arc<HttpClient>,
}
```

Each effect still controls how it uses the environment:

```rust
use stillwater::effect::prelude::*;

fn fetch_user(id: UserId) -> impl Effect<Output = User, Error = DbError, Env = AppEnv> {
    from_async(move |env: &AppEnv| {
        let db = env.db.clone();
        async move { db.fetch_user(id).await }
    })
}
```

## Composing Parallel And Sequential Work

Parallel work often appears inside a larger sequential workflow. Use normal Rust control flow around the async helper calls:

```rust
use stillwater::effect::prelude::*;

async fn build_dashboard(user_id: UserId, env: &AppEnv) -> Result<Dashboard, AppError> {
    let user = fetch_user(user_id).run(env).await?;

    let (activity, recommendations, alerts) = par3(
        fetch_activity(user.id),
        fetch_recommendations(user.id),
        fetch_alerts(user.id),
        env,
    ).await;

    Ok(Dashboard {
        user,
        activity: activity?,
        recommendations: recommendations?,
        alerts: alerts?,
    })
}
```

For a second parallel phase, build another effect collection after the first phase succeeds:

```rust
async fn load_and_save(ids: Vec<UserId>, env: &AppEnv) -> Result<Vec<Receipt>, Vec<AppError>> {
    let load_effects: Vec<BoxedEffect<User, AppError, AppEnv>> = ids
        .into_iter()
        .map(|id| fetch_user(id).boxed())
        .collect();

    let users = par_all(load_effects, env).await?;

    let save_effects: Vec<BoxedEffect<Receipt, AppError, AppEnv>> = users
        .into_iter()
        .map(|user| save_user_snapshot(user).boxed())
        .collect();

    par_all(save_effects, env).await
}
```

## Practical Patterns

### Parallel Validation

Use `par_all` when expensive validation checks can run independently and the user should see all failures:

```rust
async fn validate_signup(data: SignupData, env: &AppEnv) -> Result<ValidSignup, Vec<SignupError>> {
    let effects: Vec<BoxedEffect<FieldCheck, SignupError, AppEnv>> = vec![
        validate_email(data.email).boxed(),
        validate_username(data.username).boxed(),
        validate_password(data.password).boxed(),
    ];

    let checks = par_all(effects, env).await?;
    Ok(ValidSignup::from_checks(checks))
}
```

### Health Checks

Use fixed-arity helpers when each subsystem has a distinct result:

```rust
async fn health(env: &AppEnv) -> HealthReport {
    let (database, cache, queue) = par3(
        check_database(),
        check_cache(),
        check_queue(),
        env,
    ).await;

    HealthReport {
        database,
        cache,
        queue,
    }
}
```

### Rate-Limited API Imports

Use `par_all_limit` when the remote system enforces a concurrency cap:

```rust
async fn import_customers(customers: Vec<Customer>, env: &AppEnv) -> ImportSummary {
    let effects: Vec<BoxedEffect<ImportReceipt, ImportError, AppEnv>> = customers
        .into_iter()
        .map(|customer| send_customer(customer).boxed())
        .collect();

    match par_all_limit(effects, 10, env).await {
        Ok(receipts) => ImportSummary::success(receipts),
        Err(errors) => ImportSummary::failure(errors),
    }
}
```

### Fastest Completed Source

Use `race` only when "first completed" is the desired behavior:

```rust
async fn query_fastest_index(term: SearchTerm, env: &AppEnv) -> Result<SearchResults, SearchError> {
    let effects: Vec<BoxedEffect<SearchResults, SearchError, AppEnv>> = vec![
        query_primary_index(term.clone()).boxed(),
        query_replica_index(term).boxed(),
    ];

    race(effects, env).await
}
```

If a fast failure should not win, use a fallback chain:

```rust
fn query_with_fallback(term: SearchTerm) -> impl Effect<Output = SearchResults, Error = SearchError, Env = AppEnv> {
    query_primary_index(term.clone())
        .fallback_to(query_replica_index(term))
}
```

## Performance Considerations

### Actual Concurrency

The parallel helpers use async concurrency. They do not spawn OS threads by themselves; each effect must be asynchronous or otherwise yield for concurrency to matter.

```rust
use stillwater::effect::prelude::*;
use std::time::{Duration, Instant};

let start = Instant::now();

let effects: Vec<BoxedEffect<(), String, ()>> = (0..3)
    .map(|_| {
        from_async(|_: &()| async {
            tokio::time::sleep(Duration::from_millis(100)).await;
            Ok::<_, String>(())
        })
        .boxed()
    })
    .collect();

par_all(effects, &()).await.unwrap();

let elapsed = start.elapsed();
assert!(elapsed < Duration::from_millis(200));
```

### Boxing Cost

Fixed-arity helpers avoid boxing and are the best fit for small, known sets of independent effects.

Collection helpers require boxing because a vector needs one concrete type. Prefer collection helpers for dynamic or large batches where the allocation cost is dominated by I/O.

### Memory Usage

- `par_all` and `par_try_all` keep the whole batch in flight.
- `race` keeps the whole batch in flight until the first result completes.
- `par_all_limit` keeps at most `limit` effects in flight and is the safer default for large batches.

## Testing Parallel Effects

Parallel effects use the same environment pattern as sequential effects:

```rust
#[tokio::test]
async fn loads_users_in_parallel() {
    let env = TestEnv::with_users(vec![
        User::new(1),
        User::new(2),
        User::new(3),
    ]);

    let effects: Vec<BoxedEffect<User, TestError, TestEnv>> = vec![
        fetch_user(1).boxed(),
        fetch_user(2).boxed(),
        fetch_user(3).boxed(),
    ];

    let users = par_all(effects, &env).await.unwrap();
    assert_eq!(users.len(), 3);
}
```

For timing-sensitive tests, keep assertions loose enough to avoid flakes. Prefer testing result shape and concurrency limits over exact elapsed time.

## Common Pitfalls

### Do Not Parallelize Dependent Operations

```rust
// Wrong: sending the email needs the user returned by create_user.
let effects: Vec<BoxedEffect<(), AppError, AppEnv>> = vec![
    create_user(data).map(|_| ()).boxed(),
    send_welcome_email(user_id).boxed(),
];

// Right: compose dependent work sequentially.
create_user(data)
    .and_then(|user| send_welcome_email(user.id))
```

### Do Not Use `race` For "First Success"

`race` returns the first completed result. A fast error wins over a slower success. If you need "try primary, then backup," use `fallback_to` or `or_else`.

### Use `Arc`, Not `Rc`, In Shared Environments

```rust
// Wrong: Rc is not Send + Sync.
struct AppEnv {
    db: Rc<DatabasePool>,
}

// Right: Arc works in async shared environments.
#[derive(Clone)]
struct AppEnv {
    db: Arc<DatabasePool>,
}
```

### Box At Collection Boundaries

Keep individual effect builders zero-cost, and box only when placing them into a homogeneous collection:

```rust
fn fetch_user(id: UserId) -> impl Effect<Output = User, Error = DbError, Env = AppEnv> {
    from_async(move |env: &AppEnv| {
        let db = env.db.clone();
        async move { db.fetch_user(id).await }
    })
}

let effects: Vec<BoxedEffect<User, DbError, AppEnv>> = ids
    .into_iter()
    .map(|id| fetch_user(id).boxed())
    .collect();
```

## Summary

- Use `par2`, `par3`, and `par4` for small heterogeneous sets without boxing.
- Use `par_all` when all errors should be reported.
- Use `par_try_all` when one error is enough, but do not treat it as cancellation.
- Use `race` when the first completed result should decide the outcome.
- Use `par_all_limit` to protect connection pools, memory, rate limits, and other bounded resources.
