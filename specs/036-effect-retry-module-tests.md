---
number: 036
title: Effect Retry Module Tests
category: testing
priority: high
status: draft
dependencies: []
created: 2025-11-27
---

# Specification 036: Effect Retry Module Tests

**Category**: testing
**Priority**: high
**Status**: draft
**Dependencies**: None

## Context

The `src/effect/retry.rs` module provides zero-cost retry functionality for the new Effect trait system introduced in v0.11.0. This module currently has **0% test coverage** (0/79 lines), representing the largest untested component in the codebase.

The retry module contains critical resilience functionality:
- `retry()` - Retry an effect using a factory function with exponential backoff
- `retry_if()` - Conditional retry based on error predicate
- `retry_with_hooks()` - Retry with observability callbacks
- `with_timeout()` - Add timeout to any effect

These functions integrate with the existing `crate::retry::{RetryPolicy, RetryEvent, RetryExhausted, TimeoutError}` types but operate on the new zero-cost Effect trait rather than the legacy boxed effects.

## Objective

Achieve comprehensive test coverage for `src/effect/retry.rs`, targeting >90% line coverage. Tests should validate:
1. Correct retry behavior with various policies
2. Proper error propagation and retry exhaustion
3. Conditional retry logic
4. Hook invocation for observability
5. Timeout behavior

## Requirements

### Functional Requirements

#### FR-1: `retry()` Function Tests
- Test successful operation on first attempt (no retries needed)
- Test successful operation after N retries
- Test retry exhaustion when all attempts fail
- Test correct attempt counting in `RetryExhausted` result
- Test timing information in `RetryExhausted`
- Test with various `RetryPolicy` configurations (exponential, linear, constant)

#### FR-2: `retry_if()` Function Tests
- Test retryable errors are retried
- Test non-retryable errors fail immediately
- Test mixed scenarios (some retryable, some not)
- Test predicate receives correct error reference
- Test retry exhaustion for retryable errors

#### FR-3: `retry_with_hooks()` Function Tests
- Test `on_retry` hook is called for each retry attempt
- Test hook receives correct `RetryEvent` information
- Test hook is not called on first attempt
- Test hook is not called after success
- Test multiple hooks can be used for different purposes

#### FR-4: `with_timeout()` Function Tests
- Test operation completes before timeout
- Test operation times out correctly
- Test `TimeoutError::Timeout` variant returned on timeout
- Test `TimeoutError::Inner` variant returned on effect failure
- Test timeout with zero duration edge case

### Non-Functional Requirements

#### NFR-1: Test Execution Speed
- Tests should complete within 5 seconds total
- Use minimal sleep durations (1-10ms) where timing is needed
- Avoid real network or I/O operations

#### NFR-2: Determinism
- Tests must be deterministic and reproducible
- No flaky behavior from race conditions
- Controlled timing using tokio test utilities

#### NFR-3: Isolation
- Each test should be independent
- No shared state between tests
- Clean environment setup/teardown

## Acceptance Criteria

- [ ] `retry()` tests cover success on first attempt
- [ ] `retry()` tests cover success after multiple retries
- [ ] `retry()` tests cover exhaustion with all failures
- [ ] `retry()` tests verify attempt count accuracy
- [ ] `retry()` tests verify elapsed time tracking
- [ ] `retry_if()` tests cover retryable error path
- [ ] `retry_if()` tests cover non-retryable error path
- [ ] `retry_if()` tests cover predicate evaluation
- [ ] `retry_with_hooks()` tests verify hook invocation
- [ ] `retry_with_hooks()` tests verify `RetryEvent` data
- [ ] `with_timeout()` tests cover success before timeout
- [ ] `with_timeout()` tests cover timeout occurrence
- [ ] `with_timeout()` tests cover inner error propagation
- [ ] Line coverage for `src/effect/retry.rs` exceeds 90%
- [ ] All tests pass with `cargo test`
- [ ] No new clippy warnings introduced

## Technical Details

### Implementation Approach

Tests will be added to a new `#[cfg(test)]` module within `src/effect/retry.rs` or as a separate test file at `src/effect/retry_tests.rs`.

### Test Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::prelude::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;
    use std::time::Duration;

    // Helper to create a failing effect that succeeds after N attempts
    fn flaky_effect(
        fail_count: Arc<AtomicU32>,
        succeed_after: u32,
    ) -> impl Effect<Output = i32, Error = String, Env = ()> {
        from_fn(move |_| {
            let current = fail_count.fetch_add(1, Ordering::SeqCst);
            if current < succeed_after {
                Err(format!("attempt {} failed", current))
            } else {
                Ok(42)
            }
        })
    }

    #[tokio::test]
    async fn test_retry_success_first_attempt() { ... }

    #[tokio::test]
    async fn test_retry_success_after_retries() { ... }

    #[tokio::test]
    async fn test_retry_exhaustion() { ... }

    // ... additional tests
}
```

### Key Test Scenarios

1. **First Attempt Success**
   ```rust
   let effect = retry(|| pure::<_, String, ()>(42), policy);
   let result = effect.run(&()).await;
   assert!(result.is_ok());
   assert_eq!(result.unwrap().attempts(), 1);
   ```

2. **Retry Until Success**
   ```rust
   let attempts = Arc::new(AtomicU32::new(0));
   let effect = retry(
       || flaky_effect(attempts.clone(), 3),
       RetryPolicy::constant(Duration::from_millis(1)).with_max_retries(5)
   );
   let result = effect.run(&()).await;
   assert_eq!(result.unwrap().attempts(), 4); // 3 failures + 1 success
   ```

3. **Exhaustion**
   ```rust
   let effect = retry(
       || fail::<i32, _, ()>("always fails"),
       RetryPolicy::constant(Duration::from_millis(1)).with_max_retries(3)
   );
   let result = effect.run(&()).await;
   assert!(result.is_err());
   assert_eq!(result.unwrap_err().attempts(), 4);
   ```

## Dependencies

- **Prerequisites**: None
- **Affected Components**: `src/effect/retry.rs`
- **External Dependencies**: `tokio` (for async testing)

## Testing Strategy

- **Unit Tests**: Direct tests of each function
- **Integration Tests**: Test retry with real effect chains
- **Edge Cases**: Zero retries, immediate timeout, max duration policies

## Documentation Requirements

- **Code Documentation**: Add doc examples that serve as tests
- **User Documentation**: Ensure examples in retry.rs doctests are accurate

## Implementation Notes

- Use `Arc<AtomicU32>` for tracking attempts across effect recreations
- Use short durations (1-10ms) to keep tests fast
- Consider using `tokio::time::pause()` for deterministic timing tests
- Factory function pattern means each retry creates fresh effect

## Migration and Compatibility

No migration needed - this is additive test coverage.
