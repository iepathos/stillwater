---
number: 033
title: Effect Module Consolidation
category: foundation
priority: critical
status: draft
dependencies: [024]
created: 2025-11-27
---

# Specification 033: Effect Module Consolidation

**Category**: foundation
**Priority**: critical
**Status**: draft
**Dependencies**: Spec 024 (Zero-Cost Effect Trait)

## Context

Stillwater currently has two Effect implementations:

1. **`src/effect.rs`** - The original boxed implementation (~2500 lines)
   - `Effect<T, E, Env>` struct that allocates per combinator
   - Full feature set: retry, context, tracing, helper combinators

2. **`src/effect_v2/`** - The new zero-cost implementation (Spec 024)
   - `Effect` trait with zero-cost combinators
   - `BoxedEffect` for opt-in type erasure
   - Missing several features from the original

Maintaining two implementations is unnecessary complexity. The zero-cost implementation should become the only Effect system, with all features from the original ported over.

### Feature Gap Analysis

| Feature | Old `effect.rs` | New `effect_v2` | Action |
|---------|-----------------|-----------------|--------|
| Core combinators | `map`, `and_then`, `map_err`, `or_else` | ✅ Same | None |
| Reader pattern | `ask`, `asks`, `local` | ✅ Same | None |
| Parallel | `par_all`, `par_try_all`, `race`, `par_all_limit` | `par_all`, `par_try_all`, `race`, `par2-4` | Add `par_all_limit` |
| Helper combinators | `tap`, `check`, `with`, `and_then_auto`, `and_then_ref` | ❌ Missing | Port |
| Retry | `retry`, `retry_if`, `retry_with_hooks`, `with_timeout` | ❌ Missing | Port |
| Context errors | `EffectContext` trait, `.context()` | ❌ Missing | Port |
| Tracing | `.instrument()` | ❌ Missing | Port |
| Resource management | None | ✅ `bracket`, `bracket_simple` | None |
| Validation | `from_validation` | ❌ Missing | Port |

## Objective

Consolidate to a single Effect implementation by:

1. Adding all missing features to `effect_v2`
2. Removing the old `effect.rs` module
3. Renaming `effect_v2` to `effect`
4. Updating all public exports and prelude

## Requirements

### Functional Requirements

#### FR1: Port Helper Combinators to EffectExt

Add to `src/effect_v2/ext.rs`:

- **`tap`**: Perform side effect, return original value
- **`check`**: Fail with error if predicate is false
- **`with`**: Combine with another effect, returning tuple
- **`and_then_auto`**: Chain with automatic error conversion
- **`and_then_ref`**: Chain by borrowing value, return original

#### FR2: Port Retry Support

Create `src/effect_v2/retry.rs`:

- **`retry`**: Retry operation with policy, returning `RetryExhausted`
- **`retry_if`**: Retry only when predicate returns true for error
- **`retry_with_hooks`**: Retry with observability callbacks
- **`with_timeout`**: Add timeout to effect

Must integrate with existing `crate::retry::{RetryPolicy, RetryEvent, RetryExhausted, TimeoutError}`.

#### FR3: Port Context Error Support

Create `src/effect_v2/context.rs`:

- **`EffectContext` trait**: Extension trait for adding context
- **`.context(msg)`**: Wrap errors in `ContextError`
- Chainable context for `Effect<T, ContextError<E>, Env>`

Must integrate with existing `crate::ContextError`.

#### FR4: Port Tracing Support

Add to `src/effect_v2/ext.rs` (feature-gated):

- **`.instrument(span)`**: Wrap effect in tracing span

Feature-gated behind `#[cfg(feature = "tracing")]`.

#### FR5: Add Missing Constructors and Parallel Functions

- **`from_validation`**: Convert `Validation<T, E>` to effect
- **`par_all_limit`**: Parallel execution with concurrency limit

#### FR6: Module Restructuring

1. Delete `src/effect.rs`
2. Rename `src/effect_v2/` to `src/effect/`
3. Update `src/lib.rs`:
   - Change `pub mod effect_v2;` to `pub mod effect;`
   - Update re-exports to use new module
   - Update prelude

#### FR7: Public API Updates

Update `src/lib.rs` exports:

```rust
// Current
pub use effect::{Effect, EffectContext};

// New
pub use effect::{
    Effect, EffectExt, BoxedEffect,
    pure, fail, from_fn, from_async, from_result, from_validation,
    ask, asks, local,
    bracket, bracket_simple,
    par_all, par_try_all, race, par_all_limit,
};
pub use effect::context::EffectContext;
```

Update prelude:

```rust
pub mod prelude {
    pub use crate::effect::prelude::*;
    // ... other prelude items
}
```

### Non-Functional Requirements

#### NFR1: Backward Compatibility

- Provide `compat` module with deprecated type aliases
- `LegacyEffect<T, E, Env>` = `BoxedEffect<T, E, Env>`
- Clear deprecation warnings pointing to new API

#### NFR2: Test Coverage

- All ported features must have equivalent test coverage
- Migrate tests from old `effect.rs` to new module
- Ensure all existing tests pass

#### NFR3: Documentation

- All new/ported items must have rustdoc
- Examples in documentation must compile
- Internal documentation for maintainers

## Acceptance Criteria

### Helper Combinators

- [ ] **AC1**: `tap` implemented and tested
- [ ] **AC2**: `check` implemented and tested
- [ ] **AC3**: `with` implemented and tested
- [ ] **AC4**: `and_then_auto` implemented and tested
- [ ] **AC5**: `and_then_ref` implemented and tested

### Retry Support

- [ ] **AC6**: `retry` implemented with `RetryPolicy` integration
- [ ] **AC7**: `retry_if` implemented with predicate support
- [ ] **AC8**: `retry_with_hooks` implemented with callback support
- [ ] **AC9**: `with_timeout` implemented with `TimeoutError`
- [ ] **AC10**: All retry tests from old module pass

### Context Errors

- [ ] **AC11**: `EffectContext` trait implemented
- [ ] **AC12**: `.context()` works on any `Effect`
- [ ] **AC13**: Chained `.context()` on `Effect<T, ContextError<E>, Env>` works
- [ ] **AC14**: All context tests from old module pass

### Tracing

- [ ] **AC15**: `.instrument()` implemented (feature-gated)
- [ ] **AC16**: Tracing tests pass with `--features tracing`

### Constructors and Parallel

- [ ] **AC17**: `from_validation` implemented
- [ ] **AC18**: `par_all_limit` implemented with concurrency control

### Module Structure

- [ ] **AC19**: `src/effect.rs` deleted
- [ ] **AC20**: `src/effect_v2/` renamed to `src/effect/`
- [ ] **AC21**: `src/lib.rs` exports updated
- [ ] **AC22**: Prelude includes new effect items

### Compatibility

- [ ] **AC23**: `compat` module provides deprecated aliases
- [ ] **AC24**: Deprecation warnings are clear and helpful

### Tests

- [ ] **AC25**: `cargo test` passes
- [ ] **AC26**: `cargo test --features tracing` passes
- [ ] **AC27**: `cargo test --features async` passes
- [ ] **AC28**: All example code compiles

## Technical Details

### Helper Combinator Implementations

```rust
// In src/effect/ext.rs

pub trait EffectExt: Effect {
    // ... existing methods ...

    /// Perform a side effect and return the original value.
    fn tap<F, E2>(self, f: F) -> Tap<Self, F>
    where
        F: FnOnce(&Self::Output) -> E2 + Send,
        E2: Effect<Output = (), Error = Self::Error, Env = Self::Env>,
        Self::Output: Clone,
    {
        Tap { inner: self, f }
    }

    /// Fail with error if predicate is false.
    fn check<P, F>(self, predicate: P, error_fn: F) -> Check<Self, P, F>
    where
        P: FnOnce(&Self::Output) -> bool + Send,
        F: FnOnce() -> Self::Error + Send,
    {
        Check { inner: self, predicate, error_fn }
    }

    /// Combine with another effect, returning tuple.
    fn with<F, E2>(self, f: F) -> With<Self, F>
    where
        F: FnOnce(&Self::Output) -> E2 + Send,
        E2: Effect<Error = Self::Error, Env = Self::Env>,
        Self::Output: Clone,
    {
        With { inner: self, f }
    }

    /// Chain with automatic error conversion.
    fn and_then_auto<U, E2, F>(self, f: F) -> AndThenAuto<Self, F>
    where
        F: FnOnce(Self::Output) -> E2 + Send,
        E2: Effect<Env = Self::Env>,
        Self::Error: From<E2::Error>,
    {
        AndThenAuto { inner: self, f }
    }

    /// Chain by borrowing value, return original.
    fn and_then_ref<U, F, E2>(self, f: F) -> AndThenRef<Self, F>
    where
        F: FnOnce(&Self::Output) -> E2 + Send,
        E2: Effect<Error = Self::Error, Env = Self::Env>,
        Self::Output: Clone,
    {
        AndThenRef { inner: self, f }
    }
}
```

### Retry Implementation Approach

The retry methods need special handling because they create effects with different output types (`RetryExhausted<T>`). Two approaches:

**Option A: Standalone functions**
```rust
// In src/effect/retry.rs
pub fn retry<F, E>(
    make_effect: F,
    policy: RetryPolicy,
) -> impl Effect<Output = RetryExhausted<E::Output>, Error = RetryExhausted<E::Error>, Env = E::Env>
where
    F: Fn() -> E + Send + 'static,
    E: Effect,
{
    // Implementation using FromAsync
}
```

**Option B: Extension trait method returning BoxedEffect**
```rust
// Methods that change the output type must return BoxedEffect
fn retry<F>(make_effect: F, policy: RetryPolicy) -> BoxedEffect<RetryExhausted<T>, RetryExhausted<E>, Env>
```

Recommend **Option A** for consistency with zero-cost pattern.

### Context Error Implementation

```rust
// In src/effect/context.rs

/// Extension trait for adding context to Effect errors.
pub trait EffectContext<T, E, Env>: Effect<Output = T, Error = E, Env = Env> {
    fn context(self, msg: impl Into<String> + Send + 'static)
        -> impl Effect<Output = T, Error = ContextError<E>, Env = Env>;
}

impl<Eff> EffectContext<Eff::Output, Eff::Error, Eff::Env> for Eff
where
    Eff: Effect,
{
    fn context(self, msg: impl Into<String> + Send + 'static)
        -> impl Effect<Output = Eff::Output, Error = ContextError<Eff::Error>, Env = Eff::Env>
    {
        Context::new(self, msg)
    }
}

// For chaining context on already-wrapped errors
impl<Eff, E> EffectExt for Eff
where
    Eff: Effect<Error = ContextError<E>>,
{
    fn context(self, msg: impl Into<String> + Send + 'static) -> impl Effect<...> {
        // Add to existing context trail
    }
}
```

### File Structure After Consolidation

```
src/
├── lib.rs                    # Updated exports
├── effect/                   # Renamed from effect_v2
│   ├── mod.rs
│   ├── trait_def.rs          # Effect trait
│   ├── ext.rs                # EffectExt with all combinators
│   ├── boxed.rs              # BoxedEffect
│   ├── combinators/          # Combinator types
│   │   ├── mod.rs
│   │   ├── pure.rs
│   │   ├── fail.rs
│   │   ├── map.rs
│   │   ├── and_then.rs
│   │   ├── tap.rs            # NEW
│   │   ├── check.rs          # NEW
│   │   ├── with.rs           # NEW
│   │   └── ...
│   ├── constructors.rs
│   ├── reader.rs
│   ├── bracket.rs
│   ├── parallel.rs           # Add par_all_limit
│   ├── retry.rs              # NEW
│   ├── context.rs            # NEW
│   ├── compat.rs
│   ├── prelude.rs
│   └── tests.rs
├── context.rs                # ContextError (existing)
├── retry.rs                  # RetryPolicy (existing)
└── ...
```

## Dependencies

### Prerequisites
- Spec 024 implemented (Zero-Cost Effect Trait) ✅

### Affected Components
- `src/lib.rs` - Public exports
- `src/effect_v2/` - Rename and extend
- `src/effect.rs` - Delete
- All code importing `Effect`

### Blocked Specifications
- Spec 025 (Documentation Update) - Should wait for this consolidation

## Testing Strategy

### Unit Tests

Port all tests from `src/effect.rs`:
- Helper combinator tests (`tap`, `check`, `with`, etc.)
- Retry tests
- Context error tests
- Tracing tests (feature-gated)

### Integration Tests

- Verify all examples compile with new API
- Test migration path from old API

### Regression Tests

- Ensure no behavior changes in ported features
- Verify error messages are equivalent

## Documentation Requirements

### Code Documentation

- All new combinator types must have rustdoc
- All new functions must have examples
- Module-level documentation updated

### Internal Documentation

- Document design decisions for porting
- Note any behavior differences from old API

## Implementation Notes

### Migration Order

1. **Phase 1**: Add missing features to `effect_v2`
   - Helper combinators
   - Retry support
   - Context errors
   - Tracing
   - `from_validation`, `par_all_limit`

2. **Phase 2**: Rename and restructure
   - Rename `effect_v2` to `effect`
   - Update `lib.rs` exports
   - Update prelude

3. **Phase 3**: Remove old implementation
   - Delete `src/effect.rs`
   - Update any internal references

4. **Phase 4**: Verify
   - Run all tests
   - Run all examples
   - Check documentation builds

### Breaking Changes

This is a breaking change. Users must:
1. Update imports from `stillwater::Effect` to `stillwater::effect::prelude::*`
2. Change return types from `Effect<T, E, Env>` to `impl Effect<...>` or `BoxedEffect<...>`
3. Change constructors from `Effect::pure(x)` to `pure(x)`
4. Add `.boxed()` where type erasure is needed

The `compat` module provides a migration path but is deprecated.

## Success Metrics

### Quantitative
- All tests pass
- No increase in compile time (should decrease)
- No runtime performance regression

### Qualitative
- Single, clear Effect API
- Consistent with `futures` crate patterns
- Easy migration for existing users

---

*"One Effect to rule them all."*
