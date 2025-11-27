---
number: 034
title: Effect with_flat Tuple Combinators
category: foundation
priority: high
status: draft
dependencies: [024]
created: 2025-11-27
---

# Specification 034: Effect with_flat Tuple Combinators

**Category**: foundation
**Priority**: high
**Status**: draft
**Dependencies**: Spec 024 (Zero-Cost Effect Trait)

## Context

### The Problem

When chaining dependent effects using the `with` combinator, tuples become nested:

```rust
fetch_order(id)
    .with(|order| fetch_inventory(&order.sku))        // (Order, Inventory)
    .with(|(order, _)| fetch_customer(&order.cust_id)) // ((Order, Inventory), Customer)
    .with(|((order, _), _)| fetch_shipping(&order.addr))
    // (((Order, Inventory), Customer), Shipping) - deeply nested!
```

This nested tuple structure is awkward to destructure and work with:

```rust
.and_then(|(((order, inventory), customer), shipping)| {
    // Confusing tuple nesting
    process(&order, &inventory, &customer, &shipping)
})
```

### The Solution

Provide `with3`, `with4`, etc. combinators that flatten tuples at each step:

```rust
fetch_order(id)
    .with(|order| fetch_inventory(&order.sku))          // (Order, Inventory)
    .with3(|(order, _)| fetch_customer(&order.cust_id))  // (Order, Inventory, Customer) - FLAT!
    .with4(|(order, _, _)| fetch_shipping(&order.addr))  // (Order, Inventory, Customer, Shipping)
    .and_then(|(order, inventory, customer, shipping)| {
        // Clean flat tuple
        process(&order, &inventory, &customer, &shipping)
    })
```

### Philosophy Alignment

From PHILOSOPHY.md: *"Composition over complexity"* and *"Types guide, don't restrict"*

These combinators:
1. **Compose** with existing `with` - they extend, not replace
2. **Guide** developers toward flat, ergonomic tuple patterns
3. **Avoid macros** - pure combinator-based solution
4. **Zero-cost** - compile to the same code as manual nesting

### Prior Art

- **Scala ZIO**: `zipWith` with tuple flattening
- **Haskell**: Applicative instances with tuple flattening via `<*>`
- **TypeScript fp-ts**: `sequenceT` with flat tuples
- **Rust iterators**: No direct equivalent (iterators use different patterns)

## Objective

Add `with3`, `with4`, `with5`, `with6`, `with7`, and `with8` combinators to the Effect extension trait that:

1. Take a 2-tuple effect and add a third element, producing a flat 3-tuple
2. Maintain zero-cost abstraction (concrete types, no boxing)
3. Preserve the "reference to first value" semantics of `with`
4. Integrate seamlessly with existing combinator chains

## Requirements

### Functional Requirements

#### FR-1: with3 Combinator

Given an `Effect<(T1, T2), E, Env>`, provide `with3` that:
- Takes a function `f: FnOnce(&(T1, T2)) -> Effect<T3, E, Env>`
- Returns `Effect<(T1, T2, T3), E, Env>` - a flat 3-tuple

```rust
fn with3<E2, F>(self, f: F) -> With3<Self, F, E2>
where
    Self: Effect<Output = (T1, T2)>,
    Self::Output: Clone,
    F: FnOnce(&Self::Output) -> E2 + Send,
    E2: Effect<Error = Self::Error, Env = Self::Env>;
```

#### FR-2: with4 Combinator

Given an `Effect<(T1, T2, T3), E, Env>`, provide `with4` that:
- Takes a function `f: FnOnce(&(T1, T2, T3)) -> Effect<T4, E, Env>`
- Returns `Effect<(T1, T2, T3, T4), E, Env>` - a flat 4-tuple

#### FR-3: with5 through with8 Combinators

Continue the pattern up to 8 elements, covering most practical use cases:
- `with5`: `(T1..T4)` → `(T1..T5)`
- `with6`: `(T1..T5)` → `(T1..T6)`
- `with7`: `(T1..T6)` → `(T1..T7)`
- `with8`: `(T1..T7)` → `(T1..T8)`

#### FR-4: Value Semantics

All `withN` combinators must:
- Pass a **reference** to the accumulated tuple to the function
- **Clone** the accumulated values for the output tuple
- Require `Clone` bound on the tuple elements
- Execute sequentially (first effect, then second)

#### FR-5: Error Handling

- Fail-fast semantics: if either effect fails, the combined effect fails
- Error type must match between both effects
- No error accumulation (use Validation for that)

### Non-Functional Requirements

#### NFR-1: Zero-Cost

- Each combinator must return a concrete type (e.g., `With3<E, F, E2>`)
- No heap allocation for combinator creation
- Compile to equivalent manual code

#### NFR-2: Type Inference

- Type inference should work without explicit annotations in typical use
- Error messages should be clear when types don't match

#### NFR-3: Ergonomic Naming

- Use `with3`, `with4`, etc. naming convention
- The number indicates the **output tuple size**, not the input

## Acceptance Criteria

### Core Functionality

- [ ] **AC1**: `with3` method exists and produces flat 3-tuples
- [ ] **AC2**: `with4` method exists and produces flat 4-tuples
- [ ] **AC3**: `with5` through `with8` methods exist
- [ ] **AC4**: All methods require `Clone` on accumulated tuple
- [ ] **AC5**: Error in first effect propagates correctly
- [ ] **AC6**: Error in second effect propagates correctly

### Zero-Cost

- [ ] **AC7**: `With3<E, F, E2>` is a concrete struct implementing `Effect`
- [ ] **AC8**: `With4`, `With5`, etc. are concrete structs
- [ ] **AC9**: No heap allocations in combinator chain (verified by benchmark)

### Integration

- [ ] **AC10**: Works with `and_then` in same chain
- [ ] **AC11**: Works with `map` to transform final tuple
- [ ] **AC12**: Works with `boxed()` for type erasure
- [ ] **AC13**: Works with environment access (`asks`)

### Chaining

- [ ] **AC14**: Can chain: `effect.with(f).with3(g).with4(h)`
- [ ] **AC15**: Each step produces correct flat tuple type

## Technical Details

### Implementation Approach

#### With3 Combinator Type

```rust
// src/effect/combinators/with3.rs

use std::marker::PhantomData;
use crate::effect::trait_def::Effect;

/// An effect that combines a 2-tuple effect with another, producing a flat 3-tuple.
///
/// Created by [`EffectExt::with3`](crate::effect::ext::EffectExt::with3).
#[derive(Debug)]
pub struct With3<E, F, E2> {
    pub(crate) inner: E,
    pub(crate) f: F,
    pub(crate) _marker: PhantomData<E2>,
}

impl<E, F, E2, T1, T2, T3> Effect for With3<E, F, E2>
where
    E: Effect<Output = (T1, T2)>,
    T1: Clone + Send,
    T2: Clone + Send,
    F: FnOnce(&(T1, T2)) -> E2 + Send,
    E2: Effect<Output = T3, Error = E::Error, Env = E::Env>,
    T3: Send,
{
    type Output = (T1, T2, T3);
    type Error = E::Error;
    type Env = E::Env;

    async fn run(self, env: &Self::Env) -> Result<Self::Output, Self::Error> {
        let (v1, v2) = self.inner.run(env).await?;
        let v3 = (self.f)(&(v1.clone(), v2.clone())).run(env).await?;
        Ok((v1, v2, v3))
    }
}
```

#### With4 Combinator Type

```rust
// src/effect/combinators/with4.rs

use std::marker::PhantomData;
use crate::effect::trait_def::Effect;

/// An effect that combines a 3-tuple effect with another, producing a flat 4-tuple.
#[derive(Debug)]
pub struct With4<E, F, E2> {
    pub(crate) inner: E,
    pub(crate) f: F,
    pub(crate) _marker: PhantomData<E2>,
}

impl<E, F, E2, T1, T2, T3, T4> Effect for With4<E, F, E2>
where
    E: Effect<Output = (T1, T2, T3)>,
    T1: Clone + Send,
    T2: Clone + Send,
    T3: Clone + Send,
    F: FnOnce(&(T1, T2, T3)) -> E2 + Send,
    E2: Effect<Output = T4, Error = E::Error, Env = E::Env>,
    T4: Send,
{
    type Output = (T1, T2, T3, T4);
    type Error = E::Error;
    type Env = E::Env;

    async fn run(self, env: &Self::Env) -> Result<Self::Output, Self::Error> {
        let (v1, v2, v3) = self.inner.run(env).await?;
        let v4 = (self.f)(&(v1.clone(), v2.clone(), v3.clone())).run(env).await?;
        Ok((v1, v2, v3, v4))
    }
}
```

#### Extension Trait Methods

```rust
// In src/effect/ext.rs

impl<E: Effect> EffectExt for E {
    /// Combine a 2-tuple effect with another, producing a flat 3-tuple.
    ///
    /// # Example
    ///
    /// ```rust
    /// fetch_user(id)
    ///     .with(|user| fetch_orders(&user.id))        // (User, Orders)
    ///     .with3(|(user, _)| fetch_prefs(&user.id))   // (User, Orders, Prefs)
    /// ```
    fn with3<E2, F, T1, T2, T3>(self, f: F) -> With3<Self, F, E2>
    where
        Self: Effect<Output = (T1, T2)>,
        T1: Clone + Send,
        T2: Clone + Send,
        F: FnOnce(&(T1, T2)) -> E2 + Send,
        E2: Effect<Output = T3, Error = Self::Error, Env = Self::Env>,
        T3: Send,
    {
        With3 {
            inner: self,
            f,
            _marker: PhantomData,
        }
    }

    /// Combine a 3-tuple effect with another, producing a flat 4-tuple.
    fn with4<E2, F, T1, T2, T3, T4>(self, f: F) -> With4<Self, F, E2>
    where
        Self: Effect<Output = (T1, T2, T3)>,
        T1: Clone + Send,
        T2: Clone + Send,
        T3: Clone + Send,
        F: FnOnce(&(T1, T2, T3)) -> E2 + Send,
        E2: Effect<Output = T4, Error = Self::Error, Env = Self::Env>,
        T4: Send,
    {
        With4 {
            inner: self,
            f,
            _marker: PhantomData,
        }
    }

    // Continue pattern for with5, with6, with7, with8...
}
```

### Module Structure

```
src/effect/combinators/
├── mod.rs           # Add with3, with4, etc. modules
├── with.rs          # Existing With<E, F, E2>
├── with3.rs         # With3<E, F, E2>
├── with4.rs         # With4<E, F, E2>
├── with5.rs         # With5<E, F, E2>
├── with6.rs         # With6<E, F, E2>
├── with7.rs         # With7<E, F, E2>
└── with8.rs         # With8<E, F, E2>
```

### Why Not a Generic Solution?

A generic `withN` could theoretically work with HLists or const generics, but:

1. **HLists** add significant complexity and poor ergonomics
2. **Const generics** can't easily express "tuple of N elements"
3. **Explicit implementations** are clearer, zero-cost, and sufficient
4. **8 elements** covers virtually all practical cases

The explicit approach aligns with Stillwater's philosophy of pragmatism over theoretical purity.

## Dependencies

### Prerequisites
- Spec 024 (Zero-Cost Effect Trait) - for trait-based design

### Affected Components
- `EffectExt` trait - new methods
- Effect prelude - new exports
- Combinator module - new files

### External Dependencies
- None

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_with3_produces_flat_tuple() {
        let effect = pure::<_, String, ()>(1)
            .with(|_| pure(2))                  // (1, 2)
            .with3(|(a, b)| pure(*a + *b));     // (1, 2, 3)

        assert_eq!(effect.run(&()).await, Ok((1, 2, 3)));
    }

    #[tokio::test]
    async fn test_with4_produces_flat_tuple() {
        let effect = pure::<_, String, ()>(1)
            .with(|_| pure(2))
            .with3(|_| pure(3))
            .with4(|_| pure(4));

        assert_eq!(effect.run(&()).await, Ok((1, 2, 3, 4)));
    }

    #[tokio::test]
    async fn test_with3_error_in_first_effect() {
        let effect = fail::<(i32, i32), String, ()>("first error".into())
            .with3(|_| pure(3));

        assert_eq!(effect.run(&()).await, Err("first error".into()));
    }

    #[tokio::test]
    async fn test_with3_error_in_second_effect() {
        let effect = pure::<_, String, ()>((1, 2))
            .with3(|_| fail::<i32, _, _>("second error".into()));

        assert_eq!(effect.run(&()).await, Err("second error".into()));
    }

    #[tokio::test]
    async fn test_with_chain_full() {
        // Full chain from 2-tuple to 8-tuple
        let effect = pure::<_, String, ()>(1)
            .with(|_| pure(2))
            .with3(|_| pure(3))
            .with4(|_| pure(4))
            .with5(|_| pure(5))
            .with6(|_| pure(6))
            .with7(|_| pure(7))
            .with8(|_| pure(8));

        assert_eq!(effect.run(&()).await, Ok((1, 2, 3, 4, 5, 6, 7, 8)));
    }

    #[tokio::test]
    async fn test_with3_accesses_previous_values() {
        let effect = pure::<_, String, ()>(10)
            .with(|x| pure(*x * 2))                    // (10, 20)
            .with3(|(a, b)| pure(*a + *b));            // (10, 20, 30)

        assert_eq!(effect.run(&()).await, Ok((10, 20, 30)));
    }

    #[tokio::test]
    async fn test_with_chain_with_and_then() {
        let effect = pure::<_, String, ()>(1)
            .with(|_| pure(2))
            .with3(|_| pure(3))
            .and_then(|(a, b, c)| pure(a + b + c));

        assert_eq!(effect.run(&()).await, Ok(6));
    }

    #[tokio::test]
    async fn test_with_chain_with_map() {
        let effect = pure::<_, String, ()>(1)
            .with(|_| pure(2))
            .with3(|_| pure(3))
            .map(|(a, b, c)| a * b * c);

        assert_eq!(effect.run(&()).await, Ok(6));
    }

    #[tokio::test]
    async fn test_with3_with_environment() {
        struct Env { multiplier: i32 }

        let effect = asks::<_, String, _, _>(|env: &Env| env.multiplier)
            .with(|m| pure(*m * 2))
            .with3(|(m, _)| pure(*m * 3));

        let env = Env { multiplier: 5 };
        assert_eq!(effect.run(&env).await, Ok((5, 10, 15)));
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
        #[test]
        fn prop_with3_preserves_values(a: i32, b: i32, c: i32) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let effect = pure::<_, String, ()>((a, b))
                    .with3(|_| pure(c));

                let result = effect.run(&()).await;
                prop_assert_eq!(result, Ok((a, b, c)));
            })
        }

        #[test]
        fn prop_with4_preserves_values(a: i32, b: i32, c: i32, d: i32) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let effect = pure::<_, String, ()>((a, b, c))
                    .with4(|_| pure(d));

                let result = effect.run(&()).await;
                prop_assert_eq!(result, Ok((a, b, c, d)));
            })
        }
    }
}
```

### Benchmark Tests

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_with_chain(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("with_chain_4_elements", |b| {
        b.iter(|| {
            rt.block_on(async {
                let effect = pure::<_, String, ()>(1)
                    .with(|_| pure(2))
                    .with3(|_| pure(3))
                    .with4(|_| pure(4));
                black_box(effect.run(&()).await)
            })
        })
    });

    c.bench_function("manual_nested_and_then_4_elements", |b| {
        b.iter(|| {
            rt.block_on(async {
                let effect = pure::<_, String, ()>(1)
                    .and_then(|a| {
                        pure(2).and_then(move |b| {
                            pure(3).and_then(move |c| {
                                pure(4).map(move |d| (a, b, c, d))
                            })
                        })
                    });
                black_box(effect.run(&()).await)
            })
        })
    });
}
```

## Documentation Requirements

### Code Documentation

```rust
/// Combine a 2-tuple effect with another, producing a flat 3-tuple.
///
/// This combinator enables ergonomic chaining of dependent effects without
/// deeply nested tuple structures. The function receives a reference to the
/// accumulated tuple, and all values are cloned into the output.
///
/// # When to Use
///
/// Use `with3` when:
/// - You have an effect producing `(T1, T2)` from a previous `with` call
/// - You need to add a third dependent value
/// - You want a flat `(T1, T2, T3)` output instead of `((T1, T2), T3)`
///
/// # Example
///
/// ```rust
/// use stillwater::effect::prelude::*;
///
/// // Without with3 - nested tuples
/// let nested = fetch_order(id)
///     .with(|o| fetch_inventory(&o.sku))
///     .with(|(o, _)| fetch_customer(&o.cust_id));
/// // Type: Effect<((Order, Inventory), Customer), ...>
///
/// // With with3 - flat tuple
/// let flat = fetch_order(id)
///     .with(|o| fetch_inventory(&o.sku))
///     .with3(|(o, _)| fetch_customer(&o.cust_id));
/// // Type: Effect<(Order, Inventory, Customer), ...>
/// ```
///
/// # Chaining Pattern
///
/// ```rust
/// fetch_first()
///     .with(|a| fetch_second(a))     // (A, B)
///     .with3(|(a, _)| fetch_third(a)) // (A, B, C)
///     .with4(|_| fetch_fourth())      // (A, B, C, D)
///     .and_then(|(a, b, c, d)| process(a, b, c, d))
/// ```
///
/// # See Also
///
/// - [`with`](Self::with) - Combine two effects into a 2-tuple
/// - [`with4`](Self::with4) - Extend a 3-tuple to a 4-tuple
/// - [`zip`] - Combine independent effects (no dependency on first's value)
```

### User Documentation

Add to guide:

```markdown
## Flat Tuple Accumulation

When chaining dependent effects, use `with`, `with3`, `with4`, etc. to
accumulate values in flat tuples:

```rust
// Each step adds to the flat tuple
let profile = fetch_user(id)
    .with(|user| fetch_orders(&user.id))        // (User, Orders)
    .with3(|(user, _)| fetch_prefs(&user.id))   // (User, Orders, Prefs)
    .with4(|(user, _, _)| fetch_avatar(&user.avatar_url))
    .map(|(user, orders, prefs, avatar)| {
        // Clean destructuring!
        UserProfile { user, orders, prefs, avatar }
    });
```

### Why This Pattern?

Without flat tuple combinators:
```rust
// Nested tuples are hard to work with
.map(|(((user, orders), prefs), avatar)| ...)
```

With flat tuple combinators:
```rust
// Clean and readable
.map(|(user, orders, prefs, avatar)| ...)
```

### When to Use What

| Combinator | Input | Output | Use When |
|------------|-------|--------|----------|
| `with` | `T` | `(T, U)` | Starting a chain |
| `with3` | `(T1, T2)` | `(T1, T2, T3)` | Adding third element |
| `with4` | `(T1, T2, T3)` | `(T1..T4)` | Adding fourth element |
| ... | ... | ... | ... |
```

## Implementation Notes

### Clone Requirement

All tuple elements must implement `Clone` because:
1. The function receives a **reference** to the tuple
2. The output tuple needs **owned** values
3. We can't move out of the reference

This matches the existing `with` combinator behavior.

### Why Not Move Semantics?

We could theoretically pass ownership to the function and require it to return
the values it wants to keep. But this would be:
1. More complex for users
2. Error-prone (easy to forget to return a value)
3. Inconsistent with `with` which uses references

The clone approach is simpler and matches user expectations.

### Compile Time Considerations

Each `WithN` type adds to compile time. With 6 new types (with3-with8), this is
acceptable. The alternative (macros or generics) would likely have worse
compile-time impact.

## Migration and Compatibility

### Backward Compatibility

Purely additive - no breaking changes.

### Migration Path

Users can incrementally adopt `withN` to flatten existing nested tuple chains:

```rust
// Before: nested tuples
effect.with(f).with(g).with(h)
// Type: (((A, B), C), D)

// After: flat tuples (incremental change)
effect.with(f).with3(g).with4(h)
// Type: (A, B, C, D)
```

---

*"Flat is better than nested."* — Zen of Python (applies here too)
