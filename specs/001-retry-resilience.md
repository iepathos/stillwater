---
number: 1
title: Retry and Resilience Patterns
category: foundation
priority: high
status: draft
dependencies: []
created: 2025-01-24
---

# Specification 001: Retry and Resilience Patterns

**Category**: foundation
**Priority**: high
**Status**: draft
**Dependencies**: None (builds on existing Effect type)

## Context

Distributed systems and I/O operations fail. Network requests timeout, databases become temporarily unavailable, rate limits get hit. Production-grade applications need resilience patterns to handle transient failures gracefully.

Currently, stillwater's `Effect` type provides `or_else` for basic error recovery, but it lacks:
- Automatic retry with configurable policies
- Backoff strategies (exponential, linear, constant)
- Jitter to prevent thundering herd
- Circuit breakers to fail fast when downstream is unhealthy
- Timeout handling integrated with effects
- Retry budgets to prevent infinite retry storms

### Why This Matters for Stillwater

Stillwater's philosophy is **"pure core, imperative shell"**—keeping business logic pure while effects handle I/O at boundaries. Resilience patterns are fundamentally about **how** effects execute, not **what** they compute. This makes them a natural fit for the Effect type.

By modeling retry policies as **pure data** and retry execution as **effect combinators**, we maintain the stillwater philosophy:
- **Pure**: `RetryPolicy` is just data—no side effects, easily testable
- **Composable**: Policies can be combined and transformed
- **Declarative**: Describe *what* retry behavior you want, not *how* to implement it

### Prior Art

- **tokio-retry**: Good retry logic but not integrated with effect patterns
- **backoff**: Solid backoff algorithms but callback-based
- **tower**: Excellent middleware approach but requires tower ecosystem buy-in
- **resilience4j (Java)**: Comprehensive resilience library, good API design inspiration

## Objective

Add retry and resilience capabilities to stillwater that:
1. Feel native to the Effect type (not bolted-on)
2. Model policies as pure, composable data structures
3. Integrate with existing Effect combinators
4. Provide sensible defaults while allowing full customization
5. Support both sync and async operations

## Requirements

### Functional Requirements

#### FR-1: Retry Policies as Pure Data
```rust
// Policies are just data - no behavior, no side effects
let policy = RetryPolicy::exponential(Duration::from_millis(100))
    .with_max_retries(5)
    .with_max_delay(Duration::from_secs(30))
    .with_jitter(0.1);

// Policies are Clone, Debug, PartialEq - easy to test and inspect
assert_eq!(policy.max_retries(), Some(5));
```

#### FR-2: Built-in Retry Strategies
- **Constant**: Fixed delay between retries
- **Linear**: Delay increases linearly (100ms, 200ms, 300ms, ...)
- **Exponential**: Delay doubles each retry (100ms, 200ms, 400ms, ...)
- **Fibonacci**: Delay follows Fibonacci sequence
- **Custom**: User-provided delay function

#### FR-3: Jitter Support
```rust
// Add randomness to prevent thundering herd
RetryPolicy::exponential(Duration::from_millis(100))
    .with_jitter(0.25)  // ±25% randomness

// Full jitter (AWS recommended)
RetryPolicy::exponential(Duration::from_millis(100))
    .with_full_jitter()  // Random between 0 and calculated delay

// Decorrelated jitter
RetryPolicy::exponential(Duration::from_millis(100))
    .with_decorrelated_jitter()
```

#### FR-4: Retry Conditions
```rust
// Retry only on specific errors
let policy = RetryPolicy::exponential(Duration::from_millis(100))
    .retry_if(|err: &AppError| err.is_transient())
    .retry_if(|err: &AppError| matches!(err, AppError::Timeout | AppError::RateLimit));

// Never retry certain errors
let policy = RetryPolicy::exponential(Duration::from_millis(100))
    .stop_on(|err: &AppError| err.is_permanent());
```

#### FR-5: Effect Integration
```rust
// Primary API: combinator on Effect
let effect = fetch_data()
    .with_retry(RetryPolicy::exponential(Duration::from_millis(100)).with_max_retries(3));

// Alternative: wrap existing effect
let effect = Effect::retry(fetch_data(), policy);

// With condition
let effect = fetch_data()
    .retry_if(
        RetryPolicy::exponential(Duration::from_millis(100)),
        |err| err.is_transient()
    );
```

#### FR-6: Retry Events and Observability
```rust
// Hook into retry lifecycle for logging/metrics
let effect = fetch_data()
    .with_retry(policy)
    .on_retry(|event: RetryEvent<E>| {
        log::warn!(
            "Retry attempt {} after {:?}: {:?}",
            event.attempt,
            event.delay,
            event.error
        );
        Effect::pure(())
    });
```

#### FR-7: Timeout Integration
```rust
// Timeout for individual attempts
let effect = fetch_data()
    .with_timeout(Duration::from_secs(5))
    .with_retry(policy);

// Timeout for entire retry sequence
let effect = fetch_data()
    .with_retry(policy)
    .with_total_timeout(Duration::from_secs(60));
```

#### FR-8: Circuit Breaker (Phase 2)
```rust
// Fail fast when downstream is unhealthy
let breaker = CircuitBreaker::new()
    .with_failure_threshold(5)
    .with_success_threshold(2)
    .with_half_open_timeout(Duration::from_secs(30));

let effect = fetch_data()
    .with_circuit_breaker(breaker);
```

### Non-Functional Requirements

#### NFR-1: Zero-Cost When Not Used
- No runtime overhead for effects that don't use retry
- Retry policy storage should be minimal (stack-allocated where possible)

#### NFR-2: Predictable Memory Usage
- Retry state should not grow unboundedly
- Failed attempt errors should be discardable (not accumulated by default)

#### NFR-3: Testability
- Retry policies must be testable without actual delays
- Provide test utilities for simulating retry scenarios
- Support deterministic jitter for reproducible tests

#### NFR-4: Async-First
- All retry operations must be async-compatible
- Use tokio::time for delays (feature-gated)
- Support cancellation via standard async patterns

## Acceptance Criteria

- [ ] `RetryPolicy` struct with builder pattern for configuration
- [ ] Constant, linear, exponential, and Fibonacci backoff strategies
- [ ] Jitter support (proportional, full, decorrelated)
- [ ] `with_retry` combinator on `Effect<T, E, Env>`
- [ ] `retry_if` combinator with predicate for conditional retry
- [ ] `with_timeout` combinator for per-attempt timeout
- [ ] `on_retry` hook for observability
- [ ] `RetryError<E>` type that wraps the final error with attempt count
- [ ] Comprehensive unit tests for all retry strategies
- [ ] Property-based tests for backoff calculations
- [ ] Integration tests demonstrating real-world patterns
- [ ] Documentation with examples in module docs
- [ ] Example file: `examples/retry_patterns.rs`

## Technical Details

### Implementation Approach

#### Core Types

```rust
/// A retry policy describing how to retry failed operations.
///
/// Policies are pure data - they describe retry behavior but don't execute it.
#[derive(Debug, Clone, PartialEq)]
pub struct RetryPolicy {
    strategy: RetryStrategy,
    max_retries: Option<u32>,
    max_delay: Option<Duration>,
    jitter: JitterStrategy,
    // Note: retry_if predicate stored separately due to Fn trait object requirements
}

#[derive(Debug, Clone, PartialEq)]
pub enum RetryStrategy {
    /// Fixed delay between attempts
    Constant(Duration),
    /// Delay increases linearly: base * attempt
    Linear { base: Duration },
    /// Delay doubles: base * 2^attempt
    Exponential { base: Duration },
    /// Delay follows Fibonacci: fib(attempt) * base
    Fibonacci { base: Duration },
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum JitterStrategy {
    #[default]
    None,
    /// Add ±percentage randomness to delay
    Proportional(f64),
    /// Random delay between 0 and calculated delay
    Full,
    /// Decorrelated jitter (AWS style)
    Decorrelated,
}

/// Information about a retry attempt, passed to hooks
#[derive(Debug, Clone)]
pub struct RetryEvent<'a, E> {
    /// Which attempt just failed (1-indexed)
    pub attempt: u32,
    /// The error from the failed attempt
    pub error: &'a E,
    /// Delay before next attempt (if retrying)
    pub next_delay: Option<Duration>,
    /// Total elapsed time since first attempt
    pub elapsed: Duration,
}

/// Error type returned when all retries are exhausted
#[derive(Debug, Clone)]
pub struct RetryExhausted<E> {
    /// The error from the final attempt
    pub final_error: E,
    /// Total number of attempts made
    pub attempts: u32,
    /// Total time spent retrying
    pub total_duration: Duration,
}
```

#### Effect Extensions

```rust
impl<T, E, Env> Effect<T, E, Env>
where
    T: Send + 'static,
    E: Send + 'static,
    Env: Sync + 'static,
{
    /// Retry this effect according to the given policy.
    ///
    /// On failure, the effect will be re-executed after a delay determined
    /// by the policy. Retries continue until success or the policy's limits
    /// are reached.
    ///
    /// # Examples
    ///
    /// ```
    /// use stillwater::{Effect, RetryPolicy};
    /// use std::time::Duration;
    ///
    /// let effect = fetch_remote_data()
    ///     .with_retry(
    ///         RetryPolicy::exponential(Duration::from_millis(100))
    ///             .with_max_retries(3)
    ///     );
    /// ```
    pub fn with_retry(self, policy: RetryPolicy) -> Effect<T, RetryExhausted<E>, Env>
    where
        Self: Clone,
    {
        // Implementation
    }

    /// Retry this effect only when the predicate returns true.
    ///
    /// Non-retryable errors immediately propagate without retry.
    pub fn retry_if<P>(self, policy: RetryPolicy, predicate: P) -> Effect<T, E, Env>
    where
        P: Fn(&E) -> bool + Send + Sync + 'static,
        Self: Clone,
    {
        // Implementation
    }

    /// Add a timeout to this effect.
    ///
    /// If the effect doesn't complete within the duration, it fails
    /// with a timeout error.
    pub fn with_timeout(self, duration: Duration) -> Effect<T, TimeoutError<E>, Env> {
        // Implementation
    }

    /// Execute a side effect on each retry attempt.
    ///
    /// Useful for logging, metrics, or other observability.
    pub fn on_retry<F>(self, f: F) -> Self
    where
        F: Fn(RetryEvent<'_, E>) -> Effect<(), E, Env> + Send + Sync + 'static,
    {
        // Implementation
    }
}
```

#### Delay Calculation

```rust
impl RetryPolicy {
    /// Calculate the delay before attempt N (0-indexed).
    ///
    /// Returns None if no more retries should be attempted.
    pub fn delay_for_attempt(&self, attempt: u32) -> Option<Duration> {
        // Check max_retries
        if let Some(max) = self.max_retries {
            if attempt >= max {
                return None;
            }
        }

        // Calculate base delay from strategy
        let base_delay = match &self.strategy {
            RetryStrategy::Constant(d) => *d,
            RetryStrategy::Linear { base } => *base * (attempt + 1),
            RetryStrategy::Exponential { base } => {
                *base * 2u32.saturating_pow(attempt)
            }
            RetryStrategy::Fibonacci { base } => {
                *base * fibonacci(attempt)
            }
        };

        // Apply max_delay cap
        let capped = match self.max_delay {
            Some(max) => base_delay.min(max),
            None => base_delay,
        };

        // Apply jitter
        let jittered = self.jitter.apply(capped);

        Some(jittered)
    }
}
```

### Architecture Changes

This feature adds a new module to stillwater:

```
src/
├── lib.rs           # Re-export retry types
├── effect.rs        # Add retry combinators
├── retry/
│   ├── mod.rs       # Module root, re-exports
│   ├── policy.rs    # RetryPolicy, RetryStrategy, JitterStrategy
│   ├── executor.rs  # Internal retry execution logic
│   ├── error.rs     # RetryExhausted, TimeoutError
│   └── testing.rs   # Test utilities (MockClock, etc.)
```

### Data Structures

```rust
// Core policy - pure data, no behavior
pub struct RetryPolicy {
    strategy: RetryStrategy,
    max_retries: Option<u32>,
    max_delay: Option<Duration>,
    jitter: JitterStrategy,
}

// Strategy variants
pub enum RetryStrategy {
    Constant(Duration),
    Linear { base: Duration },
    Exponential { base: Duration },
    Fibonacci { base: Duration },
}

// Jitter options
pub enum JitterStrategy {
    None,
    Proportional(f64),
    Full,
    Decorrelated,
}

// Error wrapper with metadata
pub struct RetryExhausted<E> {
    pub final_error: E,
    pub attempts: u32,
    pub total_duration: Duration,
}

// Timeout error
pub enum TimeoutError<E> {
    Timeout { duration: Duration },
    Inner(E),
}
```

### APIs and Interfaces

#### Builder API for RetryPolicy

```rust
impl RetryPolicy {
    // Constructors
    pub fn constant(delay: Duration) -> Self;
    pub fn linear(base: Duration) -> Self;
    pub fn exponential(base: Duration) -> Self;
    pub fn fibonacci(base: Duration) -> Self;

    // Configuration
    pub fn with_max_retries(self, n: u32) -> Self;
    pub fn with_max_delay(self, d: Duration) -> Self;
    pub fn with_jitter(self, factor: f64) -> Self;
    pub fn with_full_jitter(self) -> Self;
    pub fn with_decorrelated_jitter(self) -> Self;

    // Inspection
    pub fn max_retries(&self) -> Option<u32>;
    pub fn max_delay(&self) -> Option<Duration>;
    pub fn delay_for_attempt(&self, attempt: u32) -> Option<Duration>;
}
```

#### Effect Combinators

```rust
impl<T, E, Env> Effect<T, E, Env> {
    pub fn with_retry(self, policy: RetryPolicy) -> Effect<T, RetryExhausted<E>, Env>;
    pub fn retry_if<P>(self, policy: RetryPolicy, predicate: P) -> Effect<T, E, Env>;
    pub fn with_timeout(self, duration: Duration) -> Effect<T, TimeoutError<E>, Env>;
    pub fn on_retry<F>(self, f: F) -> Self;
}
```

## Dependencies

- **Prerequisites**: None (builds on existing Effect type)
- **Affected Components**:
  - `effect.rs` - Add retry combinators
  - `lib.rs` - Re-export retry types
- **External Dependencies**:
  - `tokio` (existing, for async delays)
  - `rand` (new, for jitter - feature-gated)

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    // Policy construction
    #[test]
    fn test_exponential_policy_defaults();
    #[test]
    fn test_policy_builder_chain();
    #[test]
    fn test_policy_is_clone_and_eq();

    // Delay calculations
    #[test]
    fn test_constant_delay();
    #[test]
    fn test_linear_delay();
    #[test]
    fn test_exponential_delay();
    #[test]
    fn test_fibonacci_delay();
    #[test]
    fn test_max_delay_cap();
    #[test]
    fn test_max_retries_limit();

    // Jitter
    #[test]
    fn test_proportional_jitter_bounds();
    #[test]
    fn test_full_jitter_bounds();
    #[test]
    fn test_no_jitter_is_deterministic();
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_retry_succeeds_on_third_attempt() {
    let attempts = Arc::new(AtomicU32::new(0));
    let attempts_clone = attempts.clone();

    let effect = Effect::from_async(move |_: &()| {
        let attempts = attempts_clone.clone();
        async move {
            let n = attempts.fetch_add(1, Ordering::SeqCst);
            if n < 2 {
                Err("transient failure")
            } else {
                Ok("success")
            }
        }
    });

    let result = effect
        .with_retry(RetryPolicy::constant(Duration::from_millis(10)).with_max_retries(5))
        .run(&())
        .await;

    assert_eq!(result, Ok("success"));
    assert_eq!(attempts.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn test_retry_exhausted_returns_final_error();

#[tokio::test]
async fn test_retry_if_skips_non_retryable_errors();

#[tokio::test]
async fn test_timeout_triggers_correctly();

#[tokio::test]
async fn test_on_retry_hook_called();
```

### Property-Based Tests

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn exponential_delay_never_decreases(attempts in 0u32..20) {
        let policy = RetryPolicy::exponential(Duration::from_millis(100));
        let delays: Vec<_> = (0..attempts)
            .filter_map(|a| policy.delay_for_attempt(a))
            .collect();

        for window in delays.windows(2) {
            prop_assert!(window[0] <= window[1]);
        }
    }

    #[test]
    fn max_delay_is_respected(
        base in 1u64..1000,
        max in 1u64..10000,
        attempt in 0u32..100
    ) {
        let policy = RetryPolicy::exponential(Duration::from_millis(base))
            .with_max_delay(Duration::from_millis(max));

        if let Some(delay) = policy.delay_for_attempt(attempt) {
            prop_assert!(delay <= Duration::from_millis(max));
        }
    }

    #[test]
    fn jitter_stays_within_bounds(
        base in 100u64..1000,
        factor in 0.0f64..1.0
    ) {
        let policy = RetryPolicy::constant(Duration::from_millis(base))
            .with_jitter(factor);

        let delay = policy.delay_for_attempt(0).unwrap();
        let min = Duration::from_millis((base as f64 * (1.0 - factor)) as u64);
        let max = Duration::from_millis((base as f64 * (1.0 + factor)) as u64);

        prop_assert!(delay >= min && delay <= max);
    }
}
```

### Performance Tests

```rust
#[test]
fn test_retry_policy_is_stack_allocated() {
    assert!(std::mem::size_of::<RetryPolicy>() <= 64);
}

#[tokio::test]
async fn test_no_overhead_for_successful_effects() {
    // Measure baseline
    let start = Instant::now();
    for _ in 0..10000 {
        Effect::<_, String, ()>::pure(42).run(&()).await.unwrap();
    }
    let baseline = start.elapsed();

    // Measure with retry (no actual retries)
    let start = Instant::now();
    let policy = RetryPolicy::exponential(Duration::from_millis(100));
    for _ in 0..10000 {
        Effect::<_, String, ()>::pure(42)
            .with_retry(policy.clone())
            .run(&())
            .await
            .unwrap();
    }
    let with_retry = start.elapsed();

    // Should be within 2x (generous margin for measurement noise)
    assert!(with_retry < baseline * 2);
}
```

## Documentation Requirements

### Code Documentation
- Comprehensive rustdoc for all public types and methods
- Examples in doc comments for every public API
- Module-level documentation explaining concepts

### User Documentation
- Add "Retry and Resilience" section to user guide
- Document common patterns and best practices
- Provide migration guide from manual retry loops

### Architecture Updates
- Update DESIGN.md with retry module design decisions
- Document why policies are pure data vs behavior

## Implementation Notes

### Cloning Effects for Retry

The `with_retry` combinator requires the effect to be clonable. Since `Effect` contains a `Box<dyn FnOnce>`, we need to either:

1. **Option A**: Require `Clone` bound on the effect (user must create cloneable effects)
2. **Option B**: Take a factory function instead of an effect
3. **Option C**: Use `Arc<dyn Fn>` internally for retryable effects

**Recommendation**: Option B for ergonomics:

```rust
// Instead of cloning the effect, take a factory
pub fn with_retry<F>(factory: F, policy: RetryPolicy) -> Effect<T, RetryExhausted<E>, Env>
where
    F: Fn() -> Effect<T, E, Env> + Send + 'static
```

Or provide both APIs:
```rust
// For clonable effects
effect.with_retry(policy)

// For non-clonable effects
Effect::retry_with(|| fetch_data(), policy)
```

### Jitter Implementation

Use `rand::thread_rng()` for jitter. Feature-gate behind `rand` feature:

```toml
[features]
default = []
jitter = ["dep:rand"]
```

Without the feature, jitter methods compile but log a warning and return unjittered delays.

### Timeout Implementation

Use `tokio::time::timeout`. Timeout wraps the effect execution:

```rust
pub fn with_timeout(self, duration: Duration) -> Effect<T, TimeoutError<E>, Env> {
    Effect::from_async(move |env| async move {
        match tokio::time::timeout(duration, self.run(env)).await {
            Ok(Ok(value)) => Ok(value),
            Ok(Err(e)) => Err(TimeoutError::Inner(e)),
            Err(_) => Err(TimeoutError::Timeout { duration }),
        }
    })
}
```

### Thread Safety

- `RetryPolicy` is `Send + Sync` (pure data)
- `RetryEvent` contains a reference, lifetime-bound to retry loop
- All combinators maintain `Send + 'static` bounds

## Migration and Compatibility

### Breaking Changes
None - this is a purely additive feature.

### Feature Flags
```toml
[features]
default = ["retry"]
retry = []           # Core retry functionality
jitter = ["retry", "dep:rand"]  # Jitter support
```

### Deprecations
None.

## Future Considerations (Out of Scope)

These are explicitly NOT part of this specification but may be added later:

1. **Circuit Breaker**: Fail fast when downstream is unhealthy
2. **Bulkhead**: Limit concurrent executions
3. **Rate Limiter**: Control request rate
4. **Retry Budgets**: Limit total retries across all operations
5. **Hedged Requests**: Send parallel requests, take first success
6. **Adaptive Retry**: Adjust policy based on observed success rates

These would be separate specifications building on this foundation.

---

*"Retry policies are just data. Retry execution is just an effect. Keep them separate, keep them composable."*
