---
number: 016
title: Traverse and Sequence Utilities
category: foundation
priority: medium
status: draft
dependencies: []
created: 2025-11-22
---

# Specification 016: Traverse and Sequence Utilities

**Category**: foundation
**Priority**: medium
**Status**: draft
**Dependencies**: None

## Context

In functional programming, `traverse` and `sequence` are fundamental operations for working with collections of effects:

- **`sequence`**: Turn `Vec<Validation<T, E>>` into `Validation<Vec<T>, E>`
- **`traverse`**: Map a function `T -> Validation<U, E>` over `Vec<T>`, producing `Validation<Vec<U>, E>`

Stillwater partially has this via `Validation::all_vec()`, but it's not generalized and lacks traverse.

These operations enable clean composition of validation and effect-producing operations over collections.

## Objective

Provide `traverse` and `sequence` utilities for both `Validation` and `Effect`, enabling idiomatic functional transformation of collections.

## Requirements

### Functional Requirements

- `traverse(vec, f)` for Validation
- `sequence(vec)` for Validation (already exists as `all_vec`, generalize)
- `traverse(vec, f)` for Effect
- `sequence(vec)` for Effect
- Support iterators, not just Vec
- Preserve error accumulation for Validation
- Type-safe composition

### Acceptance Criteria

- [ ] `traverse` function for Validation implemented
- [ ] `sequence` function for Validation (generalized from `all_vec`)
- [ ] `traverse` function for Effect implemented
- [ ] `sequence` function for Effect implemented
- [ ] Works with any `IntoIterator`
- [ ] Comprehensive examples
- [ ] Tests verify correctness
- [ ] Documentation guide created

## Technical Details

### Implementation

```rust
/// Traverse a collection with a validation function.
///
/// Applies `f` to each element, accumulating all errors if any fail.
///
/// # Example
///
/// ```rust
/// let numbers = vec!["1", "2", "3", "invalid"];
/// let result = traverse(numbers, |s| parse_number(s));
/// // Fails with all parsing errors
/// ```
pub fn traverse<T, U, E, F, I>(iter: I, f: F) -> Validation<Vec<U>, E>
where
    I: IntoIterator<Item = T>,
    F: Fn(T) -> Validation<U, E>,
    E: Semigroup,
{
    let validations: Vec<_> = iter.into_iter().map(f).collect();
    Validation::all_vec(validations)
}

/// Sequence a collection of validations.
///
/// # Example
///
/// ```rust
/// let vals = vec![
///     Validation::success(1),
///     Validation::success(2),
///     Validation::success(3),
/// ];
/// let result = sequence(vals);
/// assert_eq!(result, Validation::success(vec![1, 2, 3]));
/// ```
pub fn sequence<T, E, I>(iter: I) -> Validation<Vec<T>, E>
where
    I: IntoIterator<Item = Validation<T, E>>,
    E: Semigroup,
{
    Validation::all_vec(iter.into_iter())
}

// Effect versions (fail-fast semantics)

pub fn traverse_effect<T, U, E, Env, F, I>(
    iter: I,
    f: F,
) -> Effect<Vec<U>, E, Env>
where
    I: IntoIterator<Item = T>,
    F: Fn(T) -> Effect<U, E, Env>,
{
    Effect::from_fn(move |env| {
        iter.into_iter()
            .map(|item| f(item).run(env))
            .collect::<Result<Vec<_>, _>>()
    })
}

pub fn sequence_effect<T, E, Env, I>(
    iter: I,
) -> Effect<Vec<T>, E, Env>
where
    I: IntoIterator<Item = Effect<T, E, Env>>,
{
    Effect::from_fn(move |env| {
        iter.into_iter()
            .map(|effect| effect.run(env))
            .collect::<Result<Vec<_>, _>>()
    })
}
```

## Documentation Requirements

- Update `docs/guide/02-validation.md` with traverse examples
- Add `docs/guide/12-traverse-patterns.md`
- Examples in rustdoc

## Success Metrics

- Clear, ergonomic API
- Zero overhead vs manual implementation
- Positive user feedback
