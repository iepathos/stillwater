---
number: 037
title: Effect Parallel Execution Tests
category: testing
priority: high
status: draft
dependencies: []
created: 2025-11-27
---

# Specification 037: Effect Parallel Execution Tests

**Category**: testing
**Priority**: high
**Status**: draft
**Dependencies**: None

## Context

The `src/effect/parallel.rs` module provides parallel execution functions for the zero-cost Effect system. Current coverage is **42% (18/43 lines)**, with significant gaps in:

- Error accumulation paths in `par_all`
- First-error behavior in `par_try_all`
- Race semantics in `race`
- Concurrency limiting in `par_all_limit`

These functions are critical for real-world applications that need to execute multiple effects concurrently, and incomplete test coverage risks subtle bugs in error handling and concurrency behavior.

### Uncovered Lines (from tarpaulin)
```
src/effect/parallel.rs: 50, 94, 119, 125-126, 129, 131, 134-135, 190, 208, 230, 242-244, 246, 248-249, 251-254, 258-259, 261
```

## Objective

Achieve comprehensive test coverage for `src/effect/parallel.rs`, targeting >90% line coverage. Tests should validate:
1. Correct parallel execution semantics
2. Error accumulation vs fail-fast behavior
3. Race winner selection
4. Concurrency limiting behavior
5. Environment sharing across parallel effects

## Requirements

### Functional Requirements

#### FR-1: `par_all()` Function Tests
- Test all effects succeed → returns all results
- Test some effects fail → returns all errors (not just first)
- Test all effects fail → returns all errors
- Test empty collection → returns empty result
- Test single effect → works correctly
- Test effects actually run in parallel (timing verification)
- Test error order matches input order

#### FR-2: `par_try_all()` Function Tests
- Test all effects succeed → returns all results
- Test first effect fails → returns first error only
- Test later effect fails → returns that error
- Test failure short-circuits (other effects may not complete)
- Test empty collection → returns empty result

#### FR-3: `race()` Function Tests
- Test first to succeed wins
- Test slower effects don't affect result
- Test all fail → returns all errors
- Test single effect → returns its result
- Test empty collection handling
- Test winner is actually first to complete (timing)

#### FR-4: `par_all_limit()` Function Tests (if implemented)
- Test respects concurrency limit
- Test all effects eventually complete
- Test error handling with limited concurrency
- Test limit of 1 (sequential execution)
- Test limit >= effect count (no limiting)

#### FR-5: Environment Sharing Tests
- Test environment is correctly passed to all parallel effects
- Test `Sync` requirement for cross-thread sharing
- Test environment cloning behavior

### Non-Functional Requirements

#### NFR-1: Test Execution Speed
- Tests should complete within 5 seconds total
- Use timing verification with controlled delays (10-50ms)

#### NFR-2: Concurrency Verification
- Tests should verify actual parallel execution
- Use timing to confirm effects run concurrently, not sequentially

#### NFR-3: Determinism
- Error ordering tests must be deterministic
- Race tests should use delays to control winner

## Acceptance Criteria

- [ ] `par_all` tests cover all-success path
- [ ] `par_all` tests cover partial-failure path with error accumulation
- [ ] `par_all` tests cover all-failure path
- [ ] `par_all` tests verify parallel execution via timing
- [ ] `par_try_all` tests cover all-success path
- [ ] `par_try_all` tests cover first-error behavior
- [ ] `par_try_all` tests verify fail-fast semantics
- [ ] `race` tests cover first-to-succeed semantics
- [ ] `race` tests cover all-fail error accumulation
- [ ] `race` tests verify timing-based winner selection
- [ ] Environment sharing tests pass
- [ ] Line coverage for `src/effect/parallel.rs` exceeds 90%
- [ ] All tests pass with `cargo test`
- [ ] No race conditions or flaky tests

## Technical Details

### Implementation Approach

Tests will be added to a `#[cfg(test)]` module in `src/effect/parallel.rs`.

### Test Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::prelude::*;
    use std::time::{Duration, Instant};

    // Helper: effect that succeeds after a delay
    fn delayed_success<T: Clone + Send + 'static>(
        value: T,
        delay: Duration,
    ) -> BoxedEffect<T, String, ()> {
        from_async(move |_| {
            let value = value.clone();
            async move {
                tokio::time::sleep(delay).await;
                Ok(value)
            }
        }).boxed()
    }

    // Helper: effect that fails after a delay
    fn delayed_failure(
        error: String,
        delay: Duration,
    ) -> BoxedEffect<i32, String, ()> {
        from_async(move |_| {
            let error = error.clone();
            async move {
                tokio::time::sleep(delay).await;
                Err(error)
            }
        }).boxed()
    }

    #[tokio::test]
    async fn test_par_all_all_succeed() {
        let effects = vec![
            pure(1).boxed(),
            pure(2).boxed(),
            pure(3).boxed(),
        ];
        let result = par_all(effects, &()).await;
        assert_eq!(result, Ok(vec![1, 2, 3]));
    }

    #[tokio::test]
    async fn test_par_all_accumulates_errors() {
        let effects: Vec<BoxedEffect<i32, String, ()>> = vec![
            pure(1).boxed(),
            fail("error1".to_string()).boxed(),
            pure(3).boxed(),
            fail("error2".to_string()).boxed(),
        ];
        let result = par_all(effects, &()).await;
        assert_eq!(result, Err(vec!["error1".to_string(), "error2".to_string()]));
    }

    #[tokio::test]
    async fn test_par_all_runs_in_parallel() {
        let delay = Duration::from_millis(50);
        let effects = vec![
            delayed_success(1, delay),
            delayed_success(2, delay),
            delayed_success(3, delay),
        ];

        let start = Instant::now();
        let result = par_all(effects, &()).await;
        let elapsed = start.elapsed();

        assert!(result.is_ok());
        // If parallel: ~50ms, if sequential: ~150ms
        assert!(elapsed < Duration::from_millis(100));
    }

    #[tokio::test]
    async fn test_race_first_wins() {
        let effects = vec![
            delayed_success(1, Duration::from_millis(10)),  // Winner
            delayed_success(2, Duration::from_millis(100)),
            delayed_success(3, Duration::from_millis(100)),
        ];

        let result = race(effects, &()).await;
        assert_eq!(result, Ok(1));
    }

    #[tokio::test]
    async fn test_race_all_fail() {
        let effects: Vec<BoxedEffect<i32, String, ()>> = vec![
            fail("e1".to_string()).boxed(),
            fail("e2".to_string()).boxed(),
        ];

        let result = race(effects, &()).await;
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 2);
    }

    // ... additional tests
}
```

### Key Test Scenarios

1. **Error Accumulation in par_all**
   - Verifies ALL errors are collected, not just the first
   - Critical for debugging production issues

2. **Timing-Based Race Tests**
   - Uses controlled delays to ensure deterministic winner
   - Verifies race semantics (first to complete wins)

3. **Parallel Execution Verification**
   - Timing tests confirm concurrent execution
   - Sequential would take N * delay, parallel takes ~delay

## Dependencies

- **Prerequisites**: None
- **Affected Components**: `src/effect/parallel.rs`
- **External Dependencies**: `tokio` (for async testing and timing)

## Testing Strategy

- **Unit Tests**: Direct tests of each function
- **Timing Tests**: Verify concurrent execution
- **Edge Cases**: Empty collections, single effect, all fail/succeed

## Documentation Requirements

- **Code Documentation**: Ensure doctest examples are accurate
- **User Documentation**: Update parallel effects guide if needed

## Implementation Notes

- Use `tokio::time::sleep` for controlled delays
- Use `Instant::now()` and `elapsed()` for timing verification
- Keep delays short (10-50ms) for fast tests
- Consider `#[ignore]` for slower timing tests if needed
- BoxedEffect required for homogeneous collections

## Migration and Compatibility

No migration needed - this is additive test coverage.
