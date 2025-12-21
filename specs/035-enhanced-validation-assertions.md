---
number: 35
title: Enhanced Validation Assertions
category: testing
priority: medium
status: ready
dependencies: []
created: 2025-11-28
---

# Specification 035: Enhanced Validation Assertions

**Category**: testing
**Priority**: medium
**Status**: ready
**Dependencies**: None

## Context

Stillwater already provides basic assertion macros (`assert_success!`, `assert_failure!`, `assert_validation_errors!`) for testing validation results. However, these macros can be enhanced to provide better error messages, more ergonomic APIs, and additional testing utilities that benefit any code using the `Validation` type.

The current macros work but provide minimal context when failures occur. Enhanced versions should:
- Show detailed error information when assertions fail
- Support optional error count assertions
- Provide formatted error output for debugging
- Work seamlessly with any error type that implements `Debug`

## Objective

Enhance the existing validation assertion macros with:
1. Better error formatting and display
2. Error count assertions
3. Generic helper functions for formatting validation errors
4. Consistent, ergonomic API that works with any `Validation<T, E>`

## Requirements

### Functional Requirements

1. **Enhanced `assert_success!` Macro**
   - Panics with formatted error list if validation failed
   - Shows error count in panic message
   - Formats errors using `Debug` trait
   - Works with any error type

2. **Enhanced `assert_failure!` Macro**
   - Panics if validation succeeded
   - Optional error count parameter: `assert_failure!(result, 3)` asserts exactly 3 errors
   - Shows formatted errors in panic message when count mismatches
   - Works with both single errors and error collections

3. **Error Formatting via `FormatErrors` Trait**
   - Unified trait-based approach for formatting errors
   - `FormatErrors::format_errors(&self) -> String` - trait method
   - Works with `Vec<E>`, `NonEmptyVec<E>`, single errors, or any `Debug` type
   - Provides numbered list output for collections (`Vec`, `NonEmptyVec`)
   - Falls back to pretty-print `Debug` for other types
   - Clear, readable output suitable for test failures

4. **Consistent API**
   - All macros accept any `Validation<T, E>` where bounds are appropriate
   - Error messages are actionable and include context
   - Macros work in both sync and async test contexts

### Non-Functional Requirements

- Macros should provide clear, actionable error messages
- Formatting helpers should handle edge cases (empty errors, large error lists)
- No performance overhead in release builds (macros only used in tests)
- Error messages should be readable even with complex error types

## Acceptance Criteria

- [ ] `assert_success!(result)` shows formatted errors on failure
- [ ] `assert_success!(result)` shows error count in panic message
- [ ] `assert_failure!(result)` panics on success with value shown
- [ ] `assert_failure!(result, n)` checks exact error count
- [ ] `assert_failure!(result, n)` shows formatted errors on count mismatch
- [ ] `FormatErrors` trait implemented for `Vec<E>`
- [ ] `FormatErrors` trait implemented for `NonEmptyVec<E>`
- [ ] `FormatErrors` fallback works for single error values via blanket impl
- [ ] `ErrorCount` trait implemented for `Vec<E>`, `NonEmptyVec<E>`, `String`, `&str`
- [ ] All macros work with custom error types implementing required traits
- [ ] Error messages are readable and actionable
- [ ] Helper functions use `#[track_caller]` for accurate panic locations

## Technical Details

### Implementation Approach

```rust
// src/testing.rs - Enhanced macros

/// Assert that validation result is successful.
///
/// Borrows the result, allowing it to be used after the assertion.
/// Uses `FormatErrors` trait for formatted error output.
#[macro_export]
macro_rules! assert_success {
    ($result:expr) => {
        match &$result {
            $crate::Validation::Success(_) => {}
            $crate::Validation::Failure(errors) => {
                use $crate::testing::FormatErrors;
                panic!(
                    "Expected successful validation, got failure:\n{}",
                    errors.format_errors()
                );
            }
        }
    };
}

/// Assert that validation result failed.
///
/// Borrows the result, allowing it to be used after the assertion.
/// The optional count parameter requires `E: ErrorCount`.
#[macro_export]
macro_rules! assert_failure {
    ($result:expr) => {
        match &$result {
            $crate::Validation::Failure(_) => {}
            $crate::Validation::Success(v) => {
                panic!(
                    "Expected validation failure, got success: {:?}",
                    v
                );
            }
        }
    };
    // Note: This variant requires E: ErrorCount + FormatErrors
    ($result:expr, $count:expr) => {
        match &$result {
            $crate::Validation::Failure(errors) => {
                use $crate::testing::{ErrorCount, FormatErrors};
                let actual_count = errors.error_count();
                if actual_count != $count {
                    panic!(
                        "Expected {} validation error(s), got {}:\n{}",
                        $count,
                        actual_count,
                        errors.format_errors()
                    );
                }
            }
            $crate::Validation::Success(v) => {
                panic!(
                    "Expected validation failure with {} error(s), got success: {:?}",
                    $count,
                    v
                );
            }
        }
    };
}
```

### Error Formatting

```rust
// src/testing.rs - Traits and helper functions

use crate::NonEmptyVec;
use std::fmt::Debug;

// ============================================================================
// FormatErrors Trait
// ============================================================================

/// Trait for formatting errors in a human-readable way.
///
/// Provides numbered list output for collections and pretty-printed
/// Debug output for single values.
pub trait FormatErrors {
    /// Format errors for display in test failure messages.
    fn format_errors(&self) -> String;
}

impl<E: Debug> FormatErrors for Vec<E> {
    fn format_errors(&self) -> String {
        format_numbered_list(self.iter())
    }
}

impl<E: Debug> FormatErrors for NonEmptyVec<E> {
    fn format_errors(&self) -> String {
        format_numbered_list(self.iter())
    }
}

impl<E: Debug> FormatErrors for [E] {
    fn format_errors(&self) -> String {
        format_numbered_list(self.iter())
    }
}

impl FormatErrors for String {
    fn format_errors(&self) -> String {
        format!("{:#?}", self)
    }
}

impl FormatErrors for str {
    fn format_errors(&self) -> String {
        format!("{:#?}", self)
    }
}

/// Format an iterator of errors as a numbered list.
///
/// # Example Output
/// ```text
///   1. InvalidEmail("bad@")
///   2. AgeTooYoung(15)
/// ```
#[track_caller]
pub fn format_numbered_list<'a, E: Debug + 'a>(
    errors: impl Iterator<Item = &'a E>,
) -> String {
    let formatted: Vec<_> = errors
        .enumerate()
        .map(|(i, e)| format!("  {}. {:?}", i + 1, e))
        .collect();

    if formatted.is_empty() {
        "(no errors)".to_string()
    } else {
        formatted.join("\n")
    }
}

/// Format any Debug type with pretty-printing (fallback).
#[track_caller]
pub fn format_validation_errors<E: Debug>(errors: &E) -> String {
    format!("{:#?}", errors)
}

// ============================================================================
// ErrorCount Trait
// ============================================================================

/// Trait for counting errors in various error types.
///
/// The `assert_failure!(result, count)` macro variant requires this trait.
/// Implement this for custom error types that wrap multiple errors.
pub trait ErrorCount {
    /// Return the number of individual errors.
    fn error_count(&self) -> usize;
}

impl<E> ErrorCount for Vec<E> {
    fn error_count(&self) -> usize {
        self.len()
    }
}

impl<E> ErrorCount for NonEmptyVec<E> {
    fn error_count(&self) -> usize {
        self.len()
    }
}

impl<E> ErrorCount for [E] {
    fn error_count(&self) -> usize {
        self.len()
    }
}

impl ErrorCount for String {
    fn error_count(&self) -> usize {
        1
    }
}

impl ErrorCount for str {
    fn error_count(&self) -> usize {
        1
    }
}

impl<E: ErrorCount + ?Sized> ErrorCount for Box<E> {
    fn error_count(&self) -> usize {
        (**self).error_count()
    }
}

impl<E: ErrorCount + ?Sized> ErrorCount for &E {
    fn error_count(&self) -> usize {
        (**self).error_count()
    }
}
```

### Architecture Changes

- Enhance existing `src/testing.rs` module
- Add `FormatErrors` trait for unified error formatting
- Add `ErrorCount` trait for generic error counting
- Improve macro error messages with trait-based formatting
- Add `#[track_caller]` on helper functions for better panic locations
- Support `NonEmptyVec<E>` alongside `Vec<E>` for error collections

### Data Structures

- `FormatErrors` trait - Unified approach for formatting errors as human-readable strings
- `ErrorCount` trait - Generic way to count errors in different types
- No new data structures needed

### APIs and Interfaces

```rust
// Enhanced macros
assert_success!(validation)              // E: FormatErrors
assert_failure!(validation)              // No trait bounds
assert_failure!(validation, error_count) // E: ErrorCount + FormatErrors

// Traits (public API)
pub trait FormatErrors {
    fn format_errors(&self) -> String;
}

pub trait ErrorCount {
    fn error_count(&self) -> usize;
}

// Helper functions (public API)
testing::format_numbered_list<E: Debug>(impl Iterator<Item = &E>) -> String
testing::format_validation_errors<E: Debug>(&E) -> String  // fallback

// Built-in implementations
// FormatErrors: Vec<E>, NonEmptyVec<E>, [E], String, str
// ErrorCount: Vec<E>, NonEmptyVec<E>, [E], String, str, Box<E>, &E
```

## Dependencies

- **Prerequisites**: None (enhancement of existing functionality)
- **Affected Components**: `src/testing.rs`
- **External Dependencies**: None (only uses std)

## Testing Strategy

- **Unit Tests**:
  - Macro behavior with success/failure cases
  - Error count assertions with `Vec<E>` and `NonEmptyVec<E>`
  - `FormatErrors` trait output for all built-in implementations
  - `ErrorCount` trait output for all built-in implementations
  - Edge cases (empty Vec, single errors, many errors)

- **Doc Tests**:
  - Examples in macro documentation
  - Examples in trait documentation (`FormatErrors`, `ErrorCount`)
  - Examples in helper function documentation

- **Integration Tests**:
  - Real-world usage with custom error types implementing both traits
  - `NonEmptyVec<E>` error scenarios
  - Complex validation scenarios with nested error types

## Documentation Requirements

- **Code Documentation**:
  - Comprehensive rustdoc on all public functions, macros, and traits
  - Examples showing before/after for enhanced error messages
  - Usage examples with `Vec<E>`, `NonEmptyVec<E>`, and custom error types
  - Document trait bounds required for each macro variant

- **User Documentation**:
  - README section on testing utilities
  - No migration guide needed (no breaking changes)

- **Architecture Updates**:
  - Document `FormatErrors` trait and when to implement it
  - Document `ErrorCount` trait and when to implement it
  - Note that `assert_failure!(val, count)` requires both traits on error type

## Implementation Notes

- Preserve backward compatibility with existing macro usage
- Error formatting should degrade gracefully for large error lists
- Use `#[track_caller]` on helper functions for better panic locations
- The `assert_validation_errors!` macro can remain as-is for exact error matching
- Macros borrow the result (`match &$result`) to preserve usability after assertion
- The `assert_failure!(val, count)` variant adds trait bounds (`E: ErrorCount + FormatErrors`)
- Built-in trait implementations cover common types; users implement for custom error types

## Migration and Compatibility

**Backward Compatibility**: Enhanced macros maintain the same basic API, so existing code continues to work. The improvements are in the panic message quality.

**No Breaking Changes**:
- Existing `assert_success!(val)` calls work identically (now uses `FormatErrors` trait)
- Existing `assert_failure!(val)` calls work identically (no new trait bounds)
- New variant `assert_failure!(val, count)` is additive

**Trait Bounds Note**:
- `assert_success!` now requires `E: FormatErrors` (satisfied by all common error types)
- `assert_failure!(val)` has no trait bounds (unchanged)
- `assert_failure!(val, count)` requires `E: ErrorCount + FormatErrors`
- Built-in impls for `Vec<E>`, `NonEmptyVec<E>`, `String`, `str` satisfy these bounds

**Optional Migration**: Users can optionally:
- Implement `FormatErrors` and `ErrorCount` for custom error types
- Use `format_numbered_list` helper for custom formatting

## Files to Create/Modify

```
src/testing.rs (modify - enhance macros, add FormatErrors and ErrorCount traits)
src/lib.rs (modify - re-export FormatErrors and ErrorCount from testing module)
tests/testing_utilities.rs (modify - add tests for NonEmptyVec and new traits)
```

## Example Usage

### Before (Current Implementation)

```rust
use stillwater::{Validation, assert_success, assert_failure};

#[test]
fn test_validation() {
    let result = validate_user(input);
    assert_success!(result);

    let result = validate_bad_input(input);
    assert_failure!(result);
    // Panic: "Expected Success, got Failure: [Error1, Error2, Error3]"
    // Hard to read with complex errors
}
```

### After (Enhanced Implementation)

```rust
use stillwater::{Validation, assert_success, assert_failure};

#[test]
fn test_validation() {
    let result = validate_user(input);
    assert_success!(result);
    // Panic: "Expected successful validation, got failure:
    //   1. InvalidEmail("not-an-email")
    //   2. AgeTooYoung(15)
    //   3. MissingField("phone_number")"

    let result = validate_bad_input(input);
    assert_failure!(result, 2);  // Assert exactly 2 errors
    // Panic: "Expected 2 validation error(s), got 3:
    //   1. InvalidEmail("not-an-email")
    //   2. AgeTooYoung(15)
    //   3. MissingField("phone_number")"
}
```

### Using Helper Functions

```rust
use stillwater::testing::{format_numbered_list, FormatErrors};

#[test]
fn custom_assertion() {
    let result = validate_user(input);

    if let Validation::Failure(errors) = result {
        // Using the trait method
        println!("Validation errors:\n{}", errors.format_errors());

        // Or using the helper function directly
        println!("Validation errors:\n{}", format_numbered_list(errors.iter()));

        assert!(errors.len() < 5, "Too many errors");
    }
}
```

### With NonEmptyVec Errors

```rust
use stillwater::{NonEmptyVec, Validation, assert_success, assert_failure};

fn validate_strict(input: &str) -> Validation<String, NonEmptyVec<String>> {
    if input.is_empty() {
        Validation::failure(NonEmptyVec::singleton("Input cannot be empty".to_string()))
    } else {
        Validation::success(input.to_string())
    }
}

#[test]
fn test_with_nonempty_vec() {
    let result = validate_strict("");
    assert_failure!(result, 1);  // Works with NonEmptyVec

    let result = validate_strict("valid");
    assert_success!(result);
}
```

### With Custom Error Types

```rust
use stillwater::testing::{ErrorCount, FormatErrors, format_numbered_list};

#[derive(Debug)]
enum AppError {
    Database(String),
    Validation(Vec<ValidationError>),
    Network(String),
}

impl ErrorCount for AppError {
    fn error_count(&self) -> usize {
        match self {
            AppError::Validation(errors) => errors.len(),
            _ => 1,
        }
    }
}

impl FormatErrors for AppError {
    fn format_errors(&self) -> String {
        match self {
            AppError::Validation(errors) => format_numbered_list(errors.iter()),
            other => format!("{:#?}", other),
        }
    }
}

#[test]
fn test_with_custom_errors() {
    let result: Validation<User, AppError> = validate_and_save(input);

    // Both ErrorCount and FormatErrors are implemented
    assert_failure!(result, 2);  // Works with custom error type
}
```
