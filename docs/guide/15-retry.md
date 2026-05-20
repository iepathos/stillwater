# Retry and Resilience Patterns

## Overview

Stillwater provides retry helpers for Effect-based computations. Retry policies are pure data structures, and retry execution happens at the effect boundary through free functions in `stillwater::effect::retry` or the effect prelude when the `async` feature is enabled.

## Why Retry Patterns?

Network requests fail. Databases have hiccups. External APIs rate-limit you. Robust applications need to handle transient failures gracefully:

Without retry:

```rust
let data = fetch_data().run(&env).await?;
```

With retry:

```rust
use stillwater::effect::retry::retry;
use stillwater::RetryPolicy;
use std::time::Duration;

let data = retry(
    || fetch_data(),
    RetryPolicy::exponential(Duration::from_millis(100))
        .with_max_retries(3),
)
.run(&env)
.await?
.into_value();
```

## RetryPolicy

`RetryPolicy` is a pure data structure describing retry behavior. It is composable and testable without executing any effects.

### Creating Policies

```rust
use stillwater::RetryPolicy;
use std::time::Duration;

let constant = RetryPolicy::constant(Duration::from_millis(100))
    .with_max_retries(5);

let linear = RetryPolicy::linear(Duration::from_millis(100))
    .with_max_retries(5);

let exponential = RetryPolicy::exponential(Duration::from_millis(100))
    .with_max_retries(5);

let fibonacci = RetryPolicy::fibonacci(Duration::from_millis(100))
    .with_max_retries(5);
```

### Policy Configuration

```rust
use stillwater::RetryPolicy;
use std::time::Duration;

let policy = RetryPolicy::exponential(Duration::from_millis(100))
    .with_max_retries(5)
    .with_max_delay(Duration::from_secs(30));
```

### Jitter Support

Jitter adds randomness to delays, preventing the "thundering herd" problem when many clients retry simultaneously. Enable it with the `jitter` feature:

```toml
stillwater = { version = "1.0", features = ["jitter"] }
```

```rust
use stillwater::RetryPolicy;
use std::time::Duration;

let policy = RetryPolicy::exponential(Duration::from_millis(100))
    .with_jitter(0.25)
    .with_max_retries(5);
```

## Retry Functions

### `retry` - Basic Retry

Retries an effect until it succeeds or retries are exhausted. The factory creates a fresh effect for each attempt.

```rust
use stillwater::effect::prelude::*;
use stillwater::RetryPolicy;
use std::time::Duration;

let effect = retry(
    || pure::<_, String, ()>(42),
    RetryPolicy::exponential(Duration::from_millis(100))
        .with_max_retries(3),
);

let success = effect.run(&()).await.unwrap();
assert_eq!(success.into_value(), 42);
```

### `retry_if` - Conditional Retry

Only retries when a predicate returns true for the error. Use this to distinguish transient errors from permanent failures.

```rust
use stillwater::effect::prelude::*;
use stillwater::RetryPolicy;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
enum ApiError {
    Transient,
    Permanent,
}

let effect = retry_if(
    || fail::<(), _, ()>(ApiError::Permanent),
    RetryPolicy::constant(Duration::from_millis(10)).with_max_retries(3),
    |err| matches!(err, ApiError::Transient),
);

assert_eq!(effect.run(&()).await, Err(ApiError::Permanent));
```

### `retry_with_hooks` - Retry With Observability

`retry_with_hooks` invokes a synchronous callback before each retry. Use the hook for logging, metrics, or lightweight alerting.

```rust
use stillwater::effect::prelude::*;
use stillwater::{RetryEvent, RetryPolicy};
use std::time::Duration;

let effect = retry_with_hooks(
    || pure::<_, String, ()>(42),
    RetryPolicy::exponential(Duration::from_millis(100)).with_max_retries(3),
    |event: &RetryEvent<'_, String>| {
        tracing::warn!(
            attempt = event.attempt,
            next_delay = ?event.next_delay,
            "retrying failed operation"
        );
    },
);
```

The `RetryEvent` contains:

- `attempt` - Which attempt just failed, using 1-based numbering
- `error` - The error that occurred
- `next_delay` - How long until the next retry, or `None` when exhausted
- `elapsed` - Total time elapsed since the first attempt

## Timeout Support

### `with_timeout`

Wrap an effect with a timeout:

```rust
use stillwater::effect::prelude::*;
use stillwater::TimeoutError;
use std::time::Duration;

let effect = with_timeout(
    from_async(|_: &()| async {
        tokio::time::sleep(Duration::from_millis(50)).await;
        Ok::<_, String>(42)
    }),
    Duration::from_millis(1),
);

match effect.run(&()).await {
    Err(TimeoutError::Timeout { duration }) => {
        assert_eq!(duration, Duration::from_millis(1));
    }
    other => panic!("expected timeout, got {:?}", other),
}
```

### Combining Retry With Timeout

A common pattern is a per-attempt timeout inside a retry factory:

```rust
use stillwater::effect::prelude::*;
use stillwater::{RetryPolicy, TimeoutError};
use std::time::Duration;

let effect = retry(
    || {
        with_timeout(fetch_data(), Duration::from_secs(5))
            .map_err(|err| match err {
                TimeoutError::Timeout { .. } => ApiError::Transient("timeout".into()),
                TimeoutError::Inner(err) => err,
            })
    },
    RetryPolicy::exponential(Duration::from_millis(100))
        .with_max_retries(3),
);
```

## Result Types

### `RetryExhausted<E>`

`retry` and `retry_with_hooks` return retry metadata on both outcomes:

- `Ok(RetryExhausted<T>)` when an attempt eventually succeeds
- `Err(RetryExhausted<E>)` when retries are exhausted

The wrapper currently stores the inner value in `final_error` and exposes `into_value()` for extraction.

```rust
use std::time::Duration;

pub struct RetryExhausted<E> {
    pub final_error: E,
    pub attempts: u32,
    pub total_duration: Duration,
}
```

### `TimeoutError<E>`

`with_timeout` wraps timeout and inner errors:

```rust
pub enum TimeoutError<E> {
    Timeout { duration: Duration },
    Inner(E),
}
```

## Real-World Patterns

### HTTP Client With Retry

```rust
use stillwater::effect::prelude::*;
use stillwater::RetryPolicy;
use std::time::Duration;

#[derive(Debug, Clone)]
enum HttpError {
    Timeout,
    ServerError(u16),
    ClientError(u16),
}

fn is_retryable(err: &HttpError) -> bool {
    matches!(err, HttpError::Timeout | HttpError::ServerError(_))
}

let effect = retry_if(
    || http_get(url),
    RetryPolicy::exponential(Duration::from_millis(100))
        .with_max_retries(5)
        .with_max_delay(Duration::from_secs(30)),
    is_retryable,
);
```

### Database Connection With Hooks

```rust
use stillwater::effect::prelude::*;
use stillwater::RetryPolicy;
use std::time::Duration;

let effect = retry_with_hooks(
    || connect_to_db(),
    RetryPolicy::exponential(Duration::from_secs(1))
        .with_max_retries(10)
        .with_max_delay(Duration::from_secs(60)),
    |event| {
        if event.attempt >= 3 {
            tracing::error!(attempt = event.attempt, "database connection still failing");
        }
    },
);
```

### Robust API Call

Combine per-attempt timeout, conditional retry, max delay, jitter, and hooks when calling an unreliable external service:

```rust
use stillwater::effect::prelude::*;
use stillwater::{RetryEvent, RetryPolicy, TimeoutError};
use std::time::Duration;

#[derive(Debug, Clone)]
enum ApiError {
    Transient(String),
    Permanent(String),
}

fn is_retryable(err: &ApiError) -> bool {
    matches!(err, ApiError::Transient(_))
}

let policy = RetryPolicy::exponential(Duration::from_millis(500))
    .with_max_retries(5)
    .with_max_delay(Duration::from_secs(30))
    .with_jitter(0.25);

let effect = retry_with_hooks(
    || {
        with_timeout(call_api(), Duration::from_secs(10))
            .map_err(|err| match err {
                TimeoutError::Timeout { .. } => ApiError::Transient("timeout".into()),
                TimeoutError::Inner(err) => err,
            })
            .and_then(|response| {
                if response.status().is_server_error() {
                    fail(ApiError::Transient(format!("status {}", response.status())))
                } else if response.status().is_client_error() {
                    fail(ApiError::Permanent(format!("status {}", response.status())))
                } else {
                    pure(response)
                }
            })
    },
    policy,
    |event: &RetryEvent<'_, ApiError>| {
        tracing::warn!(
            attempt = event.attempt,
            next_delay = ?event.next_delay,
            "retrying API call"
        );
    },
);
```

For conditional retry without hooks, use `retry_if` around the same per-attempt effect:

```rust
let effect = retry_if(
    || {
        with_timeout(call_api(), Duration::from_secs(10))
            .map_err(|err| match err {
                TimeoutError::Timeout { .. } => ApiError::Transient("timeout".into()),
                TimeoutError::Inner(err) => err,
            })
    },
    RetryPolicy::exponential(Duration::from_millis(500)).with_max_retries(5),
    is_retryable,
);
```

### Policy Testing

`RetryPolicy` is just data, so you can test retry timing without running any effects:

```rust
use stillwater::RetryPolicy;
use std::time::Duration;

let policy = RetryPolicy::exponential(Duration::from_millis(100))
    .with_max_retries(3)
    .with_max_delay(Duration::from_millis(250));

assert_eq!(policy.delay_for_attempt(0), Some(Duration::from_millis(100)));
assert_eq!(policy.delay_for_attempt(1), Some(Duration::from_millis(200)));
assert_eq!(policy.delay_for_attempt(2), Some(Duration::from_millis(250)));
assert_eq!(policy.delay_for_attempt(3), None);
```

### Circuit-Breaker Integration

Stillwater does not include a circuit breaker in the retry module. Keep circuit state in your environment and use `from_fn` or `check` before retrying:

```rust
use stillwater::effect::prelude::*;

fn guarded_call() -> impl Effect<Output = ApiResponse, Error = ApiError, Env = AppEnv> {
    from_fn(|env: &AppEnv| {
        if env.circuit_breaker.is_open("api") {
            Err(ApiError::Permanent("circuit open".into()))
        } else {
            Ok(())
        }
    })
    .and_then(|_| call_api())
}

let effect = retry_if(
    || guarded_call(),
    RetryPolicy::exponential(Duration::from_millis(250)).with_max_retries(3),
    |err| matches!(err, ApiError::Transient(_)),
);
```

## Behavior Notes

- `retry` and `retry_with_hooks` preserve retry metadata on success and failure.
- `retry_if` returns the original success or error type directly.
- Retry factories should create a fresh effect for each attempt.
- Hooks are synchronous; keep them lightweight and non-blocking.
- Jitter requires the `jitter` feature.
- Timeout helpers require the `async` feature.

## Best Practices

1. Use exponential backoff for most network and service calls.
2. Add jitter when many clients might retry simultaneously.
3. Set `max_delay` to prevent unreasonably long waits.
4. Use `retry_if` to avoid retrying permanent errors such as auth failures.
5. Add per-attempt timeouts for operations that might hang.
6. Use `retry_with_hooks` for retry logs and metrics.

## See Also

- [examples/retry_patterns.rs](../../examples/retry_patterns.rs) - Comprehensive retry examples
- [Parallel Effects](11-parallel-effects.md) - Running effects concurrently
- [Error Context](04-error-context.md) - Adding context to errors
