# Retry and Resilience Patterns

## Overview

Stillwater provides comprehensive retry and resilience capabilities for Effect-based computations. Following the "pure core, imperative shell" philosophy, retry policies are pure data structures that describe retry behavior declaratively, while the actual retry execution happens at the effect boundaries.

## Why Retry Patterns?

Network requests fail. Databases have hiccups. External APIs rate-limit you. Robust applications need to handle transient failures gracefully:

Without retry:
```rust
// Fragile - fails on any transient error
let data = fetch_data().run(&env).await?;
```

With retry:
```rust
// Resilient - retries transient failures with backoff
let data = Effect::retry(
    || fetch_data(),
    RetryPolicy::exponential(Duration::from_millis(100))
        .with_max_retries(3)
).run(&env).await?;
```

## RetryPolicy

`RetryPolicy` is a pure data structure describing retry behavior. It's composable and testable without executing any effects.

### Creating Policies

```rust
use stillwater::RetryPolicy;
use std::time::Duration;

// Constant delay between retries
let constant = RetryPolicy::constant(Duration::from_millis(100))
    .with_max_retries(5);

// Linear backoff: 100ms, 200ms, 300ms, ...
let linear = RetryPolicy::linear(Duration::from_millis(100))
    .with_max_retries(5);

// Exponential backoff: 100ms, 200ms, 400ms, 800ms, ...
let exponential = RetryPolicy::exponential(Duration::from_millis(100))
    .with_max_retries(5);

// Fibonacci backoff: 100ms, 100ms, 200ms, 300ms, 500ms, ...
let fibonacci = RetryPolicy::fibonacci(Duration::from_millis(100))
    .with_max_retries(5);
```

### Policy Configuration

```rust
let policy = RetryPolicy::exponential(Duration::from_millis(100))
    .with_max_retries(5)        // Maximum 5 retry attempts
    .with_max_delay(Duration::from_secs(30));  // Cap delay at 30s
```

### Jitter Support

Jitter adds randomness to delays, preventing the "thundering herd" problem when many clients retry simultaneously. Enable with the `jitter` feature:

```toml
stillwater = { version = "0.8", features = ["jitter"] }
```

```rust
// Add ±25% randomness to delays
let policy = RetryPolicy::exponential(Duration::from_millis(100))
    .with_jitter(0.25)
    .with_max_retries(5);
```

## Effect Retry Methods

### `Effect::retry()` - Basic Retry

Retries an effect until it succeeds or retries are exhausted.

```rust
use stillwater::{Effect, RetryPolicy};
use std::time::Duration;

let effect = Effect::retry(
    || fetch_data(),  // Factory function creates fresh effect each attempt
    RetryPolicy::exponential(Duration::from_millis(100))
        .with_max_retries(3)
);

match effect.run(&env).await {
    Ok(success) => {
        println!("Succeeded after {} attempts", success.attempts);
        let value = success.into_value();
    }
    Err(exhausted) => {
        println!("Failed after {} attempts: {}", exhausted.attempts, exhausted.final_error);
    }
}
```

### `Effect::retry_if()` - Conditional Retry

Only retry when a predicate returns true for the error. Useful for distinguishing transient from permanent errors.

```rust
#[derive(Debug, Clone)]
enum ApiError {
    Transient(String),  // Network timeout, 503, etc.
    Permanent(String),  // 401 Unauthorized, 404 Not Found
}

let effect = Effect::retry_if(
    || call_api(),
    RetryPolicy::exponential(Duration::from_millis(100))
        .with_max_retries(5),
    |err| matches!(err, ApiError::Transient(_)),  // Only retry transient errors
);
```

### `Effect::retry_with_hooks()` - Retry with Observability

Get callbacks before each retry for logging, metrics, or custom behavior.

```rust
let effect = Effect::retry_with_hooks(
    || fetch_data(),
    RetryPolicy::exponential(Duration::from_millis(100))
        .with_max_retries(5),
    |event| {
        // Called before each retry
        log::warn!(
            "Attempt {} failed: {:?}, retrying in {:?}",
            event.attempt,
            event.error,
            event.next_delay
        );
        metrics::counter!("api.retry").increment(1);
    },
);
```

The `RetryEvent` contains:
- `attempt` - Which attempt just failed (1-indexed)
- `error` - The error that occurred
- `next_delay` - How long until next retry (None if exhausted)
- `elapsed` - Total time elapsed since first attempt

## Timeout Support

### `Effect::with_timeout()`

Add a timeout to any effect:

```rust
use stillwater::TimeoutError;

let effect = fetch_data()
    .with_timeout(Duration::from_secs(5));

match effect.run(&env).await {
    Ok(data) => println!("Got data: {:?}", data),
    Err(TimeoutError::Timeout { duration }) => {
        println!("Timed out after {:?}", duration);
    }
    Err(TimeoutError::Inner(e)) => {
        println!("Inner error: {:?}", e);
    }
}
```

### Combining Retry with Timeout

A common pattern is per-attempt timeouts with retry:

```rust
let effect = Effect::retry(
    || {
        fetch_data()
            .with_timeout(Duration::from_secs(5))
            .map_err(|e| format!("{}", e))
    },
    RetryPolicy::exponential(Duration::from_millis(100))
        .with_max_retries(3)
);
```

## Error Types

### `RetryExhausted<E>`

Returned when all retries fail:

```rust
pub struct RetryExhausted<E> {
    pub final_error: E,      // The last error encountered
    pub attempts: u32,       // Total number of attempts made
    pub elapsed: Duration,   // Total time spent retrying
}
```

### `RetrySuccess<T>`

Returned on successful retry:

```rust
pub struct RetrySuccess<T> {
    value: T,
    pub attempts: u32,       // How many attempts it took
    pub elapsed: Duration,   // Total time including retries
}

impl<T> RetrySuccess<T> {
    pub fn into_value(self) -> T { self.value }
    pub fn value(&self) -> &T { &self.value }
}
```

### `TimeoutError<E>`

Wraps timeout or inner errors:

```rust
pub enum TimeoutError<E> {
    Timeout { duration: Duration },
    Inner(E),
}
```

## Real-World Patterns

### HTTP Client with Retry

```rust
#[derive(Debug, Clone)]
enum HttpError {
    Timeout,
    ServerError(u16),  // 5xx
    ClientError(u16),  // 4xx
}

fn is_retryable(err: &HttpError) -> bool {
    matches!(err, HttpError::Timeout | HttpError::ServerError(_))
}

let effect = Effect::retry_if(
    || http_get(url),
    RetryPolicy::exponential(Duration::from_millis(100))
        .with_max_retries(5)
        .with_max_delay(Duration::from_secs(30)),
    is_retryable,
);
```

### Database Connection with Circuit Breaker Pattern

```rust
let effect = Effect::retry_with_hooks(
    || connect_to_db(),
    RetryPolicy::exponential(Duration::from_secs(1))
        .with_max_retries(10)
        .with_max_delay(Duration::from_secs(60)),
    |event| {
        if event.attempt >= 3 {
            log::error!("Database connection failing, alerting on-call");
            alert_oncall(&format!("DB connection failed {} times", event.attempt));
        }
    },
);
```

### Robust API Call

Combining per-attempt timeout, conditional retry, and observability:

```rust
let effect = Effect::retry_if(
    || {
        call_api()
            .with_timeout(Duration::from_secs(10))
            .map_err(|e| match e {
                TimeoutError::Timeout { .. } => ApiError::Transient("timeout".into()),
                TimeoutError::Inner(e) => e,
            })
    },
    RetryPolicy::exponential(Duration::from_millis(500))
        .with_max_retries(5)
        .with_max_delay(Duration::from_secs(30))
        .with_jitter(0.25),
    |err| matches!(err, ApiError::Transient(_)),
);
```

## Best Practices

1. **Use exponential backoff** for most cases - it prevents overwhelming recovering services
2. **Add jitter** when multiple clients might retry simultaneously
3. **Set max_delay** to prevent unreasonably long waits
4. **Use retry_if** to avoid retrying permanent errors (auth failures, not found, etc.)
5. **Add per-attempt timeouts** to prevent hanging on slow operations
6. **Log retries** with retry_with_hooks for operational visibility
7. **Consider circuit breakers** for critical dependencies (alert after N failures)

## See Also

- [examples/retry_patterns.rs](../../examples/retry_patterns.rs) - Comprehensive retry examples
- [Parallel Effects](11-parallel-effects.md) - Running effects concurrently
- [Error Context](04-error-context.md) - Adding context to errors
