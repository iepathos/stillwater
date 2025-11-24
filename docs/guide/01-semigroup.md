# Semigroup: The Foundation for Combining Values

## What is a Semigroup?

A Semigroup is a simple but powerful concept: a type with an associative binary operation that combines two values into one.

In Rust terms, it's a trait with one method:

```rust
pub trait Semigroup: Sized {
    fn combine(self, other: Self) -> Self;
}
```

The key property is **associativity**: the order of combining doesn't matter.

```rust
// These two operations must produce the same result:
a.combine(b).combine(c) == a.combine(b.combine(c))
```

## Why Does This Matter?

Semigroup is the foundation for error accumulation in Stillwater. When validating multiple fields, we need to combine error collections:

```rust
// Validation 1 fails with: vec!["Email invalid"]
// Validation 2 fails with: vec!["Age too young"]
// Combined failure:         vec!["Email invalid", "Age too young"]
```

To combine these errors, we need a Semigroup implementation for `Vec<E>`.

## Built-in Implementations

Stillwater provides Semigroup implementations for common types:

### Vectors

Concatenates two vectors:

```rust
use stillwater::Semigroup;

let v1 = vec![1, 2, 3];
let v2 = vec![4, 5, 6];
assert_eq!(v1.combine(v2), vec![1, 2, 3, 4, 5, 6]);

// Empty vectors work too
let empty: Vec<i32> = vec![];
let values = vec![1, 2, 3];
assert_eq!(empty.combine(values), vec![1, 2, 3]);
```

### Strings

Concatenates two strings:

```rust
use stillwater::Semigroup;

let s1 = "Hello, ".to_string();
let s2 = "World!".to_string();
assert_eq!(s1.combine(s2), "Hello, World!");
```

### Tuples

Combines tuples component-wise (up to 12 elements):

```rust
use stillwater::Semigroup;

let t1 = (vec![1], "a".to_string());
let t2 = (vec![2], "b".to_string());
assert_eq!(
    t1.combine(t2),
    (vec![1, 2], "ab".to_string())
);

// Works with larger tuples
let t1 = (vec![1], "a".to_string(), vec![2]);
let t2 = (vec![3], "b".to_string(), vec![4]);
assert_eq!(
    t1.combine(t2),
    (vec![1, 3], "ab".to_string(), vec![2, 4])
);
```

## Implementing Semigroup for Custom Types

You can implement Semigroup for your own error types:

```rust
use stillwater::Semigroup;

#[derive(Debug, PartialEq)]
struct ValidationErrors {
    errors: Vec<String>,
}

impl Semigroup for ValidationErrors {
    fn combine(mut self, other: Self) -> Self {
        self.errors.extend(other.errors);
        self
    }
}

// Usage
let e1 = ValidationErrors { errors: vec!["Email invalid".to_string()] };
let e2 = ValidationErrors { errors: vec!["Age too young".to_string()] };
let combined = e1.combine(e2);

assert_eq!(combined.errors, vec!["Email invalid", "Age too young"]);
```

### More Complex Examples

You can implement Semigroup for domain-specific error types:

```rust
use stillwater::Semigroup;

#[derive(Debug, PartialEq)]
enum ValidationError {
    InvalidEmail(String),
    AgeTooYoung { min: u8, actual: u8 },
    PasswordTooShort { min: usize, actual: usize },
}

#[derive(Debug, PartialEq)]
struct ValidationResult {
    errors: Vec<ValidationError>,
}

impl Semigroup for ValidationResult {
    fn combine(mut self, other: Self) -> Self {
        self.errors.extend(other.errors);
        self
    }
}
```

## Important: Ownership Semantics

The `combine` method takes ownership of both values:

```rust
use stillwater::Semigroup;

let v1 = vec![1, 2, 3];
let v2 = vec![4, 5, 6];
let result = v1.combine(v2);

// v1 and v2 are now moved, can't use them anymore
// This won't compile:
// println!("{:?}", v1);
```

If you need to preserve the original values, clone them first:

```rust
use stillwater::Semigroup;

let v1 = vec![1, 2, 3];
let v2 = vec![4, 5, 6];
let result = v1.clone().combine(v2.clone());

// v1 and v2 are still usable
assert_eq!(v1, vec![1, 2, 3]);
assert_eq!(v2, vec![4, 5, 6]);
assert_eq!(result, vec![1, 2, 3, 4, 5, 6]);
```

## Associativity Law

All Semigroup implementations must be associative. This means:

```rust
use stillwater::Semigroup;

let a = vec![1, 2];
let b = vec![3, 4];
let c = vec![5, 6];

// These produce the same result
let left = a.clone().combine(b.clone()).combine(c.clone());
let right = a.combine(b.combine(c));

assert_eq!(left, right);
// Both equal [1, 2, 3, 4, 5, 6]
```

This property is crucial for validation: it means we can combine errors in any order and get the same result.

## Why Not Monoid?

You might wonder why Stillwater uses Semigroup instead of Monoid (which adds an "empty" element). The reason is practical:

1. **Not all error types have a meaningful "empty"** - What's an empty custom error?
2. **Simpler API** - Less complexity for users
3. **Validation::all handles the empty case** - You don't combine zero validations

If you need a Monoid, you can easily extend Semigroup:

```rust
use stillwater::Semigroup;

trait Monoid: Semigroup {
    fn empty() -> Self;
}

impl<T> Monoid for Vec<T> {
    fn empty() -> Self {
        Vec::new()
    }
}
```

## Real-World Example: Form Validation Errors

Here's how Semigroup enables multi-field form validation:

```rust
use stillwater::{Semigroup, Validation};

#[derive(Debug, PartialEq, Clone)]
enum FormError {
    InvalidEmail(String),
    PasswordTooShort,
    AgeTooYoung,
}

// Vec<FormError> is already a Semigroup!
fn validate_form(
    email: &str,
    password: &str,
    age: u8,
) -> Validation<(), Vec<FormError>> {
    let email_check = if email.contains('@') {
        Validation::success(())
    } else {
        Validation::failure(vec![FormError::InvalidEmail(email.to_string())])
    };

    let password_check = if password.len() >= 8 {
        Validation::success(())
    } else {
        Validation::failure(vec![FormError::PasswordTooShort])
    };

    let age_check = if age >= 18 {
        Validation::success(())
    } else {
        Validation::failure(vec![FormError::AgeTooYoung])
    };

    // Validation::all uses Semigroup to combine errors!
    Validation::all((email_check, password_check, age_check))
        .map(|_| ())
}

// Usage
match validate_form("invalid", "short", 15) {
    Validation::Success(_) => println!("Valid!"),
    Validation::Failure(errors) => {
        println!("Errors: {:?}", errors);
        // Prints all 3 errors:
        // [InvalidEmail("invalid"), PasswordTooShort, AgeTooYoung]
    }
}
```

Without Semigroup, we couldn't combine the error vectors automatically!

## Testing Your Semigroup Implementation

When implementing Semigroup, test the associativity law:

```rust
use stillwater::Semigroup;

#[derive(Debug, PartialEq, Clone)]
struct MyErrors(Vec<String>);

impl Semigroup for MyErrors {
    fn combine(mut self, other: Self) -> Self {
        self.0.extend(other.0);
        self
    }
}

#[test]
fn test_associativity() {
    let a = MyErrors(vec!["a".to_string()]);
    let b = MyErrors(vec!["b".to_string()]);
    let c = MyErrors(vec!["c".to_string()]);

    let left = a.clone().combine(b.clone()).combine(c.clone());
    let right = a.combine(b.combine(c));

    assert_eq!(left, right);
}
```

## Summary

- **Semigroup** is a type with an associative `combine` operation
- **Associativity** means combining order doesn't matter
- **Built-in implementations** for Vec, String, and tuples
- **Custom implementations** are easy to write
- **Foundation for validation** error accumulation

## Next Steps

Now that you understand Semigroup, learn how it powers [Validation](02-validation.md)!
