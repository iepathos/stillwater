---
number: 006
title: Helper Combinators for Ergonomic Composition
category: optimization
priority: medium
status: draft
dependencies: [003, 005]
created: 2025-11-21
---

# Specification 006: Helper Combinators for Ergonomic Composition

**Category**: optimization
**Priority**: medium
**Status**: draft
**Dependencies**: Spec 003 (Effect Type), Spec 005 (IO Module)

## Context

While the core Effect combinators (`map`, `and_then`, `map_err`) are sufficient, common patterns emerge that require boilerplate. Helper combinators reduce this boilerplate and make code more readable.

Based on our pain point analysis and design work, we identified these high-value helpers:
- `tap()` - Perform side effect, return original value
- `check()` - Conditional failure
- `with()` - Combine effects, keep both results
- `and_then_auto()` - Auto-convert errors via From trait
- `and_then_ref()` - Borrow value, avoid cloning

These follow functional programming best practices and idiomatic Rust patterns.

## Objective

Implement helper combinator methods on Effect that reduce boilerplate for common patterns while maintaining clarity and type safety.

## Requirements

### Functional Requirements

- Implement `tap()` for side effects that return original value
- Implement `check()` for conditional failures with predicates
- Implement `with()` for combining effects and keeping both results
- Implement `and_then_auto()` for automatic error conversion
- Implement `and_then_ref()` for reference-based chaining
- All helpers should compose with existing Effect methods
- Maintain type safety and clear error messages

### Non-Functional Requirements

- Zero-cost abstractions (inline where possible)
- Clear documentation with examples
- Type inference works for common cases
- Helpers feel natural to Rust developers

## Acceptance Criteria

- [ ] `tap()` method implemented on Effect<T, E, Env> where T: Clone
- [ ] `check()` method implemented on Effect<T, E, Env>
- [ ] `with()` method implemented on Effect<T, E, Env>
- [ ] `and_then_auto()` method implemented with From bound
- [ ] `and_then_ref()` method implemented where T: Clone
- [ ] All methods have comprehensive rustdoc
- [ ] All methods have usage examples in docs
- [ ] Comprehensive tests (>95% coverage)
- [ ] Integration tests showing real-world usage

## Technical Details

### Implementation Approach

```rust
impl<T, E, Env> Effect<T, E, Env>
where
    T: Send + 'static,
    E: Send + 'static,
{
    /// Perform a side effect and return the original value
    ///
    /// Useful for logging, metrics, or other operations that don't
    /// affect the main computation.
    ///
    /// # Example
    /// ```rust
    /// user_effect
    ///     .tap(|user| IO::write(|logger| logger.info(format!("Created: {}", user.id))))
    ///     // user is returned unchanged
    /// ```
    pub fn tap<F>(self, f: F) -> Self
    where
        F: FnOnce(&T) -> Effect<(), E, Env> + Send + 'static,
        T: Clone,
    {
        self.and_then(move |value| {
            let value_clone = value.clone();
            f(&value).map(move |_| value_clone)
        })
    }

    /// Fail with error if predicate is false
    ///
    /// Provides a declarative way to express validation conditions.
    ///
    /// # Example
    /// ```rust
    /// user_effect
    ///     .check(|user| user.age >= 18, || AppError::AgeTooYoung)
    /// ```
    pub fn check<P, F>(self, predicate: P, error_fn: F) -> Self
    where
        P: FnOnce(&T) -> bool + Send + 'static,
        F: FnOnce() -> E + Send + 'static,
    {
        self.and_then(move |value| {
            if predicate(&value) {
                Effect::pure(value)
            } else {
                Effect::fail(error_fn())
            }
        })
    }

    /// Combine with another effect, returning both values
    ///
    /// Useful when you need results from multiple effects.
    ///
    /// # Example
    /// ```rust
    /// user_effect
    ///     .with(|user| fetch_orders(user.id))
    ///     .map(|(user, orders)| /* use both */)
    /// ```
    pub fn with<U, F>(self, f: F) -> Effect<(T, U), E, Env>
    where
        F: FnOnce(&T) -> Effect<U, E, Env> + Send + 'static,
        U: Send + 'static,
        T: Clone,
    {
        self.and_then(move |value| {
            let value_clone = value.clone();
            f(&value).map(move |other| (value_clone, other))
        })
    }

    /// Chain effect with automatic error conversion
    ///
    /// Eliminates manual `.map_err(E::from)` calls when error types differ.
    ///
    /// # Example
    /// ```rust
    /// user_effect
    ///     .and_then_auto(|user| validate_user(user))  // Different error type
    ///     .and_then_auto(|user| save_user(user))      // Another different error type
    /// ```
    pub fn and_then_auto<U, E2, F>(self, f: F) -> Effect<U, E, Env>
    where
        F: FnOnce(T) -> Effect<U, E2, Env> + Send + 'static,
        U: Send + 'static,
        E2: Send + 'static,
        E: From<E2>,
    {
        self.and_then(move |value| {
            f(value).map_err(E::from)
        })
    }

    /// Chain effect by borrowing value, then returning it
    ///
    /// Avoids multiple clones when you need to use a value in multiple effects
    /// but only care about the final result.
    ///
    /// # Example
    /// ```rust
    /// user_effect
    ///     .and_then_ref(|user| save_audit_log(user))  // Borrows user
    ///     .and_then_ref(|user| send_notification(user)) // Borrows user
    ///     // user is returned (cloned once per call, not accumulated)
    /// ```
    pub fn and_then_ref<U, F>(self, f: F) -> Self
    where
        F: FnOnce(&T) -> Effect<U, E, Env> + Send + 'static,
        U: Send + 'static,
        T: Clone,
    {
        self.and_then(move |value| {
            let value_clone = value.clone();
            f(&value).map(move |_| value_clone)
        })
    }
}
```

### Architecture Changes

- Add methods to Effect impl block in `src/effect.rs`
- No new modules needed

### Data Structures

No new data structures - these are methods on existing Effect type.

### APIs and Interfaces

See Implementation Approach above.

## Dependencies

- **Prerequisites**: Spec 003 (Effect type), Spec 005 (IO module for examples)
- **Affected Components**: Effect type (adds new methods)
- **External Dependencies**: None

## Testing Strategy

### Unit Tests

```rust
#[tokio::test]
async fn test_tap() {
    let mut called = false;
    let call_flag = &mut called as *mut bool;

    let effect = Effect::<_, String, ()>::pure(42)
        .tap(move |value| {
            unsafe { *call_flag = true; }
            Effect::pure(())
        });

    let result = effect.run(&()).await;
    assert_eq!(result, Ok(42));
    assert!(called);  // Side effect occurred
}

#[tokio::test]
async fn test_check_success() {
    let effect = Effect::<_, String, ()>::pure(25)
        .check(|age| *age >= 18, || "too young".to_string());

    assert_eq!(effect.run(&()).await, Ok(25));
}

#[tokio::test]
async fn test_check_failure() {
    let effect = Effect::<_, String, ()>::pure(15)
        .check(|age| *age >= 18, || "too young".to_string());

    assert_eq!(effect.run(&()).await, Err("too young".to_string()));
}

#[tokio::test]
async fn test_with() {
    let effect = Effect::<_, String, ()>::pure(5)
        .with(|value| Effect::pure(*value * 2));

    assert_eq!(effect.run(&()).await, Ok((5, 10)));
}

#[tokio::test]
async fn test_and_then_auto() {
    #[derive(Debug, PartialEq)]
    enum Error1 {
        Fail,
    }

    #[derive(Debug, PartialEq)]
    enum Error2 {
        Other(Error1),
    }

    impl From<Error1> for Error2 {
        fn from(e: Error1) -> Self {
            Error2::Other(e)
        }
    }

    let effect1 = Effect::<_, Error1, ()>::fail(Error1::Fail);
    let effect2 = effect1.and_then_auto(|_| Effect::<i32, Error2, ()>::pure(42));

    assert_eq!(effect2.run(&()).await, Err(Error2::Other(Error1::Fail)));
}

#[tokio::test]
async fn test_and_then_ref() {
    let effect = Effect::<_, String, ()>::pure(42)
        .and_then_ref(|value| {
            assert_eq!(*value, 42);
            Effect::pure("processed")
        });

    assert_eq!(effect.run(&()).await, Ok(42));
}

#[tokio::test]
async fn test_composition() {
    // All helpers should compose naturally
    let effect = Effect::<_, String, ()>::pure(20)
        .check(|age| *age >= 18, || "too young".to_string())
        .tap(|age| Effect::pure(println!("Age: {}", age)))
        .with(|age| Effect::pure(*age * 2))
        .map(|(age, double)| age + double);

    assert_eq!(effect.run(&()).await, Ok(60));  // 20 + 40
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_real_world_user_registration() {
    #[derive(Clone, Debug, PartialEq)]
    struct User {
        id: u64,
        email: String,
        age: u8,
    }

    #[derive(Debug, PartialEq)]
    enum AppError {
        AgeTooYoung,
        EmailExists,
        DbError,
    }

    struct Env {
        db: Database,
        logger: Logger,
        email_service: EmailService,
    }

    async fn register_user(user: User) -> Effect<User, AppError, Env> {
        Effect::pure(user)
            // Validate age
            .check(|u| u.age >= 18, || AppError::AgeTooYoung)

            // Check email doesn't exist
            .and_then_ref(|u| {
                IO::read(|db: &Database| db.email_exists(&u.email))
                    .and_then(|exists| {
                        if exists {
                            Effect::fail(AppError::EmailExists)
                        } else {
                            Effect::pure(())
                        }
                    })
            })

            // Save to database
            .and_then_ref(|u| {
                IO::write(|db: &Database| db.save(u))
                    .map_err(|_| AppError::DbError)
            })

            // Send welcome email (non-critical, log but don't fail)
            .tap(|u| {
                IO::write(|email: &EmailService| email.send_welcome(&u.email))
                    .or_else(|err| {
                        IO::write(|logger: &Logger| {
                            logger.warn(format!("Failed to send email: {:?}", err))
                        })
                    })
            })
    }

    // Test implementation...
}
```

## Documentation Requirements

### Code Documentation

- Comprehensive rustdoc for each helper
- Examples showing real use cases
- Explain when to use each helper
- Note Clone requirements where applicable

### User Documentation

- Add "Helper Combinators" section to README
- Create section in docs/guide/03-effects.md
- Show before/after comparisons (with vs without helpers)
- Document common patterns

### Architecture Updates

- Document design rationale in DESIGN.md
- Explain which helpers solve which pain points

## Implementation Notes

### Clone Requirements

Several helpers require `T: Clone`:
- `tap()` - needs to clone to return original
- `with()` - needs to clone to return tuple
- `and_then_ref()` - needs to clone to return after borrow

This is acceptable because:
- Most domain types are cheap to clone
- For expensive types, users can use Arc<T>
- Alternative would be more complex lifetimes

### Performance

- All helpers inline to direct code
- Clone overhead typically negligible for domain types
- Users can profile and optimize if needed

### When to Use

**Use `tap()` when:**
- Logging, metrics, notifications
- Side effects that don't affect main flow

**Use `check()` when:**
- Simple predicate validation
- Conditional failures

**Use `with()` when:**
- Need multiple effect results
- Combining user with their orders, profile, etc.

**Use `and_then_auto()` when:**
- Composing effects with different error types
- Error types implement From

**Use `and_then_ref()` when:**
- Multiple effects need same value
- Want to avoid accumulating clones

## Migration and Compatibility

No migration needed - these are additive features.

## Open Questions

1. Should we add `Effect::all()` for parallel execution?
   ```rust
   Effect::all(vec![fetch1, fetch2, fetch3])  // Run in parallel
   ```
   - Decision: YES, add to this spec

2. Should `tap()` allow fallible side effects?
   - Current: Effect<(), E, Env>
   - Alternative: Effect<(), E2, Env> where E: From<E2>
   - Decision: Current is fine, use `and_then_ref` if need different error

3. Should we add `try_tap()` that can fail?
   - Decision: No, use `and_then_ref` instead

## Additional Helper: Parallel Execution

```rust
impl<T, E, Env> Effect<T, E, Env>
where
    T: Send + 'static,
    E: Send + 'static,
{
    /// Run multiple effects in parallel
    ///
    /// # Example
    /// ```rust
    /// let users = Effect::all(vec![
    ///     fetch_user(1),
    ///     fetch_user(2),
    ///     fetch_user(3),
    /// ]);  // All fetched concurrently
    /// ```
    pub fn all<I>(effects: I) -> Effect<Vec<T>, E, Env>
    where
        I: IntoIterator<Item = Effect<T, E, Env>> + Send + 'static,
    {
        Effect {
            run_fn: Box::new(move |env| {
                Box::pin(async move {
                    let futures: Vec<_> = effects
                        .into_iter()
                        .map(|effect| (effect.run_fn)(env))
                        .collect();

                    let results = futures::future::try_join_all(futures).await?;
                    Ok(results)
                })
            }),
        }
    }
}
```

Add tests for parallel execution in this spec.
