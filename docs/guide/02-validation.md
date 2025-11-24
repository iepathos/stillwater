# Validation with Error Accumulation

## The Problem

Standard `Result` types short-circuit on the first error:

```rust
fn validate_form(data: FormData) -> Result<ValidForm, Error> {
    let email = validate_email(data.email)?;     // ❌ Stops here if invalid
    let password = validate_password(data.pwd)?; // Never reached
    let age = validate_age(data.age)?;           // Never reached

    Ok(ValidForm { email, password, age })
}
```

If the email is invalid, the user doesn't learn about password or age errors. They have to submit the form multiple times, fixing one error at a time. Frustrating!

## The Solution: Validation

Stillwater's `Validation` type accumulates ALL errors:

```rust
use stillwater::prelude::*;

fn validate_form(data: FormData) -> Validation<ValidForm, Vec<Error>> {
    Validation::all((
        validate_email(data.email),
        validate_password(data.pwd),
        validate_age(data.age),
    ))
    .map(|(email, password, age)| ValidForm { email, password, age })
}
```

Now all three validations run, and the user sees all errors at once!

## Core API

### Creating Validations

```rust
use stillwater::Validation;

// Success
let v = Validation::success(42);

// Failure
let v = Validation::failure(vec!["error"]);

// From Result
let v = Validation::from_result(Ok(42));
let v = Validation::from_result(Err("error"));
```

### Pattern Matching

```rust
use stillwater::Validation;

match validation {
    Validation::Success(value) => println!("Valid: {}", value),
    Validation::Failure(errors) => println!("Errors: {:?}", errors),
}
```

### Checking Status

```rust
use stillwater::Validation;

let v = Validation::success(42);
assert!(v.is_success());
assert!(!v.is_failure());

let v = Validation::failure(vec!["error"]);
assert!(!v.is_success());
assert!(v.is_failure());
```

### Combining Validations

```rust
use stillwater::Validation;

// Combine with tuples (up to 12 items)
let result = Validation::all((
    validate_email(email),
    validate_password(password),
    validate_age(age),
));

// Combine Vec of same type
let result = Validation::all_vec(vec![
    validate_item(item1),
    validate_item(item2),
    validate_item(item3),
]);
```

### Transforming Validations

```rust
use stillwater::Validation;

// Transform success value
let v = Validation::success(21);
let doubled = v.map(|x| x * 2);
assert_eq!(doubled, Validation::success(42));

// Transform error value
let v = Validation::failure(vec!["oops"]);
let formatted = v.map_err(|e| format!("Error: {:?}", e));

// Chain dependent validation
let result = validate_email(email)
    .and_then(|email| check_email_available(email));
```

### Converting to Result

```rust
use stillwater::Validation;

let v = Validation::success(42);
let r: Result<i32, Vec<String>> = v.into_result();
assert_eq!(r, Ok(42));

let v = Validation::failure(vec!["error"]);
let r: Result<i32, Vec<String>> = v.into_result();
assert_eq!(r, Err(vec!["error"]));
```

## Error Accumulation with Semigroup

For `Validation::all()` to work, your error type must implement `Semigroup`:

```rust
pub trait Semigroup {
    fn combine(self, other: Self) -> Self;
}
```

Common implementations:
- `Vec<T>`: Concatenate vectors
- `String`: Concatenate strings
- `(A, B) where A: Semigroup, B: Semigroup`: Combine components

Example:
```rust
use stillwater::Semigroup;

impl Semigroup for Vec<ValidationError> {
    fn combine(mut self, mut other: Self) -> Self {
        self.extend(other);
        self
    }
}
```

See [Semigroup guide](01-semigroup.md) for details.

## Real-World Example

```rust
use stillwater::{Validation, Semigroup};

#[derive(Debug, PartialEq)]
enum ValidationError {
    InvalidEmail(String),
    PasswordTooShort { min: usize, actual: usize },
    AgeTooYoung { min: u8, actual: u8 },
}

fn validate_email(email: &str) -> Validation<String, Vec<ValidationError>> {
    if email.contains('@') && email.contains('.') {
        Validation::success(email.to_string())
    } else {
        Validation::failure(vec![
            ValidationError::InvalidEmail(email.to_string())
        ])
    }
}

fn validate_password(pwd: &str) -> Validation<String, Vec<ValidationError>> {
    const MIN_LEN: usize = 8;
    if pwd.len() >= MIN_LEN {
        Validation::success(pwd.to_string())
    } else {
        Validation::failure(vec![
            ValidationError::PasswordTooShort {
                min: MIN_LEN,
                actual: pwd.len(),
            }
        ])
    }
}

fn validate_age(age: u8) -> Validation<u8, Vec<ValidationError>> {
    const MIN_AGE: u8 = 18;
    if age >= MIN_AGE {
        Validation::success(age)
    } else {
        Validation::failure(vec![
            ValidationError::AgeTooYoung {
                min: MIN_AGE,
                actual: age,
            }
        ])
    }
}

#[derive(Debug)]
struct User {
    email: String,
    password: String,
    age: u8,
}

fn validate_registration(
    email: &str,
    password: &str,
    age: u8,
) -> Validation<User, Vec<ValidationError>> {
    Validation::all((
        validate_email(email),
        validate_password(password),
        validate_age(age),
    ))
    .map(|(email, password, age)| User { email, password, age })
}

// Usage
match validate_registration("invalid", "short", 15) {
    Validation::Success(user) => println!("✓ Registered: {:?}", user),
    Validation::Failure(errors) => {
        println!("✗ {} errors:", errors.len());
        for err in errors {
            println!("  - {:?}", err);
        }
    }
}

// Output:
// ✗ 3 errors:
//   - InvalidEmail("invalid")
//   - PasswordTooShort { min: 8, actual: 5 }
//   - AgeTooYoung { min: 18, actual: 15 }
```

## When to Use Validation

**Use Validation when**:
- Validating user input (forms, APIs)
- You want to report ALL errors at once
- Validations are independent (order doesn't matter)

**Use Result when**:
- Operations depend on previous results
- Short-circuit is desired (fail fast)
- Single error is sufficient

## Patterns

### Independent Field Validation

```rust
use stillwater::Validation;

// All fields validated independently
Validation::all((
    validate_email(data.email),
    validate_phone(data.phone),
    validate_address(data.address),
))
```

### Dependent Validation

```rust
use stillwater::Validation;

// First validate, then check dependencies
validate_email(email)
    .and_then(|email| {
        check_email_not_taken(email)
    })
```

### Mixed Validation

```rust
use stillwater::Validation;

// Combine independent and dependent
Validation::all((
    validate_email(email),
    validate_password(password),
))
.and_then(|(email, password)| {
    // Now check if combination is valid
    check_credentials_not_weak(email, password)
})
```

### Validating Collections

```rust
use stillwater::Validation;

fn validate_all_items(items: Vec<Item>) -> Validation<Vec<ValidItem>, Vec<Error>> {
    let validations: Vec<_> = items
        .into_iter()
        .map(|item| validate_item(item))
        .collect();

    Validation::all_vec(validations)
}
```

### Building Complex Types

```rust
use stillwater::Validation;

struct Config {
    host: String,
    port: u16,
    timeout: u64,
}

fn validate_config(input: ConfigInput) -> Validation<Config, Vec<Error>> {
    Validation::all((
        validate_host(&input.host),
        validate_port(input.port),
        validate_timeout(input.timeout),
    ))
    .map(|(host, port, timeout)| Config { host, port, timeout })
}
```

## Testing

Validation is pure - testing is trivial:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_email() {
        let result = validate_email("user@example.com");
        assert!(result.is_success());
    }

    #[test]
    fn test_invalid_email() {
        let result = validate_email("invalid");
        assert!(result.is_failure());
    }

    #[test]
    fn test_accumulation() {
        let result = validate_registration("bad", "short", 15);

        match result {
            Validation::Failure(errors) => {
                assert_eq!(errors.len(), 3);
            }
            _ => panic!("Expected failure"),
        }
    }

    #[test]
    fn test_partial_failure() {
        // Valid email, invalid password and age
        let result = validate_registration("user@example.com", "short", 15);

        match result {
            Validation::Failure(errors) => {
                assert_eq!(errors.len(), 2);
            }
            _ => panic!("Expected failure"),
        }
    }
}
```

## Advanced: Custom Error Types

You can use any error type that implements Semigroup:

```rust
use stillwater::{Validation, Semigroup};

#[derive(Debug, Clone)]
struct ValidationContext {
    field: String,
    message: String,
}

#[derive(Debug, Clone)]
struct ValidationErrors {
    errors: Vec<ValidationContext>,
}

impl Semigroup for ValidationErrors {
    fn combine(mut self, other: Self) -> Self {
        self.errors.extend(other.errors);
        self
    }
}

fn validate_with_context(
    field: &str,
    value: &str,
) -> Validation<String, ValidationErrors> {
    if value.is_empty() {
        Validation::failure(ValidationErrors {
            errors: vec![ValidationContext {
                field: field.to_string(),
                message: "Field is required".to_string(),
            }],
        })
    } else {
        Validation::success(value.to_string())
    }
}
```

## Performance Considerations

Validation has minimal overhead:
- Success case: just wraps a value (zero-cost)
- Failure case: creates error collection
- Combining: uses efficient vector extension

The main cost is creating error objects, which you'd do anyway.

## Common Pitfalls

### Don't use `?` for accumulation

```rust
// ❌ Wrong: short-circuits on first error
fn validate(data: Data) -> Validation<Valid, Vec<Error>> {
    let email = validate_email(data.email)?;  // Stops here!
    let age = validate_age(data.age)?;
    // ...
}

// ✓ Right: accumulates errors
fn validate(data: Data) -> Validation<Valid, Vec<Error>> {
    Validation::all((
        validate_email(data.email),
        validate_age(data.age),
    ))
}
```

### Remember to map after all()

```rust
// ❌ Wrong: returns tuple instead of User
fn validate(email: &str, age: u8) -> Validation<(String, u8), Vec<Error>> {
    Validation::all((
        validate_email(email),
        validate_age(age),
    ))
}

// ✓ Right: map tuple to User
fn validate(email: &str, age: u8) -> Validation<User, Vec<Error>> {
    Validation::all((
        validate_email(email),
        validate_age(age),
    ))
    .map(|(email, age)| User { email, age })
}
```

## Summary

- **Validation** accumulates all errors instead of short-circuiting
- Use **Validation::all()** for independent validations
- Use **and_then()** for dependent validations
- Error types must implement **Semigroup**
- Testing is easy because validation is **pure**

## Next Steps

- Learn about [Effect composition](03-effects.md)
- See [full example](../../examples/form_validation.rs)
- Read the [API docs](https://docs.rs/stillwater)
