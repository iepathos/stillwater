---
number: 007
title: Try Trait Integration for Question Mark Operator
category: ergonomics
priority: medium
status: draft
dependencies: [002, 003]
created: 2025-11-21
---

# Specification 007: Try Trait Integration for Question Mark Operator

**Category**: ergonomics
**Priority**: medium
**Status**: draft
**Dependencies**: Spec 002 (Validation Type), Spec 003 (Effect Type)

## Context

Rust's `?` operator provides ergonomic error propagation for Result and Option types. Currently, Validation and Effect require explicit error handling, leading to verbose code with repeated `.map_err()` calls.

This addresses one of the identified pain points: "Frequent .map_err() appears everywhere."

By implementing the Try trait (currently unstable), we can enable `?` operator support for both Validation and Effect, making error handling much more ergonomic while maintaining type safety.

Note: This requires nightly Rust due to unstable `try_trait_v2` feature. For stable Rust users, we'll provide alternative helpers (like `and_then_auto()` from Spec 006).

## Objective

Implement the Try trait for Validation<T, E> and Effect<T, E, Env> to enable `?` operator support, making error propagation ergonomic while maintaining type safety.

## Requirements

### Functional Requirements

- Implement Try trait for Validation<T, E>
- Implement Try trait for Effect<T, E, Env>
- Support `?` in functions returning Validation
- Support `?` in functions returning Effect
- Provide clear compiler errors for type mismatches
- Work seamlessly with existing combinators
- Document nightly requirement and alternatives

### Non-Functional Requirements

- Zero runtime overhead
- Type inference works naturally
- Clear error messages
- Works with existing Result/Option interop
- Graceful degradation for stable Rust users

## Acceptance Criteria

- [ ] Try trait implemented for Validation<T, E>
- [ ] Try trait implemented for Effect<T, E, Env>
- [ ] `?` operator works in validation functions
- [ ] `?` operator works in effect-returning functions
- [ ] Mixing Result and Validation with `?` works correctly
- [ ] Mixing Result and Effect with `?` works correctly
- [ ] Comprehensive tests (>95% coverage)
- [ ] Documentation explains nightly requirement
- [ ] Documentation shows stable alternatives
- [ ] Feature flag for Try trait support

## Technical Details

### Implementation Approach

```rust
#![feature(try_trait_v2)]

use std::ops::{ControlFlow, FromResidual, Try};

// Try trait for Validation
impl<T, E> Try for Validation<T, E> {
    type Output = T;
    type Residual = Validation<std::convert::Infallible, E>;

    fn from_output(output: Self::Output) -> Self {
        Validation::Success(output)
    }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            Validation::Success(value) => ControlFlow::Continue(value),
            Validation::Failure(error) => ControlFlow::Break(Validation::Failure(error)),
        }
    }
}

impl<T, E> FromResidual<Validation<std::convert::Infallible, E>> for Validation<T, E> {
    fn from_residual(residual: Validation<std::convert::Infallible, E>) -> Self {
        match residual {
            Validation::Failure(error) => Validation::Failure(error),
            Validation::Success(_) => unreachable!(),
        }
    }
}

// Allow ? to convert Result to Validation
impl<T, E> FromResidual<Result<std::convert::Infallible, E>> for Validation<T, E> {
    fn from_residual(residual: Result<std::convert::Infallible, E>) -> Self {
        match residual {
            Err(error) => Validation::Failure(error),
            Ok(_) => unreachable!(),
        }
    }
}

// Try trait for Effect
impl<T, E, Env> Try for Effect<T, E, Env>
where
    T: Send + 'static,
    E: Send + 'static,
{
    type Output = T;
    type Residual = Effect<std::convert::Infallible, E, Env>;

    fn from_output(output: Self::Output) -> Self {
        Effect::pure(output)
    }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        // Note: Try trait requires non-async branch, so we can't truly support
        // async ? operator without language changes. This is a limitation.
        //
        // For now, document that ? works for sync Effects constructed via
        // Effect::from_fn, but async Effects need .await before ?
        unimplemented!("Effect is async and cannot support ? directly. Use .await then ? on Result")
    }
}
```

### Limitation: Effect and Async

After investigation, the Try trait cannot work with async Effect directly because:
1. Try::branch must be synchronous
2. Effect wraps async computation (BoxFuture)
3. We can't .await inside Try::branch

**Solution**: Document pattern of `.await?` instead of `?` alone:

```rust
// This works:
async fn process_user(id: u64) -> Result<User, AppError> {
    let user = fetch_user(id).run(&env).await?;  // .await then ?
    let validated = validate_user(user).await?;
    Ok(validated)
}

// This doesn't work (Try can't await):
async fn process_user(id: u64) -> Effect<User, AppError, Env> {
    let user = fetch_user(id)?;  // ❌ Can't ? an Effect directly
    Ok(user)
}
```

**Decision**: Focus Try implementation on Validation only, document that Effect should use `.await?` pattern.

### Revised Implementation Approach

```rust
#![cfg_attr(feature = "try_trait", feature(try_trait_v2))]

#[cfg(feature = "try_trait")]
mod try_impl {
    use super::*;
    use std::ops::{ControlFlow, FromResidual, Try};

    // Try trait for Validation
    impl<T, E> Try for Validation<T, E> {
        type Output = T;
        type Residual = Validation<std::convert::Infallible, E>;

        fn from_output(output: Self::Output) -> Self {
            Validation::Success(output)
        }

        fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
            match self {
                Validation::Success(value) => ControlFlow::Continue(value),
                Validation::Failure(error) => ControlFlow::Break(Validation::Failure(error)),
            }
        }
    }

    impl<T, E> FromResidual<Validation<std::convert::Infallible, E>> for Validation<T, E> {
        fn from_residual(residual: Validation<std::convert::Infallible, E>) -> Self {
            match residual {
                Validation::Failure(error) => Validation::Failure(error),
                Validation::Success(_) => unreachable!(),
            }
        }
    }

    // Allow ? to convert Result to Validation
    impl<T, E> FromResidual<Result<std::convert::Infallible, E>> for Validation<T, E> {
        fn from_residual(residual: Result<std::convert::Infallible, E>) -> Self {
            match residual {
                Err(error) => Validation::Failure(error),
                Ok(_) => unreachable!(),
            }
        }
    }
}
```

### Architecture Changes

- Add feature flag: `try_trait` in Cargo.toml
- Conditional compilation for Try implementations
- Documentation explaining nightly requirement
- Examples showing both `?` and alternative patterns

### APIs and Interfaces

With Try trait (nightly + feature flag):
```rust
fn validate_registration(data: RegistrationData) -> Validation<User, Vec<ValidationError>> {
    let email = validate_email(&data.email)?;
    let password = validate_password(&data.password)?;
    let age = validate_age(data.age)?;

    Validation::success(User { email, password, age })
}
```

Without Try trait (stable Rust):
```rust
fn validate_registration(data: RegistrationData) -> Validation<User, Vec<ValidationError>> {
    Validation::all((
        validate_email(&data.email),
        validate_password(&data.password),
        validate_age(data.age),
    ))
    .map(|(email, password, age)| User { email, password, age })
}
```

For Effect (always use .await?):
```rust
async fn process_user(id: u64) -> Result<ProcessedUser, AppError> {
    let user = fetch_user(id).run(&env).await?;
    let enriched = enrich_user(user).run(&env).await?;
    Ok(enriched)
}
```

## Dependencies

- **Prerequisites**: Spec 002 (Validation), Spec 003 (Effect)
- **Affected Components**: Validation and Effect types
- **External Dependencies**: Requires nightly Rust + try_trait_v2 feature

## Testing Strategy

### Unit Tests

```rust
#![cfg(feature = "try_trait")]

#[test]
fn test_validation_question_mark() {
    fn parse_and_validate(s: &str) -> Validation<i32, Vec<String>> {
        let num: i32 = s.parse()
            .map_err(|e| vec![format!("Parse error: {}", e)])?;

        if num >= 0 {
            Validation::success(num)
        } else {
            Validation::failure(vec!["Number must be positive".to_string()])
        }
    }

    assert_eq!(parse_and_validate("42"), Validation::Success(42));
    assert!(parse_and_validate("-5").is_failure());
    assert!(parse_and_validate("abc").is_failure());
}

#[test]
fn test_mixing_result_and_validation() {
    fn process(s: &str) -> Validation<String, Vec<String>> {
        // Result from standard library function
        let parsed: i32 = s.parse()
            .map_err(|_| vec!["Invalid number".to_string()])?;

        // Validation from our function
        let validated = validate_positive(parsed)?;

        Validation::success(format!("Processed: {}", validated))
    }

    fn validate_positive(n: i32) -> Validation<i32, Vec<String>> {
        if n > 0 {
            Validation::success(n)
        } else {
            Validation::failure(vec!["Must be positive".to_string()])
        }
    }

    assert_eq!(
        process("42"),
        Validation::Success("Processed: 42".to_string())
    );
    assert!(process("-5").is_failure());
    assert!(process("abc").is_failure());
}

#[tokio::test]
async fn test_effect_await_question_mark() {
    async fn fetch_and_process(id: u64) -> Result<String, String> {
        let user = fetch_user(id).run(&()).await?;
        let processed = process_user(user).run(&()).await?;
        Ok(processed)
    }

    fn fetch_user(id: u64) -> Effect<String, String, ()> {
        if id > 0 {
            Effect::pure(format!("User {}", id))
        } else {
            Effect::fail("Invalid ID".to_string())
        }
    }

    fn process_user(user: String) -> Effect<String, String, ()> {
        Effect::pure(format!("Processed: {}", user))
    }

    assert_eq!(
        fetch_and_process(42).await,
        Ok("Processed: User 42".to_string())
    );
    assert_eq!(
        fetch_and_process(0).await,
        Err("Invalid ID".to_string())
    );
}
```

### Integration Tests

```rust
#![cfg(feature = "try_trait")]

#[test]
fn test_real_world_form_validation_with_question_mark() {
    #[derive(Debug, PartialEq)]
    enum ValidationError {
        InvalidEmail,
        PasswordTooShort,
        AgeTooYoung,
    }

    fn validate_form(
        email: &str,
        password: &str,
        age: u8,
    ) -> Validation<RegistrationData, Vec<ValidationError>> {
        let email = validate_email(email)?;
        let password = validate_password(password)?;
        let age = validate_age(age)?;

        Validation::success(RegistrationData {
            email,
            password,
            age,
        })
    }

    fn validate_email(email: &str) -> Validation<String, Vec<ValidationError>> {
        if email.contains('@') {
            Validation::success(email.to_string())
        } else {
            Validation::failure(vec![ValidationError::InvalidEmail])
        }
    }

    fn validate_password(pwd: &str) -> Validation<String, Vec<ValidationError>> {
        if pwd.len() >= 8 {
            Validation::success(pwd.to_string())
        } else {
            Validation::failure(vec![ValidationError::PasswordTooShort])
        }
    }

    fn validate_age(age: u8) -> Validation<u8, Vec<ValidationError>> {
        if age >= 18 {
            Validation::success(age)
        } else {
            Validation::failure(vec![ValidationError::AgeTooYoung])
        }
    }

    // Valid form
    let result = validate_form("user@example.com", "password123", 25);
    assert!(result.is_success());

    // Note: With ?, we get fail-fast behavior, not error accumulation
    // First error stops the chain
    let result = validate_form("invalid", "short", 15);
    assert_eq!(
        result,
        Validation::Failure(vec![ValidationError::InvalidEmail])
    );
    // Only sees first error (email), doesn't check password or age
}
```

## Documentation Requirements

### Code Documentation

- Comprehensive rustdoc for Try implementations
- Examples showing `?` usage
- Examples showing stable alternatives
- Explain fail-fast vs accumulation trade-off
- Document nightly requirement clearly

### User Documentation

- Add "Try Trait Support" section to README
- Create guide in docs/guide/04-try-trait.md
- Show when to use `?` vs `Validation::all()`
- Explain Effect + async + ? pattern

### Architecture Updates

- Document Try trait decision in DESIGN.md
- Explain limitation with async Effect
- Show feature flag usage

## Implementation Notes

### Feature Flag Setup

```toml
[features]
default = []
try_trait = []

[package.metadata.docs.rs]
features = ["try_trait"]
rustdoc-args = ["--cfg", "docsrs"]
```

### Nightly Rust

Users must use nightly and enable feature:
```rust
#![feature(try_trait_v2)]
```

In Cargo.toml:
```toml
[dependencies]
stillwater = { version = "0.1", features = ["try_trait"] }
```

### Fail-Fast vs Accumulation

Important trade-off to document:

**With `?` (fail-fast)**:
```rust
fn validate(data: Data) -> Validation<ValidData, Vec<Error>> {
    let a = validate_a(&data.a)?;  // Stops here on failure
    let b = validate_b(&data.b)?;  // Never reached if a fails
    let c = validate_c(&data.c)?;  // Never reached if a or b fails
    Validation::success(ValidData { a, b, c })
}
// Only reports FIRST error
```

**With `Validation::all()` (accumulation)**:
```rust
fn validate(data: Data) -> Validation<ValidData, Vec<Error>> {
    Validation::all((
        validate_a(&data.a),  // All three always run
        validate_b(&data.b),
        validate_c(&data.c),
    ))
    .map(|(a, b, c)| ValidData { a, b, c })
}
// Reports ALL errors at once
```

**Guideline**:
- Use `?` for sequential dependencies (each step needs previous result)
- Use `Validation::all()` for independent validations (better UX)

### Stable Rust Alternatives

For users on stable Rust:
1. Use `Validation::all()` for parallel validation
2. Use `and_then()` for sequential validation
3. Use `and_then_auto()` for automatic error conversion (Spec 006)

## Migration and Compatibility

- Feature is opt-in via feature flag
- No breaking changes to existing API
- Stable Rust users can ignore this feature entirely

## Open Questions

1. Should we provide a proc macro to generate fail-fast validation without `?`?
   - Decision: No, `and_then()` is sufficient, don't add macro complexity

2. Should we warn about fail-fast behavior in docs?
   - Decision: Yes, clearly document the trade-off

3. Should we wait for Try trait stabilization?
   - Decision: Implement behind feature flag now, users can opt in on nightly

4. Can we make Effect work with `?` somehow?
   - Decision: No, fundamental limitation. Document `.await?` pattern instead.
