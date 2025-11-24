# Homogeneous Validation

When you have an enum where each variant forms a Semigroup, but different variants cannot be combined, use homogeneous validation to ensure type consistency before combining.

## The Problem

Many Rust programs use discriminated unions (enums) where each variant represents a different type of data, but each variant can be combined with other values of the same variant:

```rust
enum Aggregate {
    Sum(f64),      // Sum + Sum = Sum (valid)
    Count(usize),  // Count + Count = Count (valid)
    // But: Sum + Count = ??? (type error!)
}
```

In a typical scenario, you might write:

```rust
impl Semigroup for Aggregate {
    fn combine(self, other: Self) -> Self {
        match (self, other) {
            (Aggregate::Sum(a), Aggregate::Sum(b)) => Aggregate::Sum(a + b),
            (Aggregate::Count(a), Aggregate::Count(b)) => Aggregate::Count(a + b),
            _ => panic!("Type mismatch!"), // 💥 Crashes at runtime!
        }
    }
}
```

### Why This Matters

This pattern appears frequently in real-world code:

**Aggregation Pipelines**: MapReduce systems where parallel workers combine results
```rust
// Workers return different aggregate types
let results = vec![worker1.result(), worker2.result(), worker3.result()];
// If types don't match, the program crashes!
```

**JSON/Config Merging**: Combining configuration from multiple sources
```rust
// serde_json::Value has variants for Object, Array, Number, etc.
// Merging an Object with an Array doesn't make sense
```

**Database Query Results**: Combining results from sharded queries
```rust
enum QueryResult {
    Rows(Vec<Row>),
    Count(usize),
    Affected(usize),
}
// Each shard should return the same type
```

**Plugin Systems**: Aggregating outputs from multiple plugins
```rust
enum PluginOutput {
    Metrics(Vec<Metric>),
    Logs(Vec<LogEntry>),
    Events(Vec<Event>),
}
```

### The Traditional Bad Options

When faced with this problem, developers typically choose one of two bad options:

1. **Panic on mismatch** - Crashes the program at runtime
2. **Silent coercion** - Produces wrong results without warning

## Pure Core, Imperative Shell

Stillwater follows the principle of **"pure core, imperative shell"**, where:

- **Pure core**: Business logic (like `Semigroup::combine`) is pure and total
- **Imperative shell**: Validation happens at I/O boundaries

This means:
- Keep `Semigroup::combine` simple and panic-free (after validation)
- Validate homogeneity at system boundaries (YAML → types, I/O → domain, workers → aggregation)
- Accumulate ALL errors (don't fail fast)
- Provide clear error messages for debugging

## Solution: Homogeneous Validation

Stillwater provides utilities to validate homogeneity before combining:

```rust
use stillwater::validation::homogeneous::validate_homogeneous;
use stillwater::Validation;
use std::mem::discriminant;

let items = vec![
    Aggregate::Count(5),
    Aggregate::Sum(10.0),    // Wrong type!
    Aggregate::Count(3),
];

let result = validate_homogeneous(
    items,
    |a| discriminant(a),
    |idx, _got, _expected| format!("Type mismatch at index {}", idx),
);

match result {
    Validation::Success(items) => {
        // All items have the same type, safe to combine
        let combined = items.into_iter()
            .reduce(|a, b| a.combine(b))
            .unwrap();
    }
    Validation::Failure(errors) => {
        // errors = ["Type mismatch at index 1"]
        // All errors reported at once!
        eprintln!("Validation errors: {:?}", errors);
    }
}
```

### Key Features

1. **Error Accumulation**: Reports ALL type mismatches, not just the first one
2. **Flexible Error Types**: You provide the error constructor
3. **Generic Discriminant**: Works with `std::mem::discriminant` or custom logic
4. **Zero-Cost Abstraction**: No runtime overhead compared to manual validation

## API Overview

### Core Functions

#### `validate_homogeneous`

Validates that all items in a collection have the same discriminant:

```rust
pub fn validate_homogeneous<T, D, E>(
    items: Vec<T>,
    discriminant: impl Fn(&T) -> D,
    make_error: impl Fn(usize, &T, &T) -> E,
) -> Validation<Vec<T>, Vec<E>>
where
    D: Eq,
```

**Parameters:**
- `items`: Collection to validate
- `discriminant`: Function to extract discriminant for comparison
- `make_error`: Function to create error for type mismatch

**Returns:**
- `Validation::Success(items)` if all items have the same discriminant
- `Validation::Failure(errors)` with ALL mismatches if heterogeneous

#### `combine_homogeneous`

Convenience function that validates and combines in one step:

```rust
pub fn combine_homogeneous<T, D, E>(
    items: Vec<T>,
    discriminant: impl Fn(&T) -> D,
    make_error: impl Fn(usize, &T, &T) -> E,
) -> Validation<T, Vec<E>>
where
    T: Semigroup,
    D: Eq,
```

This is equivalent to:
```rust
validate_homogeneous(items, discriminant, make_error)
    .map(|items| items.into_iter().reduce(|a, b| a.combine(b)).unwrap())
```

### Helper Types

#### `DiscriminantName` Trait

Provides human-readable names for discriminants:

```rust
pub trait DiscriminantName {
    fn discriminant_name(&self) -> &'static str;
}
```

Example implementation:
```rust
impl DiscriminantName for Aggregate {
    fn discriminant_name(&self) -> &'static str {
        match self {
            Aggregate::Count(_) => "Count",
            Aggregate::Sum(_) => "Sum",
            Aggregate::Average(_, _) => "Average",
        }
    }
}
```

#### `TypeMismatchError`

A standardized error type for type mismatches:

```rust
pub struct TypeMismatchError {
    pub index: usize,
    pub expected: String,
    pub got: String,
}
```

Use with `DiscriminantName`:
```rust
let result = validate_homogeneous(
    items,
    |a| std::mem::discriminant(a),
    TypeMismatchError::new,
);
```

## Examples

### Example 1: Aggregation Pipeline

```rust
use stillwater::validation::homogeneous::combine_homogeneous;
use stillwater::{Semigroup, Validation};
use std::mem::discriminant;

#[derive(Clone, Debug, PartialEq)]
enum AggregateResult {
    Count(usize),
    Sum(f64),
    Average(f64, usize),
}

impl Semigroup for AggregateResult {
    fn combine(self, other: Self) -> Self {
        match (self, other) {
            (AggregateResult::Count(a), AggregateResult::Count(b)) => {
                AggregateResult::Count(a + b)
            }
            (AggregateResult::Sum(a), AggregateResult::Sum(b)) => {
                AggregateResult::Sum(a + b)
            }
            (AggregateResult::Average(s1, c1), AggregateResult::Average(s2, c2)) => {
                AggregateResult::Average(s1 + s2, c1 + c2)
            }
            _ => unreachable!("Validated before combining"),
        }
    }
}

// Aggregate results from parallel workers
let worker_results = vec![
    AggregateResult::Count(10),
    AggregateResult::Count(20),
    AggregateResult::Count(30),
];

let result = combine_homogeneous(
    worker_results,
    |r| discriminant(r),
    |idx, _, _| format!("Worker {} returned different type", idx),
);

match result {
    Validation::Success(combined) => {
        println!("Total count: {:?}", combined);
    }
    Validation::Failure(errors) => {
        eprintln!("Validation errors: {:?}", errors);
    }
}
```

### Example 2: JSON Config Merging

```rust
use serde_json::Value;
use stillwater::validation::homogeneous::{validate_homogeneous, DiscriminantName};
use std::mem::discriminant;

impl DiscriminantName for Value {
    fn discriminant_name(&self) -> &'static str {
        match self {
            Value::Null => "Null",
            Value::Bool(_) => "Bool",
            Value::Number(_) => "Number",
            Value::String(_) => "String",
            Value::Array(_) => "Array",
            Value::Object(_) => "Object",
        }
    }
}

// Load configs from multiple sources
let configs = vec![
    load_default_config(),   // Object
    load_user_config(),      // Object
    load_env_config(),       // Object?
];

// Ensure all are objects before merging
let result = validate_homogeneous(
    configs,
    |v| discriminant(v),
    |idx, got, expected| {
        format!(
            "Config {}: expected {}, got {}",
            idx,
            expected.discriminant_name(),
            got.discriminant_name()
        )
    },
);

match result {
    Validation::Success(configs) => {
        let merged = merge_json_objects(configs);
        // Use merged config
    }
    Validation::Failure(errors) => {
        eprintln!("Config validation errors: {:?}", errors);
    }
}
```

### Example 3: Integration with Effect

```rust
use stillwater::{Effect, IO, Validation};
use stillwater::validation::homogeneous::combine_homogeneous;

fn aggregate_with_validation(
    job_id: &str,
) -> Effect<AggregateResult, Vec<String>, Env> {
    IO::query(|env| env.load_results(job_id))
        .and_then(|results| {
            // Validation at I/O boundary
            match combine_homogeneous(
                results,
                |r| std::mem::discriminant(r),
                |idx, _, _| format!("Worker {} type mismatch", idx),
            ) {
                Validation::Success(combined) => Effect::pure(combined),
                Validation::Failure(errors) => Effect::fail(errors),
            }
        })
        .context("Aggregating results with type validation")
}
```

## Best Practices

### 1. Validate at Boundaries

Always validate at system boundaries, not in the middle of business logic:

✅ **Good**: Validate at I/O boundaries
```rust
IO::query(load_results)
    .and_then(|results| validate_and_process(results))
```

❌ **Bad**: Validate in the middle of logic
```rust
fn process(items: Vec<T>) {
    // ... business logic ...
    validate_homogeneous(items, ...);  // Too late!
    // ... more logic ...
}
```

### 2. Keep Semigroup Pure

After validation, your `Semigroup::combine` can safely use `unreachable!()`:

```rust
impl Semigroup for MyEnum {
    fn combine(self, other: Self) -> Self {
        match (self, other) {
            (A(x), A(y)) => A(x + y),
            (B(x), B(y)) => B(x + y),
            _ => unreachable!("Call validate_homogeneous first"),
        }
    }
}
```

### 3. Provide Helpful Error Messages

Use `DiscriminantName` to create clear error messages:

```rust
impl DiscriminantName for MyEnum {
    fn discriminant_name(&self) -> &'static str {
        match self {
            MyEnum::VariantA(_) => "VariantA",
            MyEnum::VariantB(_) => "VariantB",
        }
    }
}

let result = validate_homogeneous(
    items,
    |e| discriminant(e),
    TypeMismatchError::new,
);
```

### 4. Accumulate All Errors

Take advantage of error accumulation to report all issues at once:

```rust
match validate_homogeneous(...) {
    Validation::Failure(errors) => {
        // All errors are available
        for error in errors {
            eprintln!("Error: {}", error);
        }
        // Send to monitoring, add to DLQ, etc.
    }
    _ => { /* ... */ }
}
```

### 5. Compose with Other Validations

Homogeneous validation composes naturally with other validations:

```rust
let type_check = validate_homogeneous(items, discriminant, make_error);
let range_check = validate_ranges(items);

// Combine validations
let all_checks = type_check.and(range_check);
```

## Edge Cases

### Empty Collections

Empty collections always validate successfully:

```rust
let empty: Vec<MyEnum> = vec![];
let result = validate_homogeneous(empty, discriminant, make_error);
assert!(result.is_success());
```

**Why?** There are no pairs to compare, so the homogeneity property is trivially true.

### Single-Item Collections

Single-item collections always validate successfully:

```rust
let single = vec![MyEnum::A(42)];
let result = validate_homogeneous(single, discriminant, make_error);
assert!(result.is_success());
```

**Why?** The item is homogeneous with itself.

### After Validation

Once validation succeeds, you're guaranteed that all items have the same discriminant. It's safe to use `unreachable!()` or `panic!()` in the `combine` method for mismatched cases.

## Performance

Homogeneous validation is a zero-cost abstraction:

- **Single pass**: O(n) traversal of the collection
- **No allocations**: Besides the error vector if validation fails
- **Inlined**: Discriminant and error functions are typically inlined
- **Lazy evaluation**: Only evaluates discriminant when needed

Benchmark results show no overhead compared to manual validation.

## Common Patterns

### Pattern 1: MapReduce Aggregation

```rust
// Validate before reducing
let aggregated = combine_homogeneous(
    worker_results,
    |r| discriminant(r),
    |idx, _, _| format!("Worker {} mismatch", idx),
)?;
```

### Pattern 2: Config Merging

```rust
// Validate configs are same type before merging
validate_homogeneous(configs, discriminant, make_error)
    .and_then(|configs| merge_configs(configs))
```

### Pattern 3: Database Sharding

```rust
// Validate shard results are consistent
combine_homogeneous(
    shard_results,
    |r| discriminant(r),
    |idx, _, _| format!("Shard {} inconsistent", idx),
)
```

### Pattern 4: Plugin Composition

```rust
// Validate all plugins return same output type
validate_homogeneous(
    plugin_outputs,
    |o| discriminant(o),
    |idx, _, _| format!("Plugin {} incompatible", idx),
)
```

## Related Concepts

- **Semigroup**: Homogeneous validation enables safe Semigroup usage with enums
- **Validation**: Uses the Validation type for error accumulation
- **Effect**: Composes naturally at I/O boundaries
- **Pure core, imperative shell**: Validation at boundaries, pure logic in core

## Further Reading

- [Validation Guide](./02-validation.md) - Learn about error accumulation
- [Semigroup Guide](./11-semigroup.md) - Understand combining operations
- [Effect Guide](./03-effects.md) - Compose with I/O operations
- [Philosophy](../../PHILOSOPHY.md) - Pure core, imperative shell pattern
