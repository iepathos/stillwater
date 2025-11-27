---
number: 039
title: Bracket Resource Management Tests
category: testing
priority: medium
status: draft
dependencies: []
created: 2025-11-27
---

# Specification 039: Bracket Resource Management Tests

**Category**: testing
**Priority**: medium
**Status**: draft
**Dependencies**: None

## Context

The `src/effect/bracket.rs` module implements the bracket pattern for safe resource management - ensuring resources are always released even when errors occur. Current coverage is **42% (8/19 lines)**, with gaps in:

- Debug trait implementation
- Error during use phase (release still runs)
- Error during release phase

### Uncovered Lines (from tarpaulin)
```
src/effect/bracket.rs: 36-40, 153, 198-202
```

The bracket pattern is critical for:
- File handle management
- Database connections
- Network sockets
- Any acquire/use/release pattern

Without comprehensive tests, we cannot guarantee that resources are properly released in all error scenarios.

## Objective

Achieve comprehensive test coverage for `src/effect/bracket.rs`, targeting >90% line coverage. Tests should validate:
1. Normal acquire → use → release flow
2. Release executes even when use fails
3. Correct error propagation
4. Resource cleanup guarantees

## Requirements

### Functional Requirements

#### FR-1: Happy Path Tests
- Test acquire succeeds → use succeeds → release runs → returns use result
- Test resource is available during use phase
- Test release receives the acquired resource

#### FR-2: Error During Use Phase
- Test acquire succeeds → use fails → release still runs
- Test error from use phase is returned
- Test resource is still released despite error

#### FR-3: Error During Acquire Phase
- Test acquire fails → use never runs → release never runs
- Test acquire error is returned

#### FR-4: Error During Release Phase
- Test acquire succeeds → use succeeds → release fails
- Determine error priority: use result vs release error
- Document the chosen semantics

#### FR-5: Resource Tracking
- Test that acquired resource is passed to use function
- Test that same resource is passed to release function
- Use observable side effects to verify execution order

### Non-Functional Requirements

#### NFR-1: Test Clarity
- Each test should focus on one scenario
- Use descriptive names indicating the scenario

#### NFR-2: Observable Verification
- Use `Arc<Mutex<Vec<&str>>>` or similar to track execution order
- Verify acquire, use, release called in correct order

## Acceptance Criteria

- [ ] Happy path test: acquire → use → release → returns value
- [ ] Error during use: release still executes
- [ ] Error during acquire: use and release do not execute
- [ ] Execution order verification test passes
- [ ] Resource is correctly passed through all phases
- [ ] Debug impl test (if meaningful coverage gain)
- [ ] Line coverage for `src/effect/bracket.rs` exceeds 90%
- [ ] All tests pass with `cargo test`

## Technical Details

### Implementation Approach

Tests will be added to a `#[cfg(test)]` module in `src/effect/bracket.rs` or the existing `src/effect/tests.rs`.

### Test Structure

```rust
#[cfg(test)]
mod bracket_tests {
    use super::*;
    use crate::effect::prelude::*;
    use std::sync::{Arc, Mutex};

    // Track execution order
    type Log = Arc<Mutex<Vec<&'static str>>>;

    fn make_log() -> Log {
        Arc::new(Mutex::new(Vec::new()))
    }

    #[tokio::test]
    async fn test_bracket_happy_path() {
        let log = make_log();
        let log_clone = log.clone();

        let effect = bracket(
            // Acquire
            {
                let log = log.clone();
                from_fn(move |_| {
                    log.lock().unwrap().push("acquire");
                    Ok::<_, String>("resource")
                })
            },
            // Use
            {
                let log = log.clone();
                move |resource| {
                    from_fn(move |_| {
                        log.lock().unwrap().push("use");
                        Ok::<_, String>(format!("used {}", resource))
                    })
                }
            },
            // Release
            {
                let log = log_clone;
                move |_resource| {
                    from_fn(move |_| {
                        log.lock().unwrap().push("release");
                        Ok::<_, String>(())
                    })
                }
            },
        );

        let result = effect.run(&()).await;

        assert_eq!(result, Ok("used resource".to_string()));
        assert_eq!(*log.lock().unwrap(), vec!["acquire", "use", "release"]);
    }

    #[tokio::test]
    async fn test_bracket_use_fails_release_still_runs() {
        let log = make_log();
        let log_clone = log.clone();

        let effect = bracket(
            // Acquire
            {
                let log = log.clone();
                from_fn(move |_| {
                    log.lock().unwrap().push("acquire");
                    Ok::<_, String>(42)
                })
            },
            // Use - FAILS
            {
                let log = log.clone();
                move |_resource| {
                    from_fn(move |_| {
                        log.lock().unwrap().push("use");
                        Err::<i32, _>("use failed".to_string())
                    })
                }
            },
            // Release - should still run!
            {
                let log = log_clone;
                move |_resource| {
                    from_fn(move |_| {
                        log.lock().unwrap().push("release");
                        Ok::<_, String>(())
                    })
                }
            },
        );

        let result = effect.run(&()).await;

        assert_eq!(result, Err("use failed".to_string()));
        // Crucially: release still ran despite use failure
        assert_eq!(*log.lock().unwrap(), vec!["acquire", "use", "release"]);
    }

    #[tokio::test]
    async fn test_bracket_acquire_fails() {
        let log = make_log();
        let log_clone = log.clone();
        let log_clone2 = log.clone();

        let effect = bracket(
            // Acquire - FAILS
            {
                let log = log.clone();
                from_fn(move |_| {
                    log.lock().unwrap().push("acquire");
                    Err::<i32, _>("acquire failed".to_string())
                })
            },
            // Use - should NOT run
            {
                let log = log_clone;
                move |_resource| {
                    from_fn(move |_| {
                        log.lock().unwrap().push("use");
                        Ok::<_, String>(0)
                    })
                }
            },
            // Release - should NOT run
            {
                let log = log_clone2;
                move |_resource| {
                    from_fn(move |_| {
                        log.lock().unwrap().push("release");
                        Ok::<_, String>(())
                    })
                }
            },
        );

        let result = effect.run(&()).await;

        assert_eq!(result, Err("acquire failed".to_string()));
        // Only acquire ran
        assert_eq!(*log.lock().unwrap(), vec!["acquire"]);
    }

    #[tokio::test]
    async fn test_bracket_resource_passed_correctly() {
        #[derive(Clone, Debug, PartialEq)]
        struct Resource { id: u32 }

        let effect = bracket(
            from_fn(|_| Ok::<_, String>(Resource { id: 42 })),
            |resource| {
                let id = resource.id;
                from_fn(move |_| Ok::<_, String>(id * 2))
            },
            |resource| {
                // Verify we got the same resource
                assert_eq!(resource.id, 42);
                pure(())
            },
        );

        let result = effect.run(&()).await;
        assert_eq!(result, Ok(84));
    }
}
```

### Key Test Scenarios

1. **Happy Path** - Normal execution, all phases succeed
2. **Use Failure** - Critical: release must still execute
3. **Acquire Failure** - Use and release should not execute
4. **Resource Flow** - Same resource flows through all phases

## Dependencies

- **Prerequisites**: None
- **Affected Components**: `src/effect/bracket.rs`
- **External Dependencies**: `tokio` for async testing

## Testing Strategy

- **Unit Tests**: Direct tests of `bracket` function
- **Order Verification**: Use shared log to verify execution order
- **Error Propagation**: Verify correct error is returned

## Documentation Requirements

- **Code Documentation**: Ensure bracket docs describe error semantics
- **User Documentation**: Consider adding to effects guide

## Implementation Notes

- Use `Arc<Mutex<Vec<&str>>>` to track execution order across closures
- The `bracket` function uses FnOnce, so closures are consumed
- Resource type should implement Clone if needed in multiple phases
- Consider what happens if release also fails (document behavior)

## Migration and Compatibility

No migration needed - this is additive test coverage.

## Open Questions

1. **Release error handling**: If use succeeds but release fails, which error is returned? Current implementation likely returns use result, losing release error. Should this be documented or changed?

2. **Panic handling**: Does bracket handle panics during use phase? Should it?
