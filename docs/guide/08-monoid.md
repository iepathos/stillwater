# Monoid: Identity Elements for Composition

The `Monoid` trait extends `Semigroup` by adding an identity element, enabling more powerful composition patterns without requiring explicit initial values.

## Overview

A **Monoid** is a `Semigroup` with an identity element. While a Semigroup provides an associative `combine` operation, a Monoid additionally provides an `empty()` element that acts as a neutral value for combination.

### Laws

For a type `M` to be a valid Monoid, it must satisfy:

1. **Associativity** (from Semigroup):
   ```rust
   a.combine(b).combine(c) == a.combine(b.combine(c))
   ```

2. **Right Identity**:
   ```rust
   a.combine(M::empty()) == a
   ```

3. **Left Identity**:
   ```rust
   M::empty().combine(a) == a
   ```

## Basic Examples

### Vec Monoid

Vectors form a monoid with the empty vector as identity:

```rust
use stillwater::{Monoid, Semigroup};

let v = vec![1, 2, 3];
let empty: Vec<i32> = Monoid::empty();

assert_eq!(v.clone().combine(empty.clone()), v);  // right identity
assert_eq!(empty.combine(v.clone()), v);          // left identity
```

### String Monoid

Strings form a monoid with the empty string as identity:

```rust
use stillwater::{Monoid, Semigroup};

let s = "hello".to_string();
let empty: String = Monoid::empty();

assert_eq!(s.clone().combine(empty.clone()), s);
assert_eq!(empty.combine(s.clone()), s);
```

### Option Monoid

Options lift a Semigroup into a Monoid with `None` as identity:

```rust
use stillwater::{Monoid, Semigroup};

let some_vec = Some(vec![1, 2, 3]);
let none: Option<Vec<i32>> = None;

// None is identity
assert_eq!(some_vec.clone().combine(none.clone()), some_vec);

// Some values combine their contents
let v1 = Some(vec![1, 2]);
let v2 = Some(vec![3, 4]);
assert_eq!(v1.combine(v2), Some(vec![1, 2, 3, 4]));
```

## Numeric Monoids

Since Rust primitives can't implement external traits and numbers have multiple valid monoid instances (addition vs multiplication), Stillwater provides wrapper types.

### Sum Monoid

Addition with 0 as identity:

```rust
use stillwater::monoid::{Sum, fold_all};
use stillwater::Semigroup;

let total = Sum(5).combine(Sum(10));
assert_eq!(total, Sum(15));

// Identity
let s = Sum(42);
let empty: Sum<i32> = Monoid::empty();
assert_eq!(s.combine(empty), Sum(42));

// Fold multiple values
let numbers = vec![Sum(1), Sum(2), Sum(3), Sum(4)];
let result = fold_all(numbers);
assert_eq!(result, Sum(10));
```

### Product Monoid

Multiplication with 1 as identity:

```rust
use stillwater::monoid::{Product, fold_all};

let result = Product(5).combine(Product(10));
assert_eq!(result, Product(50));

// Fold multiple values
let numbers = vec![Product(2), Product(3), Product(4)];
let total = fold_all(numbers);
assert_eq!(total, Product(24));
```

### Max and Min

Maximum and minimum operations (note: these are Semigroups but not Monoids without bounded types):

```rust
use stillwater::monoid::{Max, Min};
use stillwater::Semigroup;

let max = Max(5).combine(Max(10));
assert_eq!(max, Max(10));

let min = Min(5).combine(Min(10));
assert_eq!(min, Min(5));
```

For unbounded types, use `Option<Max<T>>` or `Option<Min<T>>` to get a Monoid:

```rust
use stillwater::{Monoid, Semigroup};
use stillwater::monoid::Max;

let m1: Option<Max<i32>> = Some(Max(5));
let m2: Option<Max<i32>> = Some(Max(10));
let empty: Option<Max<i32>> = Monoid::empty();

assert_eq!(m1.combine(m2), Some(Max(10)));
assert_eq!(m1.combine(empty), m1);
```

## Utility Functions

### fold_all

The `fold_all` function leverages the identity element to fold a collection without requiring an initial value:

```rust
use stillwater::monoid::fold_all;

// Combine vectors
let vecs = vec![vec![1, 2], vec![3, 4], vec![5]];
let result: Vec<i32> = fold_all(vecs);
assert_eq!(result, vec![1, 2, 3, 4, 5]);

// Combine strings
let strings = vec![
    "Hello".to_string(),
    " ".to_string(),
    "World".to_string(),
];
let result = fold_all(strings);
assert_eq!(result, "Hello World");
```

### reduce

Alias for `fold_all`:

```rust
use stillwater::monoid::reduce;

let vecs = vec![vec![1], vec![2], vec![3]];
let result: Vec<i32> = reduce(vecs);
assert_eq!(result, vec![1, 2, 3]);
```

## Tuple Monoids

Tuples of monoids are themselves monoids, combining component-wise:

```rust
use stillwater::{Monoid, Semigroup};
use stillwater::monoid::fold_all;

let t1 = (vec![1], "a".to_string());
let t2 = (vec![2], "b".to_string());

let result = t1.combine(t2);
assert_eq!(result, (vec![1, 2], "ab".to_string()));

// Identity
let empty: (Vec<i32>, String) = Monoid::empty();
assert_eq!(empty, (vec![], "".to_string()));

// Fold multiple tuples
let tuples = vec![
    (vec![1], "a".to_string()),
    (vec![2], "b".to_string()),
    (vec![3], "c".to_string()),
];
let result = fold_all(tuples);
assert_eq!(result, (vec![1, 2, 3], "abc".to_string()));
```

## Integration with Validation

Monoids work seamlessly with Validation for combining results:

```rust
use stillwater::{Validation, Monoid};
use stillwater::monoid::fold_all;

let validations = vec![
    Validation::success(vec![1]),
    Validation::success(vec![2, 3]),
    Validation::success(vec![4]),
];

let result = fold_all(validations);
assert_eq!(result, Validation::success(vec![1, 2, 3, 4]));
```

## Parallel Reduction

Monoids enable parallel reduction because the identity element allows splitting work:

```rust
use stillwater::monoid::{Sum, fold_all};

// These can be computed in parallel and combined
let chunk1 = vec![Sum(1), Sum(2), Sum(3)];
let chunk2 = vec![Sum(4), Sum(5), Sum(6)];

let result1 = fold_all(chunk1);
let result2 = fold_all(chunk2);

let total = result1.combine(result2);
assert_eq!(total, Sum(21));
```

## When to Use Monoid vs Semigroup

Use **Monoid** when:
- You need a default/empty value
- You want to fold without an initial value
- You're implementing parallel reduction
- You want more ergonomic API (`fold_all` vs `fold`)

Use **Semigroup** when:
- No natural identity element exists (e.g., non-empty lists)
- You always have at least one value to start with
- The type doesn't support a meaningful empty state

## Custom Monoid Implementations

Implement Monoid for your own types by first implementing Semigroup:

```rust
use stillwater::{Semigroup, Monoid};

#[derive(Debug, Clone, PartialEq)]
struct ValidationErrors(Vec<String>);

impl Semigroup for ValidationErrors {
    fn combine(mut self, other: Self) -> Self {
        self.0.extend(other.0);
        self
    }
}

impl Monoid for ValidationErrors {
    fn empty() -> Self {
        ValidationErrors(Vec::new())
    }
}

// Now you can use fold_all
let errors = vec![
    ValidationErrors(vec!["error1".to_string()]),
    ValidationErrors(vec!["error2".to_string()]),
];
let result = fold_all(errors);
assert_eq!(result.0, vec!["error1", "error2"]);
```

## Common Patterns

### Accumulating Results

```rust
use stillwater::monoid::{Sum, fold_all};

fn calculate_total(items: Vec<i32>) -> Sum<i32> {
    fold_all(items.into_iter().map(Sum))
}

let total = calculate_total(vec![1, 2, 3, 4, 5]);
assert_eq!(total, Sum(15));
```

### Combining Configurations

```rust
use stillwater::{Semigroup, Monoid, monoid::fold_all};

#[derive(Debug, Clone, PartialEq)]
struct Config {
    values: Vec<String>,
}

impl Semigroup for Config {
    fn combine(mut self, other: Self) -> Self {
        self.values.extend(other.values);
        self
    }
}

impl Monoid for Config {
    fn empty() -> Self {
        Config { values: Vec::new() }
    }
}

let configs = vec![
    Config { values: vec!["a".to_string()] },
    Config { values: vec!["b".to_string(), "c".to_string()] },
];

let merged = fold_all(configs);
assert_eq!(merged.values, vec!["a", "b", "c"]);
```

## Best Practices

1. **Verify laws**: Ensure your implementation satisfies identity and associativity
2. **Use property-based tests**: Test laws with many random inputs
3. **Choose appropriate wrapper**: Use Sum vs Product based on your domain
4. **Leverage fold_all**: More ergonomic than manual folding
5. **Document identity**: Make it clear what the empty value represents
6. **Consider performance**: Monoid operations should be cheap to enable frequent combination

## See Also

- [Semigroup Guide](01-semigroup.md) - Foundation for Monoid
- [Validation Guide](02-validation.md) - Using monoids for error accumulation
- [Effects Guide](03-effects.md) - Combining effects with monoids
