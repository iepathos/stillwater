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

## Extended Implementations for Collections

Stillwater provides Semigroup implementations for standard Rust collection types, enabling powerful composition patterns for configuration merging, error aggregation, and data combining.

### HashMaps and BTrees

#### HashMap<K, V: Semigroup>

Combines two maps by merging their entries. When keys conflict, their values are combined using the value's Semigroup instance:

```rust
use std::collections::HashMap;
use stillwater::Semigroup;

let mut map1 = HashMap::new();
map1.insert("errors", vec!["error1"]);
map1.insert("warnings", vec!["warn1"]);

let mut map2 = HashMap::new();
map2.insert("errors", vec!["error2"]);
map2.insert("info", vec!["info1"]);

let combined = map1.combine(map2);
// Result:
// {
//   "errors": ["error1", "error2"],  // Combined with Vec semigroup
//   "warnings": ["warn1"],            // From map1 only
//   "info": ["info1"]                 // From map2 only
// }
```

**Use case: Configuration Merging**

```rust
use std::collections::HashMap;
use stillwater::Semigroup;

#[derive(Clone)]
struct Config {
    settings: HashMap<String, String>,
}

impl Semigroup for Config {
    fn combine(self, other: Self) -> Self {
        Config {
            settings: self.settings.combine(other.settings),
        }
    }
}

// Layer configs from different sources
let default_config = Config {
    settings: [("timeout", "30"), ("retries", "3")]
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect(),
};

let user_config = Config {
    settings: [("timeout", "60"), ("debug", "true")]
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect(),
};

// user_config values override default_config values
// (String semigroup concatenates, but you'd typically use Last wrapper for configs)
```

#### BTreeMap<K, V: Semigroup>

Same as HashMap but maintains sorted keys:

```rust
use std::collections::BTreeMap;
use stillwater::Semigroup;

let mut map1 = BTreeMap::new();
map1.insert("a", vec![1, 2]);
map1.insert("b", vec![3]);

let mut map2 = BTreeMap::new();
map2.insert("a", vec![4, 5]);
map2.insert("c", vec![6]);

let combined = map1.combine(map2);
// Keys in sorted order: {a: [1,2,4,5], b: [3], c: [6]}
```

### Sets

#### HashSet<T>

Combines two sets using union:

```rust
use std::collections::HashSet;
use stillwater::Semigroup;

let set1: HashSet<_> = [1, 2, 3].iter().cloned().collect();
let set2: HashSet<_> = [3, 4, 5].iter().cloned().collect();

let combined = set1.combine(set2);
// Result: {1, 2, 3, 4, 5}
```

**Use case: Feature Flags**

```rust
use std::collections::HashSet;
use stillwater::Semigroup;

#[derive(Clone)]
struct Features {
    enabled: HashSet<String>,
}

impl Semigroup for Features {
    fn combine(self, other: Self) -> Self {
        Features {
            enabled: self.enabled.combine(other.enabled),
        }
    }
}

let base_features = Features {
    enabled: ["logging", "metrics"].iter().map(|s| s.to_string()).collect(),
};

let premium_features = Features {
    enabled: ["advanced_analytics", "priority_support"].iter().map(|s| s.to_string()).collect(),
};

let all_features = base_features.combine(premium_features);
// All features enabled
```

#### BTreeSet<T>

Same as HashSet but maintains sorted elements:

```rust
use std::collections::BTreeSet;
use stillwater::Semigroup;

let set1: BTreeSet<_> = [1, 2, 3].iter().cloned().collect();
let set2: BTreeSet<_> = [3, 4, 5].iter().cloned().collect();

let combined = set1.combine(set2);
// Elements in sorted order: {1, 2, 3, 4, 5}
```

### Option<T: Semigroup>

Lifts a Semigroup operation to Option, combining inner values when both are Some:

```rust
use stillwater::Semigroup;

let opt1 = Some(vec![1, 2]);
let opt2 = Some(vec![3, 4]);
assert_eq!(opt1.combine(opt2), Some(vec![1, 2, 3, 4]));

let none: Option<Vec<i32>> = None;
let some = Some(vec![1, 2]);
assert_eq!(none.clone().combine(some.clone()), some);
assert_eq!(some.clone().combine(none), some);

let none1: Option<Vec<i32>> = None;
let none2: Option<Vec<i32>> = None;
assert_eq!(none1.combine(none2), None);
```

**Combination rules:**
- `Some(a).combine(Some(b))` = `Some(a.combine(b))`
- `Some(a).combine(None)` = `Some(a)`
- `None.combine(Some(b))` = `Some(b)`
- `None.combine(None)` = `None`

**Use case: Optional Error Accumulation**

```rust
use stillwater::Semigroup;

fn validate_optional_field(
    value: Option<String>,
) -> Option<Vec<String>> {
    value.and_then(|v| {
        if v.is_empty() {
            Some(vec!["Field cannot be empty".to_string()])
        } else {
            None // No errors
        }
    })
}

let error1 = Some(vec!["Error 1".to_string()]);
let error2 = None;
let error3 = Some(vec!["Error 2".to_string()]);

let all_errors = error1.combine(error2).combine(error3);
// Some(vec!["Error 1", "Error 2"])
```

## Wrapper Types for Alternative Semantics

Sometimes you want different combining behavior. Stillwater provides wrapper types for common alternatives:

### First<T> - Keep First Value

Always keeps the first (left) value, discarding the second:

```rust
use stillwater::{First, Semigroup};

let first = First(1).combine(First(2));
assert_eq!(first.0, 1); // Keeps first

// Useful for configuration: first definition wins
let config_value = First("default").combine(First("override"));
assert_eq!(config_value.0, "default");
```

**Use case: Default Values**

```rust
use std::collections::HashMap;
use stillwater::{First, Semigroup};

// Use First wrapper to keep default config values
let defaults: HashMap<String, First<i32>> =
    [("timeout", First(30)), ("retries", First(3))]
        .iter()
        .cloned()
        .collect();

let user_config: HashMap<String, First<i32>> =
    [("timeout", First(60))]  // User only overrides timeout
        .iter()
        .cloned()
        .collect();

// Combine: user config "wins" by being first
let final_config = user_config.combine(defaults);
// timeout is 60, retries is 3
```

### Last<T> - Keep Last Value

Always keeps the last (right) value, discarding the first:

```rust
use stillwater::{Last, Semigroup};

let last = Last(1).combine(Last(2));
assert_eq!(last.0, 2); // Keeps last

// Useful for configuration: last definition wins (override)
let config_value = Last("default").combine(Last("override"));
assert_eq!(config_value.0, "override");
```

**Use case: Layered Configuration**

```rust
use std::collections::HashMap;
use stillwater::{Last, Semigroup};

// Build config from multiple layers (last wins)
let default_cfg: HashMap<String, Last<String>> =
    [("env", Last("production".into())), ("debug", Last("false".into()))]
        .iter()
        .cloned()
        .collect();

let env_cfg: HashMap<String, Last<String>> =
    [("debug", Last("true".into()))]  // Override from environment
        .iter()
        .cloned()
        .collect();

let final_cfg = default_cfg.combine(env_cfg);
// debug is "true" (env_cfg overrides)
// env is "production" (from defaults)
```

### Intersection<Set> - Set Intersection

Alternative to the default union operation for sets:

```rust
use std::collections::HashSet;
use stillwater::{Intersection, Semigroup};

let set1: HashSet<_> = [1, 2, 3].iter().cloned().collect();
let set2: HashSet<_> = [2, 3, 4].iter().cloned().collect();

let i1 = Intersection(set1);
let i2 = Intersection(set2);
let result = i1.combine(i2);

let expected: HashSet<_> = [2, 3].iter().cloned().collect();
assert_eq!(result.0, expected); // Only common elements
```

**Use case: Required Permissions**

```rust
use std::collections::HashSet;
use stillwater::{Intersection, Semigroup};

// User must have ALL these permissions (intersection)
let admin_perms: HashSet<_> =
    ["read", "write", "delete", "admin"].iter().cloned().collect();
let user_perms: HashSet<_> =
    ["read", "write", "delete"].iter().cloned().collect();

let effective_perms = Intersection(admin_perms).combine(Intersection(user_perms));
// Result: ["read", "write", "delete"] - what user actually has
```

## Real-World Example: Error Aggregation by Type

Here's how these implementations enable sophisticated error handling:

```rust
use std::collections::HashMap;
use stillwater::Semigroup;

type ErrorsByType = HashMap<String, Vec<String>>;

fn validate_user_data(data: UserData) -> ErrorsByType {
    let mut errors = HashMap::new();

    // Validation errors
    if !data.email.contains('@') {
        errors.insert("validation".to_string(), vec!["Invalid email".to_string()]);
    }

    errors
}

fn check_permissions(user: &User) -> ErrorsByType {
    let mut errors = HashMap::new();

    if !user.has_permission("create") {
        errors.insert("permission".to_string(), vec!["Unauthorized".to_string()]);
    }

    errors
}

// Combine error maps - errors of same type accumulate
let validation_errors = validate_user_data(data);
let permission_errors = check_permissions(&user);

let all_errors = validation_errors.combine(permission_errors);
// {
//   "validation": ["Invalid email", ...],
//   "permission": ["Unauthorized", ...]
// }
```

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
- **Built-in implementations** for:
  - Vec, String, and tuples (up to 12 elements)
  - HashMap, BTreeMap (merge with value combining)
  - HashSet, BTreeSet (union)
  - Option (lifts inner Semigroup)
- **Wrapper types** for alternative semantics:
  - `First<T>` - keeps first value
  - `Last<T>` - keeps last value
  - `Intersection<Set>` - set intersection instead of union
- **Custom implementations** are easy to write
- **Foundation for validation** error accumulation and configuration merging

## Next Steps

Now that you understand Semigroup, learn how it powers [Validation](02-validation.md)!
