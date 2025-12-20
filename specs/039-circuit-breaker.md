---
number: 039
title: Circuit Breaker
category: foundation
priority: high
status: draft
dependencies: [024]
created: 2025-12-20
---

# Specification 039: Circuit Breaker

**Category**: foundation
**Priority**: high
**Status**: draft
**Dependencies**: Spec 024 (Zero-Cost Effect Trait)

## Context

### The Problem

When calling external services or unreliable subsystems, failures can cascade:

```rust
// Without circuit breaker - hammering a dead service
for request in requests {
    match call_external_service(request).await {  // Times out after 30s each
        Ok(response) => process(response),
        Err(_) => log_error(),  // Keep trying, keep failing
    }
}
// 100 requests × 30s timeout = 50 minutes of waiting for a dead service
```

This causes:
1. **Resource exhaustion** - threads/connections blocked on dead services
2. **Latency spikes** - users wait for timeouts instead of fast failures
3. **Cascade failures** - one failing service brings down the whole system
4. **No recovery time** - constant retries prevent the failing service from recovering

### The Solution

The Circuit Breaker pattern provides automatic failure detection and fast-fail behavior:

```rust
let breaker = CircuitBreaker::new(CircuitBreakerConfig {
    failure_threshold: 5,
    success_threshold: 2,
    open_duration: Duration::from_secs(30),
});

for request in requests {
    match breaker.call(|| call_external_service(request)).await {
        Ok(response) => process(response),
        Err(CircuitBreakerError::Open) => {
            // Fail fast - service known to be down
            use_fallback(request);
        }
        Err(CircuitBreakerError::ServiceError(e)) => {
            // Actual service error - breaker tracks this
            log_error(e);
        }
    }
}
```

### State Machine

```
     ┌─────────────────────────────────────────────────┐
     │                                                 │
     ▼                                                 │
┌─────────┐  failure_count >= threshold   ┌──────────┐│
│ CLOSED  │ ─────────────────────────────▶│  OPEN    ││
│         │                               │          ││
│ (normal)│◀─────────────────────────────│(blocking)││
└─────────┘  success_count >= threshold   └────┬─────┘│
     ▲       in half-open                      │      │
     │                                         │      │
     │       open_duration elapsed             │      │
     │                                         ▼      │
     │                                   ┌──────────┐ │
     │                                   │HALF-OPEN │ │
     │                                   │          │─┘
     │                                   │(testing) │ failure in half-open
     │                                   └────┬─────┘
     │                                        │
     └────────────────────────────────────────┘
           success_count >= threshold
```

- **Closed**: Normal operation. Failures counted. Trips to Open when threshold reached.
- **Open**: Fast-fail all requests. After timeout, transitions to Half-Open.
- **Half-Open**: Allows limited test requests. Success closes circuit, failure re-opens.

### Why Not Use Mindset?

While mindset provides a full state machine implementation, circuit breaker is a **fixed, well-known pattern** that:

1. Has exactly 3 states (never user-defined)
2. Has fixed transition logic (no custom guards)
3. Requires atomic operations for thread safety
4. Needs no checkpointing or persistence
5. Should be zero-cost when not used

A direct implementation in stillwater is simpler and more efficient than layering on a general-purpose state machine.

### Philosophy Alignment

From PHILOSOPHY.md:
- *"Composition over complexity"* - Composes with retry, timeout, and Effect
- *"Types guide, don't restrict"* - Clear error types distinguish breaker state from service errors
- *"Zero-cost abstractions"* - No overhead when breaker is closed (hot path)

### Prior Art

- **Netflix Hystrix**: Original circuit breaker for JVM (now deprecated)
- **resilience4j**: Modern Java circuit breaker
- **Polly**: .NET resilience library with circuit breaker
- **tokio-retry**: Rust retry with basic circuit breaker (less featured)

## Objective

Add a Circuit Breaker to Stillwater that:

1. Provides automatic failure detection with configurable thresholds
2. Fast-fails requests when circuit is open
3. Allows gradual recovery through half-open state
4. Integrates seamlessly with Effect combinators
5. Is thread-safe for concurrent access
6. Has zero overhead in the closed (normal) state

## Requirements

### Functional Requirements

#### FR-1: Circuit Breaker State

```rust
/// The current state of the circuit breaker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Normal operation. Requests flow through, failures are counted.
    Closed,
    /// Circuit is open. All requests fail immediately.
    Open,
    /// Testing recovery. Limited requests allowed through.
    HalfOpen,
}
```

#### FR-2: Configuration

```rust
/// Configuration for a circuit breaker.
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of failures before opening the circuit.
    /// Default: 5
    pub failure_threshold: u32,

    /// Number of successes in half-open before closing.
    /// Default: 2
    pub success_threshold: u32,

    /// How long to stay open before transitioning to half-open.
    /// Default: 30 seconds
    pub open_duration: Duration,

    /// Optional: Maximum concurrent requests in half-open state.
    /// Default: 1
    pub half_open_max_concurrent: u32,

    /// Optional: Sliding window for failure counting in closed state.
    /// None means failures never expire.
    /// Default: None
    pub failure_window: Option<Duration>,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 2,
            open_duration: Duration::from_secs(30),
            half_open_max_concurrent: 1,
            failure_window: None,
        }
    }
}
```

#### FR-3: Circuit Breaker Core

```rust
/// A circuit breaker for protecting against cascading failures.
///
/// Thread-safe and designed for concurrent access.
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: AtomicU8,  // Encoded CircuitState
    failure_count: AtomicU32,
    success_count: AtomicU32,
    last_failure_time: AtomicU64,  // Unix timestamp millis
    half_open_permits: AtomicU32,
}

impl CircuitBreaker {
    /// Create a new circuit breaker with the given configuration.
    pub fn new(config: CircuitBreakerConfig) -> Self;

    /// Create a circuit breaker with default configuration.
    pub fn with_defaults() -> Self;

    /// Get the current state of the circuit.
    pub fn state(&self) -> CircuitState;

    /// Check if a request should be allowed through.
    ///
    /// Returns `Ok(())` if allowed, `Err(CircuitBreakerError::Open)` if blocked.
    pub fn check(&self) -> Result<(), CircuitBreakerError<()>>;

    /// Record a successful operation.
    pub fn record_success(&self);

    /// Record a failed operation.
    pub fn record_failure(&self);

    /// Manually reset the circuit to closed state.
    pub fn reset(&self);

    /// Get metrics about the circuit breaker.
    pub fn metrics(&self) -> CircuitBreakerMetrics;
}
```

#### FR-4: Error Types

```rust
/// Error type for circuit breaker operations.
#[derive(Debug, Clone)]
pub enum CircuitBreakerError<E> {
    /// The circuit is open; request was rejected without calling the service.
    Open {
        /// When the circuit will transition to half-open.
        retry_after: Duration,
    },

    /// The service call failed.
    ServiceError(E),

    /// Half-open permits exhausted; too many concurrent test requests.
    HalfOpenBusy,
}

impl<E: std::error::Error> std::error::Error for CircuitBreakerError<E> {}

impl<E> CircuitBreakerError<E> {
    /// Returns true if the circuit is open (fast-fail case).
    pub fn is_open(&self) -> bool;

    /// Returns true if this is a service error (the call was attempted).
    pub fn is_service_error(&self) -> bool;

    /// Map the inner service error type.
    pub fn map_service_error<F, E2>(self, f: F) -> CircuitBreakerError<E2>
    where
        F: FnOnce(E) -> E2;
}
```

#### FR-5: Async Execution

```rust
impl CircuitBreaker {
    /// Execute an async operation through the circuit breaker.
    ///
    /// - If closed: executes the operation, tracks success/failure
    /// - If open: returns `Err(CircuitBreakerError::Open)` immediately
    /// - If half-open: allows limited concurrent operations
    pub async fn call<F, Fut, T, E>(&self, f: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, E>>;

    /// Execute with a predicate to determine what counts as failure.
    ///
    /// By default, any `Err` is a failure. This allows treating certain
    /// errors (e.g., validation errors) as successes for circuit purposes.
    pub async fn call_with<F, Fut, T, E, P>(
        &self,
        f: F,
        is_failure: P,
    ) -> Result<T, CircuitBreakerError<E>>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, E>>,
        P: FnOnce(&E) -> bool;
}
```

#### FR-6: Effect Integration

```rust
/// Extension trait for applying circuit breaker to effects.
pub trait CircuitBreakerExt: Effect {
    /// Wrap this effect with a circuit breaker.
    ///
    /// The circuit breaker tracks successes and failures of the underlying
    /// effect and will fast-fail when open.
    fn with_circuit_breaker(
        self,
        breaker: &CircuitBreaker,
    ) -> WithCircuitBreaker<Self>
    where
        Self: Sized;

    /// Wrap with circuit breaker, using a predicate to determine failures.
    fn with_circuit_breaker_if<P>(
        self,
        breaker: &CircuitBreaker,
        is_failure: P,
    ) -> WithCircuitBreakerIf<Self, P>
    where
        Self: Sized,
        P: FnOnce(&Self::Error) -> bool;
}

/// An effect wrapped with circuit breaker protection.
pub struct WithCircuitBreaker<'a, E> {
    inner: E,
    breaker: &'a CircuitBreaker,
}

impl<'a, E: Effect> Effect for WithCircuitBreaker<'a, E> {
    type Output = E::Output;
    type Error = CircuitBreakerError<E::Error>;
    type Env = E::Env;

    async fn run(self, env: &Self::Env) -> Result<Self::Output, Self::Error> {
        self.breaker.check()?;

        match self.inner.run(env).await {
            Ok(value) => {
                self.breaker.record_success();
                Ok(value)
            }
            Err(e) => {
                self.breaker.record_failure();
                Err(CircuitBreakerError::ServiceError(e))
            }
        }
    }
}
```

#### FR-7: Metrics and Observability

```rust
/// Metrics about circuit breaker state.
#[derive(Debug, Clone)]
pub struct CircuitBreakerMetrics {
    /// Current state.
    pub state: CircuitState,

    /// Total successful calls since creation.
    pub total_successes: u64,

    /// Total failed calls since creation.
    pub total_failures: u64,

    /// Total rejected calls (when open) since creation.
    pub total_rejected: u64,

    /// Current failure count in the sliding window (if configured).
    pub current_failure_count: u32,

    /// Time until half-open transition (if currently open).
    pub time_until_half_open: Option<Duration>,
}

impl CircuitBreaker {
    /// Subscribe to state change events.
    ///
    /// Returns a receiver that yields events when the circuit state changes.
    #[cfg(feature = "events")]
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<CircuitBreakerEvent>;
}

/// Events emitted by circuit breaker.
#[derive(Debug, Clone)]
pub enum CircuitBreakerEvent {
    /// Circuit transitioned from Closed to Open.
    Opened {
        failure_count: u32,
        last_error: String,
    },

    /// Circuit transitioned from Open to HalfOpen.
    HalfOpened,

    /// Circuit transitioned from HalfOpen to Closed.
    Closed {
        success_count: u32,
    },

    /// Circuit transitioned from HalfOpen to Open (test request failed).
    RejectedHalfOpen {
        error: String,
    },
}
```

#### FR-8: Builder Pattern

```rust
/// Builder for creating circuit breakers with fluent API.
pub struct CircuitBreakerBuilder {
    config: CircuitBreakerConfig,
}

impl CircuitBreakerBuilder {
    pub fn new() -> Self;

    /// Set failure threshold before opening.
    pub fn failure_threshold(mut self, count: u32) -> Self;

    /// Set success threshold before closing from half-open.
    pub fn success_threshold(mut self, count: u32) -> Self;

    /// Set duration to stay open before half-open.
    pub fn open_duration(mut self, duration: Duration) -> Self;

    /// Set maximum concurrent requests in half-open state.
    pub fn half_open_permits(mut self, count: u32) -> Self;

    /// Set sliding window for failure counting.
    pub fn failure_window(mut self, duration: Duration) -> Self;

    /// Build the circuit breaker.
    pub fn build(self) -> CircuitBreaker;
}

// Usage:
let breaker = CircuitBreaker::builder()
    .failure_threshold(10)
    .open_duration(Duration::from_secs(60))
    .build();
```

#### FR-9: Composition with Retry

Circuit breaker should compose naturally with retry:

```rust
// Retry within circuit breaker - retries count as single attempt
let effect = call_service()
    .retry(RetryPolicy::exponential(3))
    .with_circuit_breaker(&breaker);

// Circuit breaker within retry - each retry is tracked separately
let effect = call_service()
    .with_circuit_breaker(&breaker)
    .retry(RetryPolicy::constant(3));
```

#### FR-10: Shared State

Circuit breakers are typically shared across multiple call sites:

```rust
// Create once, share across handlers
lazy_static! {
    static ref PAYMENT_SERVICE_BREAKER: CircuitBreaker =
        CircuitBreaker::builder()
            .failure_threshold(5)
            .open_duration(Duration::from_secs(30))
            .build();
}

// In handler A
async fn checkout() {
    payment_call().with_circuit_breaker(&PAYMENT_SERVICE_BREAKER).run(&env).await
}

// In handler B
async fn refund() {
    refund_call().with_circuit_breaker(&PAYMENT_SERVICE_BREAKER).run(&env).await
}
```

### Non-Functional Requirements

#### NFR-1: Thread Safety

- All operations must be safe for concurrent access
- Use atomic operations for state transitions
- No mutex contention in the hot path (closed state)

#### NFR-2: Zero-Cost Closed State

- When circuit is closed, overhead should be minimal
- Fast path: atomic load + increment
- No allocations on the hot path

#### NFR-3: Deterministic Behavior

- State transitions must be deterministic
- No race conditions in threshold checking
- Consistent behavior under concurrent load

#### NFR-4: Time Handling

- Use monotonic time for duration calculations
- Handle clock skew gracefully
- No panics on time overflow

## Acceptance Criteria

### Core Functionality

- [ ] **AC1**: `CircuitBreaker::new()` creates breaker in Closed state
- [ ] **AC2**: `check()` returns `Ok(())` when closed
- [ ] **AC3**: `check()` returns `Err(Open)` when open
- [ ] **AC4**: Failures increment counter, success resets it (in closed)
- [ ] **AC5**: Circuit opens when failure_threshold reached
- [ ] **AC6**: Circuit transitions to half-open after open_duration
- [ ] **AC7**: Success in half-open increments success counter
- [ ] **AC8**: Circuit closes when success_threshold reached in half-open
- [ ] **AC9**: Failure in half-open immediately re-opens circuit

### Async Execution

- [ ] **AC10**: `call()` executes function when closed
- [ ] **AC11**: `call()` returns immediately when open
- [ ] **AC12**: `call()` tracks success/failure automatically
- [ ] **AC13**: `call_with()` respects custom failure predicate

### Effect Integration

- [ ] **AC14**: `with_circuit_breaker()` wraps effect correctly
- [ ] **AC15**: Wrapped effect returns `CircuitBreakerError<E>`
- [ ] **AC16**: Composes with `map`, `and_then`, `map_err`
- [ ] **AC17**: Composes with `retry` in both orders

### Thread Safety

- [ ] **AC18**: Safe concurrent `check()` calls
- [ ] **AC19**: Safe concurrent `record_success/failure` calls
- [ ] **AC20**: Correct behavior under high concurrency
- [ ] **AC21**: No data races (passes miri)

### Metrics

- [ ] **AC22**: `metrics()` returns accurate counts
- [ ] **AC23**: `state()` reflects current state
- [ ] **AC24**: Events emitted on state transitions (if feature enabled)

### Edge Cases

- [ ] **AC25**: Handles rapid open/close transitions
- [ ] **AC26**: Handles time overflow gracefully
- [ ] **AC27**: `reset()` forces circuit to closed state
- [ ] **AC28**: Half-open permits limit concurrent test requests

## Technical Details

### Implementation Approach

#### State Encoding

Use a single atomic for state to enable lock-free transitions:

```rust
// src/circuit_breaker/state.rs

use std::sync::atomic::{AtomicU8, Ordering};

const CLOSED: u8 = 0;
const OPEN: u8 = 1;
const HALF_OPEN: u8 = 2;

impl CircuitBreaker {
    fn get_state(&self) -> CircuitState {
        match self.state.load(Ordering::Acquire) {
            CLOSED => CircuitState::Closed,
            OPEN => CircuitState::Open,
            HALF_OPEN => CircuitState::HalfOpen,
            _ => unreachable!(),
        }
    }

    fn transition(&self, from: u8, to: u8) -> bool {
        self.state
            .compare_exchange(from, to, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }
}
```

#### Core Implementation

```rust
// src/circuit_breaker/mod.rs

use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::{Duration, Instant};

pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: AtomicU8,
    failure_count: AtomicU32,
    success_count: AtomicU32,
    opened_at: AtomicU64,  // Instant as nanos since start
    start: Instant,        // Reference point for time
    half_open_permits: AtomicU32,

    // Metrics (optional, behind feature flag)
    #[cfg(feature = "metrics")]
    total_successes: AtomicU64,
    #[cfg(feature = "metrics")]
    total_failures: AtomicU64,
    #[cfg(feature = "metrics")]
    total_rejected: AtomicU64,
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config: config.clone(),
            state: AtomicU8::new(CLOSED),
            failure_count: AtomicU32::new(0),
            success_count: AtomicU32::new(0),
            opened_at: AtomicU64::new(0),
            start: Instant::now(),
            half_open_permits: AtomicU32::new(config.half_open_max_concurrent),
            #[cfg(feature = "metrics")]
            total_successes: AtomicU64::new(0),
            #[cfg(feature = "metrics")]
            total_failures: AtomicU64::new(0),
            #[cfg(feature = "metrics")]
            total_rejected: AtomicU64::new(0),
        }
    }

    pub fn check(&self) -> Result<(), CircuitBreakerError<()>> {
        loop {
            match self.get_state() {
                CircuitState::Closed => return Ok(()),

                CircuitState::Open => {
                    // Check if we should transition to half-open
                    let opened_at = self.opened_at.load(Ordering::Acquire);
                    let elapsed = self.start.elapsed().as_nanos() as u64 - opened_at;
                    let open_duration_nanos = self.config.open_duration.as_nanos() as u64;

                    if elapsed >= open_duration_nanos {
                        // Try to transition to half-open
                        if self.transition(OPEN, HALF_OPEN) {
                            self.success_count.store(0, Ordering::Release);
                            self.half_open_permits.store(
                                self.config.half_open_max_concurrent,
                                Ordering::Release,
                            );
                            // Loop again to handle half-open
                            continue;
                        }
                        // Another thread transitioned; re-check state
                        continue;
                    }

                    let retry_after = Duration::from_nanos(open_duration_nanos - elapsed);
                    #[cfg(feature = "metrics")]
                    self.total_rejected.fetch_add(1, Ordering::Relaxed);
                    return Err(CircuitBreakerError::Open { retry_after });
                }

                CircuitState::HalfOpen => {
                    // Try to acquire a permit
                    let permits = self.half_open_permits.load(Ordering::Acquire);
                    if permits == 0 {
                        return Err(CircuitBreakerError::HalfOpenBusy);
                    }
                    if self
                        .half_open_permits
                        .compare_exchange(permits, permits - 1, Ordering::AcqRel, Ordering::Acquire)
                        .is_ok()
                    {
                        return Ok(());
                    }
                    // CAS failed, retry
                    continue;
                }
            }
        }
    }

    pub fn record_success(&self) {
        #[cfg(feature = "metrics")]
        self.total_successes.fetch_add(1, Ordering::Relaxed);

        match self.get_state() {
            CircuitState::Closed => {
                // Reset failure count on success
                self.failure_count.store(0, Ordering::Release);
            }
            CircuitState::HalfOpen => {
                // Return permit
                self.half_open_permits.fetch_add(1, Ordering::Release);

                // Increment success count
                let count = self.success_count.fetch_add(1, Ordering::AcqRel) + 1;

                // Check if we should close
                if count >= self.config.success_threshold {
                    if self.transition(HALF_OPEN, CLOSED) {
                        self.failure_count.store(0, Ordering::Release);
                    }
                }
            }
            CircuitState::Open => {
                // Shouldn't happen, but handle gracefully
            }
        }
    }

    pub fn record_failure(&self) {
        #[cfg(feature = "metrics")]
        self.total_failures.fetch_add(1, Ordering::Relaxed);

        match self.get_state() {
            CircuitState::Closed => {
                let count = self.failure_count.fetch_add(1, Ordering::AcqRel) + 1;

                if count >= self.config.failure_threshold {
                    if self.transition(CLOSED, OPEN) {
                        let now = self.start.elapsed().as_nanos() as u64;
                        self.opened_at.store(now, Ordering::Release);
                    }
                }
            }
            CircuitState::HalfOpen => {
                // Return permit
                self.half_open_permits.fetch_add(1, Ordering::Release);

                // Immediately re-open
                if self.transition(HALF_OPEN, OPEN) {
                    let now = self.start.elapsed().as_nanos() as u64;
                    self.opened_at.store(now, Ordering::Release);
                }
            }
            CircuitState::Open => {
                // Shouldn't happen, but handle gracefully
            }
        }
    }

    pub fn reset(&self) {
        self.state.store(CLOSED, Ordering::Release);
        self.failure_count.store(0, Ordering::Release);
        self.success_count.store(0, Ordering::Release);
    }
}
```

#### Async Call Wrapper

```rust
impl CircuitBreaker {
    pub async fn call<F, Fut, T, E>(&self, f: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, E>>,
    {
        self.check().map_err(|e| match e {
            CircuitBreakerError::Open { retry_after } => {
                CircuitBreakerError::Open { retry_after }
            }
            CircuitBreakerError::HalfOpenBusy => CircuitBreakerError::HalfOpenBusy,
            CircuitBreakerError::ServiceError(()) => unreachable!(),
        })?;

        match f().await {
            Ok(value) => {
                self.record_success();
                Ok(value)
            }
            Err(e) => {
                self.record_failure();
                Err(CircuitBreakerError::ServiceError(e))
            }
        }
    }

    pub async fn call_with<F, Fut, T, E, P>(
        &self,
        f: F,
        is_failure: P,
    ) -> Result<T, CircuitBreakerError<E>>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, E>>,
        P: FnOnce(&E) -> bool,
    {
        self.check().map_err(|e| match e {
            CircuitBreakerError::Open { retry_after } => {
                CircuitBreakerError::Open { retry_after }
            }
            CircuitBreakerError::HalfOpenBusy => CircuitBreakerError::HalfOpenBusy,
            CircuitBreakerError::ServiceError(()) => unreachable!(),
        })?;

        match f().await {
            Ok(value) => {
                self.record_success();
                Ok(value)
            }
            Err(e) => {
                if is_failure(&e) {
                    self.record_failure();
                } else {
                    self.record_success();
                }
                Err(CircuitBreakerError::ServiceError(e))
            }
        }
    }
}
```

### Module Structure

```
src/circuit_breaker/
├── mod.rs              # Module exports and CircuitBreaker struct
├── config.rs           # CircuitBreakerConfig and builder
├── state.rs            # CircuitState enum and atomic state handling
├── error.rs            # CircuitBreakerError type
├── metrics.rs          # CircuitBreakerMetrics (behind feature flag)
├── effect.rs           # Effect integration (WithCircuitBreaker)
└── events.rs           # Event broadcasting (behind feature flag)
```

### Feature Flags

```toml
[features]
default = []
circuit-breaker-metrics = []   # Enable detailed metrics
circuit-breaker-events = []    # Enable event broadcasting
```

## Dependencies

### Prerequisites

- Spec 024 (Zero-Cost Effect Trait) - for Effect integration

### Affected Components

- Effect prelude - new exports
- Effect extension trait - new methods

### External Dependencies

- `tokio::sync::broadcast` (optional, for events)
- No other external dependencies

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_starts_closed() {
        let breaker = CircuitBreaker::with_defaults();
        assert_eq!(breaker.state(), CircuitState::Closed);
    }

    #[test]
    fn test_opens_after_threshold() {
        let breaker = CircuitBreaker::builder()
            .failure_threshold(3)
            .build();

        breaker.record_failure();
        assert_eq!(breaker.state(), CircuitState::Closed);

        breaker.record_failure();
        assert_eq!(breaker.state(), CircuitState::Closed);

        breaker.record_failure();
        assert_eq!(breaker.state(), CircuitState::Open);
    }

    #[test]
    fn test_success_resets_failure_count() {
        let breaker = CircuitBreaker::builder()
            .failure_threshold(3)
            .build();

        breaker.record_failure();
        breaker.record_failure();
        breaker.record_success();  // Resets count

        breaker.record_failure();
        breaker.record_failure();
        assert_eq!(breaker.state(), CircuitState::Closed);  // Still closed

        breaker.record_failure();
        assert_eq!(breaker.state(), CircuitState::Open);
    }

    #[test]
    fn test_check_fails_when_open() {
        let breaker = CircuitBreaker::builder()
            .failure_threshold(1)
            .build();

        breaker.record_failure();
        assert!(matches!(
            breaker.check(),
            Err(CircuitBreakerError::Open { .. })
        ));
    }

    #[tokio::test]
    async fn test_transitions_to_half_open() {
        let breaker = CircuitBreaker::builder()
            .failure_threshold(1)
            .open_duration(Duration::from_millis(10))
            .build();

        breaker.record_failure();
        assert_eq!(breaker.state(), CircuitState::Open);

        tokio::time::sleep(Duration::from_millis(20)).await;

        // Trigger state check
        let _ = breaker.check();
        assert_eq!(breaker.state(), CircuitState::HalfOpen);
    }

    #[tokio::test]
    async fn test_half_open_closes_on_success() {
        let breaker = CircuitBreaker::builder()
            .failure_threshold(1)
            .success_threshold(2)
            .open_duration(Duration::from_millis(10))
            .build();

        breaker.record_failure();
        tokio::time::sleep(Duration::from_millis(20)).await;

        // First check transitions to half-open
        assert!(breaker.check().is_ok());
        breaker.record_success();
        assert_eq!(breaker.state(), CircuitState::HalfOpen);

        // Second success closes circuit
        assert!(breaker.check().is_ok());
        breaker.record_success();
        assert_eq!(breaker.state(), CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_half_open_reopens_on_failure() {
        let breaker = CircuitBreaker::builder()
            .failure_threshold(1)
            .open_duration(Duration::from_millis(10))
            .build();

        breaker.record_failure();
        tokio::time::sleep(Duration::from_millis(20)).await;

        assert!(breaker.check().is_ok());
        breaker.record_failure();
        assert_eq!(breaker.state(), CircuitState::Open);
    }

    #[test]
    fn test_reset_forces_closed() {
        let breaker = CircuitBreaker::builder()
            .failure_threshold(1)
            .build();

        breaker.record_failure();
        assert_eq!(breaker.state(), CircuitState::Open);

        breaker.reset();
        assert_eq!(breaker.state(), CircuitState::Closed);
    }
}
```

### Async Execution Tests

```rust
#[cfg(test)]
mod async_tests {
    use super::*;

    #[tokio::test]
    async fn test_call_succeeds_when_closed() {
        let breaker = CircuitBreaker::with_defaults();

        let result = breaker
            .call(|| async { Ok::<_, String>(42) })
            .await;

        assert_eq!(result, Ok(42));
    }

    #[tokio::test]
    async fn test_call_fails_fast_when_open() {
        let breaker = CircuitBreaker::builder()
            .failure_threshold(1)
            .build();

        breaker.record_failure();

        let result = breaker
            .call(|| async {
                panic!("Should not be called!");
                #[allow(unreachable_code)]
                Ok::<_, String>(42)
            })
            .await;

        assert!(matches!(result, Err(CircuitBreakerError::Open { .. })));
    }

    #[tokio::test]
    async fn test_call_tracks_failures() {
        let breaker = CircuitBreaker::builder()
            .failure_threshold(2)
            .build();

        let _ = breaker
            .call(|| async { Err::<i32, _>("error") })
            .await;

        assert_eq!(breaker.state(), CircuitState::Closed);

        let _ = breaker
            .call(|| async { Err::<i32, _>("error") })
            .await;

        assert_eq!(breaker.state(), CircuitState::Open);
    }

    #[tokio::test]
    async fn test_call_with_custom_failure_predicate() {
        let breaker = CircuitBreaker::builder()
            .failure_threshold(2)
            .build();

        #[derive(Debug)]
        enum Error {
            Transient,
            Validation,
        }

        // Validation errors don't count as failures
        let _ = breaker
            .call_with(
                || async { Err::<i32, _>(Error::Validation) },
                |e| matches!(e, Error::Transient),
            )
            .await;

        let _ = breaker
            .call_with(
                || async { Err::<i32, _>(Error::Validation) },
                |e| matches!(e, Error::Transient),
            )
            .await;

        // Still closed - validation errors weren't counted
        assert_eq!(breaker.state(), CircuitState::Closed);
    }
}
```

### Concurrency Tests

```rust
#[cfg(test)]
mod concurrency_tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_concurrent_failures() {
        let breaker = Arc::new(CircuitBreaker::builder()
            .failure_threshold(100)
            .build());

        let handles: Vec<_> = (0..100)
            .map(|_| {
                let b = Arc::clone(&breaker);
                tokio::spawn(async move {
                    b.record_failure();
                })
            })
            .collect();

        for h in handles {
            h.await.unwrap();
        }

        assert_eq!(breaker.state(), CircuitState::Open);
    }

    #[tokio::test]
    async fn test_half_open_permits() {
        let breaker = Arc::new(CircuitBreaker::builder()
            .failure_threshold(1)
            .half_open_permits(2)
            .open_duration(Duration::from_millis(10))
            .build());

        breaker.record_failure();
        tokio::time::sleep(Duration::from_millis(20)).await;

        // First two checks should succeed
        assert!(breaker.check().is_ok());
        assert!(breaker.check().is_ok());

        // Third should fail (no permits)
        assert!(matches!(
            breaker.check(),
            Err(CircuitBreakerError::HalfOpenBusy)
        ));
    }
}
```

### Effect Integration Tests

```rust
#[cfg(test)]
mod effect_tests {
    use super::*;
    use stillwater::effect::prelude::*;

    #[tokio::test]
    async fn test_effect_with_circuit_breaker() {
        let breaker = CircuitBreaker::with_defaults();

        let effect = pure::<i32, String, ()>(42)
            .with_circuit_breaker(&breaker);

        let result = effect.run(&()).await;
        assert_eq!(result, Ok(42));
    }

    #[tokio::test]
    async fn test_effect_circuit_breaker_tracks_failure() {
        let breaker = CircuitBreaker::builder()
            .failure_threshold(1)
            .build();

        let effect = fail::<i32, String, ()>("error".into())
            .with_circuit_breaker(&breaker);

        let _ = effect.run(&()).await;
        assert_eq!(breaker.state(), CircuitState::Open);
    }

    #[tokio::test]
    async fn test_compose_retry_then_breaker() {
        let breaker = CircuitBreaker::builder()
            .failure_threshold(1)
            .build();

        // Retry wraps the call, breaker sees one failure after retries exhausted
        let effect = fail::<i32, String, ()>("error".into())
            .retry(RetryPolicy::constant(2).with_delay(Duration::ZERO))
            .with_circuit_breaker(&breaker);

        let _ = effect.run(&()).await;

        // Only one failure recorded (after retries)
        assert_eq!(breaker.state(), CircuitState::Open);
    }
}
```

## Documentation Requirements

### Code Documentation

Comprehensive rustdoc for all public types and methods.

### User Documentation

Add circuit breaker guide to effect documentation:

```markdown
## Circuit Breaker: Protection from Cascading Failures

### Basic Usage

```rust
use stillwater::circuit_breaker::{CircuitBreaker, CircuitBreakerError};

let breaker = CircuitBreaker::builder()
    .failure_threshold(5)
    .open_duration(Duration::from_secs(30))
    .build();

match breaker.call(|| fetch_from_service()).await {
    Ok(data) => process(data),
    Err(CircuitBreakerError::Open { retry_after }) => {
        log::warn!("Service unavailable, retry in {:?}", retry_after);
        use_cached_data()
    }
    Err(CircuitBreakerError::ServiceError(e)) => {
        log::error!("Service error: {}", e);
        handle_error(e)
    }
}
```

### With Effects

```rust
let effect = call_external_api()
    .with_circuit_breaker(&breaker)
    .map_err(|e| match e {
        CircuitBreakerError::Open { .. } => AppError::ServiceUnavailable,
        CircuitBreakerError::ServiceError(e) => AppError::from(e),
    });
```
```

## Implementation Notes

### Time Handling

Use `Instant::now()` as a reference point and store elapsed time as u64 nanos. This avoids issues with system time changes.

### Memory Ordering

- `Acquire` for reads that depend on other data
- `Release` for writes that other threads will read
- `AcqRel` for read-modify-write operations
- `Relaxed` only for metrics (doesn't affect correctness)

### Half-Open Permit Management

Permits must be returned on both success and failure to prevent permit leakage. Use a guard pattern if necessary:

```rust
struct HalfOpenGuard<'a> {
    breaker: &'a CircuitBreaker,
    released: bool,
}

impl Drop for HalfOpenGuard<'_> {
    fn drop(&mut self) {
        if !self.released {
            self.breaker.half_open_permits.fetch_add(1, Ordering::Release);
        }
    }
}
```

## Migration and Compatibility

### Backward Compatibility

Purely additive - no breaking changes to existing code.

### Integration with Existing Retry

Works naturally with existing `retry` combinators. Order matters:
- `effect.retry().with_circuit_breaker()` - retries count as one attempt
- `effect.with_circuit_breaker().retry()` - each attempt tracked separately

---

*"The circuit breaker is to distributed systems what the fuse is to electrical systems: a simple mechanism that prevents catastrophic failure."*
