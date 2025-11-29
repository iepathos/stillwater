---
number: 1
title: Enhanced Validation Assertions
category: testing
priority: medium
status: draft
dependencies: []
created: 2025-11-28
---

# Specification 001: Enhanced Validation Assertions

**Category**: testing
**Priority**: medium
**Status**: draft
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

3. **Error Formatting Helpers**
   - `format_validation_errors<E: Debug>(errors: &E) -> String` - generic formatter
   - Works with `Vec<E>`, single errors, or any `Debug` type
   - Provides numbered list output for collections
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
- [ ] `format_validation_errors` works with `Vec<E>`
- [ ] `format_validation_errors` works with single error values
- [ ] All macros work with custom error types
- [ ] Error messages are readable and actionable

## Technical Details

### Implementation Approach

```rust
// src/testing.rs - Enhanced macros

/// Assert that validation result is successful
#[macro_export]
macro_rules! assert_success {
    ($result:expr) => {
        match &$result {
            $crate::Validation::Success(_) => {}
            $crate::Validation::Failure(errors) => {
                panic!(
                    "Expected successful validation, got failure:\n{}",
                    $crate::testing::format_validation_errors(errors)
                );
            }
        }
    };
}

/// Assert that validation result failed
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
    ($result:expr, $count:expr) => {
        match &$result {
            $crate::Validation::Failure(errors) => {
                let actual_count = $crate::testing::error_count(errors);
                if actual_count != $count {
                    panic!(
                        "Expected {} validation error(s), got {}:\n{}",
                        $count,
                        actual_count,
                        $crate::testing::format_validation_errors(errors)
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
// src/testing.rs - Helper functions

/// Format validation errors for display in test failures.
///
/// Works with any error type that implements Debug. For collections,
/// provides a numbered list. For single values, shows the debug output.
pub fn format_validation_errors<E: std::fmt::Debug>(errors: &E) -> String {
    format!("{:#?}", errors)
}

/// Specialized formatting for Vec<E>
pub fn format_error_vec<E: std::fmt::Debug>(errors: &[E]) -> String {
    if errors.is_empty() {
        return "(no errors)".to_string();
    }

    errors
        .iter()
        .enumerate()
        .map(|(i, e)| format!("  {}. {:?}", i + 1, e))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Get error count from various error types
pub fn error_count<E>(errors: &E) -> usize
where
    E: ErrorCount,
{
    errors.error_count()
}

/// Trait for counting errors
pub trait ErrorCount {
    fn error_count(&self) -> usize;
}

impl<E> ErrorCount for Vec<E> {
    fn error_count(&self) -> usize {
        self.len()
    }
}

impl<E> ErrorCount for &[E] {
    fn error_count(&self) -> usize {
        self.len()
    }
}

impl ErrorCount for String {
    fn error_count(&self) -> usize {
        1
    }
}

impl ErrorCount for &str {
    fn error_count(&self) -> usize {
        1
    }
}

// Implement for common error types
impl<E> ErrorCount for Box<E>
where
    E: ErrorCount,
{
    fn error_count(&self) -> usize {
        (**self).error_count()
    }
}
```

### Architecture Changes

- Enhance existing `src/testing.rs` module
- Add `ErrorCount` trait for generic error counting
- Improve macro error messages
- Add helper functions for error formatting

### Data Structures

- `ErrorCount` trait - Generic way to count errors in different types
- No new data structures needed

### APIs and Interfaces

```rust
// Enhanced macros
assert_success!(validation)
assert_failure!(validation)
assert_failure!(validation, error_count)

// Helper functions (public API)
testing::format_validation_errors<E: Debug>(&E) -> String
testing::format_error_vec<E: Debug>(&[E]) -> String
testing::error_count<E: ErrorCount>(&E) -> usize

// Trait
trait ErrorCount {
    fn error_count(&self) -> usize;
}
```

## Dependencies

- **Prerequisites**: None (enhancement of existing functionality)
- **Affected Components**: `src/testing.rs`
- **External Dependencies**: None (only uses std)

## Testing Strategy

- **Unit Tests**:
  - Macro behavior with success/failure cases
  - Error count assertions
  - Error formatting output
  - Edge cases (empty errors, single errors, many errors)

- **Doc Tests**:
  - Examples in macro documentation
  - Examples in helper function documentation

- **Integration Tests**:
  - Real-world usage with custom error types
  - Complex validation scenarios

## Documentation Requirements

- **Code Documentation**:
  - Comprehensive rustdoc on all public functions and macros
  - Examples showing before/after for enhanced error messages
  - Usage examples with different error types

- **User Documentation**:
  - README section on testing utilities
  - Migration guide for existing macro users (if breaking changes)

- **Architecture Updates**:
  - Document `ErrorCount` trait and when to implement it

## Implementation Notes

- Preserve backward compatibility with existing macro usage
- Error formatting should degrade gracefully for large error lists
- Consider using `#[track_caller]` on helper functions for better panic locations
- The `assert_validation_errors!` macro can remain as-is for exact error matching

## Migration and Compatibility

**Backward Compatibility**: Enhanced macros maintain the same basic API, so existing code continues to work. The improvements are in the panic message quality.

**No Breaking Changes**:
- Existing `assert_success!(val)` calls work identically
- Existing `assert_failure!(val)` calls work identically
- New variant `assert_failure!(val, count)` is additive

**Optional Migration**: Users can optionally use new helper functions for custom test utilities.

## Files to Create/Modify

```
src/testing.rs (modify - enhance macros and add helpers)
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
use stillwater::testing::{format_validation_errors, format_error_vec};

#[test]
fn custom_assertion() {
    let result = validate_user(input);

    if let Validation::Failure(errors) = result {
        println!("Validation errors:\n{}", format_error_vec(&errors));
        assert!(errors.len() < 5, "Too many errors");
    }
}
```

### With Custom Error Types

```rust
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

#[test]
fn test_with_custom_errors() {
    let result: Validation<User, AppError> = validate_and_save(input);

    assert_failure!(result, 2);  // Works with custom error type
}
```
