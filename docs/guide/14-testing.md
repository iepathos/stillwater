# Testing with Stillwater

## The Problem

Testing code that uses validation and effects requires:

- Setting up mock environments repeatedly
- Writing verbose assertions for `Validation` types
- Creating test data and checking results manually
- Property-based testing for comprehensive coverage

Without proper utilities, test code becomes verbose and repetitive, making tests harder to write and maintain.

## The Solution: Testing Utilities

Stillwater provides ergonomic testing utilities that make writing tests concise and expressive:

```rust
use stillwater::prelude::*;

#[test]
fn test_user_validation() {
    let result = validate_user("user@example.com", 25);
    assert_success!(result);
}
```

## MockEnv Builder

The `MockEnv` builder creates test environments by composing dependencies:

```rust
use stillwater::testing::MockEnv;

struct Database {
    users: Vec<User>,
}

struct Config {
    min_age: i32,
}

#[test]
fn test_with_mock_env() {
    // Build a mock environment with multiple dependencies
    let env = MockEnv::new()
        .with(|| Config { min_age: 18 })
        .with(|| Database { users: vec![] })
        .build();

    let ((_, config), db) = env;
    assert_eq!(config.min_age, 18);
    assert!(db.users.is_empty());
}
```

### Building Complex Environments

For more complex setups:

```rust
#[test]
fn test_complex_env() {
    let env = MockEnv::new()
        .with(|| Config { min_age: 18 })
        .with(|| Database::with_test_data())
        .with(|| "auth_token_123")
        .build();

    let (((_, config), db), token) = env;

    // Use your mocked environment
    let result = process_request(&config, &db, token);
    assert_success!(result);
}
```

## Assertion Macros

Stillwater provides three assertion macros for testing `Validation` types:

### `assert_success!`

Assert that a validation succeeds:

```rust
#[test]
fn test_valid_email() {
    let result = validate_email("user@example.com");
    assert_success!(result);
}
```

This will panic if the validation is a `Failure`, showing the errors.

### `assert_failure!`

Assert that a validation fails:

```rust
#[test]
fn test_invalid_email() {
    let result = validate_email("invalid");
    assert_failure!(result);
}
```

This will panic if the validation is a `Success`.

### `assert_validation_errors!`

Assert that a validation fails with specific errors:

```rust
#[test]
fn test_specific_errors() {
    let result = validate_email("invalid");
    assert_validation_errors!(
        result,
        vec!["Email must contain @".to_string()]
    );
}
```

This checks both that the validation failed AND that the errors match exactly.

## Testing Patterns

### Testing Error Accumulation

Validation accumulates all errors:

```rust
#[test]
fn test_accumulates_all_errors() {
    let result = validate_user("invalid", 15);
    assert_failure!(result);

    match result {
        Validation::Failure(errors) => {
            assert_eq!(errors.len(), 2);
            assert!(errors.contains(&"Invalid email".to_string()));
            assert!(errors.contains(&"Must be 18+".to_string()));
        }
        _ => panic!("Expected failure"),
    }
}
```

### Testing Validation Composition

Test how validations combine:

```rust
#[test]
fn test_validation_and() {
    let v1 = validate_email("user@example.com");
    let v2 = validate_age(25);

    let result = v1.and(v2);
    assert_success!(result);

    match result {
        Validation::Success((email, age)) => {
            assert_eq!(email, "user@example.com");
            assert_eq!(age, 25);
        }
        _ => panic!("Expected success"),
    }
}
```

### Testing with Effects

Test effects using mock environments:

```rust
#[test]
fn test_effect_composition() {
    let env = MockEnv::new()
        .with(|| Database::with_test_data())
        .build();

    let (_, db) = env;

    let effect = Effect::from(|_: &Database| {
        Ok::<i32, String>(42)
    });

    let result = effect.run(&db);
    assert_eq!(result, Ok(42));
}
```

## Property-Based Testing

Enable property-based testing with the `proptest` feature:

```toml
[dev-dependencies]
proptest = "1.0"
```

Stillwater provides `Arbitrary` instances for `Validation`:

```rust
#[cfg(feature = "proptest")]
mod proptest_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_validation_map_preserves_success(value: i32) {
            let val = Validation::<_, Vec<String>>::success(value);
            let mapped = val.map(|x| x * 2);
            assert_success!(mapped);
        }

        #[test]
        fn test_email_validation(
            local in "[a-z]{1,10}",
            domain in "[a-z]{1,10}",
            tld in "[a-z]{2,5}"
        ) {
            let email = format!("{}@{}.{}", local, domain, tld);
            let result = validate_email(&email);
            assert_success!(result);
        }
    }
}
```

### Property Testing Patterns

Test invariants that should always hold:

```rust
proptest! {
    #[test]
    fn test_success_always_is_success(value: i32) {
        let val = Validation::<_, Vec<String>>::success(value);
        assert!(val.is_success());
    }

    #[test]
    fn test_failure_always_is_failure(errors: Vec<String>) {
        prop_assume!(!errors.is_empty());
        let val = Validation::<i32, _>::failure(errors);
        assert!(val.is_failure());
    }

    #[test]
    fn test_map_preserves_failure(errors: Vec<String>) {
        prop_assume!(!errors.is_empty());
        let val = Validation::<i32, _>::failure(errors);
        let mapped = val.map(|x| x * 2);
        assert_failure!(mapped);
    }
}
```

## Testing Best Practices

### 1. Use Descriptive Test Names

```rust
#[test]
fn test_email_validation_rejects_missing_at_symbol() {
    let result = validate_email("invalid.com");
    assert_failure!(result);
}
```

### 2. Test Both Success and Failure Cases

```rust
#[test]
fn test_age_validation_accepts_adults() {
    let result = validate_age(18);
    assert_success!(result);
}

#[test]
fn test_age_validation_rejects_minors() {
    let result = validate_age(17);
    assert_failure!(result);
}
```

### 3. Test Error Accumulation Explicitly

```rust
#[test]
fn test_validates_all_fields_at_once() {
    let result = validate_user("invalid", 15);

    match result {
        Validation::Failure(errors) => {
            assert!(errors.len() > 1, "Should accumulate multiple errors");
        }
        _ => panic!("Expected validation to fail"),
    }
}
```

### 4. Use MockEnv for Complex Dependencies

```rust
#[test]
fn test_with_realistic_environment() {
    let env = MockEnv::new()
        .with(|| Config::from_env())
        .with(|| Database::with_fixtures())
        .with(|| Cache::empty())
        .build();

    // Test your code with the full environment
}
```

## Integration with Test Frameworks

### Using with Standard Test Framework

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use stillwater::prelude::*;

    #[test]
    fn test_my_function() {
        let result = my_validation_function();
        assert_success!(result);
    }
}
```

### Using with Tokio Test

For async tests:

```rust
#[cfg(test)]
mod async_tests {
    use super::*;
    use stillwater::prelude::*;

    #[tokio::test]
    async fn test_async_validation() {
        let result = async_validate().await;
        assert_success!(result);
    }
}
```

## Complete Example

Here's a complete example showing all testing utilities:

```rust
use stillwater::prelude::*;

#[derive(Debug, Clone, PartialEq)]
struct User {
    email: String,
    age: i32,
}

fn validate_email(email: &str) -> Validation<String, Vec<String>> {
    if email.contains('@') {
        Validation::success(email.to_string())
    } else {
        Validation::failure(vec!["Invalid email".to_string()])
    }
}

fn validate_age(age: i32) -> Validation<i32, Vec<String>> {
    if age >= 18 {
        Validation::success(age)
    } else {
        Validation::failure(vec!["Must be 18+".to_string()])
    }
}

fn validate_user(email: &str, age: i32) -> Validation<User, Vec<String>> {
    Validation::all((validate_email(email), validate_age(age)))
        .map(|(email, age)| User { email, age })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_user() {
        let result = validate_user("user@example.com", 25);
        assert_success!(result);
    }

    #[test]
    fn test_invalid_email() {
        let result = validate_user("invalid", 25);
        assert_failure!(result);
    }

    #[test]
    fn test_underage() {
        let result = validate_user("user@example.com", 15);
        assert_failure!(result);
    }

    #[test]
    fn test_multiple_errors() {
        let result = validate_user("invalid", 15);

        match result {
            Validation::Failure(errors) => {
                assert_eq!(errors.len(), 2);
            }
            _ => panic!("Expected failure"),
        }
    }
}
```

## Next Steps

- See [examples/validation.rs](../../examples/validation.rs) for more examples
- Check [tests/testing_utilities.rs](../../tests/testing_utilities.rs) for comprehensive test patterns
- Read [Validation Guide](02-validation.md) for more on validation
- Read [Effects Guide](03-effects.md) for testing effects
