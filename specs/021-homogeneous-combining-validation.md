---
number: 021
title: Homogeneous Combining Validation
category: foundation
priority: medium
status: draft
dependencies: [002, 011]
created: 2025-11-23
---

# Specification 021: Homogeneous Combining Validation

**Category**: foundation
**Priority**: medium
**Status**: draft
**Dependencies**: Spec 002 (Validation), Spec 011 (Monoid)

## Context

Many Rust programs use discriminated unions (enums) where each variant forms a Semigroup, but combining across variants is a type error. This pattern appears frequently in:

**Aggregation Pipelines** (like Prodigy):
```rust
enum Aggregate {
    Sum(f64),      // Sum + Sum = Sum (valid)
    Count(usize),  // Count + Count = Count (valid)
    // But: Sum + Count = ??? (type error!)
}
```

**JSON/Config Merging**:
```rust
// serde_json::Value
enum Value {
    Object(Map),   // Object + Object = merged object
    Array(Vec),    // Array + Array = concatenated array
    Number(f64),   // Number + Number = sum
    // But: Object + Array = ??? (type error!)
}
```

**Database Query Results**:
```rust
enum QueryResult {
    Rows(Vec<Row>),      // Rows + Rows = combined rows
    Count(usize),        // Count + Count = total count
    Affected(usize),     // Affected + Affected = total affected
    // But: Rows + Count = ??? (type error!)
}
```

**Plugin Systems**:
```rust
enum PluginOutput {
    Metrics(Vec<Metric>),
    Logs(Vec<LogEntry>),
    Events(Vec<Event>),
    // Combining outputs of same type makes sense, mixing doesn't
}
```

### The Problem

Rust's type system can't prevent mixing variants at compile time when:
- The discriminant is determined at runtime (from YAML config, user input, etc.)
- Values come from parallel workers with dynamic types
- Plugin systems return heterogeneous results

This forces two bad options:

1. **Panic on mismatch** - crashes the program
   ```rust
   impl Semigroup for Aggregate {
       fn combine(self, other: Self) -> Self {
           match (self, other) {
               (Sum(a), Sum(b)) => Sum(a + b),
               (Count(a), Count(b)) => Count(a + b),
               _ => panic!("Type mismatch!"), // 💥 Crash!
           }
       }
   }
   ```

2. **Silent coercion** - produces wrong results
   ```rust
   (Sum(a), Count(b)) => Sum(a + b as f64), // ❌ Silently wrong!
   ```

### Stillwater's Better Approach

Following Stillwater's philosophy of **"fail completely"** (accumulate ALL errors) and **"pure core, imperative shell"** (validation at boundaries), we need:

- **Validate homogeneity** before combining
- **Accumulate ALL type errors** (not just first mismatch)
- **Fail gracefully** with clear error messages
- **Keep Semigroup pure** (only called after validation)

## Objective

Provide validation utilities for ensuring collections of enum values are homogeneous before combining with Semigroup, following Stillwater's error accumulation and boundary validation patterns.

## Requirements

### Functional Requirements

- Validation function to check collection homogeneity
- Accumulates ALL type mismatches (not fail-fast)
- Returns `Validation<T, Vec<E>>` for error accumulation
- Works with any enum via discriminant function
- Convenience function to validate + combine in one step
- Composable with existing Semigroup/Monoid traits
- Generic over error type (user provides error constructor)
- Zero-cost abstraction when types are validated

### Non-Functional Requirements

- Zero runtime overhead compared to manual validation
- Type-safe validation guarantees
- Clear, predictable semantics
- Integration with existing Validation ecosystem
- Comprehensive documentation with real-world examples

## Acceptance Criteria

- [ ] `validate_homogeneous()` function implemented
- [ ] `combine_homogeneous()` convenience function implemented
- [ ] Accumulates ALL type errors in single validation pass
- [ ] Returns `Validation<T, Vec<E>>` for composability
- [ ] Works with `std::mem::discriminant` or custom discriminant functions
- [ ] Property-based tests verify correctness
- [ ] Integration tests with Validation and Effect
- [ ] Real-world examples (aggregation, JSON merging, etc.)
- [ ] Documentation guide: `docs/guide/09-homogeneous-validation.md`
- [ ] All tests pass
- [ ] Rustdoc examples compile and run

## Technical Details

### Implementation Approach

#### Module Location

Place in `src/validation/homogeneous.rs` (new module under validation).

#### Core Validation Function

```rust
/// Validate that all items in a collection have the same discriminant.
///
/// This is useful for enums where each variant forms a Semigroup, but
/// different variants cannot be combined. Validates homogeneity before
/// combining with the Semigroup trait.
///
/// # Arguments
///
/// * `items` - Collection to validate
/// * `discriminant` - Function to extract discriminant for comparison
/// * `make_error` - Function to create error for type mismatch
///
/// # Returns
///
/// - `Validation::Success(items)` if all items have same discriminant
/// - `Validation::Failure(errors)` with ALL mismatches if heterogeneous
///
/// # Example
///
/// ```rust
/// use stillwater::validation::homogeneous::validate_homogeneous;
/// use stillwater::Validation;
/// use std::mem::discriminant;
///
/// #[derive(Clone, PartialEq, Debug)]
/// enum Aggregate {
///     Count(usize),
///     Sum(f64),
/// }
///
/// let items = vec![
///     Aggregate::Count(5),
///     Aggregate::Sum(10.0),    // Wrong type!
///     Aggregate::Count(3),
///     Aggregate::Sum(20.0),    // Also wrong!
/// ];
///
/// let result = validate_homogeneous(
///     items,
///     |a| discriminant(a),
///     |idx, got, expected| {
///         format!("Index {}: expected Count, got Sum", idx)
///     },
/// );
///
/// match result {
///     Validation::Success(items) => {
///         // All same type, safe to combine
///     }
///     Validation::Failure(errors) => {
///         // errors = [
///         //   "Index 1: expected Count, got Sum",
///         //   "Index 3: expected Count, got Sum"
///         // ]
///         assert_eq!(errors.len(), 2); // ALL errors reported!
///     }
/// }
/// ```
pub fn validate_homogeneous<T, D, E>(
    items: Vec<T>,
    discriminant: impl Fn(&T) -> D,
    make_error: impl Fn(usize, &T, &T) -> E,
) -> Validation<Vec<T>, Vec<E>>
where
    D: Eq,
{
    if items.is_empty() {
        return Validation::success(items);
    }

    let expected = discriminant(&items[0]);
    let errors: Vec<E> = items
        .iter()
        .enumerate()
        .skip(1)
        .filter(|(_, item)| discriminant(item) != expected)
        .map(|(idx, item)| make_error(idx, item, &items[0]))
        .collect();

    if errors.is_empty() {
        Validation::success(items)
    } else {
        Validation::failure(errors)
    }
}
```

#### Convenience: Validate + Combine

```rust
/// Validate homogeneity and combine in one step.
///
/// This is a convenience function that validates all items have the same
/// discriminant, then combines them using their Semigroup instance.
///
/// # Example
///
/// ```rust
/// use stillwater::validation::homogeneous::combine_homogeneous;
/// use stillwater::{Semigroup, Validation};
/// use std::mem::discriminant;
///
/// #[derive(Clone, PartialEq, Debug)]
/// enum Aggregate {
///     Count(usize),
///     Sum(f64),
/// }
///
/// impl Semigroup for Aggregate {
///     fn combine(self, other: Self) -> Self {
///         match (self, other) {
///             (Aggregate::Count(a), Aggregate::Count(b)) => {
///                 Aggregate::Count(a + b)
///             }
///             (Aggregate::Sum(a), Aggregate::Sum(b)) => {
///                 Aggregate::Sum(a + b)
///             }
///             // Safe to panic - only called after validation
///             _ => unreachable!("Validated before combining"),
///         }
///     }
/// }
///
/// let items = vec![
///     Aggregate::Count(5),
///     Aggregate::Count(3),
///     Aggregate::Count(2),
/// ];
///
/// let result = combine_homogeneous(
///     items,
///     |a| discriminant(a),
///     |idx, _got, _expected| {
///         format!("Type mismatch at index {}", idx)
///     },
/// );
///
/// match result {
///     Validation::Success(combined) => {
///         assert_eq!(combined, Aggregate::Count(10));
///     }
///     Validation::Failure(errors) => {
///         // Handle validation errors
///     }
/// }
/// ```
pub fn combine_homogeneous<T, D, E>(
    items: Vec<T>,
    discriminant: impl Fn(&T) -> D,
    make_error: impl Fn(usize, &T, &T) -> E,
) -> Validation<T, Vec<E>>
where
    T: Semigroup,
    D: Eq,
{
    validate_homogeneous(items, discriminant, make_error).map(|items| {
        items
            .into_iter()
            .reduce(|a, b| a.combine(b))
            .expect("Validated non-empty")
    })
}
```

#### Helper: Discriminant Name Extraction

```rust
/// Helper trait for types that can provide their discriminant name.
///
/// This is useful for generating helpful error messages.
pub trait DiscriminantName {
    fn discriminant_name(&self) -> &'static str;
}

/// Create a helpful error message for type mismatches.
///
/// # Example
///
/// ```rust
/// use stillwater::validation::homogeneous::{
///     validate_homogeneous, TypeMismatchError
/// };
///
/// #[derive(Clone, Debug)]
/// enum Aggregate {
///     Count(usize),
///     Sum(f64),
/// }
///
/// impl DiscriminantName for Aggregate {
///     fn discriminant_name(&self) -> &'static str {
///         match self {
///             Aggregate::Count(_) => "Count",
///             Aggregate::Sum(_) => "Sum",
///         }
///     }
/// }
///
/// let result = validate_homogeneous(
///     items,
///     |a| std::mem::discriminant(a),
///     TypeMismatchError::new,
/// );
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeMismatchError {
    pub index: usize,
    pub expected: String,
    pub got: String,
}

impl TypeMismatchError {
    pub fn new<T: DiscriminantName>(index: usize, got: &T, expected: &T) -> Self {
        TypeMismatchError {
            index,
            expected: expected.discriminant_name().to_string(),
            got: got.discriminant_name().to_string(),
        }
    }
}

impl std::fmt::Display for TypeMismatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Type mismatch at index {}: expected {}, got {}",
            self.index, self.expected, self.got
        )
    }
}

impl std::error::Error for TypeMismatchError {}
```

### Architecture Changes

- New module: `src/validation/homogeneous.rs`
- Update `src/validation/mod.rs` to include homogeneous module
- Re-export key functions in prelude if appropriate
- No changes to existing Semigroup trait (stays pure)

### Integration Patterns

#### Prodigy's Aggregation Use Case

```rust
use stillwater::validation::homogeneous::combine_homogeneous;
use stillwater::Semigroup;

// Prodigy's AggregateResult enum
enum AggregateResult {
    Count(usize),
    Sum(f64),
    Average(f64, usize),
    // ... more variants
}

// Semigroup stays pure (panics only after validation)
impl Semigroup for AggregateResult {
    fn combine(self, other: Self) -> Self {
        match (self, other) {
            (Count(a), Count(b)) => Count(a + b),
            (Sum(a), Sum(b)) => Sum(a + b),
            (Average(s1, c1), Average(s2, c2)) => {
                Average(s1 + s2, c1 + c2)
            }
            _ => panic!("Type mismatch - call validate_homogeneous first"),
        }
    }
}

// Validation at boundary (MapReduce aggregation)
pub fn aggregate_map_results(
    results: Vec<AggregateResult>
) -> Validation<AggregateResult, Vec<String>> {
    combine_homogeneous(
        results,
        |r| std::mem::discriminant(r),
        |idx, got, expected| {
            format!(
                "Agent {} returned {:?}, expected {:?}",
                idx,
                discriminant_name(got),
                discriminant_name(expected)
            )
        },
    )
}
```

#### JSON Config Merging

```rust
use serde_json::Value;
use stillwater::validation::homogeneous::validate_homogeneous;

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

// Validate before merging configs
fn merge_configs(configs: Vec<Value>) -> Validation<Value, Vec<String>> {
    validate_homogeneous(
        configs,
        |v| std::mem::discriminant(v),
        |idx, got, expected| {
            format!(
                "Config {}: expected {}, got {}",
                idx,
                expected.discriminant_name(),
                got.discriminant_name()
            )
        },
    )
    .and_then(|configs| {
        // Now safe to merge
        merge_json_values(configs)
    })
}
```

#### Effect Integration

```rust
use stillwater::Effect;

// Compose with Effect for I/O boundaries
fn aggregate_with_validation(
    job_id: &str
) -> Effect<AggregateResult, Vec<String>, Env> {
    IO::query(|env| env.load_results(job_id))
        .and_then(|results| {
            // Validation at I/O boundary
            match combine_homogeneous(results, discriminant, make_error) {
                Validation::Success(combined) => Effect::pure(combined),
                Validation::Failure(errors) => Effect::fail(errors),
            }
        })
        .context("Aggregating results with type validation")
}
```

## Dependencies

- **Prerequisites**:
  - Spec 002 (Validation type) - uses Validation for error accumulation
  - Spec 011 (Monoid trait) - relates to combining semantics
- **Affected Components**:
  - `src/validation/mod.rs` - add homogeneous module
  - Documentation updates
- **External Dependencies**: None (uses std only)

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum TestEnum {
        A(i32),
        B(String),
    }

    impl Semigroup for TestEnum {
        fn combine(self, other: Self) -> Self {
            match (self, other) {
                (TestEnum::A(x), TestEnum::A(y)) => TestEnum::A(x + y),
                (TestEnum::B(x), TestEnum::B(y)) => TestEnum::B(x + &y),
                _ => panic!("Should be validated before combining"),
            }
        }
    }

    #[test]
    fn test_homogeneous_validates_successfully() {
        let items = vec![TestEnum::A(1), TestEnum::A(2), TestEnum::A(3)];

        let result = validate_homogeneous(
            items,
            |e| std::mem::discriminant(e),
            |idx, _, _| format!("Error at {}", idx),
        );

        assert!(result.is_success());
    }

    #[test]
    fn test_heterogeneous_accumulates_all_errors() {
        let items = vec![
            TestEnum::A(1),
            TestEnum::B("wrong1".into()),
            TestEnum::A(2),
            TestEnum::B("wrong2".into()),
        ];

        let result = validate_homogeneous(
            items,
            |e| std::mem::discriminant(e),
            |idx, _, _| format!("Error at {}", idx),
        );

        match result {
            Validation::Failure(errors) => {
                assert_eq!(errors.len(), 2);
                assert!(errors.contains(&"Error at 1".to_string()));
                assert!(errors.contains(&"Error at 3".to_string()));
            }
            _ => panic!("Expected failure"),
        }
    }

    #[test]
    fn test_combine_homogeneous_validates_and_combines() {
        let items = vec![TestEnum::A(1), TestEnum::A(2), TestEnum::A(3)];

        let result = combine_homogeneous(
            items,
            |e| std::mem::discriminant(e),
            |idx, _, _| format!("Error at {}", idx),
        );

        match result {
            Validation::Success(combined) => {
                assert_eq!(combined, TestEnum::A(6));
            }
            _ => panic!("Expected success"),
        }
    }

    #[test]
    fn test_empty_collection_validates() {
        let items: Vec<TestEnum> = vec![];

        let result = validate_homogeneous(
            items,
            |e| std::mem::discriminant(e),
            |idx, _, _| format!("Error at {}", idx),
        );

        assert!(result.is_success());
    }

    #[test]
    fn test_single_item_validates() {
        let items = vec![TestEnum::A(42)];

        let result = validate_homogeneous(
            items,
            |e| std::mem::discriminant(e),
            |idx, _, _| format!("Error at {}", idx),
        );

        assert!(result.is_success());
    }
}
```

### Property-Based Tests

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_homogeneous_always_validates(
        values in prop::collection::vec(any::<i32>(), 1..100)
    ) {
        let items: Vec<TestEnum> = values
            .into_iter()
            .map(TestEnum::A)
            .collect();

        let result = validate_homogeneous(
            items,
            |e| std::mem::discriminant(e),
            |idx, _, _| format!("Error at {}", idx),
        );

        prop_assert!(result.is_success());
    }

    #[test]
    fn prop_heterogeneous_finds_all_mismatches(
        a_count in 1usize..10,
        b_indices in prop::collection::hash_set(0usize..20, 1..5)
    ) {
        let mut items = vec![];
        for i in 0..20 {
            if b_indices.contains(&i) {
                items.push(TestEnum::B(format!("b{}", i)));
            } else if i < a_count {
                items.push(TestEnum::A(i as i32));
            }
        }

        if items.is_empty() {
            return Ok(());
        }

        let result = validate_homogeneous(
            items,
            |e| std::mem::discriminant(e),
            |idx, _, _| idx,
        );

        match result {
            Validation::Failure(errors) => {
                prop_assert_eq!(errors.len(), b_indices.len());
            }
            Validation::Success(_) => {
                prop_assert_eq!(b_indices.len(), 0);
            }
        }
    }
}
```

### Integration Tests

```rust
#[test]
fn test_integration_with_validation_and_effect() {
    use stillwater::{Effect, IO, Validation};

    // Simulate loading results from different workers
    fn load_results(env: &TestEnv) -> Vec<TestEnum> {
        vec![TestEnum::A(1), TestEnum::A(2), TestEnum::A(3)]
    }

    let effect: Effect<TestEnum, Vec<String>, TestEnv> =
        IO::query(load_results)
            .and_then(|results| {
                match combine_homogeneous(
                    results,
                    |e| std::mem::discriminant(e),
                    |idx, _, _| format!("Worker {} type mismatch", idx),
                ) {
                    Validation::Success(combined) => Effect::pure(combined),
                    Validation::Failure(errors) => Effect::fail(errors),
                }
            })
            .context("Aggregating worker results");

    let env = TestEnv::new();
    let result = effect.run(&env);

    assert_eq!(result, Ok(TestEnum::A(6)));
}
```

## Documentation Requirements

### Code Documentation

- Comprehensive rustdoc for all public functions
- Document validation semantics and error accumulation
- Examples for common use cases (aggregation, JSON, databases)
- Explain relationship to Semigroup trait
- Document best practices for discriminant functions

### User Documentation

- New guide: `docs/guide/09-homogeneous-validation.md`
- Add section to README showing the pattern
- Update PHILOSOPHY.md with validation at boundaries pattern
- FAQ entry: "How to combine enum variants safely?"

### Example Guide Structure

```markdown
# Homogeneous Validation

When you have an enum where each variant forms a Semigroup, but
different variants can't be combined, use homogeneous validation.

## The Problem
[Explain the pattern and why it's needed]

## Pure Core, Imperative Shell
[Show how validation happens at boundaries]

## Examples
- Aggregation pipelines
- JSON config merging
- Database query results
- Plugin systems

## Best Practices
- Always validate at I/O boundaries
- Keep Semigroup pure (panic after validation)
- Use Validation for error accumulation
- Provide helpful error messages
```

## Implementation Notes

### Design Decisions

**Why not add `TrySemigroup` trait?**
- Violates mathematical semantics (semigroup is total operation)
- Increases complexity with two parallel traits
- Validation is a boundary concern, not a combining concern
- Keeps Semigroup pure and mathematically sound

**Why accumulate all errors instead of fail-fast?**
- Follows Stillwater's "fail completely" philosophy
- Better UX (report all type mismatches at once)
- Consistent with Validation type behavior
- Easier debugging in production

**Why generic discriminant function?**
- Works with `std::mem::discriminant` out of the box
- Allows custom discriminant logic if needed
- Enables comparison beyond Rust's type system
- Flexible for various enum patterns

**Why generic error constructor?**
- Allows users to provide context-rich errors
- Works with any error type (String, custom errors, etc.)
- Enables localization and custom formatting
- Composes with existing error types

### Gotchas

- Empty collections validate successfully (no pairs to check)
- Single-item collections always validate (nothing to compare)
- Discriminant function must be consistent (same value for same variant)
- After validation, Semigroup combine can safely panic on mismatch

### Best Practices

- **Validate at boundaries**: YAML → types, I/O → domain, workers → aggregation
- **Use with Effect**: Compose validation with I/O operations
- **Helpful errors**: Include context in error messages (index, expected, got)
- **Keep Semigroup pure**: Only call after validation, panic if mismatch

## Migration and Compatibility

### Breaking Changes

None - this is a pure addition to the validation module.

### Compatibility

- Fully backward compatible
- No changes to existing Semigroup trait
- Opt-in usage via validation module
- Composes with existing Validation/Effect types

### Adoption Path

```rust
// Before: Panic-based (crashes on type mismatch)
impl Semigroup for MyEnum {
    fn combine(self, other: Self) -> Self {
        match (self, other) {
            (A(x), A(y)) => A(x + y),
            _ => panic!("Type mismatch!"),
        }
    }
}

let items = vec![A(1), B(2), A(3)]; // Mixed types!
let result = items.into_iter().reduce(|a, b| a.combine(b)); // 💥 Panics

// After: Validation-based (graceful error handling)
impl Semigroup for MyEnum {
    fn combine(self, other: Self) -> Self {
        match (self, other) {
            (A(x), A(y)) => A(x + y),
            (B(x), B(y)) => B(x + y),
            _ => unreachable!("Call validate_homogeneous first"),
        }
    }
}

let items = vec![A(1), B(2), A(3)]; // Mixed types
let result = combine_homogeneous(
    items,
    |e| std::mem::discriminant(e),
    |idx, got, expected| format!("Type error at {}", idx),
);

match result {
    Validation::Success(combined) => { /* use combined */ }
    Validation::Failure(errors) => {
        // errors = ["Type error at 1"]
        // Handle gracefully, add to DLQ, etc.
    }
}
```

## Related Patterns

- **Semigroup**: Homogeneous validation enables safe usage
- **Validation**: Error accumulation for multiple type mismatches
- **Effect**: Composition at I/O boundaries
- **Pure core, imperative shell**: Validation at system boundaries

## Success Metrics

- Zero performance overhead compared to manual validation
- Property tests verify all errors are found
- Integration with Validation/Effect works seamlessly
- Documentation is clear with practical examples
- Positive user feedback from Prodigy and other users

## Future Enhancements

Potential additions in later versions:

- **Parallel validation**: Use rayon for large collections
- **Custom equality**: Beyond discriminant comparison
- **Validation cache**: Memoize discriminant checks
- **Error recovery**: Attempt coercion strategies
- **Metrics**: Track validation failures for monitoring
- **NonEmptyVec integration**: Guarantee at least one item

## Real-World Use Cases

### Prodigy MapReduce

```rust
// Aggregate results from parallel workers
let results: Vec<AggregateResult> = workers
    .into_iter()
    .map(|w| w.result())
    .collect();

// Validate before combining
let aggregated = combine_homogeneous(
    results,
    |r| std::mem::discriminant(r),
    |idx, got, expected| AggregateError::TypeMismatch {
        worker_id: idx,
        expected: discriminant_name(expected),
        got: discriminant_name(got),
    },
)?;
```

### JSON Config Merging

```rust
// Load configs from multiple sources
let configs = vec![
    load_default_config(),   // Object
    load_user_config(),      // Object
    load_env_config(),       // Object?
];

// Ensure all are objects before merging
validate_homogeneous(
    configs,
    |v| std::mem::discriminant(v),
    |idx, got, expected| {
        format!("Config {}: expected {}, got {}",
                idx,
                expected.discriminant_name(),
                got.discriminant_name())
    },
)
.map(|configs| merge_json_objects(configs))
```

### Database Query Results

```rust
// Combine results from sharded queries
let shard_results: Vec<QueryResult> = shards
    .par_iter()
    .map(|shard| shard.query(sql))
    .collect();

// Validate consistent result type
combine_homogeneous(
    shard_results,
    |r| std::mem::discriminant(r),
    |idx, _, _| format!("Shard {} returned different type", idx),
)
```
