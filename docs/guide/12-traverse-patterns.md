# Traverse and Sequence Patterns

## The Problem

When working with collections of data that need validation or effectful processing, you often face a choice:

```rust
// Option 1: Process one at a time, manually accumulating
let mut results = Vec::new();
let mut errors = Vec::new();
for item in items {
    match validate(item) {
        Validation::Success(val) => results.push(val),
        Validation::Failure(err) => errors.extend(err),
    }
}
// Now what? How do we combine results and errors?

// Option 2: Use map and somehow convert Vec<Validation<T, E>> to Validation<Vec<T>, E>
let validations: Vec<Validation<_, _>> = items.iter().map(validate).collect();
// But how do we turn this into a single Validation?
```

Both approaches are cumbersome and error-prone. This is where **traverse** and **sequence** come in.

## The Solution: Traverse and Sequence

Stillwater provides two fundamental operations for working with collections of effects:

- **`sequence`**: Converts a collection of effects into an effect of a collection
  - `Vec<Validation<T, E>>` → `Validation<Vec<T>, E>`
  - `Vec<Effect<T, E, Env>>` → `Effect<Vec<T>, E, Env>`

- **`traverse`**: Maps a function over a collection and sequences the results
  - Equivalent to `map(f).sequence()` but more efficient

## Core Concepts

### Sequence

**Sequence** inverts the structure of nested types:

```rust
use stillwater::{Validation, traverse::sequence};

// We have: Vec<Validation<T, E>>
let validations = vec![
    Validation::success(1),
    Validation::success(2),
    Validation::success(3),
];

// We want: Validation<Vec<T>, E>
let result = sequence(validations);
assert_eq!(result, Validation::Success(vec![1, 2, 3]));
```

If any validation fails, all errors are accumulated:

```rust
use stillwater::{Validation, traverse::sequence};

let validations = vec![
    Validation::<i32, _>::failure(vec!["error 1"]),
    Validation::success(2),
    Validation::failure(vec!["error 2"]),
];

let result = sequence(validations);
match result {
    Validation::Failure(errors) => {
        assert_eq!(errors, vec!["error 1", "error 2"]);
    }
    _ => panic!("Expected failure"),
}
```

### Traverse

**Traverse** combines mapping and sequencing in one operation:

```rust
use stillwater::{Validation, traverse::traverse};

fn parse_number(s: &str) -> Validation<i32, Vec<String>> {
    s.parse()
        .map(Validation::success)
        .unwrap_or_else(|_| Validation::failure(vec![format!("Invalid: {}", s)]))
}

// Instead of: items.iter().map(parse_number).collect() then sequence
let result = traverse(vec!["1", "2", "3"], parse_number);
assert_eq!(result, Validation::Success(vec![1, 2, 3]));
```

## Validation Examples

### Validating User Input Collections

```rust
use stillwater::{Validation, traverse::traverse};

#[derive(Debug, PartialEq)]
struct Email(String);

#[derive(Debug)]
enum ValidationError {
    InvalidEmail(String),
}

fn validate_email(raw: &str) -> Validation<Email, Vec<ValidationError>> {
    if raw.contains('@') && raw.contains('.') {
        Validation::success(Email(raw.to_string()))
    } else {
        Validation::failure(vec![ValidationError::InvalidEmail(raw.to_string())])
    }
}

// Validate a list of email addresses
let emails = vec!["alice@example.com", "bob@example.com", "invalid"];
let result = traverse(emails, validate_email);

match result {
    Validation::Success(valid_emails) => {
        println!("All valid: {:?}", valid_emails);
    }
    Validation::Failure(errors) => {
        println!("Found {} invalid emails:", errors.len());
        for err in errors {
            println!("  {:?}", err);
        }
    }
}
```

### Validating Nested Data

```rust
use stillwater::{Validation, traverse::traverse};

#[derive(Debug)]
struct User {
    name: String,
    age: u8,
}

fn validate_user(name: &str, age: u8) -> Validation<User, Vec<String>> {
    let name_check = if name.is_empty() {
        Validation::failure(vec!["Name cannot be empty".to_string()])
    } else {
        Validation::success(name.to_string())
    };

    let age_check = if age >= 18 {
        Validation::success(age)
    } else {
        Validation::failure(vec![format!("Age {} too young", age)])
    };

    Validation::all((name_check, age_check))
        .map(|(name, age)| User { name, age })
}

// Validate a batch of user registrations
let registrations = vec![
    ("Alice", 25),
    ("Bob", 16),
    ("", 30),
];

let result = traverse(registrations, |(name, age)| validate_user(name, age));

match result {
    Validation::Success(users) => {
        println!("All valid: {} users registered", users.len());
    }
    Validation::Failure(errors) => {
        println!("Validation errors:");
        for err in errors {
            println!("  - {}", err);
        }
    }
}
```

### CSV Parsing with Error Accumulation

```rust
use stillwater::{Validation, traverse::traverse};

#[derive(Debug)]
struct Record {
    id: i32,
    name: String,
    score: f64,
}

fn parse_record(line: &str) -> Validation<Record, Vec<String>> {
    let parts: Vec<_> = line.split(',').collect();

    if parts.len() != 3 {
        return Validation::failure(vec![
            format!("Expected 3 fields, got {}", parts.len())
        ]);
    }

    let id_check = parts[0].parse::<i32>()
        .map(Validation::success)
        .unwrap_or_else(|_| Validation::failure(vec![
            format!("Invalid ID: {}", parts[0])
        ]));

    let name_check = if parts[1].is_empty() {
        Validation::failure(vec!["Name cannot be empty".to_string()])
    } else {
        Validation::success(parts[1].to_string())
    };

    let score_check = parts[2].parse::<f64>()
        .map(Validation::success)
        .unwrap_or_else(|_| Validation::failure(vec![
            format!("Invalid score: {}", parts[2])
        ]));

    Validation::all((id_check, name_check, score_check))
        .map(|(id, name, score)| Record { id, name, score })
}

let csv_lines = vec![
    "1,Alice,95.5",
    "2,Bob,invalid",
    "3,,88.0",
    "bad,line",
];

let result = traverse(csv_lines, parse_record);

match result {
    Validation::Success(records) => {
        println!("Parsed {} records", records.len());
    }
    Validation::Failure(errors) => {
        println!("CSV parsing errors:");
        for err in errors {
            println!("  - {}", err);
        }
    }
}
```

## Effect Examples

### Batch File Processing

```rust
use stillwater::{Effect, traverse::traverse_effect};

fn process_file(path: &str) -> Effect<String, String, ()> {
    Effect::of(move |_env| {
        Box::pin(async move {
            // Simulate file reading
            Ok(format!("Contents of {}", path))
        })
    })
}

let files = vec!["file1.txt", "file2.txt", "file3.txt"];
let effect = traverse_effect(files, |path| process_file(path));

// Run the effect
tokio_test::block_on(async {
    match effect.run(&()).await {
        Ok(contents) => {
            for content in contents {
                println!("{}", content);
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
});
```

### Database Batch Operations

```rust
use stillwater::{Effect, traverse::traverse_effect};

struct Database {
    // Database connection details
}

fn save_user(db: &Database, user: User) -> Effect<i64, String, Database> {
    Effect::of(|db| {
        Box::pin(async move {
            // Simulate database save
            Ok(42) // user ID
        })
    })
}

let users = vec![
    User { name: "Alice".to_string(), age: 25 },
    User { name: "Bob".to_string(), age: 30 },
];

let db = Database {};
let effect = traverse_effect(users, |user| save_user(&db, user));

// Run the effect
tokio_test::block_on(async {
    match effect.run(&db).await {
        Ok(ids) => {
            println!("Saved {} users with IDs: {:?}", ids.len(), ids);
        }
        Err(e) => eprintln!("Database error: {}", e),
    }
});
```

### Parallel API Calls

```rust
use stillwater::{Effect, traverse::traverse_effect};

fn fetch_user(id: i32) -> Effect<String, String, ()> {
    Effect::of(move |_env| {
        Box::pin(async move {
            // Simulate API call
            Ok(format!("User {}", id))
        })
    })
}

let user_ids = vec![1, 2, 3, 4, 5];
let effect = traverse_effect(user_ids, fetch_user);

// Effects run in parallel
tokio_test::block_on(async {
    match effect.run(&()).await {
        Ok(users) => {
            println!("Fetched {} users", users.len());
        }
        Err(e) => eprintln!("API error: {}", e),
    }
});
```

## Sequence Examples

### Sequencing Pre-computed Validations

```rust
use stillwater::{Validation, traverse::sequence};

// When you already have validations (perhaps from different sources)
let validations = vec![
    validate_field_1(),
    validate_field_2(),
    validate_field_3(),
];

let result = sequence(validations);
```

### Sequencing Effects

```rust
use stillwater::{Effect, traverse::sequence_effect};

// When you have a collection of effects to run
let effects = vec![
    Effect::pure(1),
    Effect::pure(2),
    Effect::pure(3),
];

let combined = sequence_effect(effects);

tokio_test::block_on(async {
    let result = combined.run(&()).await;
    assert_eq!(result, Ok(vec![1, 2, 3]));
});
```

## Practical Patterns

### Pattern 1: Validate Then Process

```rust
use stillwater::{Validation, Effect, traverse::traverse};

// First validate all inputs
let validation = traverse(inputs, validate_input);

// Then convert to effect and process
let effect = Effect::from_validation(validation)
    .and_then(|valid_inputs| process_batch(valid_inputs));
```

### Pattern 2: Fail Fast vs Accumulate

```rust
use stillwater::{Validation, traverse::traverse};

// Accumulate all validation errors
fn validate_all(items: Vec<Item>) -> Validation<Vec<Valid>, Vec<Error>> {
    traverse(items, validate_item)
}

// Fail on first error (use Effect instead)
fn process_all(items: Vec<Item>) -> Effect<Vec<Result>, Error, Env> {
    traverse_effect(items, process_item)
}
```

### Pattern 3: Filtering with Validation

```rust
use stillwater::{Validation, traverse::traverse};

fn validate_and_filter(items: Vec<String>) -> Validation<Vec<i32>, Vec<String>> {
    let parsed = traverse(items, |s| {
        s.parse::<i32>()
            .map(Validation::success)
            .unwrap_or_else(|_| Validation::failure(vec![format!("Invalid: {}", s)]))
    });

    parsed
}
```

### Pattern 4: Transform with Environment

```rust
use stillwater::{Effect, traverse::traverse_effect};

struct Config {
    api_key: String,
}

fn fetch_with_auth(id: i32) -> Effect<Data, Error, Config> {
    Effect::asks(move |config: &Config| {
        // Use config.api_key in request
        Data { id }
    })
}

let config = Config { api_key: "secret".to_string() };
let effect = traverse_effect(vec![1, 2, 3], fetch_with_auth);

// All requests share the same config
tokio_test::block_on(async {
    let result = effect.run(&config).await;
});
```

## When to Use What

### Use `traverse` when:
- You have a collection and a function to apply to each element
- The function returns a Validation or Effect
- You want a single result aggregating all outcomes

### Use `sequence` when:
- You already have a collection of Validations or Effects
- You need to invert the structure (Vec of Validations → Validation of Vec)

### Use `Validation::all()` when:
- You have a fixed number of validations (tuple)
- The validations are different types
- You want to combine them all

### Use `traverse` vs manual loop when:
- **traverse**: Pure transformation, all errors matter
- **manual loop**: Need early exit, complex control flow

## Performance Considerations

### Memory Efficiency
- `traverse` is more efficient than `map().sequence()` because it only creates one collection
- For large collections, consider streaming or chunking

### Parallel vs Sequential
- `traverse_effect` runs effects in parallel by default
- For CPU-bound work, this is optimal
- For I/O-bound work with rate limits, consider sequential processing

### Early Termination
- Validation accumulates ALL errors (no early exit)
- Effect stops at first error (fail-fast)
- Choose based on your error handling needs

## Common Pitfalls

### Pitfall 1: Not handling empty collections

```rust
// Empty collections return success with empty vec
let result = traverse(Vec::<i32>::new(), validate);
assert_eq!(result, Validation::Success(vec![]));

// Make sure this is the behavior you want!
```

### Pitfall 2: Mixing traverse and for loops

```rust
// Bad: Manual loop loses error accumulation
for item in items {
    validate(item)?; // Stops at first error!
}

// Good: Use traverse
traverse(items, validate)
```

### Pitfall 3: Forgetting to map after traverse

```rust
// Returns Validation<Vec<(String, i32)>, E>
traverse(items, |item| {
    Validation::all((validate_name(item.name), validate_age(item.age)))
})

// Better: Map to User
traverse(items, |item| {
    Validation::all((validate_name(item.name), validate_age(item.age)))
        .map(|(name, age)| User { name, age })
})
```

## Testing

Testing traverse operations is straightforward:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use stillwater::{Validation, traverse::traverse};

    #[test]
    fn test_traverse_all_valid() {
        let result = traverse(vec![1, 2, 3], validate_positive);
        assert!(result.is_success());
    }

    #[test]
    fn test_traverse_accumulates_errors() {
        let result = traverse(vec![1, -2, -3], validate_positive);

        match result {
            Validation::Failure(errors) => {
                assert_eq!(errors.len(), 2); // Two negative numbers
            }
            _ => panic!("Expected failure"),
        }
    }

    #[test]
    fn test_traverse_empty() {
        let result = traverse(Vec::<i32>::new(), validate_positive);
        assert_eq!(result, Validation::Success(vec![]));
    }
}
```

## Summary

- **traverse** and **sequence** invert collection structures
- Use **traverse** for Validation to accumulate ALL errors
- Use **traverse_effect** for parallel Effect execution
- Choose based on error handling needs: accumulate vs fail-fast
- Test thoroughly, especially edge cases like empty collections

## Next Steps

- Review [Validation guide](02-validation.md) for error accumulation
- See [Effects guide](03-effects.md) for async processing
- Check [examples/](../../examples/) for complete examples
- Read the [API docs](https://docs.rs/stillwater) for full details
