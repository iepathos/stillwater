---
number: 002
title: Validation Type with Error Accumulation
category: foundation
priority: critical
status: draft
dependencies: [001]
created: 2025-11-21
---

# Specification 002: Validation Type with Error Accumulation

**Category**: foundation
**Priority**: critical
**Status**: draft
**Dependencies**: Spec 001 (Semigroup Trait)

## Context

Standard Result types short-circuit on the first error, which is frustrating for validation scenarios. When a user submits a form with 5 invalid fields, we want to tell them about ALL errors, not just the first one.

The Validation type solves this by accumulating errors using the Semigroup trait. It's similar to Result but designed specifically for validation where we want to collect all failures.

This is a core type in stillwater's philosophy of making validation ergonomic and user-friendly.

## Objective

Implement a `Validation<T, E>` type that accumulates errors when combining multiple validations, enabling comprehensive error reporting for forms, APIs, and data validation scenarios.

## Requirements

### Functional Requirements

- Define `Validation<T, E>` enum with `Success(T)` and `Failure(E)` variants
- Implement `all()` method for combining validations via tuples
- Implement `all_vec()` for homogeneous collections
- Implement `map()`, `and()`, `and_then()` for composition
- Provide conversion to/from Result
- Support pattern matching on variants
- Accumulate errors using Semigroup trait

### Non-Functional Requirements

- Type-safe (different validation types in tuples)
- Ergonomic API (feels natural to Rust developers)
- Clear error messages from compiler
- Zero-cost abstractions
- Support tuple sizes up to 12
- Comprehensive documentation

## Acceptance Criteria

- [ ] Validation enum defined in `src/validation.rs`
- [ ] Success and Failure variants implemented
- [ ] `all()` works with tuples (T1,), (T1, T2), ..., (T1, ..., T12)
- [ ] `all_vec()` works with Vec<Validation<T, E>>
- [ ] `map()` transforms success values
- [ ] `and()` combines two validations
- [ ] `and_then()` chains dependent validations
- [ ] `into_result()` converts to Result
- [ ] `from_result()` converts from Result
- [ ] All methods properly accumulate errors using Semigroup
- [ ] Comprehensive test coverage (>95%)
- [ ] Documentation with examples
- [ ] No compiler warnings

## Technical Details

### Implementation Approach

```rust
/// A validation that either succeeds with a value or fails with accumulated errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Validation<T, E> {
    /// Successful validation with a value
    Success(T),
    /// Failed validation with accumulated errors
    Failure(E),
}

impl<T, E> Validation<T, E> {
    /// Create a successful validation
    pub fn success(value: T) -> Self {
        Validation::Success(value)
    }

    /// Create a failed validation
    pub fn failure(error: E) -> Self {
        Validation::Failure(error)
    }

    /// Create from Result
    pub fn from_result(result: Result<T, E>) -> Self {
        match result {
            Ok(value) => Validation::Success(value),
            Err(error) => Validation::Failure(error),
        }
    }

    /// Convert to Result
    pub fn into_result(self) -> Result<T, E> {
        match self {
            Validation::Success(value) => Ok(value),
            Validation::Failure(error) => Err(error),
        }
    }

    /// Check if successful
    pub fn is_success(&self) -> bool {
        matches!(self, Validation::Success(_))
    }

    /// Check if failed
    pub fn is_failure(&self) -> bool {
        matches!(self, Validation::Failure(_))
    }

    /// Transform success value
    pub fn map<U, F>(self, f: F) -> Validation<U, E>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Validation::Success(value) => Validation::Success(f(value)),
            Validation::Failure(error) => Validation::Failure(error),
        }
    }

    /// Transform error value
    pub fn map_err<E2, F>(self, f: F) -> Validation<T, E2>
    where
        F: FnOnce(E) -> E2,
    {
        match self {
            Validation::Success(value) => Validation::Success(value),
            Validation::Failure(error) => Validation::Failure(f(error)),
        }
    }
}

impl<T, E: Semigroup> Validation<T, E> {
    /// Combine two validations, accumulating errors
    pub fn and<U>(self, other: Validation<U, E>) -> Validation<(T, U), E> {
        match (self, other) {
            (Validation::Success(a), Validation::Success(b)) => {
                Validation::Success((a, b))
            }
            (Validation::Failure(e1), Validation::Failure(e2)) => {
                Validation::Failure(e1.combine(e2))
            }
            (Validation::Failure(e), _) => Validation::Failure(e),
            (_, Validation::Failure(e)) => Validation::Failure(e),
        }
    }

    /// Chain dependent validation
    pub fn and_then<U, F>(self, f: F) -> Validation<U, E>
    where
        F: FnOnce(T) -> Validation<U, E>,
    {
        match self {
            Validation::Success(value) => f(value),
            Validation::Failure(error) => Validation::Failure(error),
        }
    }

    /// Combine all validations in a tuple
    pub fn all<V: ValidateAll<E>>(validations: V) -> Validation<V::Output, E> {
        validations.validate_all()
    }

    /// Combine all validations in a Vec
    pub fn all_vec(validations: Vec<Validation<T, E>>) -> Validation<Vec<T>, E> {
        let mut successes = Vec::new();
        let mut failures = Vec::new();

        for validation in validations {
            match validation {
                Validation::Success(value) => successes.push(value),
                Validation::Failure(error) => failures.push(error),
            }
        }

        if failures.is_empty() {
            Validation::Success(successes)
        } else {
            Validation::Failure(
                failures.into_iter().reduce(|acc, e| acc.combine(e)).unwrap()
            )
        }
    }
}
```

### Architecture Changes

- New module: `src/validation.rs`
- New trait: `ValidateAll` for tuple validation
- Export from `src/lib.rs`
- Re-export in `prelude`

### Data Structures

```rust
pub enum Validation<T, E> {
    Success(T),
    Failure(E),
}

// Helper trait for tuple validation
pub trait ValidateAll<E: Semigroup> {
    type Output;
    fn validate_all(self) -> Validation<Self::Output, E>;
}
```

### APIs and Interfaces

See Implementation Approach above.

## Dependencies

- **Prerequisites**: Spec 001 (Semigroup trait must be implemented first)
- **Affected Components**: None (new module)
- **External Dependencies**: None

## Testing Strategy

### Unit Tests

```rust
#[test]
fn test_success_and_success() {
    let v1 = Validation::<_, Vec<&str>>::success(1);
    let v2 = Validation::<_, Vec<&str>>::success(2);
    assert_eq!(v1.and(v2), Validation::Success((1, 2)));
}

#[test]
fn test_failure_and_failure_accumulates() {
    let v1 = Validation::<i32, _>::failure(vec!["error1"]);
    let v2 = Validation::<i32, _>::failure(vec!["error2"]);
    assert_eq!(v1.and(v2), Validation::Failure(vec!["error1", "error2"]));
}

#[test]
fn test_all_with_mixed_results() {
    let v1 = Validation::<_, Vec<&str>>::success(1);
    let v2 = Validation::failure(vec!["error1"]);
    let v3 = Validation::failure(vec!["error2"]);

    let result = Validation::all((v1, v2, v3));
    assert_eq!(result, Validation::Failure(vec!["error1", "error2"]));
}

#[test]
fn test_all_success() {
    let result = Validation::all((
        Validation::<_, Vec<&str>>::success(1),
        Validation::<_, Vec<&str>>::success(2),
        Validation::<_, Vec<&str>>::success(3),
    ));
    assert_eq!(result, Validation::Success((1, 2, 3)));
}

#[test]
fn test_map_on_success() {
    let v = Validation::<_, Vec<&str>>::success(5);
    assert_eq!(v.map(|x| x * 2), Validation::Success(10));
}

#[test]
fn test_map_on_failure() {
    let v = Validation::<i32, _>::failure(vec!["error"]);
    assert_eq!(v.map(|x| x * 2), Validation::Failure(vec!["error"]));
}

#[test]
fn test_from_result() {
    assert_eq!(
        Validation::from_result(Ok::<_, Vec<&str>>(42)),
        Validation::Success(42)
    );
    assert_eq!(
        Validation::from_result(Err::<i32, _>(vec!["error"])),
        Validation::Failure(vec!["error"])
    );
}

#[test]
fn test_into_result() {
    assert_eq!(
        Validation::<_, Vec<&str>>::success(42).into_result(),
        Ok(42)
    );
    assert_eq!(
        Validation::<i32, _>::failure(vec!["error"]).into_result(),
        Err(vec!["error"])
    );
}
```

### Integration Tests

```rust
// Test with real-world validation scenario
#[test]
fn test_form_validation() {
    #[derive(Debug, PartialEq)]
    enum ValidationError {
        InvalidEmail,
        PasswordTooShort,
        AgeTooYoung,
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

    // All invalid - should accumulate all 3 errors
    let result = Validation::all((
        validate_email("invalid"),
        validate_password("short"),
        validate_age(15),
    ));

    assert_eq!(
        result,
        Validation::Failure(vec![
            ValidationError::InvalidEmail,
            ValidationError::PasswordTooShort,
            ValidationError::AgeTooYoung,
        ])
    );
}
```

## Documentation Requirements

### Code Documentation

- Comprehensive rustdoc for Validation type
- Examples for each method
- Explain difference from Result
- Show real-world validation scenarios

### User Documentation

- Add "Validation" section to README with examples
- Create guide in docs/guide/02-validation.md
- Show comparison with Result (when to use which)

### Architecture Updates

- Document Validation type in DESIGN.md
- Explain ValidateAll trait and tuple magic

## Implementation Notes

### ValidateAll Trait

Use a macro to implement for tuples:

```rust
pub trait ValidateAll<E: Semigroup> {
    type Output;
    fn validate_all(self) -> Validation<Self::Output, E>;
}

macro_rules! impl_validate_all {
    ($($T:ident),+) => {
        impl<E: Semigroup, $($T),+> ValidateAll<E> for ($(Validation<$T, E>),+) {
            type Output = ($($T),+);

            fn validate_all(self) -> Validation<Self::Output, E> {
                #[allow(non_snake_case)]
                let ($($T),+) = self;

                // Start with first validation
                // Chain with and() for the rest
                // This automatically accumulates errors
                // (implementation details)
            }
        }
    };
}

impl_validate_all!(T1);
impl_validate_all!(T1, T2);
// ... up to T1, T2, ..., T12
```

### Performance

- Validation is typically small (enum with 2 variants)
- No heap allocation for the type itself
- Errors are accumulated efficiently via Semigroup

### Edge Cases

- Empty Vec in all_vec() → Success(vec![])
- Single element tuple → works correctly
- 12+ element tuples → use nesting or all_vec()

## Migration and Compatibility

No migration needed - this is a new feature.

## Open Questions

1. Should Validation implement Try trait for `?` operator?
   - Decision: Defer to separate spec (Try trait integration)

2. Should we provide async versions?
   - Decision: No, Validation is for pure validation, not effects

3. Should we have a `recover` or `or_else` method?
   - Decision: Add if users request it, not needed for MVP
