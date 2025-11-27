---
number: 027
title: Effect zip/zip_with Combinators
category: foundation
priority: high
status: refined
dependencies: [024]
created: 2025-11-27
updated: 2025-11-27
---

# Specification 027: Effect zip/zip_with Combinators

**Category**: foundation
**Priority**: high
**Status**: refined
**Dependencies**: Spec 024 (Zero-Cost Effect Trait)

## Context

### The Problem

When combining independent effects, the current API forces users to use `and_then` with nested closures:

```rust
// Current: Awkward nesting for independent effects
let effect = fetch_user(id)
    .and_then(|user| {
        fetch_orders(id)
            .and_then(|orders| {
                fetch_preferences(id)
                    .map(|prefs| (user, orders, prefs))
            })
    });
```

This has several problems:

1. **Rightward drift**: Each level adds indentation
2. **False dependency**: `and_then` implies sequential dependency, but these are independent
3. **Verbosity**: Simple combination requires boilerplate
4. **Refactoring hazard**: Hard to reorder or parallelize

### The Solution

`zip` and `zip_with` combinators express **independent combination** clearly:

```rust
// With zip: Clear, flat, expresses independence
let effect = fetch_user(id)
    .zip(fetch_orders(id))
    .zip(fetch_preferences(id))
    .map(|((user, orders), prefs)| (user, orders, prefs));

// With zip_with: Even cleaner
let effect = fetch_user(id)
    .zip_with(fetch_orders(id), |user, orders| UserWithOrders { user, orders });

// With zip3/zip4: Best for multiple independents
let effect = zip3(
    fetch_user(id),
    fetch_orders(id),
    fetch_preferences(id),
).map(|(user, orders, prefs)| UserProfile { user, orders, prefs });
```

### Prior Art

This pattern is universal in FP:

- **Haskell**: `liftA2`, `(<*>)`, `zip`
- **Scala ZIO**: `zip`, `zipWith`, `zipPar`
- **Rust futures**: `join!`, `try_join!`, `Future::zip`
- **Rust iterators**: `Iterator::zip`

### Relationship to `with` Combinator

The existing `with` combinator is similar but serves a different purpose:

| Aspect | `zip` | `with` |
|--------|-------|--------|
| Independence | Both effects are independent | Second effect may depend on first's value |
| Signature | `zip(other: E2)` | `with(f: FnOnce(&T) -> E2)` |
| First value | Consumed | Passed by reference, then cloned |
| Use case | Combine unrelated effects | Access first result to compute second |

```rust
// zip: independent effects
fetch_user(id).zip(fetch_settings(id))

// with: second effect uses first's value by reference
fetch_user(id).with(|user| fetch_avatar(&user.avatar_url))
```

Use `zip` when effects are truly independent. Use `with` when the second effect needs to reference the first result.

## Objective

Add `zip` and `zip_with` combinators to `Effect` that enable clean composition of independent effects, following the zero-cost pattern established in Spec 024.

## Requirements

### Functional Requirements

#### FR1: Basic zip Combinator

- **MUST** provide `zip(self, other) -> Effect<(T, U), E, Env>` method
- **MUST** execute effects sequentially (first, then second) for predictable behavior
- **MUST** return both values as a tuple on success
- **MUST** fail fast on first error
- **MUST** return a concrete type (zero-cost) per Spec 024

```rust
// On EffectExt trait
fn zip<E2>(self, other: E2) -> Zip<Self, E2>
where
    E2: Effect<Error = Self::Error, Env = Self::Env>;
```

**Note on Execution Order**: Although `zip` semantically expresses *independence* (neither effect depends on the other's result), the implementation executes sequentially for simplicity and predictability. Use `zip_par` when concurrent execution is desired.

#### FR2: zip_with Combinator

- **MUST** provide `zip_with(self, other, f) -> Effect<R, E, Env>` method
- **MUST** apply function `f` to both results
- **MUST** be equivalent to `self.zip(other).map(|(a, b)| f(a, b))`
- **SHOULD** be more efficient than `zip` + `map` (single combinator struct)

```rust
fn zip_with<E2, R, F>(self, other: E2, f: F) -> ZipWith<Self, E2, F>
where
    E2: Effect<Error = Self::Error, Env = Self::Env>,
    F: FnOnce(Self::Output, E2::Output) -> R + Send;
```

#### FR3: Tuple zip Functions

- **MUST** provide `zip3(e1, e2, e3) -> Zip3<E1, E2, E3>` returning concrete type
- **MUST** provide `zip4(e1, e2, e3, e4) -> Zip4<E1, E2, E3, E4>` returning concrete type
- **SHOULD** provide up to `zip8` for common use cases (all as concrete types)
- **MUST** require all effects have same `Error` and `Env` types
- **MUST** return flat tuples `(T1, T2, T3)` not nested `((T1, T2), T3)`

#### FR4: Parallel Semantics (Optional Enhancement)

- **SHOULD** provide `zip_par` variant that guarantees concurrent execution via `tokio::join!`
- **SHOULD** provide `zip3_par`, `zip4_par` etc. for multi-way concurrent zips
- **MUST** document execution semantics clearly for both sequential and parallel variants

#### FR5: Error Handling

- **MUST** fail on first error (fail-fast semantics)
- **MUST NOT** accumulate errors (use `Validation` for that)
- **MUST** propagate error type unchanged

### Non-Functional Requirements

#### NFR1: Zero-Cost

- Combinators MUST return concrete types (no boxing)
- Chained zips MUST NOT allocate
- Size of `Zip<A, B>` MUST equal `size_of::<A>() + size_of::<B>()`

#### NFR2: Type Inference

- Type inference SHOULD work without explicit annotations
- Error messages SHOULD be clear when types don't match

#### NFR3: Integration

- MUST work with Spec 024 trait-based Effect design
- MUST work with `BoxedEffect` via `.boxed()` after zipping

## Acceptance Criteria

### Core zip

- [ ] **AC1**: `zip` method exists on `EffectExt`
- [ ] **AC2**: `pure(1).zip(pure(2))` returns `pure((1, 2))`
- [ ] **AC3**: `fail("e").zip(pure(2))` returns `fail("e")`
- [ ] **AC4**: `pure(1).zip(fail("e"))` returns `fail("e")`
- [ ] **AC5**: `Zip<A, B>` implements `Effect` trait

### zip_with

- [ ] **AC6**: `zip_with` method exists on `EffectExt`
- [ ] **AC7**: `pure(1).zip_with(pure(2), |a, b| a + b)` returns `pure(3)`
- [ ] **AC8**: `ZipWith<A, B, F>` implements `Effect` trait

### Tuple zips

- [ ] **AC9**: `zip3` function exists and returns concrete `Zip3<E1, E2, E3>` type
- [ ] **AC10**: `zip4` function exists and returns concrete `Zip4<E1, E2, E3, E4>` type
- [ ] **AC11**: `zip3(pure(1), pure(2), pure(3))` returns `pure((1, 2, 3))` - flat tuple, not nested
- [ ] **AC12**: `Zip3`, `Zip4`, etc. implement `Effect` trait directly (not via composition)

### Zero-Cost

- [ ] **AC13**: `Zip<Pure<i32>, Pure<i32>>` is stack-allocated
- [ ] **AC14**: Chaining 5 zips produces no heap allocations
- [ ] **AC15**: Benchmark shows zip chains equivalent to manual async

### Integration

- [ ] **AC16**: Works with `BoxedEffect` via `.boxed()`
- [ ] **AC17**: Works with environment access (`asks`)
- [ ] **AC18**: Works with `and_then` in same chain

## Technical Details

### Implementation Approach

#### Zip Combinator Type

```rust
// src/effect/combinators/zip.rs

/// Combines two effects, running them and returning both results.
///
/// This is zero-cost: no heap allocation occurs.
pub struct Zip<E1, E2> {
    first: E1,
    second: E2,
}

impl<E1, E2> Zip<E1, E2> {
    pub fn new(first: E1, second: E2) -> Self {
        Zip { first, second }
    }
}

impl<E1, E2> Effect for Zip<E1, E2>
where
    E1: Effect,
    E2: Effect<Error = E1::Error, Env = E1::Env>,
{
    type Output = (E1::Output, E2::Output);
    type Error = E1::Error;
    type Env = E1::Env;

    fn run(self, env: &Self::Env) -> impl Future<Output = Result<Self::Output, Self::Error>> + Send {
        async move {
            // Sequential execution (simplest, correct)
            let first_result = self.first.run(env).await?;
            let second_result = self.second.run(env).await?;
            Ok((first_result, second_result))
        }
    }
}
```

#### ZipWith Combinator Type

```rust
// src/effect/combinators/zip_with.rs

/// Combines two effects with a function.
///
/// More efficient than `zip().map()` as it's a single combinator.
pub struct ZipWith<E1, E2, F> {
    first: E1,
    second: E2,
    f: F,
}

impl<E1, E2, F, R> Effect for ZipWith<E1, E2, F>
where
    E1: Effect,
    E2: Effect<Error = E1::Error, Env = E1::Env>,
    F: FnOnce(E1::Output, E2::Output) -> R + Send,
    R: Send,
{
    type Output = R;
    type Error = E1::Error;
    type Env = E1::Env;

    fn run(self, env: &Self::Env) -> impl Future<Output = Result<R, Self::Error>> + Send {
        async move {
            let first_result = self.first.run(env).await?;
            let second_result = self.second.run(env).await?;
            Ok((self.f)(first_result, second_result))
        }
    }
}
```

#### Extension Trait Methods

```rust
// In src/effect/ext.rs

impl<E: Effect> EffectExt for E {
    /// Combine this effect with another, returning both results as a tuple.
    ///
    /// # Example
    ///
    /// ```rust
    /// let effect = fetch_user(id).zip(fetch_settings(id));
    /// // Returns Effect<(User, Settings), Error, Env>
    /// ```
    fn zip<E2>(self, other: E2) -> Zip<Self, E2>
    where
        E2: Effect<Error = Self::Error, Env = Self::Env>,
    {
        Zip::new(self, other)
    }

    /// Combine this effect with another using a function.
    ///
    /// # Example
    ///
    /// ```rust
    /// let effect = fetch_user(id)
    ///     .zip_with(fetch_settings(id), |user, settings| {
    ///         UserWithSettings { user, settings }
    ///     });
    /// ```
    fn zip_with<E2, R, F>(self, other: E2, f: F) -> ZipWith<Self, E2, F>
    where
        E2: Effect<Error = Self::Error, Env = Self::Env>,
        F: FnOnce(Self::Output, E2::Output) -> R + Send,
        R: Send,
    {
        ZipWith::new(self, other, f)
    }
}
```

#### Tuple zip Functions (Concrete Types)

```rust
// src/effect/combinators/zip.rs

/// Combines three effects into a flat tuple.
pub struct Zip3<E1, E2, E3> {
    e1: E1,
    e2: E2,
    e3: E3,
}

impl<E1, E2, E3> Zip3<E1, E2, E3> {
    pub fn new(e1: E1, e2: E2, e3: E3) -> Self {
        Zip3 { e1, e2, e3 }
    }
}

impl<E1, E2, E3> Effect for Zip3<E1, E2, E3>
where
    E1: Effect,
    E2: Effect<Error = E1::Error, Env = E1::Env>,
    E3: Effect<Error = E1::Error, Env = E1::Env>,
{
    type Output = (E1::Output, E2::Output, E3::Output);
    type Error = E1::Error;
    type Env = E1::Env;

    fn run(self, env: &Self::Env) -> impl Future<Output = Result<Self::Output, Self::Error>> + Send {
        async move {
            let r1 = self.e1.run(env).await?;
            let r2 = self.e2.run(env).await?;
            let r3 = self.e3.run(env).await?;
            Ok((r1, r2, r3))
        }
    }
}

/// Combines four effects into a flat tuple.
pub struct Zip4<E1, E2, E3, E4> {
    e1: E1,
    e2: E2,
    e3: E3,
    e4: E4,
}

impl<E1, E2, E3, E4> Effect for Zip4<E1, E2, E3, E4>
where
    E1: Effect,
    E2: Effect<Error = E1::Error, Env = E1::Env>,
    E3: Effect<Error = E1::Error, Env = E1::Env>,
    E4: Effect<Error = E1::Error, Env = E1::Env>,
{
    type Output = (E1::Output, E2::Output, E3::Output, E4::Output);
    type Error = E1::Error;
    type Env = E1::Env;

    fn run(self, env: &Self::Env) -> impl Future<Output = Result<Self::Output, Self::Error>> + Send {
        async move {
            let r1 = self.e1.run(env).await?;
            let r2 = self.e2.run(env).await?;
            let r3 = self.e3.run(env).await?;
            let r4 = self.e4.run(env).await?;
            Ok((r1, r2, r3, r4))
        }
    }
}

// Continue pattern for Zip5 through Zip8...
```

```rust
// src/effect/constructors.rs

/// Combine three effects into a tuple.
///
/// # Example
///
/// ```rust
/// let effect = zip3(
///     fetch_user(id),
///     fetch_orders(id),
///     fetch_preferences(id),
/// );
/// // Returns Zip3<...> with Output = (User, Vec<Order>, Preferences)
/// ```
pub fn zip3<E1, E2, E3>(e1: E1, e2: E2, e3: E3) -> Zip3<E1, E2, E3>
where
    E1: Effect,
    E2: Effect<Error = E1::Error, Env = E1::Env>,
    E3: Effect<Error = E1::Error, Env = E1::Env>,
{
    Zip3::new(e1, e2, e3)
}

/// Combine four effects into a tuple.
pub fn zip4<E1, E2, E3, E4>(e1: E1, e2: E2, e3: E3, e4: E4) -> Zip4<E1, E2, E3, E4>
where
    E1: Effect,
    E2: Effect<Error = E1::Error, Env = E1::Env>,
    E3: Effect<Error = E1::Error, Env = E1::Env>,
    E4: Effect<Error = E1::Error, Env = E1::Env>,
{
    Zip4::new(e1, e2, e3, e4)
}

// Continue pattern for zip5 through zip8...
```

#### Macro for zip_all (Deferred)

A `zip_all!` macro for arbitrary arity is deferred to a future enhancement. The complexity of tuple flattening in declarative macros makes this better suited for:
1. A procedural macro, or
2. Manual `zip3`-`zip8` functions (recommended approach)

The concrete `Zip3`, `Zip4`, etc. types cover most practical use cases without macro complexity.

### Parallel zip Variant (Future Enhancement)

```rust
/// Parallel zip - guarantees concurrent execution.
///
/// Unlike `zip`, this uses `tokio::join!` to run both effects concurrently.
pub struct ZipPar<E1, E2> {
    first: E1,
    second: E2,
}

impl<E1, E2> Effect for ZipPar<E1, E2>
where
    E1: Effect,
    E2: Effect<Error = E1::Error, Env = E1::Env>,
{
    type Output = (E1::Output, E2::Output);
    type Error = E1::Error;
    type Env = E1::Env;

    fn run(self, env: &Self::Env) -> impl Future<Output = Result<Self::Output, Self::Error>> + Send {
        async move {
            let (first_result, second_result) = tokio::join!(
                self.first.run(env),
                self.second.run(env),
            );
            Ok((first_result?, second_result?))
        }
    }
}
```

### Module Structure

```
src/effect/
├── combinators/
│   ├── mod.rs
│   ├── zip.rs           # Zip<E1, E2>
│   ├── zip_with.rs      # ZipWith<E1, E2, F>
│   └── zip_par.rs       # ZipPar<E1, E2> (optional)
├── ext.rs               # Add zip, zip_with methods
├── constructors.rs      # Add zip3, zip4, etc.
└── prelude.rs           # Export zip functions
```

## Dependencies

### Prerequisites
- Spec 024 (Zero-Cost Effect Trait) - for trait-based design

### Affected Components
- `EffectExt` trait - new methods
- Effect prelude - new exports

### External Dependencies
- `tokio` for parallel variant (optional)

## Testing Strategy

**Note on API**: Tests use `.execute(&env)` which is a convenience wrapper around `.run(env).await`. Both are equivalent:
- `effect.run(&env).await` - direct trait method
- `effect.execute(&env).await` - ergonomic helper (if provided)

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_zip_both_success() {
        let effect = pure::<_, String, ()>(1).zip(pure(2));
        assert_eq!(effect.execute(&()).await, Ok((1, 2)));
    }

    #[tokio::test]
    async fn test_zip_first_fails() {
        let effect = fail::<i32, _, ()>("error").zip(pure(2));
        assert_eq!(effect.execute(&()).await, Err("error"));
    }

    #[tokio::test]
    async fn test_zip_second_fails() {
        let effect = pure::<_, String, ()>(1).zip(fail("error"));
        assert_eq!(effect.execute(&()).await, Err("error"));
    }

    #[tokio::test]
    async fn test_zip_with() {
        let effect = pure::<_, String, ()>(2)
            .zip_with(pure(3), |a, b| a * b);
        assert_eq!(effect.execute(&()).await, Ok(6));
    }

    #[tokio::test]
    async fn test_zip3() {
        let effect = zip3(
            pure::<_, String, ()>(1),
            pure(2),
            pure(3),
        );
        assert_eq!(effect.execute(&()).await, Ok((1, 2, 3)));
    }

    #[tokio::test]
    async fn test_zip4() {
        let effect = zip4(
            pure::<_, String, ()>(1),
            pure(2),
            pure(3),
            pure(4),
        );
        assert_eq!(effect.execute(&()).await, Ok((1, 2, 3, 4)));
    }

    #[tokio::test]
    async fn test_zip_chain() {
        let effect = pure::<_, String, ()>(1)
            .zip(pure(2))
            .zip(pure(3))
            .map(|((a, b), c)| a + b + c);
        assert_eq!(effect.execute(&()).await, Ok(6));
    }

    #[tokio::test]
    async fn test_zip_with_and_then() {
        let effect = pure::<_, String, ()>(1)
            .zip(pure(2))
            .and_then(|(a, b)| pure(a + b));
        assert_eq!(effect.execute(&()).await, Ok(3));
    }

    #[tokio::test]
    async fn test_zip_with_environment() {
        struct Env { multiplier: i32 }

        let effect = asks::<_, String, _, _>(|env: &Env| env.multiplier)
            .zip(pure(10));

        let env = Env { multiplier: 5 };
        assert_eq!(effect.execute(&env).await, Ok((5, 10)));
    }

    #[tokio::test]
    async fn test_zip_boxed() {
        let effects: Vec<BoxedEffect<i32, String, ()>> = vec![
            pure(1).zip(pure(2)).map(|(a, b)| a + b).boxed(),
            pure(3).zip(pure(4)).map(|(a, b)| a + b).boxed(),
        ];

        let mut results = Vec::new();
        for effect in effects {
            results.push(effect.execute(&()).await.unwrap());
        }
        assert_eq!(results, vec![3, 7]);
    }
}
```

### Property Tests

```rust
#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        // zip is associative up to tuple restructuring
        #[test]
        fn prop_zip_associative(a: i32, b: i32, c: i32) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let left = pure::<_, String, ()>(a)
                    .zip(pure(b))
                    .zip(pure(c))
                    .map(|((x, y), z)| (x, y, z));

                let right = pure::<_, String, ()>(a)
                    .zip(pure(b).zip(pure(c)))
                    .map(|(x, (y, z))| (x, y, z));

                prop_assert_eq!(
                    left.execute(&()).await,
                    right.execute(&()).await
                );
            })
        }

        // zip with pure is identity-like
        #[test]
        fn prop_zip_with_unit(x: i32) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let effect = pure::<_, String, ()>(x)
                    .zip(pure(()))
                    .map(|(v, _)| v);

                prop_assert_eq!(effect.execute(&()).await, Ok(x));
            })
        }
    }
}
```

### Benchmark Tests

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_zip(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("zip_2_effects", |b| {
        b.iter(|| {
            rt.block_on(async {
                let effect = pure::<_, String, ()>(1).zip(pure(2));
                black_box(effect.execute(&()).await)
            })
        })
    });

    c.bench_function("zip_5_effects", |b| {
        b.iter(|| {
            rt.block_on(async {
                let effect = pure::<_, String, ()>(1)
                    .zip(pure(2))
                    .zip(pure(3))
                    .zip(pure(4))
                    .zip(pure(5));
                black_box(effect.execute(&()).await)
            })
        })
    });

    c.bench_function("and_then_chain_equivalent", |b| {
        b.iter(|| {
            rt.block_on(async {
                let effect = pure::<_, String, ()>(1)
                    .and_then(|a| pure(2).and_then(move |b| pure((a, b))));
                black_box(effect.execute(&()).await)
            })
        })
    });
}
```

## Documentation Requirements

### Code Documentation

```rust
/// Combine this effect with another, returning both results as a tuple.
///
/// `zip` is useful when you have two independent effects and need both results.
/// Unlike `and_then`, which expresses sequential dependency, `zip` expresses
/// that both effects are independent and can potentially run in parallel.
///
/// # Execution Order
///
/// The current implementation runs effects sequentially for simplicity.
/// Use `zip_par` if you need guaranteed parallel execution.
///
/// # Error Handling
///
/// Uses fail-fast semantics: if either effect fails, the combined effect
/// fails with that error. Errors are not accumulated.
///
/// For error accumulation, use `Validation::all()` instead.
///
/// # Example
///
/// ```rust
/// use stillwater::effect::prelude::*;
///
/// // Independent effects - order doesn't matter
/// let effect = fetch_user(id)
///     .zip(fetch_settings(id))
///     .map(|(user, settings)| UserProfile { user, settings });
///
/// // Chain multiple zips
/// let effect = fetch_a()
///     .zip(fetch_b())
///     .zip(fetch_c())
///     .map(|((a, b), c)| combine(a, b, c));
///
/// // Or use zip3 for cleaner syntax
/// let effect = zip3(fetch_a(), fetch_b(), fetch_c())
///     .map(|(a, b, c)| combine(a, b, c));
/// ```
///
/// # See Also
///
/// - `zip_with` - combine with a function directly
/// - `zip3`, `zip4`, etc. - combine multiple effects
/// - `and_then` - for dependent/sequential effects
/// - `par_all` - for parallel execution with collection input
```

### User Documentation

Add to guide documentation:

```markdown
## Combining Independent Effects

When you have multiple effects that don't depend on each other, use `zip`:

```rust
// These fetches are independent
let profile = fetch_user(id)
    .zip(fetch_orders(id))
    .zip(fetch_preferences(id))
    .map(|((user, orders), prefs)| UserProfile { user, orders, prefs });
```

Compare to `and_then` which expresses dependency:

```rust
// Order depends on user - must use and_then
let orders = fetch_user(id)
    .and_then(|user| fetch_orders_for(user.id));
```

### zip vs and_then

| Use `zip` when... | Use `and_then` when... |
|-------------------|------------------------|
| Effects are independent | Later effect needs earlier result |
| Order doesn't matter | Specific order required |
| Combining unrelated data | Building on previous result |
```

## Implementation Notes

### Design Decisions

| Decision | Rationale |
|----------|-----------|
| Sequential execution for `zip` | Simpler, predictable ordering, no runtime dependency, easy to reason about |
| Separate `zip_par` for concurrent | Explicit about execution semantics; users opt-in to concurrency |
| Concrete `ZipN` types (not `impl Effect`) | Enables type naming, consistent with `Zip<E1, E2>` pattern |
| `zip_with` as separate type | More efficient than `zip().map()` - single combinator struct |
| zipN functions up to 8 | Covers most practical cases without macro complexity |
| Flat tuples `(a, b, c)` not nested | Ergonomic access; consistent with FP conventions |

### Future Enhancements

1. **`zip_par` combinator**: Concurrent execution via `tokio::join!` (defined in spec, implement when needed)
2. **`zip_all!` procedural macro**: Arbitrary arity with auto-flattening (deferred - concrete types preferred)
3. **HList-based zip**: Type-safe arbitrary arity (complex, low priority)
4. **Applicative syntax**: `(effect1, effect2, effect3).tupled()` via trait extension
5. **`zip_into` variant**: Combine effects with different error types via `Into<E>` conversion

## Migration and Compatibility

- **Breaking changes**: None (additive)
- **New API surface**: `zip`, `zip_with`, `zip3`-`zip8` functions

---

*"Express independence, not false dependency."*
