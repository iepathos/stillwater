---
number: 018
title: Testing Utilities and Helpers
category: testing
priority: medium
status: draft
dependencies: []
created: 2025-11-22
---

# Specification 018: Testing Utilities and Helpers

**Category**: testing
**Priority**: medium
**Status**: draft
**Dependencies**: None

## Context

Testing code that uses Stillwater should be ergonomic. Users need utilities for:

- Creating mock environments easily
- Asserting on Validation success/failure
- Testing Effects without real I/O
- Property-based testing helpers

Currently, users must manually create test environments and write verbose assertions.

## Objective

Provide testing utilities that make writing tests with Stillwater types and effects ergonomic and concise.

## Requirements

### Functional Requirements

- `MockEnv` builder for creating test environments
- Assertion helpers: `assert_success!`, `assert_failure!`, `assert_validation_errors!`
- `TestEffect` wrapper for deterministic testing
- Property-based testing support (with proptest)
- Example test patterns

### Acceptance Criteria

- [ ] `MockEnv` builder implemented
- [ ] Assertion macros for Validation
- [ ] Test helpers for Effect
- [ ] Proptest Arbitrary instances for Validation
- [ ] Comprehensive examples in `tests/` directory
- [ ] Documentation guide: `docs/guide/14-testing.md`
- [ ] All test helpers tested

## Technical Details

### MockEnv Builder

```rust
/// Builder for creating test environments.
///
/// # Example
///
/// ```rust
/// let env = MockEnv::new()
///     .with_database(mock_db)
///     .with_config(test_config)
///     .build();
///
/// let result = my_effect().run(&env);
/// ```
pub struct MockEnv<Env> {
    env: Env,
}

impl MockEnv<()> {
    pub fn new() -> Self {
        Self { env: () }
    }
}

impl<Env> MockEnv<Env> {
    pub fn with<F, T>(self, f: F) -> MockEnv<(Env, T)>
    where
        F: FnOnce() -> T,
    {
        MockEnv {
            env: (self.env, f()),
        }
    }

    pub fn build(self) -> Env {
        self.env
    }
}
```

### Assertion Macros

```rust
/// Assert a validation succeeds.
///
/// # Example
///
/// ```rust
/// let val = validate_email("test@example.com");
/// assert_success!(val);
/// ```
#[macro_export]
macro_rules! assert_success {
    ($validation:expr) => {
        match $validation {
            Validation::Success(_) => {},
            Validation::Failure(e) => {
                panic!("Expected Success, got Failure: {:?}", e);
            }
        }
    };
}

/// Assert a validation fails.
#[macro_export]
macro_rules! assert_failure {
    ($validation:expr) => {
        match $validation {
            Validation::Failure(_) => {},
            Validation::Success(v) => {
                panic!("Expected Failure, got Success: {:?}", v);
            }
        }
    };
}

/// Assert validation fails with specific errors.
#[macro_export]
macro_rules! assert_validation_errors {
    ($validation:expr, $expected:expr) => {
        match $validation {
            Validation::Failure(errors) => {
                assert_eq!(errors, $expected);
            }
            Validation::Success(v) => {
                panic!("Expected Failure, got Success: {:?}", v);
            }
        }
    };
}
```

### Property-Based Testing Support

```rust
#[cfg(feature = "proptest")]
use proptest::prelude::*;

#[cfg(feature = "proptest")]
impl<T: Arbitrary, E: Arbitrary> Arbitrary for Validation<T, E> {
    type Parameters = (T::Parameters, E::Parameters);
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
        let (t_params, e_params) = args;
        prop_oneof![
            any_with::<T>(t_params).prop_map(Validation::success),
            any_with::<E>(e_params).prop_map(Validation::failure),
        ]
        .boxed()
    }
}
```

## Documentation Requirements

- Comprehensive testing guide: `docs/guide/14-testing.md`
- Example tests in `tests/` showing patterns
- Rustdoc examples for all helpers

## Success Metrics

- Reduces test boilerplate significantly
- Positive user feedback on ergonomics
- Comprehensive test examples
